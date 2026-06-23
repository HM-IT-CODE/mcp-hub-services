mod hub;
mod mcp;
mod models;
mod proxy;
mod registry;

use std::sync::Arc;
use axum::{routing::{get, post}, Router};
use reqwest::Client;
use tracing::info;

use hub::{
    HubContext, add_node, get_node, handle_mcp, health_check,
    list_nodes, remove_node, set_node_active,
};
use registry::NodeRegistry;

const DEFAULT_PORT: u16 = 3030;
const DB_PATH: &str     = "sentinel-hub.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("sentinel_hub=info".parse()?))
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let db_path = std::env::var("HUB_DB").unwrap_or_else(|_| DB_PATH.to_string());

    let state = Arc::new(HubContext {
        registry: NodeRegistry::open(&db_path)?,
        client:   Client::new(),
    });

    let app = Router::new()
        .route("/mcp",              post(handle_mcp))
        .route("/nodes",            get(list_nodes).post(add_node))
        .route("/nodes/:id",        get(get_node).delete(remove_node))
        .route("/nodes/:id/active", post(set_node_active))
        .route("/health",           get(health_check))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    info!("sentinel-hub listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
