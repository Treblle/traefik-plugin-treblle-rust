use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::env;
use tokio::time::{interval, Duration};

fn generate_mock_user(id: u32) -> serde_json::Value {
    let mut rng = rand::thread_rng();
    json!({
        "id": id,
        "name": format!("User {}", id),
        "email": format!("user{}@example.com", id),
        "password": format!("password{}", rng.gen::<u32>()),
        "credit_card": format!("{:04} {:04} {:04} {:04}",
            rng.gen_range(1000..9999),
            rng.gen_range(1000..9999),
            rng.gen_range(1000..9999),
            rng.gen_range(1000..9999)
        ),
    })
}

async fn send_request(
    client: &Client,
    url: &str,
    content_type: &str,
    payload: &str,
    retry_count: u32,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut attempts = 0;
    loop {
        let request = client
            .post(url)
            .header("Content-Type", content_type)
            .body(payload.to_string());
        match request.send().await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < retry_count => {
                eprintln!("Error sending request (attempt {}): {}", attempts + 1, e);
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();
    let interval_duration = env::var("INTERVAL_DURATION")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u64>()
        .expect("INTERVAL_DURATION must be a valid u64");

    let mut interval = interval(Duration::from_secs(interval_duration));
    let mut request_count = 0;

    let base_url = env::var("CONSUMER_URL").unwrap_or_else(|_| "http://traefik".to_string());

    println!("Producer service started. Sending requests to {}", base_url);

    loop {
        interval.tick().await;
        request_count += 1;

        let (endpoint, content_type, payload) = match request_count % 5 {
            0 => (
                "/consume",
                "application/json",
                generate_mock_user(request_count).to_string(),
            ),
            1 => (
                "/consume",
                "application/json",
                json!({
                    "id": request_count,
                    "message": "Test JSON message",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })
                .to_string(),
            ),
            2 => (
                "/consume",
                "text/plain",
                format!("Plain text message {}", request_count),
            ),
            3 => (
                "/consume",
                "application/xml",
                format!(
                    "<message><id>{}</id><text>Test XML message</text></message>",
                    request_count
                ),
            ),
            _ => (
                "/blacklisted-example",
                "application/json",
                json!({
                    "id": request_count,
                    "message": "This request should be blacklisted, ignored by Treblle middleware, but still passed to Consumer",
                })
                .to_string(),
            ),
        };

        let url = format!("{}{}", base_url, endpoint);

        match send_request(&client, &url, content_type, &payload, 3).await {
            Ok(response) => {
                println!(
                    "Sent request {} to {} ({}): Status {}",
                    request_count,
                    url,
                    content_type,
                    response.status()
                );

                match response.text().await {
                    Ok(body) => println!("Response body: {}", body),
                    Err(e) => eprintln!("Error reading response body: {}", e),
                }
            }
            Err(e) => {
                eprintln!(
                    "Error sending request {} after 3 attempts: {}",
                    request_count, e
                );
            }
        }
    }
}
