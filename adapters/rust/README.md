# trace-rs

Rust adapter for griid-trace. Lightweight instrumentation library that appends to `trace.jsonl`.

## Installation

```bash
cargo add trace-rs
```

## Usage

```rust
use trace_rs::{init, span};

fn main() {
    // Initialize (reads TRACE_PATH env var, defaults to ./trace.jsonl)
    init();
    
    // Wrap code blocks with spans
    span!("llm_call", model = "claude-3.5", {
        // Your agent code here
        let response = call_llm("Hello");
    });
    
    span!("tool_call", tool = "search", {
        let results = search_web("query");
    });
}
```

## API

### `init()`

Initialize the tracer. Reads `TRACE_PATH` environment variable by default.

### `span!(name, key = value, { ... })`

Macro for creating spans.

- `name`: Span name (e.g., "llm_call", "tool_call")
- `key = value`: Arbitrary attributes
- `{ ... }`: Code block to instrument

## Convention Attributes

For LLM calls:
- `model`: Model name
- `tokens_in`: Input token count
- `tokens_out`: Output token count
- `latency_ms`: Latency in milliseconds
- `cost_usd`: Cost in USD

For tool calls:
- `tool`: Tool name
- `latency_ms`: Latency in milliseconds

For root spans:
- `total_latency_ms`: Total trace latency
- `total_cost_usd`: Total trace cost
- `input`: Input prompt/query
