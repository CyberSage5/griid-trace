import { TraceEvent } from '../types';

export interface SpanNode {
  event: TraceEvent;
  children: SpanNode[];
  depth: number;
  startMs: number;
  durationMs: number;
}

export function parseTraceJsonl(content: string): TraceEvent[] {
  const events: TraceEvent[] = [];
  for (const line of content.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    events.push(JSON.parse(trimmed) as TraceEvent);
  }
  return events;
}

export function buildSpanTree(events: TraceEvent[]): SpanNode[] {
  const byId = new Map<string, SpanNode>();
  const roots: SpanNode[] = [];

  for (const event of events) {
    byId.set(event.span_id, {
      event,
      children: [],
      depth: 0,
      startMs: new Date(event.ts).getTime(),
      durationMs: event.attrs.latency_ms ?? 100,
    });
  }

  for (const event of events) {
    const node = byId.get(event.span_id)!;
    if (event.parent_id && byId.has(event.parent_id)) {
      byId.get(event.parent_id)!.children.push(node);
    } else if (!event.parent_id) {
      roots.push(node);
    }
  }

  const assignDepth = (node: SpanNode, depth: number) => {
    node.depth = depth;
    node.children.forEach((c) => assignDepth(c, depth + 1));
  };
  roots.forEach((r) => assignDepth(r, 0));

  return roots;
}

export function flattenTree(nodes: SpanNode[]): SpanNode[] {
  const result: SpanNode[] = [];
  const walk = (node: SpanNode) => {
    result.push(node);
    node.children.forEach(walk);
  };
  nodes.forEach(walk);
  return result;
}

export function computeTraceBounds(events: TraceEvent[]) {
  if (events.length === 0) return { startMs: 0, endMs: 0, totalMs: 1 };
  const times = events.map((e) => new Date(e.ts).getTime());
  const startMs = Math.min(...times);
  const endMs = Math.max(
    ...events.map((e) => new Date(e.ts).getTime() + (e.attrs.latency_ms ?? 0))
  );
  return { startMs, endMs, totalMs: Math.max(endMs - startMs, 1) };
}

export function totalCost(events: TraceEvent[]): number {
  return events.reduce((sum, e) => sum + (e.attrs.cost_usd ?? 0), 0);
}

export function totalTokens(events: TraceEvent[]): number {
  return events.reduce(
    (sum, e) => sum + (e.attrs.tokens_in ?? 0) + (e.attrs.tokens_out ?? 0),
    0
  );
}

export interface DiffResult {
  added: TraceEvent[];
  removed: TraceEvent[];
  changed: { before: TraceEvent; after: TraceEvent }[];
  costDelta: number;
  latencyDelta: number;
}

export function diffTraces(a: TraceEvent[], b: TraceEvent[]): DiffResult {
  const mapA = new Map(a.map((e) => [`${e.name}:${e.span_id}`, e]));
  const mapB = new Map(b.map((e) => [`${e.name}:${e.span_id}`, e]));

  const added: TraceEvent[] = [];
  const removed: TraceEvent[] = [];
  const changed: { before: TraceEvent; after: TraceEvent }[] = [];

  for (const [key, event] of mapB) {
    if (!mapA.has(key)) added.push(event);
  }
  for (const [key, event] of mapA) {
    if (!mapB.has(key)) removed.push(event);
    else {
      const after = mapB.get(key)!;
      if (JSON.stringify(event.attrs) !== JSON.stringify(after.attrs)) {
        changed.push({ before: event, after });
      }
    }
  }

  const latency = (events: TraceEvent[]) =>
    events.reduce((s, e) => s + (e.attrs.latency_ms ?? 0), 0);

  return {
    added,
    removed,
    changed,
    costDelta: totalCost(b) - totalCost(a),
    latencyDelta: latency(b) - latency(a),
  };
}
