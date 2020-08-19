use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::env;

use crate::types::{Error, OAuthCred, Token};

fn get_env_var(name: &str) -> Result<String, Error> {
    env::var(name).map_err(|e| {
        Error::EnvironmentError(format!("Could not get '{}' from environment: {}", name, e))
    })
}

fn get_oauth_creds_from_env() -> Result<OAuthCred, Error> {
    Ok(OAuthCred {
        scope: get_env_var("SCOPE")?,
        client_id: get_env_var("CLIENT_ID")?,
        client_secret: get_env_var("CLIENT_SECRET")?,
    })
}

pub fn get_bearer_token(client: &Client) -> Result<Token, Error> {
    let cred = get_oauth_creds_from_env()?;
    let res = client
        .post("https://login.microsoftonline.com/maersk.onmicrosoft.com/oauth2/v2.0/token")
        .body(format!(
            "client_id={}&scope={}&client_secret={}&grant_type=client_credentials",
            cred.client_id, cred.scope, cred.client_secret
        ))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .unwrap();
    let s = res.status();
    let t = res.text().unwrap();
    if s.is_success() {
        let token: Token = serde_json::from_str(&t).unwrap();
        if token.get_type() != "Bearer" {
            Err(Error::UnknownError(format!(
                "Unknown token type: {}",
                token.get_type()
            )))
        } else {
            Ok(token)
        }
    } else {
        // panic!("Received a {} status code from the oauth api");
        Err(Error::OAuthError(s.as_u16(), t))
    }
}
