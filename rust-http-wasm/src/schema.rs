use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrebllePayload {
    pub api_key: String,
    pub project_id: String,
    pub version: f32,
    pub sdk: String,
    pub data: PayloadData,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PayloadData {
    pub server: ServerInfo,
    pub language: LanguageInfo,
    pub request: RequestInfo,
    pub response: ResponseInfo,
    pub errors: Vec<ErrorInfo>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ServerInfo {
    pub ip: String,
    pub timezone: String,
    pub software: Option<String>,
    pub signature: Option<String>,
    pub protocol: String,
    pub encoding: Option<String>,
    pub os: OsInfo,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct OsInfo {
    pub name: String,
    pub release: String,
    pub architecture: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LanguageInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_php: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_errors: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RequestInfo {
    pub timestamp: String,
    pub ip: String,
    pub url: String,
    pub user_agent: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ResponseInfo {
    #[serde(serialize_with = "serialize_code")]
    pub code: u32,
    #[serde(serialize_with = "serialize_size")]
    pub size: usize,
    pub load_time: f64,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorInfo {
    pub source: String,
    pub error_type: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

fn serialize_code<S>(code: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&code.to_string())
}

fn serialize_size<S>(size: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&size.to_string())
}

impl Default for ErrorInfo {
    fn default() -> Self {
        ErrorInfo {
            source: String::new(),
            error_type: String::new(),
            message: String::new(),
            file: String::new(),
            line: 0,
        }
    }
}
