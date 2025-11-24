# Add Graph Functions to GUI - Proposal

## Why

The Graph Relationships page in the dashboard currently only implements basic visualization functionality (list nodes, list edges, and discover edges for entire collection). However, the backend API and the `useGraph` hook already provide comprehensive graph operations that are not exposed in the GUI, limiting users' ability to fully interact with and manage graph relationships. Users need a complete interface to create and delete edges manually, explore node neighborhoods, find related nodes, discover paths between nodes, and manage edge discovery operations at a granular level. Without these features, users must rely on API calls or external tools to perform essential graph operations, creating a poor user experience and reducing the dashboard's value as a complete management interface.

## What Changes

This task extends the Graph Relationships page (`dashboard/src/pages/GraphPage.tsx`) to expose all available graph functions from the `useGraph` hook:

1. **Edge Management**:
   - Add UI to create edges manually between nodes (with relationship type and weight selection)
   - Add UI to delete edges (with confirmation dialog)
   - Display edge details in a modal or side panel

2. **Node Exploration**:
   - Add "View Neighbors" functionality when clicking on a node (shows all connected nodes)
   - Add "Find Related Nodes" feature with configurable max hops and relationship type filter
   - Display related nodes in a list or highlight them in the graph

3. **Path Finding**:
   - Add "Find Path" feature to discover shortest path between two selected nodes
   - Display path visually in the graph (highlight nodes and edges in the path)
   - Show path details (distance, nodes involved)

4. **Advanced Discovery**:
   - Add "Discover Edges for Node" feature (discover edges for a specific node instead of entire collection)
   - Add discovery status display (show progress, total nodes, edges created)
   - Add discovery configuration modal (similarity threshold, max per node)

5. **UI Enhancements**:
   - Add context menu on nodes (right-click to access neighbors, related, discover edges, etc.)
   - Add edge selection and details panel
   - Add path visualization controls
   - Improve node details panel with action buttons

6. **Integration**:
   - All new features MUST use existing `useGraph` hook functions
   - Maintain existing cache mechanism
   - Update graph visualization after edge creation/deletion
   - Refresh graph data after discovery operations

## Impact

- **Affected specs**: 
  - `specs/dashboard/spec.md` - Add graph GUI functionality requirements
- **Affected code**: 
  - **MODIFIED**: `dashboard/src/pages/GraphPage.tsx` - Add new UI components and functionality
  - **NEW**: `dashboard/src/components/graph/` - New graph-specific components (EdgeCreateModal, NodeDetailsPanel, PathFinder, etc.) if needed
- **Breaking change**: NO (additive changes only)
- **User benefit**: 
  - Complete graph management interface without needing external tools
  - Better exploration and understanding of graph relationships
  - More efficient graph manipulation and discovery
  - Improved user experience with comprehensive graph operations
