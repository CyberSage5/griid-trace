use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "trace")]
#[command(about = "Chrome DevTools for AI Agent Runs - Local-first observability", long_about = None)]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch interactive TUI viewer
    Tui {
        /// Trace file to watch (default: ./trace.jsonl or TRACE_PATH env var)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        
        /// Watch file for changes (default: true)
        #[arg(short, long, default_value_t = true)]
        watch: bool,
    },
    
    /// Export trace to self-contained HTML
    Export {
        #[command(subcommand)]
        export_cmd: ExportCommands,
    },
    
    /// Validate trace.jsonl format compliance
    Validate {
        /// Trace file to validate
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    
    /// Clear index and cache
    Clean,
}

#[derive(Subcommand, Debug)]
pub enum ExportCommands {
    /// Export to self-contained HTML
    Html {
        /// Input trace file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// Output HTML file
        #[arg(short, long, value_name = "OUTPUT")]
        output: PathBuf,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
