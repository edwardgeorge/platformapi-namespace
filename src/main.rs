use clap::{App, AppSettings, Arg, SubCommand};
use log::info;
use regex::Regex;
use reqwest::blocking::Client;
use std::env;

pub mod auth;
pub mod types;
use auth::get_bearer_token;
use types::{Error, NSDef, NSResponse};

const HOSTNAME_ENV_VAR: &str = "PLATFORM_API_HOSTNAME";
const CLUSTER_ENV_VAR: &str = "PLATFORM_API_CLUSTER";
const TENANT_ENV_VAR: &str = "PLATFORM_API_TENANT";

fn strip_prefix_if_exists<'a>(name: &'a str, prefix: &str) -> &'a str {
    if name.starts_with(&format!("{}-", prefix)) {
        &name[prefix.len() + 1..]
    } else {
        name
    }
}

fn validate_ttl(inp: String) -> Result<(), String> {
    let re = Regex::new(r"^(1([hd]|[0-9]h)|2([hd]|[0-4]h)|[3-7][hd]|[89]h)$").unwrap();
    if re.is_match(&inp) {
        Ok(())
    } else {
        Err(String::from("Valid TTLs are 1-24h or 1-7d"))
    }
}

fn create(
    hostname: &str,
    tenant: &str,
    cluster: &str,
    productkey: &str,
    name: &str,
    ttl: &str,
) -> Result<NSResponse, Error> {
    let client = Client::new();
    let token = get_bearer_token(&client, tenant)?;
    let url = format!("https://{}/namespace", hostname);
    let payload = NSDef {
        productkey,
        ttl,
        cluster,
        namespace: name,
    };
    info!(
        "submitting request body to {}: {}",
        url,
        serde_json::to_string(&payload).unwrap_or_else(|err| format!("error: {:?}", err))
    );
    let res = client.post(&url).bearer_auth(token).json(&payload).send();
    let resp = match res {
        Ok(r) => r,
        Err(e) => {
            if e.is_timeout() {
                return Err(Error::APITimeoutError);
            } else {
                return Err(Error::UnknownError(
                    "Got an unknown communicating with the Platform API".to_owned(),
                ));
            }
        }
    };
    let status = resp.status();
    let rtext = resp.text().unwrap();
    if status.is_success() {
        let resp = serde_json::from_str(&rtext)
            .map_err(|e| Error::UnknownError(format!("Error decoding API Response: {}", e)))?;
        Ok(resp)
    } else {
        Err(Error::APIError(status.as_u16(), rtext))
    }
}

fn main() {
    let def_hostname = env::var(HOSTNAME_ENV_VAR);
    let def_cluster = env::var(CLUSTER_ENV_VAR);
    let def_tenant = env::var(TENANT_ENV_VAR);
    env_logger::init();
    let matches = App::new("Platform API Namespace Client")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("create")
                .about("Create Dynamic Namespace")
                .arg(
                    Arg::with_name("ttl")
                        .long("ttl")
                        .validator(validate_ttl)
                        .default_value("24h")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("strip-prefix")
                        .long("strip-prefix")
                        .short("s")
                        .takes_value(false)
                        .required(false),
                )
                .arg(Arg::with_name("hostname").long("hostname").required(false))
                .arg(Arg::with_name("cluster").long("cluster").required(false))
                .arg(Arg::with_name("tenant").long("tenant").required(false))
                .arg(Arg::with_name("productkey").required(true).index(1))
                .arg(Arg::with_name("name").required(true).index(2)),
        )
        .get_matches();
    if let Some(crmatch) = matches.subcommand_matches("create") {
        let productkey = crmatch.value_of("productkey").unwrap();
        let mut name = crmatch.value_of("name").unwrap();
        let ttl = crmatch.value_of("ttl").unwrap();
        if crmatch.is_present("strip-prefix") {
            name = strip_prefix_if_exists(name, productkey);
        }
        let hostname = merge_option_and_result(matches.value_of("hostname"), &def_hostname)
            .unwrap_or_else(|e| {
                eprintln!(
                    "'--hostname' option missing and could not read {} env var: {}",
                    HOSTNAME_ENV_VAR, e
                );
                std::process::exit(1)
            });
        let cluster = merge_option_and_result(matches.value_of("cluster"), &def_cluster)
            .unwrap_or_else(|e| {
                eprintln!(
                    "'--cluster' option missing and could not read {} env var: {}",
                    CLUSTER_ENV_VAR, e
                );
                std::process::exit(1)
            });
        let tenant = merge_option_and_result(matches.value_of("tenant"), &def_tenant)
            .unwrap_or_else(|e| {
                eprintln!(
                    "'--tenant' option missing and could not read {} env var: {}",
                    TENANT_ENV_VAR, e
                );
                std::process::exit(1)
            });
        std::process::exit(
            match create(hostname, tenant, cluster, productkey, name, ttl) {
                Ok(r) => {
                    println!("{}", r);
                    0
                }
                Err(e) => {
                    eprintln!("{}", e);
                    1
                }
            },
        );
    } else {
        panic!("No subcommand");
    }
}

// apologies for this function,
// this is for missing options which should be taken from the environment,
// it munges the types into the correct ones
fn merge_option_and_result<'a, E>(
    a: Option<&'a str>,
    b: &'a Result<String, E>,
) -> Result<&'a str, &'a E> {
    match a {
        Some(v) => Ok(v),
        None => match b {
            Ok(ref v) => Ok(v),
            Err(e) => Err(e),
        },
    }
}
