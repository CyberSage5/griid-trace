use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TraceError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid timestamp format")]
    InvalidTimestamp,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// A single trace event in the trace.jsonl format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Timestamp in ISO 8601 format
    pub ts: DateTime<Utc>,
    
    /// Unique identifier for the trace
    pub trace_id: String,
    
    /// Unique identifier for this span
    pub span_id: String,
    
    /// Parent span ID (null for root spans)
    pub parent_id: Option<String>,
    
    /// Name of the span (e.g., "llm_call", "tool_call")
    pub name: String,
    
    /// Status: "ok", "error", "in_progress"
    pub status: String,
    
    /// Arbitrary attributes (model, tokens, cost, etc.)
    pub attrs: HashMap<String, serde_json::Value>,
}

impl TraceEvent {
    /// Parse a single line from trace.jsonl
    pub fn from_line(line: &str) -> Result<Self, TraceError> {
        let event: TraceEvent = serde_json::from_str(line)?;
        Ok(event)
    }
    
    /// Stream parse a trace file for memory efficiency with large files.
    /// Pre-allocates capacity and uses an 8MB read buffer for throughput.
    pub fn stream_parse<P: AsRef<Path>>(path: P) -> Result<Trace, TraceError> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let capacity = estimate_event_count(&file)?;
        let reader = BufReader::with_capacity(8 * 1024 * 1024, file);
        let events = TraceEvent::stream_parse_reader(reader, capacity)?;
        Ok(Trace::new(events))
    }

    /// Stream parse from a BufReader with optional pre-allocated capacity.
    pub fn stream_parse_reader<R: BufRead>(
        reader: R,
        capacity: usize,
    ) -> Result<Vec<TraceEvent>, TraceError> {
        let mut events = Vec::with_capacity(capacity);

        for line_result in reader.lines() {
            let line = line_result?;
            if line.is_empty() {
                continue;
            }
            let event = TraceEvent::from_line(&line)?;
            event.validate()?;
            events.push(event);
        }

        Ok(events)
    }

    /// Validate required fields per trace.jsonl v1.0 spec.
    pub fn validate(&self) -> Result<(), TraceError> {
        if self.trace_id.is_empty() {
            return Err(TraceError::MissingField("trace_id".into()));
        }
        if self.span_id.is_empty() {
            return Err(TraceError::MissingField("span_id".into()));
        }
        if self.name.is_empty() {
            return Err(TraceError::MissingField("name".into()));
        }
        if self.status.is_empty() {
            return Err(TraceError::MissingField("status".into()));
        }
        Ok(())
    }
    
    /// Check if this is a root span (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
    
    /// Get total latency if present in attrs
    pub fn total_latency_ms(&self) -> Option<f64> {
        self.attrs.get("total_latency_ms")
            .and_then(|v| v.as_f64())
    }
    
    /// Get total cost if present in attrs
    pub fn total_cost_usd(&self) -> Option<f64> {
        self.attrs.get("total_cost_usd")
            .and_then(|v| v.as_f64())
    }
    
    /// Get model name for LLM calls
    pub fn model(&self) -> Option<&str> {
        self.attrs.get("model")
            .and_then(|v| v.as_str())
    }
    
    /// Get token count for LLM calls
    pub fn tokens_in(&self) -> Option<u64> {
        self.attrs.get("tokens_in")
            .and_then(|v| v.as_u64())
    }
    
    pub fn tokens_out(&self) -> Option<u64> {
        self.attrs.get("tokens_out")
            .and_then(|v| v.as_u64())
    }
    
    /// Get latency for this span
    pub fn latency_ms(&self) -> Option<f64> {
        self.attrs.get("latency_ms")
            .and_then(|v| v.as_f64())
    }
    
    /// Get cost for this span
    pub fn cost_usd(&self) -> Option<f64> {
        self.attrs.get("cost_usd")
            .and_then(|v| v.as_f64())
    }
}

/// A collection of trace events with indexing
#[derive(Debug)]
pub struct Trace {
    pub events: Vec<TraceEvent>,
    pub trace_id: String,
}

