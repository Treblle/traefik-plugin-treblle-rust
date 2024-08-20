use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::Value;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app = Router::new().route("/api", post(receive_data));

    let addr = "0.0.0.0:3002";
    info!("Treblle API listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn receive_data(headers: HeaderMap, Json(payload): Json<Value>) -> impl IntoResponse {
    info!("Received request with headers: {:?}", headers);
    info!("Full payload: {:?}", payload);

    if let Err(e) = validate_request(&headers, &payload) {
        error!("Request validation failed: {}", e);
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    info!("Received valid data from middleware");
    StatusCode::OK.into_response()
}

fn validate_request(headers: &HeaderMap, payload: &Value) -> Result<(), String> {
    if !headers.contains_key("content-type") || headers["content-type"] != "application/json" {
        return Err("Missing or invalid Content-Type header".into());
    }

    if !headers.contains_key("x-api-key") {
        return Err("Missing x-api-key header".into());
    }

    if !validate_payload(payload) {
        return Err("Invalid payload structure".into());
    }

    Ok(())
}

fn validate_payload(payload: &Value) -> bool {
    ["api_key", "project_id", "version", "sdk", "data"]
        .iter()
        .all(|key| payload.get(*key).is_some())
}
