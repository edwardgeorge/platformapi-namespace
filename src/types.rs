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

#[derive(Debug)]
pub struct OAuthCred {
    pub scope: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum Error {
    EnvironmentError(String),
    OAuthError(u16, String),
    APIError(u16, String),
    UnknownError(String),
}
