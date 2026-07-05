/**
 * trace-ts: TypeScript adapter for griid-trace
 * Zero dependency on the viewer — appends to trace.jsonl only.
 */

import * as fs from "fs";
import * as path from "path";
import { randomUUID } from "crypto";

export interface TraceEvent {
  ts: string;
  trace_id: string;
  span_id: string;
  parent_id: string | null;
  name: string;
  status: "ok" | "error" | "in_progress";
  attrs: Record<string, unknown>;
}

let tracePath: string | null = null;

export function init(filePath?: string): void {
  tracePath = filePath ?? process.env.TRACE_PATH ?? "./trace.jsonl";
  const dir = path.dirname(path.resolve(tracePath));
  fs.mkdirSync(dir, { recursive: true });
}

function getTracePath(): string {
  if (!tracePath) init();
  return tracePath!;
}

function writeEvent(event: TraceEvent): void {
  const p = getTracePath();
  const line = JSON.stringify(event) + "\n";
  fs.appendFileSync(p, line, { encoding: "utf-8" });
}

function isoNow(): string {
  return new Date().toISOString();
}

export function setTraceId(id: string): void {
  process.env.TRACE_ID = id;
}

export function getTraceId(): string {
  return process.env.TRACE_ID ?? randomUUID();
}

export async function span<T>(
  name: string,
  attrs: Record<string, unknown>,
  fn: () => T | Promise<T>
): Promise<T> {
  const spanId = randomUUID();
  const traceId = getTraceId();
  const parentId = process.env.PARENT_SPAN_ID ?? null;
  const prevParent = process.env.PARENT_SPAN_ID;
  process.env.PARENT_SPAN_ID = spanId;

  const start = Date.now();
  writeEvent({
    ts: isoNow(),
    trace_id: traceId,
    span_id: spanId,
    parent_id: parentId,
    name,
    status: "in_progress",
    attrs,
  });

  try {
    const result = await fn();
    writeEvent({
      ts: isoNow(),
      trace_id: traceId,
      span_id: spanId,
      parent_id: parentId,
      name,
      status: "ok",
      attrs: { ...attrs, latency_ms: Date.now() - start },
    });
    return result;
  } catch (err) {
    writeEvent({
      ts: isoNow(),
      trace_id: traceId,
      span_id: spanId,
      parent_id: parentId,
      name,
      status: "error",
      attrs: {
        ...attrs,
        latency_ms: Date.now() - start,
        error: err instanceof Error ? err.message : String(err),
      },
    });
    throw err;
  } finally {
    if (prevParent) process.env.PARENT_SPAN_ID = prevParent;
    else delete process.env.PARENT_SPAN_ID;
  }
}

export async function llmCall(
  model: string,
  attrs: Record<string, unknown>,
  fn: () => Promise<unknown>
): Promise<unknown> {
  return span("llm_call", { model, ...attrs }, fn);
}

export async function toolCall<T>(
  tool: string,
  attrs: Record<string, unknown>,
  fn: () => T | Promise<T>
): Promise<T> {
  return span("tool_call", { tool, ...attrs }, fn);
}

export async function rootSpan<T>(
  input: string,
  attrs: Record<string, unknown>,
  fn: () => T | Promise<T>
): Promise<T> {
  return span("agent_execution", { input, ...attrs }, fn);
}
