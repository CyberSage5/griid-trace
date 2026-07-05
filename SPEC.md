# griid-trace v1.0 — Official Specification

**This document is the Source of Truth.** All development, features, marketing, and decisions must follow it strictly.

---

## Product Overview

**griid-trace** is Chrome DevTools for AI Agent runs — 100% local-first.

**One-line thesis**:  
Every AI agent is a black box. `griid-trace` makes it a glass box using a single local file, with live flamegraphs, timelines, and cost breakdowns. **No cloud. No login. Your data never leaves your machine.**

**Domain**: griid-trace.xyz

---

## Core Philosophy — Local-First Laws (Non-Negotiable)

| Law | Rule | Purpose |
|-----|------|--------|
| **Law #1** | No Network by Default | Zero outbound connections. Works firewalled/air-gapped. |
| **Law #2** | No Account, No Key | Instant `cargo install` → usable. |
| **Law #3** | File > API | `./trace.jsonl` is primary. API is secondary. |
| **Law #4** | Zero Telemetry | No usage stats. Fully auditable. |
| **Law #5** | Single Binary | One executable. No Docker, no heavy runtime. |

**We do NOT compete with cloud tools.** We serve users who want full control and privacy.

---

## What griid-trace IS and IS NOT

**IS**
- Local TUI + optional Desktop viewer
- Open `trace.jsonl` specification
- Rust single binary
- Tool-agnostic observability

**IS NOT**
- A cloud service
- An agent framework
- A heavy SDK with vendor lock-in
- A replacement for general logging

---

## Technology Stack (Fixed)

- **Language**: Rust 1.80+
- **TUI**: ratatui + crossterm
- **Desktop**: Tauri v2 + React + Tailwind
- **Runtime**: Tokio
- **CLI**: clap v4
- **Parsing**: serde + serde_json
- **File watching**: notify
- **Indexing**: sled (sqlite as fallback)
- **Format**: `trace.jsonl` (NDJSON)

---

## trace.jsonl Format v1.0 (Contract)

Append-only NDJSON. One event per line.

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

**Convention keys in `attrs`**:
- `llm_call`: `model`, `tokens_in`, `tokens_out`, `latency_ms`, `cost_usd` 
- `tool_call`: `tool`, `latency_ms` 
- Root spans: `total_latency_ms`, `total_cost_usd`, `input` 

**Key Conventions**
- Root span: `parent_id: null`, include `total_latency_ms`, `total_cost_usd`.
- Required attrs by type: `llm_call`, `tool_call`, etc.

---

## Installation & Quick Start

**TUI (Free)**
```bash
cargo install griid-trace
trace.tui --watch ./trace.jsonl
```

**Desktop App** — Download from griid-trace.xyz (macOS, Windows, Linux, arm64)

**Environment Variable**
`TRACE_PATH` (default: `./.trace/trace.jsonl`)

---

## CLI Commands (V1.0)

- `trace.tui [path]` — Interactive viewer
- `trace export html <input> -o <output.html>` — Self-contained report
- `trace validate <file>` — Check format compliance
- `trace clean` — Clear index

---

## TUI Key Bindings (Final)

### Global / Navigation
| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `?` | Show help / keybinding overlay |
| `Tab` | Cycle focus between panels (Table ↔ Flamegraph ↔ Detail) |
| `Ctrl+W` | Toggle watch/live mode |
| `/` | Search / filter mode |
| `Esc` | Clear search / exit mode |
| `r` | Refresh / force re-index |

### Table View (Main Span List)
| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `Space` | Select span → show details |
| `g` | Go to top |
| `G` | Go to bottom |
| `f` | Filter menu (status, name, etc.) |
| `c` | Collapse/expand current subtree |

### Flamegraph / Tree View
| Key | Action |
|-----|--------|
| `←` / `h` | Zoom out / collapse |
| `→` / `l` | Zoom in / expand focused span |
| `+` / `-` | Adjust time scale |
| `o` | Toggle orientation (horizontal flame / vertical tree) |

### Detail Pane
| Key | Action |
|-----|--------|
| `j` / `k` | Scroll JSON attrs up/down |
| `a` | Toggle raw JSON vs formatted view |
| `e` | Export current span details |

### Actions
| Key | Action |
|-----|--------|
| `e` | Export current view to HTML |
| `v` | Validate current file |
| `d` | Diff mode (V1.1) |
| `Ctrl+S` | Save current filters/view as preset |

**Mouse Support** (when available):
- Click to select
- Scroll wheel for navigation
- Drag to zoom flamegraph

**Color Scheme**:
- Green = ok
- Red = error
- Yellow = in_progress
- Blue highlights for search matches

---

## Adapters

Lightweight packages that append JSONL to `TRACE_PATH`.

**Day 1 Support Required**:
- Python (`trace-py`) — CrewAI, LangChain, LlamaIndex, AutoGen
- Rust (`trace-rs`)
- JavaScript/TypeScript

Adapters only write the file. They have zero dependency on the viewer.

### How Adapters Work

**Core Mechanism**:
1. Read the `TRACE_PATH` environment variable (default: `./.trace/trace.jsonl` or current dir).
2. On span start/end, create a `TraceEvent` JSON object matching the spec.
3. Append it as a single line to the file (atomic append for safety).
4. Optional: Auto-create the file and parent directories.

**User Flow**:
- Install adapter (`pip install trace-py` or `cargo add trace-rs`)
- Call `init()` once at program start
- Wrap code blocks with `span("name", attrs=...)` 
- `griid-trace` (TUI or export) reads the same file independently

This decouples instrumentation from viewing. Multiple processes/agents can safely append to the same file.

---

## Desktop App (Paid — $19/month)

**Lifetime option**: First 100 buyers at $199, then $299.

**Premium Features** (what justifies payment):
- Beautiful zoomable flamegraphs
- Replay mode (step-by-step execution)
- Run diff / comparison
- Advanced analytics & charts
- IDE integration
- PDF + interactive exports
- Automatic updates
- Themes & polish

**TUI remains fully capable** — Desktop is convenience + power user tools.

---

## V1.0 Scope (Strict — No Scope Creep)

**In Scope**:
- TUI with table + basic flamegraph + search
- HTML export
- Validation
- Core adapters (Python + Rust)
- Desktop stub with same core features

**Explicitly Out**:
- Any cloud features
- Multi-user / team server
- LLM evaluation / judging
- Network calls except localhost

---

## Website (griid-trace.xyz)

Must include:
- Hero with strong local-first message
- Clear TUI vs Desktop comparison
- Download section for all platforms (macOS Intel + arm64, Windows x64 + arm64, Linux AppImage/.deb)
- Full documentation
- GitHub link (star & contribute)
- Adapters examples
- Pricing page

---

## Success Definition

- Fast performance (100k spans < 200ms)
- Easy instrumentation (< 5 min for popular frameworks)
- Strong open source adoption + community contributions
- Happy paying Desktop users who love the polish

---

## This is the fence.

Any feature request, marketing claim, or code change must be checked against this document. If it violates the Local-First Laws or V1.0 scope, it is rejected.

We are building the best local observability tool for the coming wave of local AI agents.

**Approved.**  
**Follow strictly.**
