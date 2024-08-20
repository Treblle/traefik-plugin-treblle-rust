use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/consume", post(consume_request))
        .route("/blacklisted-example", post(blacklisted_example))
        .route("/sensitive-data", post(sensitive_data));

    let addr = "0.0.0.0:3000";
    println!("Consumer service listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn consume_request(headers: HeaderMap, body: String) -> impl IntoResponse {
    println!("Received request on /consume:");
    println!("Headers: {:?}", headers);
    println!("Body: {}", body);

    (StatusCode::OK, "Request received and logged")
}

async fn blacklisted_example(headers: HeaderMap, body: String) -> impl IntoResponse {
    println!("Received request on /blacklisted-example:");
    println!("Headers: {:?}", headers);
    println!("Body: {}", body);

    (StatusCode::OK, "Blacklisted request received and logged")
}

async fn sensitive_data(headers: HeaderMap, body: String) -> impl IntoResponse {
    println!("Received request on /sensitive-data:");
    println!("Headers: {:?}", headers);
    println!("Body: {}", body);

    (StatusCode::OK, "Sensitive data request received and logged")
}