impl Trace {
    /// Create a new trace from events
    pub fn new(events: Vec<TraceEvent>) -> Self {
        let trace_id = events.first()
            .map(|e| e.trace_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        Self { events, trace_id }
    }
    
    /// Build a span tree from flat events
    pub fn build_span_tree(&self) -> Vec<SpanNode> {
        let mut span_map: HashMap<String, SpanNode> = HashMap::new();
        let mut roots: Vec<SpanNode> = Vec::new();
        
        // First pass: create all nodes
        for event in &self.events {
            let node = SpanNode {
                event: event.clone(),
                children: Vec::new(),
            };
            span_map.insert(event.span_id.clone(), node);
        }
        
        // Second pass: build hierarchy
        for event in &self.events {
            if let Some(parent_id) = &event.parent_id {
                if let Some(parent_node) = span_map.get_mut(parent_id) {
                    let child = span_map.remove(&event.span_id).unwrap();
                    parent_node.children.push(child);
                }
            } else {
                // Root span
                if let Some(root) = span_map.remove(&event.span_id) {
                    roots.push(root);
                }
            }
        }
        
        roots
    }
    
    /// Get all root spans
    pub fn root_spans(&self) -> Vec<&TraceEvent> {
        self.events.iter()
            .filter(|e| e.is_root())
            .collect()
    }
    
    /// Calculate total cost across all spans
    pub fn total_cost(&self) -> f64 {
        self.events.iter()
            .filter_map(|e| e.cost_usd())
            .sum()
    }
    
    /// Calculate total latency for root spans
    pub fn total_latency(&self) -> f64 {
        self.root_spans()
            .iter()
            .filter_map(|e| e.total_latency_ms())
            .sum()
    }
}

/// A node in the span tree
#[derive(Debug, Clone)]
pub struct SpanNode {
    pub event: TraceEvent,
    pub children: Vec<SpanNode>,
}

/// Estimate event count from file size (~200 bytes per NDJSON line).
fn estimate_event_count(file: &File) -> Result<usize, TraceError> {
    let size = file.metadata()?.len() as usize;
    Ok((size / 200).max(64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_event(i: usize) -> String {
        format!(
            r#"{{"ts":"2026-07-02T15:30:{:02}.123Z","trace_id":"t_bench","span_id":"s_{}","parent_id":null,"name":"llm_call","status":"ok","attrs":{{"model":"claude-3.5","latency_ms":10,"cost_usd":0.001}}}}"#,
            i % 60,
            i
        )
    }

    #[test]
    fn test_parse_trace_event() {
        let json = r#"{
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
        }"#;
        
        let event = TraceEvent::from_line(json).unwrap();
        assert_eq!(event.trace_id, "t_abc123");
        assert_eq!(event.name, "llm_call");
        assert!(event.is_root());
        assert_eq!(event.model(), Some("claude-3.5"));
        assert_eq!(event.tokens_in(), Some(1200));
    }

    #[test]
    fn test_validate_required_fields() {
        let event = TraceEvent::from_line(&sample_event(0)).unwrap();
        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_parse_100k_spans_under_200ms() {
        let mut file = NamedTempFile::new().unwrap();
        for i in 0..100_000 {
            writeln!(file, "{}", sample_event(i)).unwrap();
        }
        file.flush().unwrap();

        let start = Instant::now();
        let trace = TraceEvent::stream_parse(file.path()).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(trace.events.len(), 100_000);
        assert!(
            elapsed.as_millis() < 2000,
            "parse took {}ms (target <200ms on release builds)",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_build_span_tree() {
        let jsonl = format!(
            "{}\n{}\n",
            sample_event(0).replace(r#""parent_id":null"#, r#""parent_id":null"#),
            sample_event(1).replace(r#""parent_id":null"#, r#""parent_id":"s_0""#)
        );
        let events: Vec<TraceEvent> = jsonl
            .lines()
            .map(TraceEvent::from_line)
            .collect::<Result<_, _>>()
            .unwrap();
        let trace = Trace::new(events);
        let tree = trace.build_span_tree();
        assert!(!tree.is_empty());
    }
}
