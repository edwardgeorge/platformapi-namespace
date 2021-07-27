use derive_builder::*;
use klap::{Annotations, Labels};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct VaultServiceAccounts {
    include_default: bool,
    service_accounts: Vec<String>,
}

impl VaultServiceAccounts {
    pub fn new() -> Self {
        VaultServiceAccounts::default()
    }
    pub fn new_no_default() -> Self {
        VaultServiceAccounts {
            include_default: false,
            service_accounts: Vec::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.service_accounts.is_empty()
    }
    pub fn service_accounts_string(&self) -> String {
        if self.service_accounts.is_empty() && !self.include_default {
            return "".to_string();
        }
        let x = self.service_accounts.join(",");
        if self.include_default {
            format!("default,{}", x)
        } else {
            x
        }
    }
}

impl std::default::Default for VaultServiceAccounts {
    fn default() -> Self {
        VaultServiceAccounts {
            include_default: true,
            service_accounts: Vec::new(),
        }
    }
}

impl std::iter::Extend<String> for VaultServiceAccounts {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = String>,
    {
        for acc in iter.into_iter() {
            if acc == "default" {
                self.include_default = true;
            } else {
                self.service_accounts.push(acc);
            }
        }
    }
}

impl Serialize for VaultServiceAccounts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("vault_config", 1)?;
        s.serialize_field("service_account_name", &self.service_accounts_string())?;
        s.end()
    }
}

// simply used for serialising so no need to take ownership of strs
#[derive(Debug, Serialize, Builder)]
#[builder(setter(into))]
pub struct NSDef<'a> {
    pub productkey: &'a str,
    pub ttl: &'a str,
    pub cluster: &'a str,
    pub namespace: &'a str,
    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Labels,
    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub annotations: Annotations,
    #[builder(default)]
    #[serde(
        skip_serializing_if = "VaultServiceAccounts::is_empty",
        rename = "vault_config"
    )]
    pub vault_service_accounts: VaultServiceAccounts,
    #[builder(default)]
    #[serde(flatten)]
    pub extra_properties: HashMap<String, Value>,
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
            scope,
            client_id,
            client_secret,
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
