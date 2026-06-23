# 📋 sentinel-hub — Estado del Proyecto (para Antigravity / continuación)

> **Lee este archivo primero.** Fecha de corte: **2026-06-23**.

---

## 🎯 Qué es

Orquestador **multi-nodo MCP**. Un solo endpoint (`:3030`) al que la IA se conecta, y
el hub reparte las llamadas a todos los nodos SQL Sentinel de la red. Igual que
Acumatica interconecta todos los PC de un almacén, pero para herramientas de IA.

---

## ✅ Hecho (compila 100% limpio — `cargo build` ✅, cero warnings)

```
src/
  main.rs              ← servidor axum, rutas, puerto 3030, env PORT/HUB_DB
  models/node.rs       ← SentinelNode (id #[serde(default)]), NodeHealth
  registry/store.rs    ← NodeRegistry sobre SQLite (bundled): list/add/remove/set_active/get_by_id
  proxy/forwarder.rs   ← forward_mcp() (POST /mcp al nodo), ping_node() (latencia)
  mcp/dispatch.rs      ← ★ enrutamiento MCP: initialize / tools/list (fan-out) / tools/call (routing)
  hub/router.rs        ← handlers HTTP
```

### ★ Enrutamiento MCP REAL — HECHO y PROBADO (2026-06-23)
- **`initialize`** → el hub responde como servidor MCP (protocolVersion, serverInfo).
- **`tools/list`** → fan-out **en paralelo** (`futures::join_all`) a todos los nodos
  activos, agrega sus herramientas **prefijadas** `{nodo}__{tool}` y anota la descripción
  con `[nodo]`. Resiliente: si un nodo está offline, lo omite sin romper.
- **`tools/call`** → enruta al nodo dueño según el prefijo, le quita el prefijo y reenvía.
- **notifications/** → responde `202` sin body (correcto en JSON-RPC).
- **PROBADO EN VIVO**: con el nodo `mcp-sql-sentinel` real en `:2020`, el hub agregó sus
  9 herramientas (EstadoCuentaCliente, EjecutarSQLDinamico, ListarConexiones, Fivetran...).

### Endpoints HTTP
| Método | Ruta | Función |
|--------|------|---------|
| POST   | `/mcp` | endpoint MCP (initialize/tools-list/tools-call) |
| GET/POST | `/nodes` | listar / registrar nodo |
| GET/DELETE | `/nodes/:id` | obtener / borrar nodo |
| POST | `/nodes/:id/active` | activar/desactivar (`{"active":bool}`) |
| GET | `/health` | ping a todos los nodos + latencia |

---

## ⏳ Pendiente

### 1. Transporte SSE (opcional)
- Hoy es POST `/mcp` (JSON-RPC sobre HTTP). Algunos clientes MCP esperan SSE. Evaluar
  si Claude Desktop / Antigravity lo consumen bien por HTTP POST o requieren SSE.

### 2. GUI / registro de nodos
- Reusar el tray nativo Win32 de `mcp-sql-sentinel`, o un `/ui` HTML simple.

### 3. Seguridad
- Token/API-key por nodo (hoy la red es plana).

### 4. Git + Publicación
- `git init` + repo GitHub (igual que node-winsvc). Dockerfile para correr en servidor central.

---

## 🚀 Cómo continuar (siguiente sesión)

1. `git init` + subir a GitHub.
2. Probar el hub conectado a **2+ nodos** SQL Sentinel reales simultáneos.
3. Conectar Claude Desktop / Antigravity al hub (`http://host:3030/mcp`) y validar end-to-end.
4. (Opcional) GUI de administración de nodos.

---

## 📍 Rutas

- **Este proyecto:** `D:\2026\RUST\HENRY MORENO-DEV\sentinel-hub\`
- **Nodo MCP base:** `D:\2026\RUST\HENRY MORENO-DEV\lab-entorno\mcp-sql-sentinel-mcp\`
- **Proyecto hermano (servicios Node):** `D:\2026\node-winsvc\`
