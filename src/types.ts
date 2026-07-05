export interface TraceEvent {
  ts: string;
  trace_id: string;
  span_id: string;
  parent_id: string | null;
  name: string;
  status: string;
  attrs: {
    model?: string;
    tokens_in?: number;
    tokens_out?: number;
    latency_ms?: number;
    cost_usd?: number;
    [key: string]: any;
  };
}

export interface Trace {
  events: TraceEvent[];
  trace_id: string;
}
