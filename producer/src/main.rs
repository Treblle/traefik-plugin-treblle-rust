use reqwest::Client;
use serde_json::json;
use std::env;
use tokio::time::{interval, sleep, Duration};

async fn send_request(
    client: &Client,
    url: &str,
    payload: serde_json::Value,
    retry_count: u32,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut attempts = 0;
    loop {
        match client.post(url).json(&payload).send().await {
            Ok(response) => return Ok(response),
            Err(e) if attempts < retry_count => {
                eprintln!("Error sending request (attempt {}): {}", attempts + 1, e);
                attempts += 1;
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();
    let mut interval = interval(Duration::from_secs(1));
    let mut request_count = 0;

    let consumer_url =
        env::var("CONSUMER_URL").unwrap_or_else(|_| "http://traefik/consume".to_string());

    println!(
        "Producer service started. Sending requests to {}",
        consumer_url
    );

    loop {
        interval.tick().await;
        request_count += 1;
        let payload = json!({
            "id": request_count,
            "message": "Test message",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        match send_request(&client, &consumer_url, payload, 3).await {
            Ok(response) => {
                println!(
                    "Sent request {}: Status {}",
                    request_count,
                    response.status()
                );

                // Optional: Print response body for debugging
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
