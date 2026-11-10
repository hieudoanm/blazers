//! Router assembly, embedded admin dashboard and server bootstrap.
//!
//! `app` builds the axum `Router`; `serve_http` opens the database pool,
//! runs migrations, wires every feature hub and starts listening.

mod assets;
use axum::middleware;
mod routes;
mod serve;

pub use routes::app;
pub use serve::serve_http;

async fn log_request(
    request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> impl axum::response::IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().to_string();
    let remote = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();
    println!("{method} {uri} {remote} {:?}", duration);
    response
}

#[cfg(test)]
mod tests;
