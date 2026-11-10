use std::sync::Arc;

use super::assets::embedded_handler;
use super::log_request;

use crate::handlers;
use crate::handlers::AppState;
use crate::{content_type, notification, rate_limit};
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use tower::service_fn;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

pub fn app(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/api/health", get(handlers::get_health))
        .route("/api/auth/register", post(handlers::post_register))
        .route("/api/auth/login", post(handlers::post_login))
        .route(
            "/api/collections",
            get(handlers::list_collections).post(handlers::create_collection),
        )
        .route(
            "/api/collections/{name}",
            get(handlers::get_collection)
                .patch(handlers::update_collection)
                .delete(handlers::delete_collection),
        )
        .route(
            "/api/collections/{name}/records",
            get(handlers::list_records).post(handlers::create_record),
        )
        .route(
            "/api/collections/{name}/records/{id}",
            get(handlers::get_record)
                .patch(handlers::update_record)
                .delete(handlers::delete_record),
        )
        .route(
            "/api/buckets",
            get(handlers::list_buckets).post(handlers::create_bucket),
        )
        .route(
            "/api/buckets/{name}",
            get(handlers::get_bucket).delete(handlers::delete_bucket),
        )
        .route(
            "/api/buckets/{name}/files",
            get(handlers::list_files).post(handlers::upload_file),
        )
        .route(
            "/api/buckets/{name}/files/{id}",
            get(handlers::download_file).delete(handlers::delete_file),
        )
        .route(
            "/api/webhooks",
            get(handlers::list_webhooks).post(handlers::create_webhook),
        )
        .route(
            "/api/webhooks/{id}",
            get(handlers::get_webhook)
                .patch(handlers::update_webhook)
                .delete(handlers::delete_webhook),
        )
        .route("/api/webhooks/{id}/logs", get(handlers::list_webhook_logs))
        .route(
            "/api/secrets",
            get(handlers::list_secrets).post(handlers::create_secret),
        )
        .route(
            "/api/secrets/{id}",
            get(handlers::get_secret)
                .patch(handlers::update_secret)
                .delete(handlers::delete_secret),
        )
        .route("/api/websockets", get(handlers::list_ws_connections))
        .route(
            "/api/websockets/{id}",
            get(handlers::get_ws_connection).delete(handlers::delete_ws_connection),
        )
        .route(
            "/api/websockets/broadcast",
            post(handlers::broadcast_ws_message),
        )
        .route("/api/websockets/{id}/send", post(handlers::send_ws_message))
        .route(
            "/api/websockets/{id}/messages",
            get(handlers::list_ws_messages),
        )
        .route(
            "/api/websockets/messages",
            get(handlers::list_all_ws_messages),
        )
        .route(
            "/api/cache",
            get(handlers::list_cache)
                .post(handlers::set_cache)
                .delete(handlers::flush_cache),
        )
        .route("/api/cache/stats", get(handlers::get_cache_stats))
        .route("/api/cache/keys", get(handlers::list_cache_keys))
        .route("/api/cache/exists/{key}", get(handlers::cache_key_exists))
        .route("/api/cache/delete", post(handlers::delete_cache_keys))
        .route(
            "/api/cache/{key}",
            get(handlers::get_cache).delete(handlers::delete_cache),
        )
        .route("/api/cache/{key}/incr", post(handlers::incr_cache))
        .route("/api/cache/{key}/append", post(handlers::append_cache))
        .route("/api/cache/{key}/expire", post(handlers::expire_cache))
        .route("/api/cache/{key}/ttl", get(handlers::get_cache_ttl))
        .route(
            "/api/cache/{key}/list",
            get(handlers::list_cache_list).post(handlers::push_cache_list),
        )
        .route("/api/cache/{key}/list/pop", post(handlers::pop_cache_list))
        .route(
            "/api/cache/{key}/hash",
            get(handlers::get_cache_hash).post(handlers::set_cache_hash),
        )
        .route(
            "/api/cache/{key}/hash/{field}",
            get(handlers::get_cache_hash_field).delete(handlers::delete_cache_hash_fields),
        )
        .route(
            "/api/notifications",
            get(handlers::handle_notifications_list)
                .post(handlers::handle_notifications_create)
                .delete(handlers::handle_notifications_clear),
        )
        .route(
            "/api/notifications/{id}",
            get(handlers::handle_notifications_get)
                .patch(handlers::handle_notifications_mark_read)
                .delete(handlers::handle_notifications_delete),
        )
        .route(
            "/api/logs",
            get(handlers::handle_logs_list)
                .post(handlers::handle_logs_create)
                .delete(handlers::handle_logs_clear),
        )
        .route(
            "/api/cronjobs",
            get(handlers::list_cron_jobs).post(handlers::create_cron_job),
        )
        .route(
            "/api/cronjobs/{id}",
            get(handlers::get_cron_job)
                .patch(handlers::update_cron_job)
                .delete(handlers::delete_cron_job),
        )
        .route("/api/cronjobs/{id}/run", post(handlers::run_cron_job))
        .route("/api/cronjobs/{id}/logs", get(handlers::list_cron_job_logs))
        .route("/api/backup", get(handlers::get_backup))
        .route("/api/export", get(crate::import_export::handle_export))
        .route("/api/import", post(crate::import_export::handle_import))
        .route(
            "/api/permissions",
            get(crate::rbac::handle_list_permissions).post(crate::rbac::handle_create_permission),
        )
        .route(
            "/api/permissions/{id}",
            delete(crate::rbac::handle_delete_permission),
        )
        .route(
            "/api/buckets/{name}/files/{id}/thumb",
            get(handlers::get_thumbnail),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_middleware,
        ))
        .layer(middleware::from_fn(content_type::require_json_content_type))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024));

    Router::new()
        .merge(api)
        .route("/api/openapi.json", get(handlers::get_openapi_json))
        .route("/api/docs", get(handlers::get_swagger_ui))
        .route("/api/docs/", get(handlers::get_swagger_ui))
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
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(log_request))
        .fallback_service(service_fn(embedded_handler))
        .with_state(state)
}
