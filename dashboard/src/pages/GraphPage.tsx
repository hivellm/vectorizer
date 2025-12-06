/**
 * Graph Relationships page - Visualize vector relationships
 * Uses vis-network for Neo4j-style graph visualization
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { Network, Options } from 'vis-network';
import { useGraph, GraphNode, GraphEdge, NeighborInfo, RelatedNodeInfo } from '@/hooks/useGraph';
import { useCollections } from '@/hooks/useCollections';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Select, SelectOption } from '@/components/ui/Select';
import LoadingSpinner from '@/components/LoadingSpinner';
import { useToastContext } from '@/providers/ToastProvider';
import { RefreshCw01 } from '@untitledui/icons';
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

// SVG icons
const SearchIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
);

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
    // getDiscoveryStatus, // TODO: Use for discovery status indicator
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

        setNodes(nodesResponse.nodes);
        setEdges(edgesResponse.edges);
      } catch (error) {
        console.error('[GraphPage] Error loading graph data:', error);
        let errorMessage = 'Unknown error';

        if (error instanceof Error) {
          errorMessage = error.message;
          // Check if it's a "Graph not enabled" error
          if (errorMessage.includes('Graph not enabled') || errorMessage.includes('not enabled')) {
            errorMessage = `Graph is not enabled for collection "${selectedCollection}". Please enable graph support in the collection configuration.`;
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

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-neutral-900 dark:text-white">Graph Relationships</h1>
          <p className="text-neutral-600 dark:text-neutral-400 mt-1">Visualize relationships between vectors</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="primary"
            size="sm"
            onClick={() => setShowEdgeCreateModal(true)}
            disabled={loading || !selectedCollection || nodes.length < 2}
          >
            Create Edge
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => setShowPathFinderModal(true)}
            disabled={loading || !selectedCollection || nodes.length < 2}
          >
            Find Path
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setDiscoveryNodeId(undefined);
              setShowDiscoveryModal(true);
            }}
            disabled={loading || !selectedCollection}
          >
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Discover Edges
          </Button>
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={loading || !selectedCollection}>
            <RefreshCw01 className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Controls */}
      <Card>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 items-start">
          <div>
            <Select
              label="Collection"
              value={selectedCollection}
              onChange={(value) => setSelectedCollection(value)}
              placeholder="Select collection"
            >
              {collections.map((col) => (
                <SelectOption key={col.name} id={col.name} value={col.name}>
                  {col.name}
                </SelectOption>
              ))}
            </Select>
          </div>
          <div>
            <Select
              label="Relationship Type"
              value={relationshipFilter}
              onChange={(value) => setRelationshipFilter(value)}
            >
              <SelectOption id="all" value="all">
                All Types
              </SelectOption>
              {relationshipTypes.map((type) => (
                <SelectOption key={type} id={type} value={type}>
                  {type}
                </SelectOption>
              ))}
            </Select>
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1">
              Search Node
            </label>
            <div className="relative">
              <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-neutral-400 z-10" />
              <Input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by ID or type..."
                className="pl-10"
              />
            </div>
          </div>
          <div className="flex items-end gap-2 min-h-[42px]">
            {networkInstanceRef.current && (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => networkInstanceRef.current?.fit({ animation: true })}
                  title="Fit to Screen"
                >
                  Fit
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    networkInstanceRef.current?.moveTo({ position: { x: 0, y: 0 }, scale: 1, animation: true });
                    setSelectedNode(null);
                  }}
                  title="Reset View"
                >
                  Reset
                </Button>
              </>
            )}
          </div>
        </div>
      </Card>

      {/* Graph Visualization */}
      <Card>
        <div className="relative">
          {(loading || rendering) && (
            <div className="absolute inset-0 flex flex-col items-center justify-center bg-white/80 dark:bg-neutral-900/80 z-10 backdrop-blur-sm">
              <LoadingSpinner size="lg" />
              <p className="mt-4 text-sm text-neutral-600 dark:text-neutral-400">
                {loading ? 'Loading graph data...' : 'Rendering graph layout...'}
              </p>
            </div>
          )}

          {!selectedCollection ? (
            <div className="text-center py-12">
              <p className="text-neutral-500 dark:text-neutral-400">Select a collection to view graph</p>
            </div>
          ) : nodes.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-neutral-500 dark:text-neutral-400">No graph data available for this collection</p>
            </div>
          ) : (
            <div
              ref={networkRef}
              className="border border-neutral-200 dark:border-neutral-700 rounded-lg"
              style={{ width: '100%', height: '800px', backgroundColor: 'transparent' }}
            />
          )}

          {/* Node details panel */}
          {selectedNode && (
            <div className="mt-4 p-4 bg-neutral-50 dark:bg-neutral-800 rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-semibold text-neutral-900 dark:text-white">Node Details</h3>
                <Button variant="ghost" size="sm" onClick={() => setSelectedNode(null)}>
                  ×
                </Button>
              </div>
              {(() => {
                const node = nodes.find((n) => n.id === selectedNode);
                if (!node) return null;
                return (
                  <div className="space-y-3">
                    <div className="space-y-2 text-sm">
                      <div>
                        <span className="font-medium text-neutral-700 dark:text-neutral-300">ID:</span>{' '}
                        <span className="text-neutral-600 dark:text-neutral-400">{node.id}</span>
                      </div>
                      <div>
                        <span className="font-medium text-neutral-700 dark:text-neutral-300">Type:</span>{' '}
                        <span className="text-neutral-600 dark:text-neutral-400">{node.node_type}</span>
                      </div>
                      {Object.keys(node.metadata).length > 0 && (
                        <div>
                          <span className="font-medium text-neutral-700 dark:text-neutral-300">Metadata:</span>
                          <pre className="mt-1 p-2 bg-white dark:bg-neutral-900 rounded text-xs overflow-auto">
                            {JSON.stringify(node.metadata, null, 2)}
                          </pre>
                        </div>
                      )}
                    </div>

                    {/* Node actions */}
                    <div className="pt-3 border-t border-neutral-200 dark:border-neutral-700">
                      <p className="text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-2">Actions</p>
                      <div className="flex flex-wrap gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleViewNeighbors(node.id)}
                        >
                          View Neighbors
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleFindRelated(node.id)}
                        >
                          Find Related
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setDiscoveryNodeId(node.id);
                            setShowDiscoveryModal(true);
                          }}
                        >
                          Discover Edges
                        </Button>
                      </div>
                    </div>

                    {/* Show neighbors if available */}
                    {neighbors.length > 0 && (
                      <div className="pt-3 border-t border-neutral-200 dark:border-neutral-700">
                        <p className="text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                          Neighbors ({neighbors.length})
                        </p>
                        <div className="space-y-1 max-h-40 overflow-y-auto">
                          {neighbors.map((neighbor) => (
                            <div
                              key={neighbor.node.id}
                              className="text-xs text-neutral-600 dark:text-neutral-400 p-2 bg-white dark:bg-neutral-900 rounded"
                            >
                              <span className="font-mono">{neighbor.node.id.substring(0, 30)}...</span>
                              <span className="ml-2 text-neutral-500">via {neighbor.edge.relationship_type}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Show related nodes if available */}
                    {relatedNodes.length > 0 && (
                      <div className="pt-3 border-t border-neutral-200 dark:border-neutral-700">
                        <p className="text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-2">
                          Related Nodes ({relatedNodes.length})
                        </p>
                        <div className="space-y-1 max-h-40 overflow-y-auto">
                          {relatedNodes.map((related) => (
                            <div
                              key={related.node.id}
                              className="text-xs text-neutral-600 dark:text-neutral-400 p-2 bg-white dark:bg-neutral-900 rounded"
                            >
                              <span className="font-mono">{related.node.id.substring(0, 30)}...</span>
                              <span className="ml-2 text-neutral-500">
                                (distance: {related.distance}, weight: {related.weight.toFixed(2)})
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                );
              })()}
            </div>
          )}
        </div>
      </Card>

      {/* Modals */}
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
