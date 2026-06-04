use axum::Router;
use job_search_tool_lib::{db::Database, http_server};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,job_search_tool_lib=debug".into()),
        )
        .init();

    let data_dir = std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            directories::UserDirs::new()
                .map(|d| d.home_dir().join(".job-search-tool"))
                .unwrap_or_else(|| PathBuf::from("/data"))
        });

    tracing::info!("data dir: {}", data_dir.display());

    let db = Database::open(&data_dir).expect("failed to open database");
    db.migrate().expect("failed to apply migrations");

    let state = Arc::new(db);

    let dist_dir = std::env::var("DIST_DIR").unwrap_or_else(|_| {
        // When built with `cargo build`, dist is at project root
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dist")
            .to_string_lossy()
            .into_owned()
    });

    tracing::info!("serving frontend from: {dist_dir}");

    let app = Router::new()
        .nest("/api", http_server::router())
        .fallback_service(ServeDir::new(&dist_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
