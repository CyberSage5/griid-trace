//! trace-rs: Rust adapter for griid-trace
//!
//! Lightweight instrumentation library that appends to trace.jsonl.
//! Zero dependency on the griid-trace viewer.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// Global trace file path
static TRACE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Initialize the tracer with a custom path
pub fn init(path: Option<PathBuf>) {
    let mut trace_path = TRACE_PATH.lock().unwrap();
    *trace_path = path.or_else(|| {
        std::env::var("TRACE_PATH")
            .ok()
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from("./trace.jsonl")))
    });
    
    // Ensure parent directory exists
    if let Some(path) = &*trace_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
    }
}

/// Get the current trace path
fn get_trace_path() -> PathBuf {
    let trace_path = TRACE_PATH.lock().unwrap();
    if let Some(path) = &*trace_path {
        path.clone()
    } else {
        drop(trace_path);
        init(None);
        let trace_path = TRACE_PATH.lock().unwrap();
        trace_path.as_ref().unwrap().clone()
    }
}

/// Write a trace event to the trace.jsonl file
fn write_event(event: &TraceEvent) {
    let path = get_trace_path();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        if let Ok(json) = serde_json::to_string(event) {
            let _ = writeln!(file, "{}", json);
            let _ = file.flush();
        }
    }
}

/// A trace event
#[derive(Debug, Serialize)]
struct TraceEvent {
    ts: String,
    trace_id: String,
    span_id: String,
    parent_id: Option<String>,
    name: String,
    status: String,
    attrs: serde_json::Value,
}

/// Span context for tracking hierarchy
pub struct Span {
    span_id: String,
    trace_id: String,
    parent_id: Option<String>,
    name: String,
    attrs: serde_json::Value,
    start_time: std::time::Instant,
}

impl Span {
    /// Create a new span
    fn new(name: &str, attrs: serde_json::Value) -> Self {
        let span_id = Uuid::new_v4().to_string();
        let trace_id = std::env::var("TRACE_ID")
            .unwrap_or_else(|_| Uuid::new_v4().to_string());
        let parent_id = std::env::var("PARENT_SPAN_ID").ok();
        
        // Set current span as parent for nested spans
        std::env::set_var("PARENT_SPAN_ID", &span_id);
        
        let start_ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        
        // Write start event
        write_event(&TraceEvent {
            ts: start_ts,
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            parent_id: parent_id.clone(),
            name: name.to_string(),
            status: "in_progress".to_string(),
            attrs: attrs.clone(),
        });
        
        Self {
            span_id,
            trace_id,
            parent_id,
            name: name.to_string(),
            attrs,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Complete the span with success
    fn complete_ok(self) {
        let latency_ms = self.start_time.elapsed().as_millis() as f64;
        
        let mut attrs = self.attrs;
        if let Some(obj) = attrs.as_object_mut() {
            obj.insert("latency_ms".to_string(), serde_json::json!(latency_ms));
        }
        
        let end_ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        
        write_event(&TraceEvent {
            ts: end_ts,
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_id: self.parent_id,
            name: self.name,
            status: "ok".to_string(),
            attrs,
        });
        
        // Restore parent span
        if let Some(parent) = self.parent_id {
            std::env::set_var("PARENT_SPAN_ID", parent);
        } else {
            std::env::remove_var("PARENT_SPAN_ID");
        }
    }
    
    /// Complete the span with error
    fn complete_error(self, error: &str) {
        let latency_ms = self.start_time.elapsed().as_millis() as f64;
        
        let mut attrs = self.attrs;
        if let Some(obj) = attrs.as_object_mut() {
            obj.insert("latency_ms".to_string(), serde_json::json!(latency_ms));
            obj.insert("error".to_string(), serde_json::json!(error));
        }
        
        let end_ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        
        write_event(&TraceEvent {
            ts: end_ts,
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_id: self.parent_id,
            name: self.name,
            status: "error".to_string(),
            attrs,
        });
        
        // Restore parent span
        if let Some(parent) = self.parent_id {
            std::env::set_var("PARENT_SPAN_ID", parent);
        } else {
            std::env::remove_var("PARENT_SPAN_ID");
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        // If not explicitly completed, mark as ok
        let latency_ms = self.start_time.elapsed().as_millis() as f64;
        
        let mut attrs = self.attrs.clone();
        if let Some(obj) = attrs.as_object_mut() {
            obj.insert("latency_ms".to_string(), serde_json::json!(latency_ms));
        }
        
        let end_ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        
        write_event(&TraceEvent {
            ts: end_ts,
            trace_id: self.trace_id.clone(),
            span_id: self.span_id.clone(),
            parent_id: self.parent_id.clone(),
            name: self.name.clone(),
            status: "ok".to_string(),
            attrs,
        });
        
        // Restore parent span
        if let Some(parent) = &self.parent_id {
            std::env::set_var("PARENT_SPAN_ID", parent);
        } else {
            std::env::remove_var("PARENT_SPAN_ID");
        }
    }
}

/// Create a new span with the given name and attributes
///
/// # Example
/// ```
/// use trace_rs::span;
///
/// span!("llm_call", model = "claude-3.5", {
///     // Your code here
///     let response = call_llm("Hello");
/// });
/// ```
#[macro_export]
macro_rules! span {
    ($name:expr, $($key:ident = $value:expr),*, $block:block) => {
        {
            let mut attrs = serde_json::Map::new();
            $(
                attrs.insert(stringify!($key).to_string(), serde_json::json!($value));
            )*
            let _span = $crate::Span::new($name, serde_json::Value::Object(attrs));
            let result = $block;
            _span.complete_ok();
            result
        }
    };
    ($name:expr, $block:block) => {
        {
            let _span = $crate::Span::new($name, serde_json::Value::Object(serde_json::Map::new()));
            let result = $block;
            _span.complete_ok();
            result
        }
    };
}

/// Set a custom trace ID
pub fn set_trace_id(trace_id: &str) {
    std::env::set_var("TRACE_ID", trace_id);
}

/// Get the current trace ID
pub fn get_trace_id() -> String {
    std::env::var("TRACE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string())
}

/// Convenience function for LLM calls
pub fn llm_call(model: &str, prompt: &str, response: &str, tokens_in: u64, tokens_out: u64) {
    span!("llm_call", 
        model = model,
        prompt = prompt,
        response = response,
        tokens_in = tokens_in,
        tokens_out = tokens_out,
        {
            // The span is automatically completed
        }
    );
}

/// Convenience function for tool calls
pub fn tool_call<F>(tool: &str, f: F) 
where
    F: FnOnce(),
{
    span!("tool_call", tool = tool, {
        f();
    });
}

/// Convenience function for root spans (agent execution)
pub fn root_span<F>(input: &str, f: F)
where
    F: FnOnce(),
{
    span!("agent_execution", input = input, {
        f();
    });
}
