use clap::{App, AppSettings, Arg, SubCommand};
use reqwest::blocking::Client;
use std::env;

pub mod auth;
pub mod types;
use auth::get_bearer_token;
use types::{Error, NSDef, NSResponse};

static HOSTNAME_ENV_VAR: &str = "PLATFORM_API_HOSTNAME";

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
        }
        Err(e) => match e {
            Error::EnvironmentError(s) => {
                eprintln!("Environment Error: {}", s);
                1
            }
            Error::OAuthError(s, m) => {
                eprintln!("Error from OAuth API, status code: {}\n{}", s, m);
                2
            }
            Error::APIError(s, m) => {
                eprintln!("Error from Platform API, status code: {}\n{}", s, m);
                3
            }
            Error::UnknownError(m) => {
                eprintln!("{}", m);
                4
            }
        },
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
                        .default_value("24h")
                        .required(false),
                )
                .arg(Arg::with_name("hostname").long("hostname").required(false))
                .arg(Arg::with_name("productkey").required(true).index(1))
                .arg(Arg::with_name("name").required(true).index(2)),
        )
        .get_matches();
    if let Some(crmatch) = matches.subcommand_matches("create") {
        let productkey = crmatch.value_of("productkey").unwrap();
        let name = crmatch.value_of("name").unwrap();
        let ttl = crmatch.value_of("ttl").unwrap();
        let hostname = crmatch.value_of("hostname");
        std::process::exit(run_create(hostname, productkey, name, ttl));
    } else {
        panic!("No subcommand");
    }
}
