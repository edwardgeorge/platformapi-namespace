use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct Token {
    #[serde(rename(deserialize = "token_type"))]
    type_: String,
    #[serde(rename(deserialize = "access_token"))]
    value: String,
}

impl Token {
    pub fn get_type(&self) -> &str {
        &self.type_
    }
}

// Display used for .bearer_auth()
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// simply used for serialising so no need to take ownership of strs
#[derive(Debug, Serialize)]
pub struct NSDef<'a> {
    pub productkey: &'a str,
    pub ttl: &'a str,
    pub cluster: &'a str,
    pub namespace: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct NSResponse {
    pub message: String,
    pub namespace: String,
    pub expiry: String,
}

impl fmt::Display for NSResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "message: {}\nnamespace: {}\nexpiry: {}",
            self.message, self.namespace, self.expiry
        )
    }
}

#[derive(Debug, Serialize)]
pub struct OAuthCred {
    scope: String,
    client_id: String,
    client_secret: String,
    grant_type: String,
}

impl OAuthCred {
    pub fn new(scope: String, client_id: String, client_secret: String) -> Self {
        OAuthCred {
            scope: scope,
            client_id: client_id,
            client_secret: client_secret,
            grant_type: String::from("client_credentials"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment Error: {0}")]
    EnvironmentError(String),
    #[error("Error from OAuth API, status code: {0}\n{1}")]
    OAuthError(u16, String),
    #[error("Error from Platform API, status code: {0}\n{1}")]
    APIError(u16, String),
    #[error("Timeout calling PlatformAPI")]
    APITimeoutError,
    #[error("{0}")]
    UnknownError(String),
}
