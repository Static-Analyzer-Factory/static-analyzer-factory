import archData from '../../../../packages/shared/src/architecture/architecture.json';
import type { ArchitectureData, ArchNode, Connection, NodeGroup } from '../../../../packages/shared/src/architecture/types';
import { useState, useRef, useEffect, useCallback, useMemo } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import './ArchitectureDiagram.css';

const data = archData as ArchitectureData;

// ── Helpers ──

/** Hex color to "r, g, b" string for rgba() CSS vars */
function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r}, ${g}, ${b}`;
}

/** Build lookup: nodeId -> layerColor */
function buildNodeLayerMap(): Map<string, string> {
  const m = new Map<string, string>();
  for (const layer of data.layers) {
    for (const node of layer.nodes) {
      m.set(node.id, layer.color);
    }
  }
  return m;
}

const nodeLayerColor = buildNodeLayerMap();

/** Build lookup: nodeId -> node label */
function buildNodeLabelMap(): Map<string, string> {
  const m = new Map<string, string>();
  for (const actor of data.actors) {
    m.set(actor.id, actor.label);
  }
  for (const layer of data.layers) {
    for (const node of layer.nodes) {
      m.set(node.id, node.label);
    }
  }
  return m;
}

const nodeLabelMap = buildNodeLabelMap();

/** Get all node IDs connected to a given node */
function getConnectedNodeIds(nodeId: string): Set<string> {
  const ids = new Set<string>();
  for (const conn of data.connections) {
    if (conn.from === nodeId) ids.add(conn.to);
    if (conn.to === nodeId) ids.add(conn.from);
  }
  return ids;
}

/** Get connections involving a node */
function getNodeConnections(nodeId: string): Array<{ id: string; label: string; direction: 'in' | 'out'; type: string }> {
  const results: Array<{ id: string; label: string; direction: 'in' | 'out'; type: string }> = [];
  for (const conn of data.connections) {
    if (conn.from === nodeId) {
      results.push({ id: conn.to, label: nodeLabelMap.get(conn.to) ?? conn.to, direction: 'out', type: conn.type });
    }
    if (conn.to === nodeId) {
      results.push({ id: conn.from, label: nodeLabelMap.get(conn.from) ?? conn.from, direction: 'in', type: conn.type });
    }
  }
  return results;
}

/** Look up a node definition by ID from any layer */
function findNode(nodeId: string): ArchNode | undefined {
  for (const layer of data.layers) {
    const found = layer.nodes.find((n) => n.id === nodeId);
    if (found) return found;
  }
  return undefined;
}

/** BFS to find all nodes on upstream and downstream paths from a starting node */
function tracePaths(startId: string): Set<string> {
  const onPath = new Set<string>([startId]);

  // Downstream BFS (follow from -> to)
  const downQueue = [startId];
  const downVisited = new Set<string>([startId]);
  while (downQueue.length > 0) {
    const current = downQueue.shift()!;
    for (const conn of data.connections) {
      if (conn.from === current && !downVisited.has(conn.to)) {
        downVisited.add(conn.to);
        downQueue.push(conn.to);
        onPath.add(conn.to);
      }
    }
  }

  // Upstream BFS (follow to -> from)
  const upQueue = [startId];
  const upVisited = new Set<string>([startId]);
  while (upQueue.length > 0) {
    const current = upQueue.shift()!;
    for (const conn of data.connections) {
      if (conn.to === current && !upVisited.has(conn.from)) {
        upVisited.add(conn.from);
        upQueue.push(conn.from);
        onPath.add(conn.from);
      }
    }
  }

  return onPath;
}

/** Get trace-relevant edge set */
function traceEdges(startId: string): Set<string> {
  const edges = new Set<string>();

  const downQueue = [startId];
  const downVisited = new Set<string>([startId]);
  while (downQueue.length > 0) {
    const current = downQueue.shift()!;
    for (const conn of data.connections) {
      if (conn.from === current && !downVisited.has(conn.to)) {
        downVisited.add(conn.to);
        downQueue.push(conn.to);
        edges.add(`${conn.from}->${conn.to}`);
      }
    }
  }

  const upQueue = [startId];
  const upVisited = new Set<string>([startId]);
  while (upQueue.length > 0) {
    const current = upQueue.shift()!;
    for (const conn of data.connections) {
      if (conn.to === current && !upVisited.has(conn.from)) {
        upVisited.add(conn.from);
        upQueue.push(conn.from);
        edges.add(`${conn.from}->${conn.to}`);
      }
    }
  }

  return edges;
}

// ── SVG Icons ──

function RobotIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <rect x="5" y="9" width="14" height="10" rx="2" />
      <circle cx="9" cy="14" r="1.5" />
      <circle cx="15" cy="14" r="1.5" />
      <path d="M12 2v4" />
      <path d="M8 9V7a4 4 0 0 1 8 0v2" />
      <path d="M2 14h3" />
      <path d="M19 14h3" />
    </svg>
  );
}

function UserIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="8" r="4" />
      <path d="M6 21v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2" />
    </svg>
  );
}

function ChevronDown() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

function TraceIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M2 12h4l3-9 4 18 3-9h4" />
    </svg>
  );
}

// ── Sub-components ──

interface ActorBarProps {
  nodeRefCallback: (id: string, el: HTMLDivElement | null) => void;
  hoveredNode: string | null;
  connectedToHovered: Set<string>;
  traceNodeIds: Set<string> | null;
  onHover: (id: string | null) => void;
  onClick: (id: string) => void;
}

function ActorBar({ nodeRefCallback, hoveredNode, connectedToHovered, traceNodeIds, onHover, onClick }: ActorBarProps) {
  return (
    <div className="arch-actor-bar">
      {data.actors.map((actor) => {
        let extraClass = '';
        if (traceNodeIds) {
          if (traceNodeIds.has(actor.id)) {
            extraClass = ' arch-actor--on-path';
          } else {
            extraClass = ' arch-actor--dimmed';
          }
        } else if (hoveredNode) {
          if (hoveredNode === actor.id) {
            extraClass = ' arch-actor--hovered';
          } else if (connectedToHovered.has(actor.id)) {
            extraClass = ' arch-actor--connected';
          } else {
            extraClass = ' arch-actor--dimmed';
          }
        }
        return (
          <div
            key={actor.id}
            className={`arch-actor${extraClass}`}
            ref={(el) => nodeRefCallback(actor.id, el)}
            data-node-id={actor.id}
            onMouseEnter={() => onHover(actor.id)}
            onMouseLeave={() => onHover(null)}
            onClick={() => onClick(actor.id)}
          >
            {actor.icon === 'robot' ? <RobotIcon /> : <UserIcon />}
            <span>{actor.label}</span>
          </div>
        );
      })}
    </div>
  );
}

interface NodeBoxProps {
  node: ArchNode;
  layerColor: string;
  index: number;
  hoveredNode: string | null;
  connectedToHovered: Set<string>;
  selectedNode: string | null;
  traceNodeIds: Set<string> | null;
  onHover: (id: string | null) => void;
  onClick: (id: string) => void;
  nodeRefCallback: (id: string, el: HTMLDivElement | null) => void;
}

function NodeBox({ node, layerColor, index, hoveredNode, connectedToHovered, selectedNode, traceNodeIds, onHover, onClick, nodeRefCallback }: NodeBoxProps) {
  let extraClass = '';
  if (traceNodeIds) {
    if (traceNodeIds.has(node.id)) {
      extraClass = ' arch-node--on-path';
    } else {
      extraClass = ' arch-node--dimmed';
    }
  } else if (hoveredNode) {
    if (hoveredNode === node.id) {
      extraClass = ' arch-node--hovered';
    } else if (connectedToHovered.has(node.id)) {
      extraClass = ' arch-node--connected';
    } else {
      extraClass = ' arch-node--dimmed';
    }
  }
  if (selectedNode === node.id) {
    extraClass += ' arch-node--selected';
  }

  return (
    <motion.div
      className={`arch-node${extraClass}`}
      style={{
        '--layer-color': layerColor,
        '--layer-color-rgb': hexToRgb(layerColor),
      } as React.CSSProperties}
      ref={(el) => nodeRefCallback(node.id, el)}
      data-node-id={node.id}
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ delay: index * 0.03, duration: 0.25, ease: 'easeOut' }}
      onMouseEnter={() => onHover(node.id)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onClick(node.id)}
    >
      <div className="arch-node-label">{node.label}</div>
      <div className="arch-node-desc">{node.desc}</div>
      {node.tags && node.tags.length > 0 && (
        <div className="arch-node-tags">
          {node.tags.map((tag) => (
            <span key={tag} className={`arch-tag arch-tag--${tag}`}>{tag}</span>
          ))}
        </div>
      )}
    </motion.div>
  );
}

interface LayerSectionProps {
  layer: typeof data.layers[number];
  isOpen: boolean;
  onToggle: () => void;
  hoveredNode: string | null;
  connectedToHovered: Set<string>;
  selectedNode: string | null;
  traceNodeIds: Set<string> | null;
  onNodeHover: (id: string | null) => void;
  onNodeClick: (id: string) => void;
  nodeRefCallback: (id: string, el: HTMLDivElement | null) => void;
}

function LayerSection({ layer, isOpen, onToggle, hoveredNode, connectedToHovered, selectedNode, traceNodeIds, onNodeHover, onNodeClick, nodeRefCallback }: LayerSectionProps) {
  const nodeCount = layer.nodes.length;

  const renderNode = (nodeId: string, index: number) => {
    const node = layer.nodes.find((n) => n.id === nodeId);
    if (!node) return null;
    return (
      <NodeBox
        key={node.id}
        node={node}
        layerColor={layer.color}
        index={index}
        hoveredNode={hoveredNode}
        connectedToHovered={connectedToHovered}
        selectedNode={selectedNode}
        traceNodeIds={traceNodeIds}
        onHover={onNodeHover}
        onClick={onNodeClick}
        nodeRefCallback={nodeRefCallback}
      />
    );
  };

  return (
    <div className="arch-layer" style={{ '--layer-color': layer.color } as React.CSSProperties}>
      <button className="arch-layer-header" onClick={onToggle} aria-expanded={isOpen}>
        <span className="arch-layer-label">{layer.id.toUpperCase()}</span>
        <span className="arch-layer-name">{layer.label}</span>
        <span className="arch-layer-count">{nodeCount}</span>
        <span className={`arch-layer-chevron${isOpen ? ' arch-layer-chevron--open' : ''}`}>
          <ChevronDown />
        </span>
      </button>
      <AnimatePresence initial={false}>
        {isOpen && (
          <motion.div
            key="content"
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.25, ease: 'easeInOut' }}
            style={{ overflow: 'hidden' }}
          >
            {layer.groups ? (
              <div className="arch-groups">
                {layer.groups.map((group) => {
                  let nodeIndex = 0;
                  return (
                    <div key={group.id} className="arch-group">
                      {group.hub && (
                        <div
                          className="arch-hub-node"
                          style={{
                            '--layer-color-rgb': hexToRgb(layer.color),
                          } as React.CSSProperties}
                        >
                          {renderNode(group.hub, nodeIndex++)}
                        </div>
                      )}
                      <div className="arch-group-label">{group.label}</div>
                      {group.children.length > 0 && (
                        <div className="arch-group-children">
                          {group.children.map((childId) => renderNode(childId, nodeIndex++))}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            ) : (
              <div className="arch-node-grid">
                {layer.nodes.map((node, i) => (
                  <NodeBox
                    key={node.id}
                    node={node}
                    layerColor={layer.color}
                    index={i}
                    hoveredNode={hoveredNode}
                    connectedToHovered={connectedToHovered}
                    selectedNode={selectedNode}
                    traceNodeIds={traceNodeIds}
                    onHover={onNodeHover}
                    onClick={onNodeClick}
                    nodeRefCallback={nodeRefCallback}
                  />
                ))}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface DetailPanelProps {
  nodeId: string;
  onClose: () => void;
  onTrace: (id: string) => void;
}

function DetailPanel({ nodeId, onClose, onTrace }: DetailPanelProps) {
  // Find the node in layers
  let node: ArchNode | null = null;
  let layerColor = '#3d9b8f';
  for (const layer of data.layers) {
    const found = layer.nodes.find((n) => n.id === nodeId);
    if (found) {
      node = found;
      layerColor = layer.color;
      break;
    }
  }
  if (!node) return null;

  const connections = getNodeConnections(nodeId);

  return (
    <motion.div
      className="arch-detail-overlay"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
    >
      <div className="arch-detail-backdrop" onClick={onClose} />
      <motion.div
        className="arch-detail-panel"
        initial={{ x: 380 }}
        animate={{ x: 0 }}
        exit={{ x: 380 }}
        transition={{ duration: 0.25, ease: 'easeOut' }}
      >
        <div className="arch-detail-header">
          <h3 style={{ color: layerColor }}>{node.label}</h3>
          <button className="arch-detail-close" onClick={onClose}>
            <CloseIcon />
          </button>
        </div>

        <p className="arch-detail-desc">{node.desc}</p>

        <div className="arch-detail-meta">
          {node.crate && (
            <div className="arch-detail-meta-row">
              <span className="arch-detail-meta-label">Crate</span>
              <span className="arch-detail-meta-value">{node.crate}</span>
            </div>
          )}
          {node.path && (
            <div className="arch-detail-meta-row">
              <span className="arch-detail-meta-label">Path</span>
              <span className="arch-detail-meta-value">{node.path}</span>
            </div>
          )}
        </div>

        {node.tags && node.tags.length > 0 && (
          <div className="arch-detail-tags">
            {node.tags.map((tag) => (
              <span key={tag} className={`arch-tag arch-tag--${tag}`}>{tag}</span>
            ))}
          </div>
        )}

        {connections.length > 0 && (
          <>
            <div className="arch-detail-section-label">Connected to</div>
            <div className="arch-detail-connections">
              {connections.map((conn, i) => (
                <div key={`${conn.id}-${conn.direction}-${i}`} className="arch-detail-connection">
                  <span className="arch-detail-conn-arrow">{conn.direction === 'out' ? '\u2192' : '\u2190'}</span>
                  <span className="arch-detail-conn-label">{conn.label}</span>
                  <span className="arch-detail-conn-type">{conn.type}</span>
                </div>
              ))}
            </div>
          </>
        )}

        <div className="arch-detail-actions">
          <button className="arch-trace-btn" onClick={() => onTrace(nodeId)}>
            <TraceIcon />
            Trace data flow
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}

interface ConnectionLine {
  from: string;
  to: string;
  type: Connection['type'];
  key: string;
}

interface NodePos {
  cx: number;
  cy: number;
  top: number;
  bottom: number;
}

interface ConnectionOverlayProps {
  nodePositions: Map<string, NodePos>;
  hoveredNode: string | null;
  connectedToHovered: Set<string>;
  traceNodeIds: Set<string> | null;
  traceEdgeKeys: Set<string> | null;
}

function ConnectionOverlay({ nodePositions, hoveredNode, connectedToHovered, traceNodeIds, traceEdgeKeys }: ConnectionOverlayProps) {
  if (nodePositions.size === 0) return null;

  const lines: ConnectionLine[] = data.connections
    .filter((c) => nodePositions.has(c.from) && nodePositions.has(c.to))
    .map((c) => ({
      from: c.from,
      to: c.to,
      type: c.type,
      key: `${c.from}->${c.to}`,
    }));

  const draw = {
    hidden: { pathLength: 0, opacity: 0 },
    visible: (i: number) => ({
      pathLength: 1,
      opacity: 1,
      transition: {
        pathLength: { delay: 0.5 + i * 0.02, duration: 0.6, ease: 'easeInOut' as const },
        opacity: { delay: 0.5 + i * 0.02, duration: 0.15 },
      },
    }),
  };

  return (
    <motion.svg
      className="arch-connection-svg"
      initial="hidden"
      animate="visible"
    >
      {lines.map((line, i) => {
        const fromPos = nodePositions.get(line.from);
        const toPos = nodePositions.get(line.to);
        if (!fromPos || !toPos) return null;

        // Determine direction
        const goingDown = fromPos.cy < toPos.cy;
        const x1 = fromPos.cx;
        const y1 = goingDown ? fromPos.bottom : fromPos.top;
        const x2 = toPos.cx;
        const y2 = goingDown ? toPos.top : toPos.bottom;
        const midY = (y1 + y2) / 2;

        const d = `M ${x1} ${y1} C ${x1} ${midY}, ${x2} ${midY}, ${x2} ${y2}`;

        // Stroke styling by type
        let stroke = 'rgba(255, 255, 255, 0.12)';
        let strokeWidth = 1.5;
        let dasharray: string | undefined;
        const sourceColor = nodeLayerColor.get(line.from) ?? '#9b9ba0';

        switch (line.type) {
          case 'uses':
            stroke = 'rgba(255, 255, 255, 0.25)';
            strokeWidth = 1.5;
            break;
          case 'data':
            stroke = sourceColor;
            strokeWidth = 1.5;
            break;
          case 'compiles-to':
            stroke = '#9b9ba0';
            strokeWidth = 1.2;
            dasharray = '8 4';
            break;
          case 'deploys':
            stroke = '#9b9ba0';
            strokeWidth = 1;
            dasharray = '2 4';
            break;
          case 'planned':
            stroke = 'rgba(255, 255, 255, 0.1)';
            strokeWidth = 1;
            dasharray = '6 4';
            break;
        }

        let opacity = line.type === 'data' ? 0.25 : 0.2;
        let extraClass = '';

        // Hover highlighting
        if (hoveredNode && !traceNodeIds) {
          const isRelevant = line.from === hoveredNode || line.to === hoveredNode;
          if (isRelevant) {
            extraClass = ' arch-connection--highlighted';
            opacity = 0.8;
          } else {
            extraClass = ' arch-connection--dimmed';
            opacity = 0.03;
          }
        }

        // Trace mode
        if (traceNodeIds && traceEdgeKeys) {
          if (traceEdgeKeys.has(line.key)) {
            extraClass = ' arch-connection--highlighted arch-trace-path';
            opacity = 0.85;
            dasharray = '12 12';
          } else {
            extraClass = ' arch-connection--dimmed';
            opacity = 0.03;
          }
        }

        return (
          <motion.path
            key={line.key}
            d={d}
            stroke={stroke}
            strokeWidth={strokeWidth}
            strokeDasharray={dasharray}
            strokeOpacity={opacity}
            strokeLinecap="round"
            className={extraClass.trim()}
            variants={draw}
            custom={i}
          />
        );
      })}
    </motion.svg>
  );
}

// ── Main Component ──

export default function ArchitectureDiagram() {
  const [collapsedLayers, setCollapsedLayers] = useState<Set<string>>(new Set());
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [traceOrigin, setTraceOrigin] = useState<string | null>(null);
  const [nodePositions, setNodePositions] = useState<Map<string, NodePos>>(new Map());

  const containerRef = useRef<HTMLDivElement>(null);
  const nodeRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  // Connected nodes for hover highlighting
  const connectedToHovered = useMemo(() => {
    if (!hoveredNode) return new Set<string>();
    return getConnectedNodeIds(hoveredNode);
  }, [hoveredNode]);

  // Trace computation
  const traceNodeIds = useMemo(() => {
    if (!traceOrigin) return null;
    return tracePaths(traceOrigin);
  }, [traceOrigin]);

  const traceEdgeKeys = useMemo(() => {
    if (!traceOrigin) return null;
    return traceEdges(traceOrigin);
  }, [traceOrigin]);

  // Node ref callback
  const nodeRefCallback = useCallback((id: string, el: HTMLDivElement | null) => {
    if (el) {
      nodeRefs.current.set(id, el);
    } else {
      nodeRefs.current.delete(id);
    }
  }, []);

  // Compute node positions relative to container
  const updatePositions = useCallback(() => {
    const container = containerRef.current;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();
    const newPositions = new Map<string, NodePos>();

    for (const [id, el] of nodeRefs.current) {
      const rect = el.getBoundingClientRect();
      newPositions.set(id, {
        cx: rect.left + rect.width / 2 - containerRect.left,
        cy: rect.top + rect.height / 2 - containerRect.top,
        top: rect.top - containerRect.top,
        bottom: rect.bottom - containerRect.top,
      });
    }

    setNodePositions(newPositions);
  }, []);

  // Update positions on layout changes
  useEffect(() => {
    // Delay to allow AnimatePresence to settle
    const timer = setTimeout(updatePositions, 300);
    return () => clearTimeout(timer);
  }, [collapsedLayers, updatePositions]);

  // ResizeObserver
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver(() => {
      updatePositions();
    });
    observer.observe(container);

    // Initial position calc
    const timer = setTimeout(updatePositions, 500);

    return () => {
      observer.disconnect();
      clearTimeout(timer);
    };
  }, [updatePositions]);

  const toggleLayer = useCallback((layerId: string) => {
    setCollapsedLayers((prev) => {
      const next = new Set(prev);
      if (next.has(layerId)) {
        next.delete(layerId);
      } else {
        next.add(layerId);
      }
      return next;
    });
  }, []);

  const handleNodeClick = useCallback((nodeId: string) => {
    setSelectedNode(nodeId);
  }, []);

  const handleCloseDetail = useCallback(() => {
    setSelectedNode(null);
  }, []);

  const handleTrace = useCallback((nodeId: string) => {
    setTraceOrigin(nodeId);
    setSelectedNode(null);
  }, []);

  const clearTrace = useCallback(() => {
    setTraceOrigin(null);
  }, []);

  return (
    <section className="arch-diagram">
      <div className="arch-inner" ref={containerRef}>
        <div className="arch-header">
          <motion.h2
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
          >
            System Architecture
          </motion.h2>
          <motion.p
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.5, delay: 0.15 }}
          >
            Click a component to inspect. Hover to see connections.
          </motion.p>
        </div>

        <ActorBar
          nodeRefCallback={nodeRefCallback}
          hoveredNode={hoveredNode}
          connectedToHovered={connectedToHovered}
          traceNodeIds={traceNodeIds}
          onHover={setHoveredNode}
          onClick={handleNodeClick}
        />

        {traceOrigin && (
          <div className="arch-trace-banner">
            <span>Tracing data flow from: {nodeLabelMap.get(traceOrigin) ?? traceOrigin}</span>
            <button className="arch-trace-clear-btn" onClick={clearTrace}>Clear trace</button>
          </div>
        )}

        <ConnectionOverlay
          nodePositions={nodePositions}
          hoveredNode={hoveredNode}
          connectedToHovered={connectedToHovered}
          traceNodeIds={traceNodeIds}
          traceEdgeKeys={traceEdgeKeys}
        />

        <div className="arch-layers">
          {data.layers.map((layer) => (
            <LayerSection
              key={layer.id}
              layer={layer}
              isOpen={!collapsedLayers.has(layer.id)}
              onToggle={() => toggleLayer(layer.id)}
              hoveredNode={hoveredNode}
              connectedToHovered={connectedToHovered}
              selectedNode={selectedNode}
              traceNodeIds={traceNodeIds}
              onNodeHover={setHoveredNode}
              onNodeClick={handleNodeClick}
              nodeRefCallback={nodeRefCallback}
            />
          ))}
        </div>
      </div>

      <AnimatePresence>
        {selectedNode && (
          <DetailPanel
            key={selectedNode}
            nodeId={selectedNode}
            onClose={handleCloseDetail}
            onTrace={handleTrace}
          />
        )}
      </AnimatePresence>
    </section>
  );
}
