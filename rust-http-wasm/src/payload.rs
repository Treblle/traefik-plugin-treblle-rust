use crate::config::Config;
use crate::error::Result;
use crate::schema::*;
use crate::utils;
use std::collections::HashMap;
use std::time::Instant;

pub struct Payload {
    pub data: TrebllePayload,
    config: Config,
}

impl Payload {
    pub fn new(config: Config) -> Self {
        Payload {
            data: TrebllePayload {
                api_key: config.api_key.clone(),
                project_id: config.project_id.clone(),
                version: 0.6,
                sdk: "rust-wasm".to_string(),
                data: PayloadData::default(),
            },
            config,
        }
    }

    pub fn update_request_info(
        &mut self,
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: &[u8],
    ) {
        self.data.data.request = utils::parse_request(method, url, headers, body, &self.config);
    }

    pub fn update_response_info(
        &mut self,
        status: u32,
        headers: HashMap<String, String>,
        body: &[u8],
        start_time: Instant,
    ) {
        self.data.data.response =
            utils::parse_response(status, headers, body, start_time, &self.config);
    }

    pub fn add_error(&mut self, error: ErrorInfo) {
        self.data.data.errors.push(error);
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.data).map_err(Into::into)
    }

    pub fn update_server_info(&mut self, protocol: String) {
        self.data.data.server = ServerInfo {
            ip: "Unknown".to_string(), // TODO: Implement server IP detection
            timezone: chrono::Local::now().format("%Z").to_string(),
            software: None,
            signature: None,
            protocol,
            encoding: None,
            os: OsInfo {
                name: std::env::consts::OS.to_string(),
                release: "Unknown".to_string(), // TODO: Implement OS release detection
                architecture: std::env::consts::ARCH.to_string(),
            },
        };
    }

    pub fn update_language_info(&mut self) {
        self.data.data.language = LanguageInfo {
            name: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            expose_php: None,
            display_errors: None,
        };
    }
}

pub fn is_json(content_type: &str) -> bool {
    utils::is_json(content_type)
}
