# Contributing to griid-trace

Thank you for your interest in contributing to griid-trace! We welcome contributions from the community.

## Code of Conduct

Be respectful, inclusive, and constructive. We're building tools for everyone.

## Getting Started

### Prerequisites
- Rust 1.80+
- Node.js 18+ (for Desktop development)
- Python 3.8+ (for Python adapter development)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/griid-trace/griid-trace.git
cd griid-trace

# Build the TUI
cargo build

# Run tests
cargo test

# Run with sample trace
cargo run -- tui examples/sample-trace.jsonl
```

### Desktop Development

```bash
# Install frontend dependencies
npm install

# Development mode
npm run tauri dev

# Build
npm run tauri build
```

## Development Guidelines

### Local-First Principles

All contributions must adhere to the Local-First Laws:

1. **No Network by Default**: Zero outbound connections
2. **No Account, No Key**: Works without authentication
3. **File > API**: trace.jsonl is primary interface
4. **Zero Telemetry**: No data collection
5. **Single Binary**: Self-contained distribution

### Code Style

- Rust: Follow `cargo fmt` and `cargo clippy`
- Python: Follow PEP 8
- TypeScript/React: Follow ESLint and Prettier

### Testing

- Add tests for new features
- Ensure all tests pass before submitting PR
- Test on multiple platforms when possible

## Areas for Contribution

### High Priority
- Performance optimization (large traces)
- Additional adapters (TypeScript, Go, etc.)
- Framework integrations (CrewAI, LangChain, AutoGen)
- Documentation improvements

### Medium Priority
- Desktop app features (replay, diff, charts)
- TUI enhancements (additional visualizations)
- Example integrations
- Bug fixes

### Low Priority
- Additional themes
- Keyboard customization
- Plugin system

## Submitting Changes

1. Fork the repository
2. Create a branch for your feature
3. Make your changes
4. Add tests
5. Ensure all tests pass
6. Submit a pull request

## Pull Request Process

1. Describe your changes clearly
2. Link related issues
3. Ensure CI passes
4. Request review from maintainers

## Code of Conduct

Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before participating.

## Security

Report vulnerabilities privately per [SECURITY.md](SECURITY.md). See [AUDIT.md](AUDIT.md) for audit information.

## License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.

## Questions?

Open an issue or join our community discussions.
