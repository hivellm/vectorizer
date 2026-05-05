/**
 * Graph Relationships page — console-themed restyle.
 *
 * Visual restyle only: the imperative vis-network mount (ref + ctor +
 * DataSet + event wiring), the cache logic, the relationship/edge
 * algorithms, and every API call site are preserved verbatim from the
 * pre-redesign version. The redesign brief has no dedicated mockup for
 * the Graph page, so this applies the established Phase 3 recipe:
 *   - `.page` + `.page-head` shell
 *   - console `Card` / `CardHead` / `CardBody`
 *   - `.field` + `<input className="input">` form controls
 *   - `.btn` / `.btn primary` for actions
 *   - `Pill` / `StatusPill` / `Icons.*` from console primitives
 *   - vis-network container is wrapped in a `Card` so it inherits the
 *     dark theme; the network ctor + ref are otherwise untouched
 *
 * Triggered modals (EdgeCreate, EdgeDetails, PathFinder, DiscoveryConfig)
 * still use the legacy `@/components/ui/*` Modal/Button/Input/Select
 * primitives. Their wiring (props, callbacks, state) is preserved here;
 * the modals themselves will be migrated in a follow-up sweep.
 *   // TODO(graph-modals): migrate modal internals to console primitives
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { Network, Options } from 'vis-network';
import { useGraph, GraphNode, GraphEdge, NeighborInfo, RelatedNodeInfo } from '@/hooks/useGraph';
import { useCollections } from '@/hooks/useCollections';
import { useToastContext } from '@/providers/ToastProvider';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
} from '@/components/console';
import EdgeCreateModal from '@/components/modals/EdgeCreateModal';
import EdgeDetailsModal from '@/components/modals/EdgeDetailsModal';
import PathFinderModal from '@/components/modals/PathFinderModal';
import DiscoveryConfigModal from '@/components/modals/DiscoveryConfigModal';

// Import vis-network CSS
import 'vis-network/styles/vis-network.css';

// Cache for graph data (in-memory cache with 5 minute TTL)
interface CachedGraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
  timestamp: number;
}

const GRAPH_CACHE_TTL = 5 * 60 * 1000; // 5 minutes in milliseconds
const graphCache = new Map<string, CachedGraphData>();

function GraphPage() {
  const {
    listNodes,
    listEdges,
    discoverEdges,
    discoverEdgesForNode,
    createEdge,
    deleteEdge,
    getNeighbors,
    findRelated,
    findPath,
    enableGraph,
    getGraphStatus,
  } = useGraph();
  const { listCollections } = useCollections();
  const toast = useToastContext();

  const [collections, setCollections] = useState<any[]>([]);
  const [selectedCollection, setSelectedCollection] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [rendering, setRendering] = useState(false);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [relationshipFilter, setRelationshipFilter] = useState<string>('all');
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<GraphEdge | null>(null);
  const [graphEnabled, setGraphEnabled] = useState<boolean | null>(null);
  const [enablingGraph, setEnablingGraph] = useState(false);

  // Modal states
  const [showEdgeCreateModal, setShowEdgeCreateModal] = useState(false);
  const [showEdgeDetailsModal, setShowEdgeDetailsModal] = useState(false);
  const [showPathFinderModal, setShowPathFinderModal] = useState(false);
  const [showDiscoveryModal, setShowDiscoveryModal] = useState(false);
  const [discoveryNodeId, setDiscoveryNodeId] = useState<string | undefined>();

  // Neighbors and related nodes
  const [neighbors, setNeighbors] = useState<NeighborInfo[]>([]);
  const [relatedNodes, setRelatedNodes] = useState<RelatedNodeInfo[]>([]);
  // pathNodes is used to store path results for later visualization
  const [, setPathNodes] = useState<GraphNode[]>([]);

  const networkRef = useRef<HTMLDivElement>(null);
  const networkInstanceRef = useRef<Network | null>(null);

  // Load collections on mount
  useEffect(() => {
    const fetchCollections = async () => {
      try {
        const data = await listCollections();
        setCollections(Array.isArray(data) ? data : []);
        if (data.length > 0 && !selectedCollection) {
          setSelectedCollection(data[0].name);
        }
      } catch (error) {
        toast.error('Failed to load collections');
      }
    };
    fetchCollections();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount

  // Load graph data when collection changes (with cache)
  useEffect(() => {
    if (!selectedCollection) return;

    let cancelled = false;

    const fetchGraphData = async (forceRefresh = false) => {
      // First check if graph is enabled for this collection
      try {
        const status = await getGraphStatus(selectedCollection);
        if (cancelled) return;

        setGraphEnabled(status.enabled);

        if (!status.enabled) {
          console.log(`[GraphPage] Graph not enabled for collection '${selectedCollection}'`);
          setNodes([]);
          setEdges([]);
          setLoading(false);
          return;
        }
      } catch (error) {
        console.warn('[GraphPage] Could not get graph status, trying to load data anyway:', error);
        // Continue trying to load data - maybe the status endpoint failed but data is available
      }

      // Check cache first
      const cacheKey = selectedCollection;
      const cached = graphCache.get(cacheKey);
      const now = Date.now();

      if (!forceRefresh && cached && (now - cached.timestamp) < GRAPH_CACHE_TTL) {
        console.log(`[GraphPage] Using cached data for collection '${selectedCollection}'`);
        setNodes(cached.nodes);
        setEdges(cached.edges);
        setLoading(false);
        return;
      }

      setLoading(true);
      try {
        console.log(`[GraphPage] Fetching graph data for collection '${selectedCollection}'...`);
        const startTime = Date.now();

        const [nodesResponse, edgesResponse] = await Promise.all([
          listNodes(selectedCollection),
          listEdges(selectedCollection), // No limit - get all edges
        ]);

        if (cancelled) return;

        // Validate responses
        if (!nodesResponse || !Array.isArray(nodesResponse.nodes)) {
          throw new Error('Invalid nodes response from API');
        }

        if (!edgesResponse || !Array.isArray(edgesResponse.edges)) {
          throw new Error('Invalid edges response from API');
        }

        const fetchTime = Date.now() - startTime;
        console.log(
          `[GraphPage] Loaded ${nodesResponse.nodes.length} nodes and ${edgesResponse.edges.length} edges in ${fetchTime}ms`
        );

        // Update cache
        graphCache.set(cacheKey, {
          nodes: nodesResponse.nodes,
          edges: edgesResponse.edges,
          timestamp: now,
        });

        setGraphEnabled(true);
        setNodes(nodesResponse.nodes);
        setEdges(edgesResponse.edges);
      } catch (error) {
        console.error('[GraphPage] Error loading graph data:', error);
        let errorMessage = 'Unknown error';

        if (error instanceof Error) {
          errorMessage = error.message;
          // Check if it's a "Graph not enabled" error
          if (errorMessage.includes('Graph not enabled') || errorMessage.includes('not enabled')) {
            setGraphEnabled(false);
            setNodes([]);
            setEdges([]);
            setLoading(false);
            return; // Don't show toast for this case - we'll show the enable button instead
          } else if (errorMessage.includes('not found')) {
            errorMessage = `Collection "${selectedCollection}" not found or graph data is not available.`;
          }
        }

        toast.error(`Failed to load graph data: ${errorMessage}`);
        setNodes([]);
        setEdges([]);
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };

    fetchGraphData();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedCollection]); // Only depend on selectedCollection

  // Initialize vis-network when nodes/edges change
  useEffect(() => {
    if (!networkRef.current || nodes.length === 0) {
      if (networkInstanceRef.current) {
        networkInstanceRef.current.destroy();
        networkInstanceRef.current = null;
      }
      return;
    }

    // Filter nodes and edges based on search and relationship filter
    const filteredNodes = nodes.filter((node) => {
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        if (
          !node.id.toLowerCase().includes(query) &&
          !node.node_type.toLowerCase().includes(query) &&
          !JSON.stringify(node.metadata).toLowerCase().includes(query)
        ) {
          return false;
        }
      }
      return true;
    });

    const filteredEdges = edges.filter((edge) => {
      if (relationshipFilter !== 'all' && edge.relationship_type !== relationshipFilter) {
        return false;
      }
      // Only include edges where both source and target are in filtered nodes
      return (
        filteredNodes.some((n) => n.id === edge.source) &&
        filteredNodes.some((n) => n.id === edge.target)
      );
    });

    // Convert to vis-network format
    const visNodes = filteredNodes.map((node) => {
      // Get node color based on type
      const getNodeColor = () => {
        switch (node.node_type) {
          case 'document':
            return { background: '#a855f7', border: '#7c3aed', highlight: { background: '#c084fc', border: '#a855f7' } };
          case 'file':
            return { background: '#f59e0b', border: '#d97706', highlight: { background: '#fbbf24', border: '#f59e0b' } };
          case 'chunk':
            return { background: '#06b6d4', border: '#0891b2', highlight: { background: '#22d3ee', border: '#06b6d4' } };
          case 'vector':
            return { background: '#6366f1', border: '#4f46e5', highlight: { background: '#818cf8', border: '#6366f1' } };
          default:
            return { background: '#8b5cf6', border: '#7c3aed', highlight: { background: '#a78bfa', border: '#8b5cf6' } };
        }
      };

      const colors = getNodeColor();
      if (selectedNode === node.id) {
        colors.background = '#3b82f6';
        colors.border = '#2563eb';
        colors.highlight = { background: '#60a5fa', border: '#3b82f6' };
      }

      return {
        id: node.id,
        label: node.id.length > 30 ? `${node.id.substring(0, 30)}...` : node.id,
        title: `${node.id}\nType: ${node.node_type}\nMetadata: ${JSON.stringify(node.metadata, null, 2)}`,
        color: colors,
        shape: 'dot',
        size: selectedNode === node.id ? 25 : 20,
        font: {
          size: selectedNode === node.id ? 14 : 12,
          face: 'Inter, system-ui, sans-serif',
          color: selectedNode === node.id ? '#1e40af' : '#374151',
        },
        borderWidth: selectedNode === node.id ? 3 : 2,
        shadow: selectedNode === node.id,
      };
    });

    const visEdges = filteredEdges.map((edge) => {
      // Get edge color based on relationship type (brighter colors for dark theme)
      const getEdgeColor = () => {
        if (selectedNode === edge.source || selectedNode === edge.target) {
          return '#60a5fa'; // Brighter blue for selected
        }
        switch (edge.relationship_type) {
          case 'SIMILAR_TO':
            return '#34d399'; // Brighter green
          case 'REFERENCES':
            return '#fbbf24'; // Brighter amber
          case 'CONTAINS':
            return '#a78bfa'; // Brighter purple
          case 'DERIVED_FROM':
            return '#22d3ee'; // Brighter cyan
          default:
            return '#94a3b8'; // Brighter gray
        }
      };

      // Calculate width - make edges more visible (minimum 2px, scale with weight)
      const baseWidth = 2;
      const weightMultiplier = Math.max(1, edge.weight || 1);
      const edgeWidth = Math.max(baseWidth, Math.min(5, baseWidth * weightMultiplier));

      return {
        id: edge.id,
        from: edge.source,
        to: edge.target,
        label: edge.relationship_type,
        color: {
          color: getEdgeColor(),
          highlight: '#60a5fa', // Bright blue when highlighted
          hover: '#60a5fa',
          opacity: 0.8, // Make edges more visible
        },
        width: edgeWidth,
        arrows: {
          to: {
            enabled: true,
            scaleFactor: 1.5, // Larger arrows
            type: 'arrow',
          },
        },
        smooth: {
          enabled: true,
          type: 'continuous',
          roundness: 0.5,
        },
        title: `${edge.relationship_type}\nWeight: ${edge.weight}`,
        selectionWidth: edgeWidth + 2, // Thicker when selected
        hoverWidth: edgeWidth + 1, // Thicker on hover
      };
    });

    const data = { nodes: visNodes, edges: visEdges };

    // Neo4j-style options
    const options: Options = {
      nodes: {
        shape: 'dot',
        font: {
          size: 12,
          face: 'Inter, system-ui, sans-serif',
        },
        borderWidth: 2,
        shadow: {
          enabled: true,
          color: 'rgba(0,0,0,0.2)',
          size: 5,
          x: 2,
          y: 2,
        },
      },
      edges: {
        width: 2, // Default width (individual edges can override)
        color: {
          color: '#94a3b8', // Brighter default color for dark theme
          highlight: '#60a5fa', // Bright blue when highlighted
          hover: '#60a5fa',
          opacity: 0.8, // More visible opacity
        },
        smooth: {
          enabled: true,
          type: 'continuous',
          roundness: 0.5,
        },
        arrows: {
          to: {
            enabled: true,
            scaleFactor: 1.5, // Larger arrows
            type: 'arrow',
          },
        },
        font: {
          size: 11, // Slightly larger font
          align: 'middle',
          color: '#e2e8f0', // Light color for dark theme
          strokeWidth: 2, // Text outline for better visibility
          strokeColor: '#1e293b', // Dark outline
        },
        selectionWidth: 4, // Thicker when selected
        hoverWidth: 3, // Thicker on hover
      },
      physics: {
        enabled: true,
        stabilization: {
          enabled: true,
          iterations: 200,
          fit: true,
        },
        barnesHut: {
          gravitationalConstant: -2000,
          centralGravity: 0.3,
          springLength: 200,
          springConstant: 0.04,
          damping: 0.09,
          avoidOverlap: 0.5,
        },
      },
      interaction: {
        hover: true,
        tooltipDelay: 100,
        zoomView: true,
        dragView: true,
        selectConnectedEdges: true,
      },
      layout: {
        improvedLayout: true,
        hierarchical: {
          enabled: false,
        },
      },
    };

    // Create or update network
    if (networkInstanceRef.current) {
      // Show loading when updating network
      setRendering(true);
      networkInstanceRef.current.setData(data);
      networkInstanceRef.current.setOptions(options);

      // Listen for stabilization complete to hide loading
      const handleStabilization = () => {
        console.log('[GraphPage] Graph stabilization complete');
        setRendering(false);
      };

      // Use stabilizationIterationsDone event (correct event name)
      networkInstanceRef.current.once('stabilizationIterationsDone', handleStabilization);

      // Fallback timeout in case event doesn't fire
      const fallbackTimeout = setTimeout(() => {
        console.log('[GraphPage] Fallback: hiding rendering indicator');
        setRendering(false);
      }, 5000);

      networkInstanceRef.current.once('stabilizationIterationsDone', () => {
        clearTimeout(fallbackTimeout);
      });
    } else {
      // Show loading when creating new network
      setRendering(true);

      const network = new Network(networkRef.current, data, options);
      networkInstanceRef.current = network;

      // Handle stabilization progress (optional - can show progress bar)
      network.on('stabilizationProgress', () => {
        // Progress tracking (can be used for progress bar if needed)
      });

      // Handle stabilization complete (layout done)
      const handleStabilization = () => {
        console.log('[GraphPage] Graph stabilization complete');
        setRendering(false);
      };

      network.once('stabilizationIterationsDone', handleStabilization);

      // Fallback: if stabilization doesn't fire, hide loading after a delay
      const fallbackTimeout = setTimeout(() => {
        console.log('[GraphPage] Fallback: hiding rendering indicator');
        setRendering(false);
      }, 5000); // 5 second fallback

      network.once('stabilizationIterationsDone', () => {
        clearTimeout(fallbackTimeout);
      });

      // Handle node selection
      network.on('selectNode', (params) => {
        if (params.nodes.length > 0) {
          setSelectedNode(params.nodes[0] as string);
        } else {
          setSelectedNode(null);
        }
      });

      // Handle edge selection — open the details modal so users can
      // inspect / delete the edge. Only fire when a single edge is
      // clicked without a node also being selected.
      network.on('selectEdge', (params) => {
        if (
          params.edges.length > 0 &&
          (!params.nodes || params.nodes.length === 0)
        ) {
          const edgeId = params.edges[0] as string;
          const edge = edges.find((e) => e.id === edgeId);
          if (edge) {
            setSelectedEdge(edge);
            setShowEdgeDetailsModal(true);
          }
        }
      });

      // Handle click on background
      network.on('click', (params) => {
        if (params.nodes.length === 0) {
          setSelectedNode(null);
        }
      });

      // Handle double click to fit
      network.on('doubleClick', (params) => {
        if (params.nodes.length > 0) {
          network.focus(params.nodes[0] as string, {
            scale: 1.2,
            animation: true,
          });
        }
      });
    }

    return () => {
      if (networkInstanceRef.current) {
        networkInstanceRef.current.destroy();
        networkInstanceRef.current = null;
      }
    };
  }, [nodes, edges, searchQuery, relationshipFilter, selectedNode]);

  // Get unique relationship types
  const relationshipTypes = Array.from(new Set(edges.map((e) => e.relationship_type))).sort();

  // Handle refresh (force cache refresh)
  const handleRefresh = useCallback(() => {
    if (!selectedCollection) return;

    // Clear cache for this collection
    graphCache.delete(selectedCollection);

    setLoading(true);
    Promise.all([listNodes(selectedCollection), listEdges(selectedCollection)])
      .then(([nodesResponse, edgesResponse]) => {
        // Validate responses
        if (!nodesResponse || !Array.isArray(nodesResponse.nodes)) {
          throw new Error('Invalid nodes response from API');
        }

        if (!edgesResponse || !Array.isArray(edgesResponse.edges)) {
          throw new Error('Invalid edges response from API');
        }

        // Update cache
        graphCache.set(selectedCollection, {
          nodes: nodesResponse.nodes,
          edges: edgesResponse.edges,
          timestamp: Date.now(),
        });

        console.log(
          `[GraphPage] Refreshed: ${nodesResponse.nodes.length} nodes and ${edgesResponse.edges.length} edges`
        );

        setNodes(nodesResponse.nodes);
        setEdges(edgesResponse.edges);
        setLoading(false);
      })
      .catch((error) => {
        console.error('[GraphPage] Error refreshing graph:', error);
        toast.error(`Failed to refresh graph: ${error instanceof Error ? error.message : 'Unknown error'}`);
        setLoading(false);
      });
  }, [selectedCollection, listNodes, listEdges, toast]);

  // Handle create edge
  const handleCreateEdge = useCallback(
    async (source: string, target: string, relationshipType: string, weight: number) => {
      if (!selectedCollection) return;

      await createEdge(selectedCollection, source, target, relationshipType, weight);
      graphCache.delete(selectedCollection);
      await handleRefresh();
    },
    [selectedCollection, createEdge, handleRefresh]
  );

  // Handle delete edge
  const handleDeleteEdge = useCallback(
    async (edgeId: string) => {
      try {
        await deleteEdge(edgeId);
        toast.success('Edge deleted successfully');
        graphCache.delete(selectedCollection);
        await handleRefresh();
      } catch (error) {
        console.error('Error deleting edge:', error);
        toast.error(error instanceof Error ? error.message : 'Failed to delete edge');
      }
    },
    [deleteEdge, selectedCollection, handleRefresh, toast]
  );

  // Handle view neighbors
  const handleViewNeighbors = useCallback(
    async (nodeId: string) => {
      if (!selectedCollection) return;

      try {
        const result = await getNeighbors(selectedCollection, nodeId);
        setNeighbors(result);
        toast.success(`Found ${result.length} neighbors`);
      } catch (error) {
        console.error('Error getting neighbors:', error);
        toast.error(error instanceof Error ? error.message : 'Failed to get neighbors');
      }
    },
    [selectedCollection, getNeighbors, toast]
  );

  // Handle find related
  const handleFindRelated = useCallback(
    async (nodeId: string, maxHops: number = 2) => {
      if (!selectedCollection) return;

      try {
        const result = await findRelated(selectedCollection, nodeId, { max_hops: maxHops });
        setRelatedNodes(result);
        toast.success(`Found ${result.length} related nodes`);
      } catch (error) {
        console.error('Error finding related nodes:', error);
        toast.error(error instanceof Error ? error.message : 'Failed to find related nodes');
      }
    },
    [selectedCollection, findRelated, toast]
  );

  // Handle find path
  const handleFindPath = useCallback(
    async (source: string, target: string) => {
      if (!selectedCollection) return [];

      try {
        const result = await findPath(selectedCollection, source, target);
        if (result.found) {
          setPathNodes(result.path);
          return result.path;
        } else {
          return [];
        }
      } catch (error) {
        console.error('Error finding path:', error);
        toast.error(error instanceof Error ? error.message : 'Failed to find path');
        return [];
      }
    },
    [selectedCollection, findPath, toast]
  );

  // Handle discover edges with config
  const handleDiscoverWithConfig = useCallback(
    async (threshold: number, maxPerNode: number) => {
      if (!selectedCollection) return;

      setLoading(true);
      try {
        let response;
        if (discoveryNodeId) {
          response = await discoverEdgesForNode(selectedCollection, discoveryNodeId, {
            similarity_threshold: threshold,
            max_per_node: maxPerNode,
          });
        } else {
          response = await discoverEdges(selectedCollection, {
            similarity_threshold: threshold,
            max_per_node: maxPerNode,
          });
        }

        toast.success(`Discovery completed: ${response.edges_created} edges created`);
        graphCache.delete(selectedCollection);
        await handleRefresh();
      } catch (error) {
        console.error('Error discovering edges:', error);
        toast.error(error instanceof Error ? error.message : 'Failed to discover edges');
      } finally {
        setLoading(false);
        setDiscoveryNodeId(undefined);
      }
    },
    [selectedCollection, discoveryNodeId, discoverEdges, discoverEdgesForNode, handleRefresh, toast]
  );

  const selectedNodeObj = selectedNode ? nodes.find((n) => n.id === selectedNode) : undefined;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Graph Relationships</h1>
          <p className="page-sub">Visualize relationships between vectors</p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button
            className="btn primary"
            onClick={() => setShowEdgeCreateModal(true)}
            disabled={loading || !selectedCollection || nodes.length < 2}
          >
            <Icons.plus size={13} />
            Create Edge
          </button>
          <button
            className="btn"
            onClick={() => setShowPathFinderModal(true)}
            disabled={loading || !selectedCollection || nodes.length < 2}
          >
            <Icons.search size={13} />
            Find Path
          </button>
          <button
            className="btn"
            onClick={() => {
              setDiscoveryNodeId(undefined);
              setShowDiscoveryModal(true);
            }}
            disabled={loading || !selectedCollection}
          >
            <Icons.sparkles size={13} />
            Discover Edges
          </button>
          <button className="btn" onClick={handleRefresh} disabled={loading || !selectedCollection}>
            <Icons.refresh size={13} />
            Refresh
          </button>
        </div>
      </div>

      <Card>
        <CardHead title="Controls" />
        <CardBody>
          <div
            className="grid"
            style={{
              gridTemplateColumns: '1fr 1fr 1fr auto',
              gap: 10,
              alignItems: 'end',
            }}
          >
            <div className="field">
              <label className="field-label" htmlFor="graph-collection">Collection</label>
              <select
                id="graph-collection"
                className="input"
                value={selectedCollection}
                onChange={(e) => setSelectedCollection(e.target.value)}
              >
                <option value="">Select collection</option>
                {collections.map((col) => (
                  <option key={col.name} value={col.name}>
                    {col.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label className="field-label" htmlFor="graph-rel-type">Relationship type</label>
              <select
                id="graph-rel-type"
                className="input"
                value={relationshipFilter}
                onChange={(e) => setRelationshipFilter(e.target.value)}
              >
                <option value="all">All Types</option>
                {relationshipTypes.map((type) => (
                  <option key={type} value={type}>
                    {type}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label className="field-label" htmlFor="graph-search">Search node</label>
              <input
                id="graph-search"
                className="input"
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by ID or type…"
              />
            </div>
            <div className="row" style={{ gap: 8 }}>
              {networkInstanceRef.current && (
                <>
                  <button
                    className="btn"
                    onClick={() => networkInstanceRef.current?.fit({ animation: true })}
                    title="Fit to Screen"
                  >
                    Fit
                  </button>
                  <button
                    className="btn"
                    onClick={() => {
                      networkInstanceRef.current?.moveTo({
                        position: { x: 0, y: 0 },
                        scale: 1,
                        animation: true,
                      });
                      setSelectedNode(null);
                    }}
                    title="Reset View"
                  >
                    Reset
                  </button>
                </>
              )}
            </div>
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <Card>
        <CardHead
          title="Visualization"
          sub={
            selectedCollection
              ? `${nodes.length} nodes · ${edges.length} edges`
              : undefined
          }
          right={
            (loading || rendering) ? (
              <Pill tone="amber">
                <span className="dot amber" />
                {loading ? 'loading' : 'rendering'}
              </Pill>
            ) : graphEnabled === true && nodes.length > 0 ? (
              <Pill tone="green">
                <span className="dot green" />
                ready
              </Pill>
            ) : undefined
          }
        />
        <CardBody>
          {!selectedCollection ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              Select a collection to view graph
            </div>
          ) : graphEnabled === false ? (
            <div
              className="col"
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 12,
                padding: '32px 16px',
                textAlign: 'center',
              }}
            >
              <Icons.layers size={32} />
              <div>
                <div style={{ color: 'var(--text-1)', fontWeight: 600 }}>Graph Not Enabled</div>
                <div style={{ color: 'var(--text-2)', marginTop: 4 }}>
                  Graph relationships are not enabled for collection &ldquo;{selectedCollection}&rdquo;
                </div>
              </div>
              <button
                className="btn primary"
                onClick={async () => {
                  setEnablingGraph(true);
                  try {
                    await enableGraph(selectedCollection);
                    toast.success(`Graph enabled for collection "${selectedCollection}"`);
                    setGraphEnabled(true);
                    // Trigger reload
                    graphCache.delete(selectedCollection);
                    handleRefresh();
                  } catch (error) {
                    const message = error instanceof Error ? error.message : 'Unknown error';
                    toast.error(`Failed to enable graph: ${message}`);
                  } finally {
                    setEnablingGraph(false);
                  }
                }}
                disabled={enablingGraph}
              >
                <Icons.bolt size={13} />
                {enablingGraph ? 'Enabling…' : 'Enable Graph'}
              </button>
            </div>
          ) : nodes.length === 0 ? (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              <div>No graph data available for this collection</div>
              <div style={{ color: 'var(--text-3, var(--text-2))', fontSize: 12, marginTop: 6 }}>
                Try discovering edges using the &ldquo;Discover Edges&rdquo; button
              </div>
            </div>
          ) : (
            <div
              ref={networkRef}
              style={{
                width: '100%',
                height: '800px',
                backgroundColor: 'var(--surface-2, transparent)',
                border: '1px solid var(--border)',
                borderRadius: 4,
              }}
            />
          )}
        </CardBody>
      </Card>

      {selectedNode && selectedNodeObj && (
        <>
          <div style={{ height: 14 }} />
          <Card>
            <CardHead
              title="Node details"
              right={
                <button className="btn sm" onClick={() => setSelectedNode(null)}>
                  <Icons.x size={11} />
                  Close
                </button>
              }
            />
            <CardBody>
              <div className="col" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div className="row" style={{ gap: 8, flexWrap: 'wrap' }}>
                  <Pill tone="muted">
                    id: <span className="mono">{selectedNodeObj.id}</span>
                  </Pill>
                  <Pill tone="teal">type: {selectedNodeObj.node_type}</Pill>
                </div>
                {Object.keys(selectedNodeObj.metadata).length > 0 && (
                  <div>
                    <div
                      style={{ color: 'var(--text-2)', fontSize: 12, marginBottom: 4 }}
                    >
                      Metadata
                    </div>
                    <pre
                      className="mono"
                      style={{
                        margin: 0,
                        padding: 10,
                        fontSize: 12,
                        background: 'var(--surface-2)',
                        border: '1px solid var(--border)',
                        borderRadius: 4,
                        overflow: 'auto',
                        color: 'var(--text-1)',
                      }}
                    >
                      {JSON.stringify(selectedNodeObj.metadata, null, 2)}
                    </pre>
                  </div>
                )}
                <div>
                  <div style={{ color: 'var(--text-2)', fontSize: 12, marginBottom: 6 }}>
                    Actions
                  </div>
                  <div className="row" style={{ gap: 8, flexWrap: 'wrap' }}>
                    <button className="btn sm" onClick={() => handleViewNeighbors(selectedNodeObj.id)}>
                      <Icons.collections size={11} />
                      View Neighbors
                    </button>
                    <button className="btn sm" onClick={() => handleFindRelated(selectedNodeObj.id)}>
                      <Icons.activity size={11} />
                      Find Related
                    </button>
                    <button
                      className="btn sm"
                      onClick={() => {
                        setDiscoveryNodeId(selectedNodeObj.id);
                        setShowDiscoveryModal(true);
                      }}
                    >
                      <Icons.sparkles size={11} />
                      Discover Edges
                    </button>
                  </div>
                </div>

                {neighbors.length > 0 && (
                  <div>
                    <div style={{ color: 'var(--text-2)', fontSize: 12, marginBottom: 6 }}>
                      Neighbors ({neighbors.length})
                    </div>
                    <div
                      className="col"
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 4,
                        maxHeight: 160,
                        overflow: 'auto',
                      }}
                    >
                      {neighbors.map((neighbor) => (
                        <div
                          key={neighbor.node.id}
                          className="row"
                          style={{
                            gap: 8,
                            alignItems: 'center',
                            padding: 8,
                            background: 'var(--surface-2)',
                            border: '1px solid var(--border)',
                            borderRadius: 4,
                            fontSize: 12,
                          }}
                        >
                          <span className="mono" style={{ color: 'var(--text-1)' }}>
                            {neighbor.node.id.length > 30
                              ? `${neighbor.node.id.substring(0, 30)}…`
                              : neighbor.node.id}
                          </span>
                          <Pill tone="muted">via {neighbor.edge.relationship_type}</Pill>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {relatedNodes.length > 0 && (
                  <div>
                    <div style={{ color: 'var(--text-2)', fontSize: 12, marginBottom: 6 }}>
                      Related nodes ({relatedNodes.length})
                    </div>
                    <div
                      className="col"
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 4,
                        maxHeight: 160,
                        overflow: 'auto',
                      }}
                    >
                      {relatedNodes.map((related) => (
                        <div
                          key={related.node.id}
                          className="row"
                          style={{
                            gap: 8,
                            alignItems: 'center',
                            padding: 8,
                            background: 'var(--surface-2)',
                            border: '1px solid var(--border)',
                            borderRadius: 4,
                            fontSize: 12,
                          }}
                        >
                          <span className="mono" style={{ color: 'var(--text-1)' }}>
                            {related.node.id.length > 30
                              ? `${related.node.id.substring(0, 30)}…`
                              : related.node.id}
                          </span>
                          <Pill tone="muted">distance: {related.distance}</Pill>
                          <Pill tone="muted">weight: {related.weight.toFixed(2)}</Pill>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </CardBody>
          </Card>
        </>
      )}

      {/* Modals — wiring preserved verbatim. Modal internals still use
          legacy `@/components/ui/*` primitives; restyle in a follow-up.
          // TODO(graph-modals): migrate modal internals to console primitives */}
      <EdgeCreateModal
        isOpen={showEdgeCreateModal}
        onClose={() => setShowEdgeCreateModal(false)}
        onCreateEdge={handleCreateEdge}
        nodes={nodes}
        preselectedSource={selectedNode || undefined}
      />

      <EdgeDetailsModal
        isOpen={showEdgeDetailsModal}
        onClose={() => {
          setShowEdgeDetailsModal(false);
          setSelectedEdge(null);
        }}
        edge={selectedEdge}
        onDelete={handleDeleteEdge}
      />

      <PathFinderModal
        isOpen={showPathFinderModal}
        onClose={() => setShowPathFinderModal(false)}
        onFindPath={handleFindPath}
        nodes={nodes}
      />

      <DiscoveryConfigModal
        isOpen={showDiscoveryModal}
        onClose={() => {
          setShowDiscoveryModal(false);
          setDiscoveryNodeId(undefined);
        }}
        onDiscover={handleDiscoverWithConfig}
        nodeId={discoveryNodeId}
      />
    </div>
  );
}

export default GraphPage;
