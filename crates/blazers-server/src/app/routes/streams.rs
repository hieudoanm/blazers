//! Realtime + docs routes merged into the main router.

use std::sync::Arc;

use axum::{Router, routing::get};

use crate::handlers::{self, AppState};
use crate::notification;

pub fn streams() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ws", get(crate::websocket::handle_ws_upgrade))
        .route(
            "/api/notifications/stream",
            get(notification::handle_notification_stream),
        )
        .route("/api/logs/stream", get(crate::log::handle_log_stream))
        .route(
            "/api/pubsub/topics",
            get(handlers::handle_pubsub_topics_list).post(handlers::handle_pubsub_topics_create),
        )
        .route(
            "/api/pubsub/topics/{name}",
            get(handlers::handle_pubsub_topics_get).delete(handlers::handle_pubsub_topics_delete),
        )
        .route(
            "/api/pubsub/topics/{name}/messages",
            get(handlers::handle_pubsub_messages_list)
                .post(handlers::handle_pubsub_messages_create),
        )
        .route(
            "/api/pubsub/{name}/stream",
            get(crate::pubsub::handle_pubsub_stream),
        )
}
