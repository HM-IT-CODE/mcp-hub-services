use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

use crate::models::SentinelNode;

/// Forwards a raw MCP JSON-RPC request to a remote node via HTTP POST /mcp
pub async fn forward_mcp(client: &Client, node: &SentinelNode, body: Value) -> Result<Value> {
    let url = format!("{}/mcp", node.url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Node \"{}\" unreachable: {}", node.name, e))?;

    let status = resp.status();
    let json: Value = resp.json().await
        .map_err(|e| anyhow!("Invalid JSON from node \"{}\": {}", node.name, e))?;

    if !status.is_success() {
        return Err(anyhow!("Node \"{}\" returned HTTP {}", node.name, status));
    }

    Ok(json)
}

/// Pings a node's /health endpoint and returns latency in ms
pub async fn ping_node(client: &Client, node: &SentinelNode) -> Option<u64> {
    let url = format!("{}/health", node.url.trim_end_matches('/'));
    let start = std::time::Instant::now();
    client.get(&url).send().await.ok().map(|_| start.elapsed().as_millis() as u64)
}
