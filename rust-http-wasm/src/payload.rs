use crate::config::Config;
use crate::error::Result;
use crate::schema::*;
use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

pub struct Payload {
    pub data: TrebllePayload,
    sensitive_keys_regex: Regex,
}

impl Payload {
    pub fn new(config: &Config) -> Self {
        let sensitive_keys_regex = Regex::new(&config.sensitive_keys_regex).unwrap();

        Payload {
            data: TrebllePayload {
                api_key: config.api_key.clone(),
                project_id: config.project_id.clone(),
                version: 0.6,
                sdk: "rust-wasm".to_string(),
                data: PayloadData::default(),
            },
            sensitive_keys_regex,
        }
    }

    pub fn update_request_info(
        &mut self,
        method: String,
        url: String,
        ip: String,
        headers: HashMap<String, String>,
        body: String,
    ) {
        self.data.data.request.method = method;
        self.data.data.request.url = url;
        self.data.data.request.ip = ip;
        self.data.data.request.headers = headers;
        self.data.data.request.body = serde_json::from_str(&body).unwrap_or(Value::String(body));
        self.data.data.request.timestamp = Utc::now().to_rfc3339();

        if let Some(user_agent) = self.data.data.request.headers.get("User-Agent") {
            self.data.data.request.user_agent = user_agent.clone();
        }
    }

    pub fn mask_sensitive_data(&mut self) {
        Self::mask_recursive(&self.sensitive_keys_regex, &mut self.data.data.request.body);
        Self::mask_recursive(
            &self.sensitive_keys_regex,
            &mut self.data.data.response.body,
        );
    }

    fn mask_recursive(regex: &Regex, value: &mut Value) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj.iter_mut() {
                    if regex.is_match(key) {
                        *val = Value::String("*****".to_string());
                    } else {
                        Self::mask_recursive(regex, val);
                    }
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    Self::mask_recursive(regex, val);
                }
            }
            _ => {}
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.data).map_err(Into::into)
    }
}

pub fn is_json(content_type: &str) -> bool {
    content_type.to_lowercase().contains("application/json")
}
