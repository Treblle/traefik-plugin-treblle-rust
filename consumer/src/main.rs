use axum::{
    routing::post,
    Router,
    Json,
    http::StatusCode,
};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/consume", post(consume_request));

    let addr = "0.0.0.0:3000";
    println!("Consumer service listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn consume_request(
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    println!("Received request: {:?}", payload);
    (StatusCode::OK, Json(payload))
}