# Proposal: remove-cluster-tools-from-mcp

## Why

In the last update, several unnecessary cluster management tools were added to the MCP interface. These tools pollute the MCP API and are not appropriate for end-user operations:

1. **MCP Purpose Mismatch**: MCP (Model Context Protocol) is designed for end-user operations like search, insert, and data retrieval. Cluster management is an administrative operation that should not be exposed via MCP.

2. **API Pollution**: Having 6 cluster management tools in MCP increases the tool count unnecessarily, making it harder for AI assistants to find relevant tools.

3. **Security Concerns**: Cluster management operations (add/remove nodes, rebalance) are administrative tasks that should only be available via REST API with proper authentication, not through MCP.

4. **User Confusion**: End users using MCP don't need cluster management tools - they need data operations (search, insert, etc.).

**Current State**:
- 6 cluster tools in MCP: `cluster_list_nodes`, `cluster_get_shard_distribution`, `cluster_rebalance`, `cluster_add_node`, `cluster_remove_node`, `cluster_get_node_info`
- These tools are available via MCP but should only be in REST API
- Total MCP tools: 32 (should be reduced to 26)

**Problem Scenarios**:
- AI assistant sees cluster tools when user just wants to search
- Cluster operations accidentally called via MCP instead of REST API
- MCP interface becomes cluttered with administrative tools
- Security risk: cluster management via MCP without proper admin authentication

## What Changes

### 1. Remove Cluster Tools from MCP

Remove the following 6 tools from MCP interface:
- `cluster_list_nodes`
- `cluster_get_shard_distribution`
- `cluster_rebalance`
- `cluster_add_node`
- `cluster_remove_node`
- `cluster_get_node_info`

### 2. Keep Cluster Tools in REST API

These operations remain available via REST API:
- `GET /api/v1/cluster/nodes` - List nodes
- `GET /api/v1/cluster/nodes/:node_id` - Get node info
- `GET /api/v1/cluster/shard-distribution` - Get shard distribution
- `POST /api/v1/cluster/rebalance` - Rebalance shards
- `POST /api/v1/cluster/nodes` - Add node
- `DELETE /api/v1/cluster/nodes/:node_id` - Remove node

### 3. Update MCP Tools List

- Remove cluster tool definitions from `src/server/mcp_tools.rs`
- Remove cluster tool handlers from `src/server/mcp_handlers.rs`
- Update tool count in discovery service (from 32 to 26)
- Update documentation to reflect reduced tool count

### 4. Update Documentation

- Update README.md to remove cluster tools from MCP tools list
- Update MCP documentation to clarify that cluster operations are REST-only
- Add note that cluster management should use REST API with proper authentication

## Impact

- **Affected code**:
  - Modified `src/server/mcp_tools.rs` - Remove 6 cluster tool definitions
  - Modified `src/server/mcp_handlers.rs` - Remove 6 cluster tool handlers
  - Modified `src/umicp/discovery.rs` - Update tool count (32 → 26)
  - Modified `README.md` - Update MCP tools documentation
  - Modified `docs/` - Update MCP documentation
- **Breaking change**: YES - MCP tools removed (but REST API unchanged)
- **User benefit**: Cleaner MCP interface focused on data operations, not administrative tasks
