use clap::{App, AppSettings, Arg, SubCommand};
use reqwest::blocking::Client;
use regex::Regex;
use std::env;

pub mod auth;
pub mod types;
use auth::get_bearer_token;
use types::{Error, NSDef, NSResponse};

static HOSTNAME_ENV_VAR: &str = "PLATFORM_API_HOSTNAME";

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

fn create(hostname: &str, productkey: &str, name: &str, ttl: &str) -> Result<NSResponse, Error> {
    let client = Client::new();
    let token = get_bearer_token(&client)?;
    if token.get_type() != "Bearer" {
        return Err(Error::UnknownError(format!(
            "Unknown token type: {}",
            token.get_type()
        )));
    }
    let url = format!("https://{}/namespace", hostname);
    let data = NSDef {
        productkey,
        ttl: ttl,
        cluster: "core-dev-west-1",
        namespace: name,
    };
    let res = client
        .post(&url)
        .bearer_auth(token)
        .json(&data)
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

fn run_create(hostname: Option<&str>, productkey: &str, name: &str, ttl: &str) -> i32 {
    let hn_from_env = env::var(HOSTNAME_ENV_VAR);
    let hn_unwrapped = match hostname {
        Some(h) => h,
        None => match hn_from_env {
            Ok(ref h) => h,
            Err(e) => {
                eprintln!(
                    "No hostname provided and env var {} could not be read: {}",
                    HOSTNAME_ENV_VAR, e
                );
                return 1;
            }
        },
    };
    match create(hn_unwrapped, productkey, name, ttl) {
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
                .arg(Arg::with_name("productkey").required(true).index(1))
                .arg(Arg::with_name("name").required(true).index(2)),
        )
        .get_matches();
    if let Some(crmatch) = matches.subcommand_matches("create") {
        let productkey = crmatch.value_of("productkey").unwrap();
        let mut name = crmatch.value_of("name").unwrap();
        let ttl = crmatch.value_of("ttl").unwrap();
        let hostname = crmatch.value_of("hostname");
        if crmatch.is_present("strip-prefix") {
            name = strip_prefix_if_exists(name, productkey);
        }
        std::process::exit(run_create(hostname, productkey, name, ttl));
    } else {
        panic!("No subcommand");
    }
}
