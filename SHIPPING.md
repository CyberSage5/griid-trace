# Shipping Checklist — griid-trace v1.0.0

Use this before tagging `v1.0.0` and publishing.

## Pre-release (maintainer)

### 1. Generate assets
```bash
python scripts/generate-icons.py
```

### 2. Verify Rust
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --verbose
cargo test --test integration_test
cargo test test_parse_100k_spans_under_200ms --release
cargo build --release
```

### 3. Verify Python adapter
```bash
cd adapters/python && pip install -e ".[dev]" && pytest -v
USE_MOCK=1 python ../../examples/langchain_example.py
```

### 4. Verify TypeScript adapter
```bash
cd adapters/typescript && npm install && npm run build
```

### 5. Verify Desktop
```bash
npm ci
npm run build
npm run tauri build
```

### 6. Smoke test TUI
```bash
cargo run --release -- tui examples/sample-trace.jsonl
cargo run --release -- tui --watch ./trace.jsonl
cargo run --release -- validate examples/sample-trace.jsonl
cargo run --release -- export html examples/sample-trace.jsonl -o report.html
```

## GitHub release

### 7. Configure secrets (one-time)
| Secret | Purpose |
|--------|---------|
| `MINISIGN_SECRET_KEY` | Sign SHA256SUMS on release |

### 8. Tag and push
```bash
git tag -a v1.0.0 -m "griid-trace v1.0.0"
git push origin v1.0.0
```

### 9. Website
Deploy `website/` to griid-trace.xyz.

## Post-release

- Verify checksums: `sha256sum -c SHA256SUMS`
- Test: `cargo install griid-trace --version 1.0.0`

## Demo

```bash
# Terminal 1
trace tui --watch trace.jsonl

# Terminal 2
USE_MOCK=1 python examples/langchain_example.py
```
