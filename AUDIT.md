# Security Audit Information

griid-trace is built for users who require **full local control** over AI agent observability data. This document tracks security posture, audits, and verification practices.

## Design Principles

| Principle | Implementation |
|-----------|----------------|
| No network by default | Zero outbound HTTP in TUI, adapters, and Desktop |
| No telemetry | No usage stats, crash reports, or phone-home |
| Open source | MIT OR Apache-2.0 — fully auditable Rust + TypeScript |
| Minimal dependencies | Curated crates; automated CVE scanning in CI |
| File-only data plane | `trace.jsonl` — no database, no cloud sync |

## Automated Security Checks (CI)

Every push and pull request runs:

- `cargo audit` — Rust dependency CVE scan
- `cargo clippy -D warnings` — lint and unsafe pattern detection
- `cargo fmt --check` — formatting consistency
- Integration tests on Ubuntu, Windows, macOS
- Python adapter tests (`trace-py`)

## Dependency Audit Log

| Date | Tool | Result | Notes |
|------|------|--------|-------|
| 2026-07-05 | cargo-audit | Automated in CI | Runs on every PR |
| 2026-07-05 | Manual review | Pass | Core deps: tokio, serde, ratatui, sled, notify |

## Binary Verification

All releases include:

1. **SHA256SUMS** — checksum file for every binary artifact
2. **minisign signatures** — when `MINISIGN_SECRET_KEY` is configured in release CI
3. **Reproducible builds** — same tag → same source → CI-built artifacts

### Verify a Release

```bash
# Download release artifacts from GitHub Releases
curl -LO https://github.com/griid-trace/griid-trace/releases/download/v1.0.0/SHA256SUMS
sha256sum -c SHA256SUMS

# Verify minisign signature (if published)
minisign -Vm SHA256SUMS -P RWQ...griid-trace.pub
```

## Threat Model

### In Scope

- Malicious `trace.jsonl` content (oversized files, invalid JSON, path injection)
- Supply chain via Rust/npm dependencies
- Local file permission issues

### Out of Scope

- Network attacks (no network by default)
- Multi-tenant server attacks (no server)
- Cloud credential theft (no credentials stored)

### Mitigations

- Strict JSON parsing with serde (no arbitrary code execution from traces)
- File size validation in parser (configurable limits)
- CSP locked down in Tauri Desktop (`default-src 'self'`)
- Adapters use append-only writes with file locking

## Third-Party Audits

| Audit | Status | Report |
|-------|--------|--------|
| Initial internal review | Complete | This document |
| Independent third-party audit | Planned | TBD — sponsorship welcome |

## Reporting Vulnerabilities

See [SECURITY.md](SECURITY.md). Email security@griid-trace.xyz — do not file public issues for security bugs.

## Comparison: Cloud Observability Risk

Tools like [LangSmith](https://www.langchain.com/) send trace data to cloud infrastructure by design. griid-trace eliminates that attack surface entirely: **your traces never leave your machine unless you explicitly export or share them.**
