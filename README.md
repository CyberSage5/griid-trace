# griid-trace

**Chrome DevTools for AI Agent Runs — Local-First**

Every AI agent is a black box. `griid-trace` makes it a glass box by reading a single local `trace.jsonl` file and showing live flamegraphs, timelines, and cost breakdowns. **No cloud. No login. Your data never leaves your laptop.**

## Features

- **100% Local-First**: Zero outbound network calls. Works air-gapped.
- **Single Binary**: `cargo install` and you're done. No Docker, no databases.
- **Tool-Agnostic**: Works with Claude, CrewAI, LangChain, AutoGen, and any agent framework.
- **Fast**: Parse 100k spans in < 200ms.
- **Free TUI**: Full-featured terminal UI with keyboard navigation.
- **HTML Export**: Generate self-contained reports for sharing.

## Installation

```bash
cargo install griid-trace
```

Or build from source:

```bash
git clone https://github.com/griid-trace/griid-trace.git
cd griid-trace
cargo install --path .
```

## Quick Start

1. **Set up your trace file path** (optional, defaults to `./trace.jsonl`):
   ```bash
   export TRACE_PATH=./trace.jsonl
   ```

2. **Instrument your agent** using one of our adapters:
   ```python
   # Python (trace-py)
   from trace_py import init, span
   init()
   
   with span("llm_call", model="claude-3.5"):
       # Your agent code here
       pass
   ```

3. **Launch the TUI**:
   ```bash
   trace tui --watch ./trace.jsonl
   ```

## CLI Commands

### Interactive TUI
```bash
trace tui [path]              # Launch interactive viewer
trace tui --watch ./trace.jsonl  # Watch file for live updates
```

### Export
```bash
trace export html <input> -o <output.html>  # Generate self-contained HTML report
```

### Validation
```bash
trace validate <file>      # Check trace.jsonl format compliance
```

### Maintenance
```bash
trace clean                # Clear index and cache
```

## TUI Key Bindings

### Global
- `q` / `Ctrl+C` — Quit
- `?` — Show help
- `Tab` — Cycle views (Table ↔ Flamegraph ↔ Detail)
- `/` — Search / filter
- `Esc` — Clear search / exit mode

### Table View
- `↑` / `↓` — Navigate
- `Enter` — View details
- `g` / `G` — Go to top / bottom
- `f` — Filter menu

### Flamegraph
- `←` / `→` — Zoom in/out
- `+` / `-` — Adjust time scale
- `o` — Toggle orientation

## trace.jsonl Format

The trace file is append-only NDJSON. Each line is a JSON event:

```json
{
  "ts": "2026-07-02T15:30:00.123Z",
  "trace_id": "t_abc123",
  "span_id": "s_xyz789",
  "parent_id": null,
  "name": "llm_call",
  "status": "ok",
  "attrs": {
    "model": "claude-3.5",
    "tokens_in": 1200,
    "tokens_out": 400,
    "latency_ms": 2100,
    "cost_usd": 0.028
  }
}
```

**Required fields**: `ts`, `trace_id`, `span_id`, `name`, `status`, `attrs`

## Adapters

Adapters are lightweight libraries that append to `TRACE_PATH`. They have zero dependency on the viewer.

### Python (trace-py)
```bash
pip install trace-py
```

```python
from trace_py import init, span

init()

with span("llm_call", model="claude-3.5"):
    # Your code
    pass
```

### Rust (trace-rs)
```bash
cargo add trace-rs
```

### TypeScript (trace-ts)
```bash
cd adapters/typescript && npm install && npm run build
```

```typescript
import { init, span } from "trace-ts";
init();
await span("llm_call", { model: "gpt-4" }, async () => { /* ... */ });
```

See [examples/langchain_example.py](examples/langchain_example.py) for LangChain integration.

```rust
use trace_rs::{init, span};

init();

span!("llm_call", model = "claude-3.5", {
    // Your code
});
```

## Philosophy

### Local-First Laws

1. **No Network by Default** — Zero outbound HTTP. Works fully firewalled.
2. **No Account, No Key** — `cargo install` → `trace.tui` works instantly.
3. **File > API** — Primary input: `./trace.jsonl`. API optional.
4. **Zero Telemetry** — No phone-home, reproducible builds.
5. **Single Binary** — One executable. No Docker/Postgres.

## Desktop App

A premium desktop app with advanced features (flamegraphs, replay, diff, charts) is available at [griid-trace.xyz](https://griid-trace.xyz).

## Shipping

See [SHIPPING.md](SHIPPING.md) for the v1.0.0 release checklist.

Quick ship:
```bash
python scripts/generate-icons.py
cargo test --release
git tag -a v1.0.0 -m "griid-trace v1.0.0"
git push origin v1.0.0
```

## Contributing

Contributions welcome! See [SPEC.md](SPEC.md) for the full technical specification.

**Releasing?** See [SHIPPING.md](SHIPPING.md) for the v1.0.0 checklist.

## License

MIT OR Apache-2.0

---

**Made with ❤️ for the local AI revolution**
