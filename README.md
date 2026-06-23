# sentinel-hub

> **Multi-node MCP orchestrator.** One endpoint, many machines. Lets an AI assistant
> (Claude Desktop, VS Code, Antigravity) reach every SQL Sentinel node on your network
> through a single connection — the way Acumatica interconnects every PC in a warehouse.

```
Claude Desktop ──► sentinel-hub (:3030) ──┬──► sentinel-node @ warehouse-pc  (:2020)
                                          ├──► sentinel-node @ erp-server     (:2020)
                                          └──► sentinel-node @ finance-pc     (:2020)
```

Instead of configuring N MCP servers by hand in every client, you point the client at
**one** hub. The hub keeps a registry of nodes and forwards/merges MCP tool calls.

---

## Architecture

```
src/
  main.rs              ← axum HTTP server (port 3030)
  models/node.rs       ← SentinelNode, NodeHealth
  registry/store.rs    ← NodeRegistry (portable SQLite, bundled)
  proxy/forwarder.rs   ← forward_mcp(), ping_node()
  hub/router.rs        ← HTTP handlers
```

### Endpoints

| Method | Path                | Purpose                                          |
| ------ | ------------------- | ------------------------------------------------ |
| POST   | `/mcp`              | MCP JSON-RPC: `initialize`, `tools/list`, `tools/call`. |
| GET    | `/nodes`            | List registered nodes.                           |
| POST   | `/nodes`            | Register a new node.                              |
| GET    | `/nodes/:id`        | Fetch a single node.                             |
| DELETE | `/nodes/:id`        | Remove a node.                                    |
| POST   | `/nodes/:id/active` | Enable/disable a node (`{"active": bool}`).       |
| GET    | `/health`           | Ping every node, report latency.                 |

### How MCP routing works

- **`initialize`** — the hub answers as an MCP server itself.
- **`tools/list`** — fans out in parallel to every active node, then merges their tools
  into one catalog. Each tool is namespaced `{node}__{tool}` so names never collide, and
  its description is tagged `[node]`. Offline nodes are skipped, not fatal.
- **`tools/call`** — the hub reads the `{node}__` prefix, strips it, and forwards the call
  to the node that owns the tool.

---

## Run

```bash
cargo run --release
# PORT=3030  HUB_DB=sentinel-hub.db   (env overrides)
```

Register a node:

```bash
curl -X POST http://localhost:3030/nodes -H "Content-Type: application/json" -d '{
  "name": "warehouse-pc",
  "url": "http://192.168.1.20:2020",
  "description": "WMS database node",
  "active": true
}'
```

Point Claude Desktop / Antigravity at `http://<hub-host>:3030/mcp`.

---

## Status

✅ **Working MVP.** Real MCP routing: `tools/list` aggregates tools across all active
nodes (parallel fan-out, namespaced per node), and `tools/call` routes to the owning
node. Verified live against a real SQL Sentinel node. Next: SSE transport, auth, admin
GUI. See `PROGRESS.md`.

---

## License

MIT © Henry Moreno
