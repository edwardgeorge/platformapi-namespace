use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use core::time::Duration;
use klap::{Annotations, Labels};
use log::info;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;
use std::io::BufReader;

mod auth;
mod metadata;
mod types;
use auth::get_bearer_token;
use metadata::metadata_from_matches;
use types::{Error, ExitError, ExtraProps, NSDef, NSDefBuilder, NSResponse, VaultServiceAccounts};

const HOSTNAME_ENV_VAR: &str = "PLATFORM_API_HOSTNAME";
const CLUSTER_ENV_VAR: &str = "PLATFORM_API_CLUSTER";
const TENANT_ENV_VAR: &str = "PLATFORM_API_TENANT";

macro_rules! option_or_env {
    ($matches:ident, $opt:expr, $var:ident) => {
        if let Some(val) = $matches.value_of($opt) {
            val.to_string()
        } else {
            match env::var($var) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "'--{}' option missing and could not read {} env var: {}",
                        $opt, $var, e
                    );
                    std::process::exit(1);
                }
            }
        }
    };
}

fn validate_ttl(inp: String) -> Result<(), String> {
    let re = Regex::new(r"^(1([hd]|[0-9]h)|2([hd]|[0-4]h)|[3-7][hd]|[89]h)$").unwrap();
    if re.is_match(&inp) {
        Ok(())
    } else {
        Err(String::from("Valid TTLs are 1-24h or 1-7d"))
    }
}

fn match_vault_service_accounts(matches: &ArgMatches<'_>) -> VaultServiceAccounts {
    let mut vsas: VaultServiceAccounts;
    if let Some(val) = matches.value_of("svcac-raw") {
        vsas = VaultServiceAccounts::new_no_default();
        vsas.extend(val.split(',').map(|v| v.trim().to_string()));
    } else {
        vsas = VaultServiceAccounts::new();
    };
    if let Some(vals) = matches.values_of("svcac") {
        vsas.extend(vals.map(|v| v.to_string()));
    }
    vsas
}

fn match_extra(matches: &ArgMatches<'_>) -> Result<ExtraProps, Error> {
    if let Some(val) = matches.value_of("extra-props") {
        if let Some(filename) = val.strip_prefix('@') {
            let f = std::fs::File::open(filename).map_err(|e| {
                Error::Option("extra-data".to_string(), val.to_string(), e.to_string())
            })?;
            serde_yaml::from_reader(BufReader::new(f)).map_err(|e| {
                Error::Option("extra-data".to_string(), val.to_string(), e.to_string())
            })
        } else {
            serde_yaml::from_str(val).map_err(|e| {
                Error::Option("extra-data".to_string(), val.to_string(), e.to_string())
            })
        }
    } else {
        Ok(HashMap::new())
    }
}

fn create(hostname: &str, tenant: &str, payload: NSDef) -> Result<NSResponse, Error> {
    let client = Client::new();
    let token = get_bearer_token(&client, tenant)?;
    let url = format!("https://{}/namespace", hostname);
    info!(
        "submitting request body to {}: {}",
        url,
        serde_json::to_string(&payload).unwrap_or_else(|err| format!("error: {:?}", err))
    );
    let res = client
        .post(&url)
        .bearer_auth(token)
        .json(&payload)
        .timeout(Duration::from_secs(60))
        .send();
    let resp = match res {
        Ok(r) => r,
        Err(e) => {
            if e.is_timeout() {
                return Err(Error::APITimeout);
            } else {
                return Err(Error::Unknown(format!(
                    "Got an unknown error communicating with the Platform API: {}",
                    e
                )));
            }
        }
    };
    let status = resp.status();
    let rtext = resp.text().unwrap();
    if status.is_success() {
        let resp = serde_json::from_str(&rtext)
            .map_err(|e| Error::Unknown(format!("Error decoding API Response: {}", e)))?;
        Ok(resp)
    } else {
        Err(Error::Api(status.as_u16(), rtext))
    }
}

