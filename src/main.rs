use clap::{App, AppSettings, Arg, SubCommand};
use reqwest::blocking::Client;
use regex::Regex;
use std::env;

pub mod auth;
pub mod types;
use auth::get_bearer_token;
use types::{Error, NSDef, NSResponse};

static HOSTNAME_ENV_VAR: &str = "PLATFORM_API_HOSTNAME";
static CLUSTER_ENV_VAR: &str = "PLATFORM_API_CLUSTER";

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

fn create(hostname: &str, cluster: &str, productkey: &str, name: &str, ttl: &str) -> Result<NSResponse, Error> {
    let client = Client::new();
    let token = get_bearer_token(&client)?;
    let url = format!("https://{}/namespace", hostname);
    let res = client
        .post(&url)
        .bearer_auth(token)
        .json(&NSDef {
            productkey,
            ttl: ttl,
            cluster: cluster,
            namespace: name,
        })
        .send()
        .unwrap();
    let status = res.status();
    let rtext = res.text().unwrap();
    if status.is_success() {
        Ok(serde_json::from_str(&rtext).unwrap())
    } else {
        Err(Error::APIError(status.as_u16(), rtext))
    }
}

fn run_create(hostname: &str, cluster: &str, productkey: &str, name: &str, ttl: &str) -> i32 {
    match create(hostname, cluster, productkey, name, ttl) {
        Ok(r) => {
            println!("{}", r);
            0
        },
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

fn main() {
    let def_hostname = env::var(HOSTNAME_ENV_VAR);
    let def_cluster = env::var(CLUSTER_ENV_VAR);
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
        let hostname = oreo(matches.value_of("hostname"), &def_hostname).unwrap_or_else(|e| {
            eprintln!("'--hostname' option missing and could not read {} env var: {}", HOSTNAME_ENV_VAR, e);
            std::process::exit(1)
        });
        let cluster = oreo(matches.value_of("cluster"), &def_cluster).unwrap_or_else(|e| {
            eprintln!("'--cluster' option missing and could not read {} env var: {}", CLUSTER_ENV_VAR, e);
            std::process::exit(1)
        });
        std::process::exit(run_create(hostname, cluster, productkey, name, ttl));
    } else {
        panic!("No subcommand");
    }
}

// apologies for this function,
// this is for missing options which should be taken from the environment
fn oreo<'a, E>(a: Option<&'a str>, b: &'a Result<String, E>) -> Result<&'a str, &'a E> {
    a.ok_or(()).or(b.as_ref().map(|v| v.as_ref()))
}
