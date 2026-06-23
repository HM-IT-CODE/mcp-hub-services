pub mod router;
pub use router::{
    HubContext,
    handle_mcp, list_nodes, add_node, get_node, remove_node, set_node_active, health_check,
};
