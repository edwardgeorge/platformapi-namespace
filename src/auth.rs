use reqwest::blocking::Client;
use std::env;
use urlencoding::decode;

use crate::types::{Error, OAuthCred, Token};

fn get_env_var(name: &str) -> Result<String, Error> {
    env::var(name).map_err(|e| {
        Error::Environment(format!("Could not get '{}' from environment: {}", name, e))
    })
}

fn get_oauth_creds_from_env() -> Result<OAuthCred, Error> {
    let mut scope = get_env_var("SCOPE")?;
    // hack to deal with already urlencoded data so that it isn't encoded twice...
    if (&scope).contains("%3A%2F%2F") {
        scope = decode(&scope).map_err(|e| Error::Unknown(e.to_string()))?;
    }
    Ok(OAuthCred::new(
        scope,
        get_env_var("CLIENT_ID")?,
        get_env_var("CLIENT_SECRET")?,
    ))
}

pub fn get_bearer_token(client: &Client, tenant: &str) -> Result<Token, Error> {
    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant
    );
    let res = client
        .post(&url)
        .form(&get_oauth_creds_from_env()?)
        .send()
        .map_err(|e| Error::Unknown(format!("Error from OAuth request: {}", e)))?;
    let s = res.status();
    let t = res
        .text()
        .map_err(|e| Error::Unknown(format!("Error obtaining body of OAuth response: {}", e)))?;
    if s.is_success() {
        let token: Token = serde_json::from_str(&t)
            .map_err(|e| Error::Unknown(format!("Error decoding OAuth API Response: {}", e)))?;
        if token.get_type() != "Bearer" {
            Err(Error::Unknown(format!(
                "Unknown token type: {}",
                token.get_type()
            )))
        } else {
            Ok(token)
        }
    } else {
        // panic!("Received a {} status code from the oauth api");
        Err(Error::OAuth(s.as_u16(), t))
    }
}
