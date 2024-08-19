use axum::{http::StatusCode, routing::post, Json, Router};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api", post(receive_data));

    let addr = "0.0.0.0:3002";
    println!("Treblle API listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn receive_data(Json(payload): Json<Value>) -> StatusCode {
    println!("Received data from middleware: {:?}", payload);
    StatusCode::OK
}
