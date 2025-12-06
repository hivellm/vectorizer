## 1. Remove Cluster Tools from MCP Tools List

- [x] 1.1 Remove `cluster_list_nodes` tool definition from src/server/mcp_tools.rs
- [x] 1.2 Remove `cluster_get_shard_distribution` tool definition
- [x] 1.3 Remove `cluster_rebalance` tool definition
- [x] 1.4 Remove `cluster_add_node` tool definition
- [x] 1.5 Remove `cluster_remove_node` tool definition
- [x] 1.6 Remove `cluster_get_node_info` tool definition
- [x] 1.7 Remove entire "Cluster Management Tools" section comment

## 2. Remove Cluster Tool Handlers

- [x] 2.1 Remove `cluster_list_nodes` handler case from src/server/mcp_handlers.rs
- [x] 2.2 Remove `cluster_get_shard_distribution` handler case
- [x] 2.3 Remove `cluster_rebalance` handler case
- [x] 2.4 Remove `cluster_add_node` handler case
- [x] 2.5 Remove `cluster_remove_node` handler case
- [x] 2.6 Remove `cluster_get_node_info` handler case
- [x] 2.7 Remove "Cluster Operations" comment section
- [x] 2.8 Remove handler function implementations (all 6 cluster handlers removed)

## 3. Update Discovery Service

- [x] 3.1 Update operations_count in src/umicp/discovery.rs (32 → 26)
- [x] 3.2 Update metadata description if it mentions tool count
- [x] 3.3 Verify discovery service still works correctly

## 4. Update Documentation

- [x] 4.1 Update README.md MCP tools section to remove cluster tools
- [x] 4.2 Update tool count in README (20 → 26, was outdated)
- [x] 4.3 Add note that cluster operations are REST-only
- [x] 4.4 Update docs/users/api/API_REFERENCE.md if it mentions MCP cluster tools
- [x] 4.5 Update any MCP-specific documentation files

## 5. Verify REST API Still Works

- [x] 5.1 Verify cluster REST endpoints still function correctly (no changes made)
- [x] 5.2 Test that cluster operations are only available via REST
- [x] 5.3 Verify authentication/authorization on REST endpoints (unchanged)

## 6. Testing

- [x] 6.1 Test MCP server starts without cluster tools
- [x] 6.2 Test MCP tool list no longer includes cluster tools
- [x] 6.3 Test that calling removed tools via MCP returns appropriate error
- [x] 6.4 Test that REST API cluster endpoints still work
- [x] 6.5 Verify discovery service reports correct tool count

---

## Implementation Summary

### Files Modified:
- `src/server/mcp_tools.rs` - Removed 6 cluster tool definitions
- `src/server/mcp_handlers.rs` - Removed 6 handler cases and 6 handler functions
- `src/umicp/discovery.rs` - Updated operations_count from 32 to 26
- `README.md` - Updated tool counts and added note about REST-only cluster ops

### Cluster tools removed from MCP:
1. `cluster_list_nodes`
2. `cluster_get_shard_distribution`
3. `cluster_rebalance`
4. `cluster_add_node`
5. `cluster_remove_node`
6. `cluster_get_node_info`

### REST endpoints unchanged:
- All cluster REST endpoints remain available at `/cluster/*`
- No authentication/authorization changes needed
