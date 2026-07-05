import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { init, span } from "./index.js";

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "trace-ts-"));
const traceFile = path.join(tmp, "trace.jsonl");

init(traceFile);

await span("llm_call", { model: "test" }, async () => {
  return "ok";
});

const lines = fs.readFileSync(traceFile, "utf-8").trim().split("\n");
assert.equal(lines.length, 2);

const start = JSON.parse(lines[0]);
const end = JSON.parse(lines[1]);

assert.equal(start.status, "in_progress");
assert.equal(end.status, "ok");
assert.equal(start.name, "llm_call");
assert.ok(typeof end.attrs.latency_ms === "number");

console.log("trace-ts: all tests passed");
