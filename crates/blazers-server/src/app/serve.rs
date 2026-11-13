use std::sync::Arc;

use crate::app::app;
use crate::app::assets::get_local_ip;
use crate::handlers::AppState;
use crate::{cache, cronjobs, notification, pubsub, rate_limit, secrets, websocket};
use blazers_core::db;

pub async fn serve_http() {
    let data_dir = db::data_dir();
    std::fs::create_dir_all(&data_dir).expect("create data dir");
    let db_path = data_dir.join("data.db");
    let pool: deadpool::managed::Pool<db::ConnectionManager> =
        deadpool::managed::Pool::builder(db::ConnectionManager { path: db_path })
            .build()
            .unwrap();

    {
        let conn = pool.get().await.expect("get db connection for migration");
        db::migrate_db(&conn).expect("migrate db");
    }

    let key = secrets::get_or_create_secrets_key(&db::data_dir()).expect("secrets key");
    let ws_hub = websocket::new_ws_hub();
    tokio::spawn(websocket::ws_hub_run(ws_hub.clone()));
    let cache_arc = std::sync::Arc::new(cache::CacheStore::new());
    {
        let conn = pool.get().await.expect("get db for cache init");
        cache_arc.load_from_db(&conn);
    }
    cache::start_eviction_loop(cache_arc.clone());
    let sse_hub = notification::new_sse_hub();
    let log_hub = crate::log::new_log_hub();
    let pubsub_hub = pubsub::new_pubsub_hub();
    let rate_limiter = std::sync::Arc::new(rate_limit::RateLimiter::new(200.0, 100.0));
    let state = Arc::new(AppState {
        db: pool,
        storage_dir: db::data_dir(),
        http_client: reqwest::Client::new(),
        secrets_key: key,
        ws_hub,
        cache: cache_arc,
        sse_hub,
        log_hub,
        pubsub_hub,
        rate_limiter,
    });

    cronjobs::start_cron_scheduler(state.clone()).await;
    tokio::spawn(crate::backups::scheduler(state.clone()));

    {
        let rate_limiter = state.rate_limiter.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                rate_limiter.cleanup();
            }
        });
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");

    let local_url = format!("http://localhost:{port}");
    let net_url = format!("http://{}:{port}", get_local_ip());
    let local_link = format!("\x1b]8;;{local_url}\x07{local_url}\x1b]8;;\x07");
    let net_link = format!("\x1b]8;;{net_url}\x07{net_url}\x1b]8;;\x07");
    let local_pad = " ".repeat(29 - local_url.len());
    let net_pad = " ".repeat(29 - net_url.len());
    println!();
    println!("  ┌─────────────────────────────────────────────┐");
    println!("  │  Server running at:                         │");
    println!("  │                                             │");
    println!("  │    ➜  Local:   {local_link}{local_pad}│");
    println!("  │    ➜  Network: {net_link}{net_pad}│");
    println!("  │                                             │");
    println!("  └─────────────────────────────────────────────┘");
    println!();

    axum::serve(
        listener,
        app(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("server error");
}

#[cfg(unix)]
async fn shutdown_signal() {
    let sigint = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("install sigint handler")
            .recv()
            .await;
    };
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install sigterm handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = sigint => {},
        _ = sigterm => {},
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
