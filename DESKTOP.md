# griid-trace Desktop App

The Desktop app provides advanced features beyond the TUI, including beautiful visualizations, replay mode, run comparison, and analytics.

## Setup

### Prerequisites

- Rust 1.80+
- Node.js 18+
- npm or yarn

### Installation

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Features

### Premium Features (Paid)

- **Advanced Flamegraph**: Interactive, zoomable flamegraph visualization
- **Replay Mode**: Step-by-step execution replay
- **Run Diff**: Compare multiple agent runs side-by-side
- **Analytics Charts**: Cost, latency, and token usage charts
- **IDE Integration**: Direct integration with popular IDEs
- **PDF + Interactive Exports**: Export traces in multiple formats
- **Automatic Updates**: Stay up-to-date automatically
- **Themes & Polish**: Beautiful dark theme with customization

### Architecture

- **Frontend**: React 18 + TypeScript + TailwindCSS
- **Backend**: Rust + Tauri v2
- **Communication**: Tauri commands for file I/O and native operations

## Development

### Project Structure

```
griid-trace/
├── src/                    # React frontend
│   ├── main.tsx           # Entry point
│   ├── App.tsx            # Main app component
│   └── index.css          # Global styles
├── tauri/                 # Tauri backend
│   ├── src/
│   │   └── main.rs       # Rust backend
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Node dependencies
├── vite.config.ts         # Vite configuration
└── tailwind.config.js     # Tailwind configuration
```

### Adding New Features

1. **Frontend**: Add React components in `src/`
2. **Backend**: Add Tauri commands in `tauri/src/main.rs`
3. **Styling**: Use TailwindCSS utility classes

## Building

### Development

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

Builds will be in `src-tauri/target/release/bundle/`

## Local-First Philosophy

The Desktop app follows the same Local-First Laws as the TUI:

1. No outbound network by default
2. No account or API key required
3. File-based input (`trace.jsonl`) > API
4. Zero telemetry
5. Single binary distribution

## License

MIT OR Apache-2.0
