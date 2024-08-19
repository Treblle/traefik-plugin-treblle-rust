# Treblle's WASM Middleware Plugin for Traefik in Rust

This plugin is a WebAssembly module that can be loaded by Traefik and used to trace the request/response of incoming HTTP traffic and send it to Treblle's APIs.

## Development Setup

This project requires Rust version 1.80.0. To install this specific version, run:

```sh
rustup install 1.80.0
rustup default 1.80.0
```

Alternatively, the project includes a `rust-toolchain.toml` file which will automatically select the correct Rust version when you're in the project directory.

Make sure you have the wasm32-wasi target installed:

```sh
rustup target add wasm32-wasi
```
