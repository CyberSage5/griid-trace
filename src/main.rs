mod cli;

use crate::cli::{Cli, Commands, ExportCommands};
use griid_trace::{
    export::export_html,
    trace::TraceEvent,
    tui,
    watcher::{read_trace_file, TraceWatcher},
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Tui { file, watch } => {
            let trace_path = resolve_trace_path(file, watch)?;
            run_tui(trace_path, watch).await?;
        }
        Commands::Export { export_cmd } => {
            match export_cmd {
                ExportCommands::Html { input, output } => {
                    export_to_html(input, output).await?;
                }
            }
        }
        Commands::Validate { file } => {
            validate_trace(file).await?;
        }
        Commands::Clean => {
            clean_index()?;
        }
    }

    Ok(())
}

fn resolve_trace_path(
    file: Option<PathBuf>,
    watch: bool,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = file {
        if path.exists() || watch {
            ensure_trace_file(&path, watch)?;
            return Ok(path);
        }
        return Err(format!("File not found: {}", path.display()).into());
    }

    if let Ok(trace_path) = env::var("TRACE_PATH") {
        let path = PathBuf::from(trace_path);
        if path.exists() || watch {
            ensure_trace_file(&path, watch)?;
            return Ok(path);
        }
    }

    let defaults = [
        PathBuf::from("./trace.jsonl"),
        PathBuf::from("./.trace/trace.jsonl"),
    ];

    for path in defaults {
        if path.exists() {
            return Ok(path);
        }
    }

    if watch {
        let path = PathBuf::from("./trace.jsonl");
        ensure_trace_file(&path, true)?;
        return Ok(path);
    }

    Err("No trace file found. Specify a path, set TRACE_PATH, or use --watch to create one.".into())
}

fn ensure_trace_file(path: &Path, create: bool) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        return Ok(());
    }
    if !create {
        return Err(format!("File not found: {}", path.display()).into());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::File::create(path)?;
    Ok(())
}

async fn run_tui(trace_path: PathBuf, watch: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Loading trace from: {}", trace_path.display());

    let trace = if trace_path.exists() && trace_path.metadata()?.len() > 0 {
        TraceEvent::stream_parse(&trace_path)?
    } else {
        griid_trace::Trace::new(vec![])
    };
    eprintln!("Loaded {} events{}", trace.events.len(), if watch { " (watching)" } else { "" });

    let watch_rx = if watch {
        let (tx, rx) = mpsc::channel();
        let path = trace_path.clone();
        tokio::spawn(async move {
            match TraceWatcher::new(&path).await {
                Ok(mut watcher) => {
                    while let Some(event) = watcher.recv().await {
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        });
        Some(rx)
    } else {
        None
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = if trace.events.len() > 10000 {
        tui::App::with_events_paged(trace.events, 1000)
    } else {
        tui::App::with_events(trace.events)
    };
    app.watch_mode = watch;
    app.trace_path = Some(trace_path);

    let res = app.run_with_watch(&mut terminal, watch_rx);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

async fn export_to_html(input: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading trace from: {}", input.display());

    let trace = TraceEvent::stream_parse(&input)?;

    println!("Generating HTML report...");
    export_html(&trace, &output)?;

    println!("Exported to: {}", output.display());
    Ok(())
}

async fn validate_trace(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating: {}", file.display());

    let events = read_trace_file(&file).await?;
    let mut errors = Vec::new();

    for (i, event) in events.iter().enumerate() {
        if let Err(e) = event.validate() {
            errors.push(format!("Line {}: {}", i + 1, e));
        }
    }

    if errors.is_empty() {
        println!("✓ Valid trace.jsonl file ({} events)", events.len());
        Ok(())
    } else {
        eprintln!("✗ Validation failed:");
        for error in errors {
            eprintln!("  {}", error);
        }
        Err("Validation failed".into())
    }
}

fn clean_index() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = PathBuf::from("./.trace/index");
    if index_path.exists() {
        std::fs::remove_dir_all(&index_path)?;
        println!("Cleared index at: {}", index_path.display());
    } else {
        println!("No index found at: {}", index_path.display());
    }
    Ok(())
}
