use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/consume", post(consume_request));

    let addr = "0.0.0.0:3000";
    println!("Consumer service listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn consume_request(headers: HeaderMap, body: String) -> impl IntoResponse {
    println!("Received request:");
    println!("Headers: {:?}", headers);
    println!("Body: {}", body);

    (StatusCode::OK, "Request received and logged")
}
