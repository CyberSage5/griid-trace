import React, { useMemo } from 'react';
import { TraceEvent } from '../types';
import { buildSpanTree, computeTraceBounds, flattenTree } from '../lib/trace';

interface FlamegraphProps {
  events: TraceEvent[];
  width?: number;
  height?: number;
  zoom?: number;
  onSpanClick?: (event: TraceEvent) => void;
}

interface LayoutNode {
  event: TraceEvent;
  x: number;
  y: number;
  width: number;
  color: string;
  depth: number;
}

export const Flamegraph: React.FC<FlamegraphProps> = ({
  events,
  width = 900,
  height = 500,
  zoom = 1,
  onSpanClick,
}) => {
  const [hovered, setHovered] = React.useState<TraceEvent | null>(null);
  const [selected, setSelected] = React.useState<TraceEvent | null>(null);

  const layout = useMemo(() => {
    if (events.length === 0) return { nodes: [] as LayoutNode[], totalMs: 1 };

    const { startMs, totalMs } = computeTraceBounds(events);
    const roots = buildSpanTree(events);
    const flat = flattenTree(roots);
    const rowHeight = 22;
    const nodes: LayoutNode[] = [];

    const layoutNode = (node: typeof flat[0], xOffset: number) => {
      const duration = node.durationMs || 100;
      const widthPct = (duration / totalMs) * 100 * zoom;
      const xPct = ((node.startMs - startMs) / totalMs) * 100 * zoom + xOffset;

      nodes.push({
        event: node.event,
        x: Math.max(0, xPct),
        y: node.depth * rowHeight + 24,
        width: Math.max(widthPct, 0.8),
        color: colorForStatus(node.event.status),
        depth: node.depth,
      });

      let childX = xPct;
      const childTotal = node.children.reduce((s, c) => s + (c.durationMs || 100), 0) || 1;
      for (const child of node.children) {
        const childW = (child.durationMs / childTotal) * widthPct;
        layoutNode(child, childX);
        childX += childW;
      }
    };

    roots.forEach((r) => layoutNode(r, 0));

    return { nodes, totalMs };
  }, [events, zoom]);

  if (events.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-gray-500">
        Open a trace file to view the flamegraph
      </div>
    );
  }

  const active = hovered ?? selected;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between text-xs text-gray-400">
        <span>Total duration: {(layout.totalMs / 1000).toFixed(2)}s • {events.length} events</span>
        <span>Click a span for details • Scroll to zoom (use slider below)</span>
      </div>

      <div className="overflow-x-auto border border-dark-600 rounded-lg bg-dark-900 p-2">
        <svg width={width} height={height} className="min-w-full">
          {layout.nodes.map((node) => (
            <g key={`${node.event.span_id}-${node.depth}`}>
              <rect
                x={`${node.x}%`}
                y={node.y}
                width={`${node.width}%`}
                height={18}
                fill={node.color}
                stroke={selected?.span_id === node.event.span_id ? '#fff' : '#30363d'}
                strokeWidth={selected?.span_id === node.event.span_id ? 2 : 1}
                rx={2}
                className="cursor-pointer transition-opacity hover:opacity-90"
                onClick={() => {
                  setSelected(node.event);
                  onSpanClick?.(node.event);
                }}
                onMouseEnter={() => setHovered(node.event)}
                onMouseLeave={() => setHovered(null)}
              />
              {node.width > 3 && (
                <text
                  x={`${node.x + node.width / 2}%`}
                  y={node.y + 13}
                  textAnchor="middle"
                  className="text-[10px] fill-white pointer-events-none select-none"
                >
                  {node.event.name}
                </text>
              )}
            </g>
          ))}
        </svg>
      </div>

      {active && (
        <div className="bg-dark-700 border border-dark-600 rounded-lg p-4 text-sm">
          <div className="flex flex-wrap gap-4 mb-2">
            <span><strong className="text-white">{active.name}</strong></span>
            <span className="text-accent-green">{active.status}</span>
            {active.attrs.model && <span>model: {active.attrs.model}</span>}
            {active.attrs.latency_ms != null && (
              <span>latency: {active.attrs.latency_ms}ms</span>
            )}
            {active.attrs.cost_usd != null && (
              <span>cost: ${active.attrs.cost_usd.toFixed(4)}</span>
            )}
          </div>
          <pre className="text-xs text-gray-400 overflow-auto max-h-32">
            {JSON.stringify(active.attrs, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
};

function colorForStatus(status: string): string {
  switch (status) {
    case 'ok': return '#3fb950';
    case 'error': return '#f85149';
    case 'in_progress': return '#d29922';
    default: return '#58a6ff';
  }
}
