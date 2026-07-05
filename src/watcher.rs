use crate::trace::{TraceEvent, TraceError};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::mpsc;

/// File watcher that tails a trace.jsonl file and emits new events
pub struct TraceWatcher {
    _watcher: RecommendedWatcher,
    event_rx: mpsc::Receiver<TraceEvent>,
}

impl TraceWatcher {
    /// Create a new watcher for the given file
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, TraceError> {
        let path = path.as_ref();
        
        let (event_tx, event_rx) = mpsc::channel(1000);
        let (watcher_tx, mut watcher_rx) = mpsc::channel(100);
        
        // Set up file system watcher
        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = watcher_tx.blocking_send(event);
                }
            },
            notify::Config::default(),
        )?;
        
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        
        // Spawn task to read file and emit events
        let path_clone = path.to_path_buf();
        tokio::spawn(async move {
            let mut last_position = 0u64;
            
            // Initial read of existing file
            if let Ok(file) = File::open(&path_clone).await {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Ok(event) = TraceEvent::from_line(&line) {
                        if event.validate().is_ok() {
                            let _ = event_tx.send(event).await;
                        }
                    }
                    last_position += line.len() as u64 + 1; // +1 for newline
                }
            }
            
            // Watch for modifications and read new lines
            while let Some(_event) = watcher_rx.recv().await {
                if let Ok(metadata) = tokio::fs::metadata(&path_clone).await {
                    let file_size = metadata.len();
                    
                    if file_size > last_position {
                        if let Ok(file) = File::open(&path_clone).await {
                            let _ = file.seek(std::io::SeekFrom::Start(last_position)).await;
                            let reader = BufReader::new(file);
                            let mut lines = reader.lines();
                            
                            while let Ok(Some(line)) = lines.next_line().await {
                                if let Ok(event) = TraceEvent::from_line(&line) {
                                    let _ = event_tx.send(event).await;
                                }
                                last_position += line.len() as u64 + 1;
                            }
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            _watcher: watcher,
            event_rx,
        })
    }
    
    /// Receive the next event from the file
    pub async fn recv(&mut self) -> Option<TraceEvent> {
        self.event_rx.recv().await
    }
}

/// Simple file reader for non-watch mode
pub async fn read_trace_file<P: AsRef<Path>>(path: P) -> Result<Vec<TraceEvent>, TraceError> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut events = Vec::new();
    
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(event) = TraceEvent::from_line(&line) {
            events.push(event);
        }
    }
    
    Ok(events)
}
