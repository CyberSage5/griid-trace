use crate::trace::{Trace, TraceEvent, TraceError};
use sled::{Db, Tree};
use std::path::Path;
use std::sync::Arc;

/// Index for fast trace lookups and span tree queries
pub struct TraceIndex {
    db: Db,
    events_tree: Tree,
    traces_tree: Tree,
}

impl TraceIndex {
    /// Open or create an index at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TraceError> {
        let db = sled::open(path)?;
        let events_tree = db.open_tree("events")?;
        let traces_tree = db.open_tree("traces")?;
        
        Ok(Self {
            db,
            events_tree,
            traces_tree,
        })
    }
    
    /// Index a single trace event
    pub fn index_event(&self, event: &TraceEvent) -> Result<(), TraceError> {
        let key = format!("{}:{}", event.trace_id, event.span_id);
        let value = serde_json::to_vec(event)?;
        self.events_tree.insert(key, value)?;
        Ok(())
    }
    
    /// Index an entire trace
    pub fn index_trace(&self, trace: &Trace) -> Result<(), TraceError> {
        // Index all events
        for event in &trace.events {
            self.index_event(event)?;
        }
        
        // Store trace metadata
        let metadata = serde_json::json!({
            "trace_id": trace.trace_id,
            "event_count": trace.events.len(),
            "total_cost": trace.total_cost(),
            "total_latency": trace.total_latency(),
        });
        self.traces_tree.insert(&trace.trace_id, metadata.to_string())?;
        
        Ok(())
    }
    
    /// Get all events for a trace
    pub fn get_trace_events(&self, trace_id: &str) -> Result<Vec<TraceEvent>, TraceError> {
        let mut events = Vec::new();
        let prefix = format!("{}:", trace_id);
        
        for item in self.events_tree.scan_prefix(&prefix) {
            let (_, value) = item?;
            let event: TraceEvent = serde_json::from_slice(&value)?;
            events.push(event);
        }
        
        events.sort_by(|a, b| a.ts.cmp(&b.ts));
        Ok(events)
    }
    
    /// Get a specific event by span_id
    pub fn get_event(&self, trace_id: &str, span_id: &str) -> Result<Option<TraceEvent>, TraceError> {
        let key = format!("{}:{}", trace_id, span_id);
        if let Some(value) = self.events_tree.get(&key)? {
            let event: TraceEvent = serde_json::from_slice(&value)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
    
    /// List all indexed traces
    pub fn list_traces(&self) -> Result<Vec<String>, TraceError> {
        let mut traces = Vec::new();
        for item in self.traces_tree.iter() {
            let (key, _) = item?;
            if let Ok(trace_id) = String::from_utf8(key.to_vec()) {
                traces.push(trace_id);
            }
        }
        Ok(traces)
    }
    
    /// Clear all indexed data
    pub fn clear(&self) -> Result<(), TraceError> {
        self.events_tree.clear()?;
        self.traces_tree.clear()?;
        Ok(())
    }
    
    /// Flush to disk
    pub fn flush(&self) -> Result<(), TraceError> {
        self.db.flush()?;
        Ok(())
    }
}

impl Drop for TraceIndex {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// In-memory index for faster operations without disk persistence
#[derive(Debug, Default)]
pub struct MemoryIndex {
    events: Vec<TraceEvent>,
    page_size: usize,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            page_size: 1000, // Load 1000 events at a time
        }
    }
    
    pub fn with_page_size(page_size: usize) -> Self {
        Self {
            events: Vec::new(),
            page_size,
        }
    }
    
    pub fn add_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
    
    pub fn add_events(&mut self, events: Vec<TraceEvent>) {
        self.events.extend(events);
    }
    
    pub fn get_events(&self) -> &[TraceEvent] {
        &self.events
    }
    
    /// Get events with pagination for large traces
    pub fn get_events_paged(&self, page: usize) -> &[TraceEvent] {
        let start = page * self.page_size;
        let end = (page + 1) * self.page_size;
        &self.events[start.min(self.events.len())..end.min(self.events.len())]
    }
    
    pub fn get_trace(&self, trace_id: &str) -> Trace {
        let events: Vec<TraceEvent> = self.events
            .iter()
            .filter(|e| e.trace_id == trace_id)
            .cloned()
            .collect();
        Trace::new(events)
    }
    
    pub fn filter_by_name(&self, name: &str) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.name == name)
            .collect()
    }
    
    pub fn filter_by_status(&self, status: &str) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.status == status)
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    pub fn page_count(&self) -> usize {
        (self.events.len() + self.page_size - 1) / self.page_size
    }
}
