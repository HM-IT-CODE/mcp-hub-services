use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::mcp;
use crate::models::{NodeHealth, SentinelNode};
use crate::proxy::ping_node;
use crate::registry::NodeRegistry;

pub type HubState = Arc<HubContext>;

pub struct HubContext {
    pub registry: NodeRegistry,
    pub client:   Client,
}

/// POST /mcp — MCP JSON-RPC endpoint. The hub aggregates tools/list across all
/// active nodes and routes tools/call to the node that owns the tool.
pub async fn handle_mcp(
    State(state): State<HubState>,
    Json(body): Json<Value>,
) -> Response {
    let method = body.get("method").and_then(Value::as_str).unwrap_or("");

    // JSON-RPC notifications (e.g. notifications/initialized) expect no response.
    if method.starts_with("notifications/") {
        return StatusCode::ACCEPTED.into_response();
    }

    let response = mcp::dispatch(&state, body).await;
    (StatusCode::OK, Json(response)).into_response()
}

/// GET /nodes — list all registered nodes
pub async fn list_nodes(State(state): State<HubState>) -> impl IntoResponse {
    match state.registry.list() {
        Ok(nodes) => (StatusCode::OK, Json(json!(nodes))),
        Err(e)    => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

/// POST /nodes — register a new node
pub async fn add_node(
    State(state): State<HubState>,
    Json(mut node): Json<SentinelNode>,
) -> impl IntoResponse {
    node.id = Uuid::new_v4().to_string();
    match state.registry.add(&node) {
        Ok(_)  => (StatusCode::CREATED, Json(json!(node))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

/// DELETE /nodes/:id — remove a node
pub async fn remove_node(
    State(state): State<HubState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.registry.remove(&id) {
        Ok(_)  => (StatusCode::OK, Json(json!({"ok": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

/// GET /nodes/:id — fetch a single node
pub async fn get_node(
    State(state): State<HubState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.registry.get_by_id(&id) {
        Ok(Some(node)) => (StatusCode::OK, Json(json!(node))),
        Ok(None)       => (StatusCode::NOT_FOUND, Json(json!({"error": "Node not found"}))),
        Err(e)         => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

/// POST /nodes/:id/active — enable or disable a node without deleting it
pub async fn set_node_active(
    State(state): State<HubState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let active = body.get("active").and_then(Value::as_bool).unwrap_or(true);
    match state.registry.set_active(&id, active) {
        Ok(_)  => (StatusCode::OK, Json(json!({"ok": true, "active": active}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

/// GET /health — ping all nodes and return their status
pub async fn health_check(State(state): State<HubState>) -> impl IntoResponse {
    let nodes = state.registry.list().unwrap_or_default();
    let mut results = Vec::new();
    for node in &nodes {
        let latency = ping_node(&state.client, node).await;
        results.push(NodeHealth {
            node_id:    node.id.clone(),
            online:     latency.is_some(),
            latency_ms: latency,
        });
    }
    (StatusCode::OK, Json(json!(results)))
}