fn main() -> Result<(), ExitError> {
    env_logger::init();
    let matches = App::new("Platform API Namespace Client")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("create")
                .about("Create Dynamic Namespace")
                .arg(
                    Arg::with_name("ttl")
                        .long("ttl")
                        .help("ttl for namespace. valid values are 1-24h or 1-7d")
                        .validator(validate_ttl)
                        .default_value("24h")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("strip-prefix")
                        .long("strip-prefix")
                        .help("strip prefix from namespace name if it is already prepended")
                        .short("s")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("labels")
                        .short("l")
                        .long("labels")
                        .required(false)
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1),
                )
                .arg(
                    Arg::with_name("annotations")
                        .short("a")
                        .long("annotation")
                        .required(false)
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                )
                .arg(
                    Arg::with_name("manifest")
                        .long("metadata-from-manifest")
                        .required_if("name", "-")
                        .takes_value(true)
                        .multiple(false)
                        .number_of_values(1)
                )
                .arg(
                    Arg::with_name("svcac")
                        .long("vault-service-account")
                        .help("add an additional service account for vault access")
                        .required(false)
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1),
                )
                .arg(
                    Arg::with_name("svcac-raw")
                        .long("vault-service-account-raw")
                        .help("service accounts for vault access. comma-separated raw list of values.")
                        .required(false)
                        .takes_value(true)
                        .multiple(false)
                        .conflicts_with("svcac")
                        .number_of_values(1),
                )
                .arg(
                    Arg::with_name("extra-props")
                        .long("extra-data")
                        .help("provide extra params to api by reading in yaml/json. value prefixed with '@' is treated as a filename.")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .number_of_values(1),
                )
                .arg(
                    Arg::with_name("debug")
                        .short("d")
                        .long("dry-run")
                        .takes_value(false)
                        .required(false)
                )
                .arg(Arg::with_name("hostname").long("hostname").required(false).takes_value(true).help("hostname of API, otherwise read from PLATFORM_API_HOSTNAME env var"))
                .arg(Arg::with_name("cluster").long("cluster").required(false).takes_value(true).help("cluster name, otherwise read from PLATFORM_API_CLUSTER env var"))
                .arg(Arg::with_name("tenant").long("tenant").required(false).takes_value(true).help("tenant info for auth, otherwise read from PLATFORM_API_TENANT env var"))
                .arg(Arg::with_name("productkey").required(true).index(1).help("product key, prepended to namespace name"))
                .arg(Arg::with_name("name").required(true).index(2).help("namespace name, appended as suffix to product key")),
        )
        .get_matches();
    if let Some(crmatch) = matches.subcommand_matches("create") {
        let productkey = crmatch.value_of("productkey").unwrap();
        let mut name = crmatch.value_of("name").unwrap().to_string();
        let ttl = crmatch.value_of("ttl").unwrap();
        let metadata = metadata_from_matches(crmatch)?;
        let mut strict_strip_prefix = false;
        if name == "-" {
            name = match metadata.name {
                Some(mname) => mname,
                None => {
                    return Err(Error::Unknown(
                        "name passed as '-' but no name provided in manifest metadata".to_string(),
                    )
                    .into());
                }
            };
            strict_strip_prefix = true;
        }
        if crmatch.is_present("strip-prefix") || strict_strip_prefix {
            if let Some(suffix) = name.strip_prefix(&format!("{}-", productkey)) {
                name = suffix.to_string();
            } else if strict_strip_prefix {
                return Err(Error::Unknown(format!(
                    "Expected that name '{}' is prefixed with product key '{}'",
                    name, productkey
                ))
                .into());
            }
        }
        let hostname: String = option_or_env!(crmatch, "hostname", HOSTNAME_ENV_VAR);
        let cluster: String = option_or_env!(crmatch, "cluster", CLUSTER_ENV_VAR);
        let tenant: String = option_or_env!(crmatch, "tenant", TENANT_ENV_VAR);
        let vsas = match_vault_service_accounts(crmatch);
        let extra = match_extra(crmatch)?;
        let labelscollected: Labels = metadata.labels.into_iter().map(|a| a.into()).collect();
        let annotationscollected: Annotations =
            metadata.annotations.into_iter().map(|a| a.into()).collect();
        let payload = NSDefBuilder::default()
            .productkey(productkey)
            .ttl(ttl)
            .cluster(cluster)
            .namespace(name)
            .labels(labelscollected)
            .annotations(annotationscollected)
            .vault_service_accounts(vsas)
            .extra_properties(extra)
            .build()
            .unwrap();
        if crmatch.occurrences_of("debug") > 0 {
            println!(
                "Would submit the following payload to the API:\n{}",
                serde_json::to_string_pretty(&payload).unwrap()
            );
            eprintln!("Dry-run, not calling API!");
            return Ok(());
        }
        //std::process::exit(match create(&hostname, &tenant, payload) {
        //    Ok(r) => {
        //        println!("{}", r);
        //        0
        //    }
        //    Err(e) => {
        //        eprintln!("{}", e);
        //        1
        //    }
        //});
        let resp = create(&hostname, &tenant, payload)?;
        println!("{}", resp);
        Ok(())
    } else {
        panic!("No subcommand");
    }
}
