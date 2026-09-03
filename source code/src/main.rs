
mod api;
mod fee;
mod models;
mod proxy;
mod state;
mod utils;

use api::{app_router, AppCtx};
use proxy::spawn_port;
use state::Store;
use tracing_subscriber::{fmt, EnvFilter};
use axum::Router;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", std::env::var("RUST_LOG").unwrap_or_else(|_| "info,axum=warn".into()));
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let store = Store::new("data/state.json")?;
    let reg = proxy::ProxyRegistry::new();

    // spawn existing ports
    for p in store.list_ports().into_iter().filter(|p| p.enabled) {
        if let Err(e) = spawn_port(reg.clone(), p).await { eprintln!("spawn port failed: {e}"); }
    }

    let ctx = AppCtx { store: store.clone(), reg: reg.clone() };
    let api = app_router(ctx);
    let ui = Router::new().nest_service("/", ServeDir::new("static"));
    let app = ui.merge(api);

    let addr = std::net::SocketAddr::from(([0,0,0,0], 8080));
    tracing::info!(?addr, "HTTP server on");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
