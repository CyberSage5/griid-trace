# trace-py

Python adapter for griid-trace. Lightweight instrumentation library that appends to `trace.jsonl`.

## Installation

```bash
pip install trace-py
```

## Usage

```python
from trace_py import init, span
import os

# Initialize (reads TRACE_PATH env var, defaults to ./trace.jsonl)
init()

# Wrap code blocks with spans
with span("llm_call", model="claude-3.5"):
    # Your agent code here
    response = call_llm("Hello")
    
with span("tool_call", tool="search"):
    results = search_web("query")
```

## API

### `init(path=None)`

Initialize the tracer. Reads `TRACE_PATH` environment variable by default.

### `span(name, **attrs)`

Context manager for creating spans.

- `name`: Span name (e.g., "llm_call", "tool_call")
- `**attrs`: Arbitrary attributes (model, tokens, cost, etc.)

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
