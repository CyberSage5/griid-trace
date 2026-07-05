use griid_trace::trace::{Trace, TraceEvent};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_sample_trace(path: &std::path::Path) {
    let content = include_str!("../examples/sample-trace.jsonl");
    std::fs::write(path, content).unwrap();
}

#[test]
fn integration_parse_sample_trace() {
    let mut file = NamedTempFile::new().unwrap();
    write_sample_trace(file.path());

    let trace = TraceEvent::stream_parse(file.path()).unwrap();
    assert_eq!(trace.events.len(), 6);
    assert!(trace.total_cost() > 0.0);
}

#[test]
fn integration_span_tree_from_sample() {
    let mut file = NamedTempFile::new().unwrap();
    write_sample_trace(file.path());

    let trace = TraceEvent::stream_parse(file.path()).unwrap();
    let roots = trace.build_span_tree();
    assert_eq!(roots.len(), 1);
    assert!(!roots[0].children.is_empty());
}

#[test]
fn integration_validate_rejects_empty_span_id() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"{{"ts":"2026-07-02T15:30:00.123Z","trace_id":"t1","span_id":"","parent_id":null,"name":"llm_call","status":"ok","attrs":{{}}}}"#
    )
    .unwrap();

    let result = TraceEvent::stream_parse(file.path());
    assert!(result.is_err());
}
