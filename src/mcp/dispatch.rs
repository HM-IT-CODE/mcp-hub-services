//! MCP JSON-RPC routing across multiple SQL Sentinel nodes.
//! - `initialize`  → the hub answers as an MCP server itself
//! - `tools/list`  → fan out to every active node, aggregate tools (prefixed per node)
//! - `tools/call`  → route to the node that owns the (prefixed) tool name

use serde_json::{json, Value};

use crate::hub::HubContext;
use crate::models::SentinelNode;
use crate::proxy::forward_mcp;

const PROTOCOL_VERSION: &str = "2024-11-05";
const SEP: &str = "__";

/// Entry point: returns the JSON-RPC response for a single MCP request.
pub async fn dispatch(ctx: &HubContext, body: Value) -> Value {
    let id     = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body.get("method").and_then(Value::as_str).unwrap_or("").to_string();

    match method.as_str() {
        "initialize" => initialize_result(id),
        "tools/list" => tools_list(ctx, id).await,
        "tools/call" => tools_call(ctx, body, id).await,
        other        => error(id, -32601, &format!("Method not routed by hub: {other}")),
    }
}

fn initialize_result(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "sentinel-hub", "version": env!("CARGO_PKG_VERSION") }
        }
    })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Tool-name prefix for a node: keeps tools from different nodes from colliding.
fn prefix_of(node: &SentinelNode) -> String {
    let safe: String = node.name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    format!("{safe}{SEP}")
}

fn active_nodes(ctx: &HubContext) -> Result<Vec<SentinelNode>, String> {
    ctx.registry.list()
        .map(|nodes| nodes.into_iter().filter(|n| n.active).collect())
        .map_err(|e| e.to_string())
}

async fn tools_list(ctx: &HubContext, id: Value) -> Value {
    let nodes = match active_nodes(ctx) {
        Ok(n)  => n,
        Err(e) => return error(id, -32603, &e),
    };

    let req = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    let calls = nodes.into_iter().map(|node| {
        let req = req.clone();
        async move {
            let result = forward_mcp(&ctx.client, &node, req).await;
            (node, result)
        }
    });

    let responses = futures::future::join_all(calls).await;

    let mut tools = Vec::new();
    for (node, result) in responses {
        let Ok(resp) = result else { continue };
        let Some(node_tools) = resp.pointer("/result/tools").and_then(Value::as_array) else { continue };

        let prefix = prefix_of(&node);
        for tool in node_tools {
            let mut t = tool.clone();
            if let Some(name) = t.get("name").and_then(Value::as_str) {
                t["name"] = json!(format!("{prefix}{name}"));
            }
            if let Some(desc) = t.get("description").and_then(Value::as_str) {
                t["description"] = json!(format!("[{}] {}", node.name, desc));
            }
            tools.push(t);
        }
    }

    json!({ "jsonrpc": "2.0", "id": id, "result": { "tools": tools } })
}

async fn tools_call(ctx: &HubContext, mut body: Value, id: Value) -> Value {
    let full_name = body.pointer("/params/name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let nodes = match active_nodes(ctx) {
        Ok(n)  => n,
        Err(e) => return error(id, -32603, &e),
    };

    for node in &nodes {
        let prefix = prefix_of(node);
        if let Some(real_name) = full_name.strip_prefix(&prefix) {
            body["params"]["name"] = json!(real_name);
            return match forward_mcp(&ctx.client, node, body).await {
                Ok(resp) => resp,
                Err(e)   => error(id, -32603, &e.to_string()),
            };
        }
    }

    error(id, -32602, &format!("No active node owns tool \"{full_name}\""))
}
