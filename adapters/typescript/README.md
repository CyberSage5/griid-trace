# trace-ts

TypeScript/JavaScript adapter for griid-trace. Appends to `trace.jsonl` — no viewer dependency.

## Install

```bash
cd adapters/typescript
npm install && npm run build
```

Or link locally:

```bash
npm link
```

## Usage

```typescript
import { init, span, llmCall, rootSpan } from "trace-ts";

init("./trace.jsonl");

await rootSpan("What is 2+2?", { agent: "demo" }, async () => {
  await llmCall("gpt-4", { tokens_in: 10, tokens_out: 5 }, async () => {
    // your LLM call
  });
});
```

## Environment

- `TRACE_PATH` — default `./trace.jsonl`
- `TRACE_ID` — optional fixed trace ID
- `PARENT_SPAN_ID` — managed automatically for nesting
