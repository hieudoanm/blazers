use rust_embed::RustEmbed;
use std::net::UdpSocket;

#[derive(RustEmbed)]
#[folder = "public/"]
pub(crate) struct Assets;

pub(crate) fn get_local_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0")
        && socket.connect("8.8.8.8:80").is_ok()
        && let Ok(addr) = socket.local_addr()
    {
        return addr.ip().to_string();
    }
    "127.0.0.1".to_string()
}

pub(crate) async fn embedded_handler(
    req: axum::http::Request<axum::body::Body>,
) -> Result<axum::response::Response<axum::body::Body>, std::convert::Infallible> {
    let path = req.uri().path().trim_start_matches('/');
    let filename = if path.is_empty() { "index.html" } else { path };
    match Assets::get(filename) {
        Some(content) => {
            let mime = mime_guess::from_path(filename).first_or_octet_stream();
            Ok(axum::response::Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(axum::body::Body::from(content.data))
                .unwrap())
        }
        None => Ok(axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("404 Not Found"))
            .unwrap()),
    }
}
