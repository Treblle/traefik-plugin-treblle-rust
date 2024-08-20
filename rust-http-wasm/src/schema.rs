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
    pub software: String,
    pub signature: String,
    pub protocol: String,
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
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RequestInfo {
    pub timestamp: String,
    pub ip: String,
    pub url: String,
    pub user_agent: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ResponseInfo {
    pub headers: HashMap<String, String>,
    pub code: u16,
    pub size: usize,
    pub load_time: f64,
    pub body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorInfo {
    pub source: String,
    pub r#type: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

impl Default for ErrorInfo {
    fn default() -> Self {
        ErrorInfo {
            source: String::new(),
            r#type: String::new(),
            message: String::new(),
            file: String::new(),
            line: 0,
        }
    }
}