use exports::traefik::http_handler::handler::Guest;

wit_bindgen::generate!({
    path: "traefik-http-handler.wit",
    world: "http-handler",
    exports: {
        "traefik:http-handler/handler": HttpHandler,
    },
});

struct HttpHandler;

impl Guest for HttpHandler {
    fn handle_request() -> i64 {
        println!("Handling request in WASM module");
        0
    }

    fn handle_response(req_ctx: i32, is_error: i32) {
        println!(
            "Handling response in WASM module. req_ctx: {}, is_error: {}",
            req_ctx, is_error
        );
    }
}

// Ensure the `handle_request` function is exported with the correct name
#[export_name = "handle_request"]
pub extern "C" fn __wasm_export_handle_request() -> i64 {
    HttpHandler::handle_request()
}

// Ensure the `handle_response` function is exported with the correct name
#[export_name = "handle_response"]
pub extern "C" fn __wasm_export_handle_response(req_ctx: i32, is_error: i32) {
    HttpHandler::handle_response(req_ctx, is_error)
}
