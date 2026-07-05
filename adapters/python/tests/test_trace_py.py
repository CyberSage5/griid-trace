import os
import json
import tempfile
import pytest
from trace_py import init, span, set_trace_id, get_trace_id


@pytest.fixture
def trace_file(tmp_path):
    path = str(tmp_path / "trace.jsonl")
    init(path)
    return path


def test_init_creates_parent_dir(tmp_path):
    path = str(tmp_path / "nested" / "trace.jsonl")
    init(path)
    assert os.path.isdir(os.path.dirname(path))


def test_span_writes_ok_events(trace_file):
    with span("llm_call", model="claude-3.5"):
        pass

    with open(trace_file) as f:
        lines = [json.loads(l) for l in f if l.strip()]

    assert len(lines) >= 2
    assert lines[0]["status"] == "in_progress"
    assert lines[-1]["status"] == "ok"
    assert lines[-1]["attrs"]["model"] == "claude-3.5"
    assert "latency_ms" in lines[-1]["attrs"]


def test_span_records_error(trace_file):
    with pytest.raises(ValueError):
        with span("tool_call", tool="calc"):
            raise ValueError("boom")

    with open(trace_file) as f:
        lines = [json.loads(l) for l in f if l.strip()]

    assert any(l["status"] == "error" for l in lines)


def test_nested_spans_have_parent(trace_file):
    with span("agent_execution", input="test"):
        with span("llm_call", model="gpt-4"):
            pass

    with open(trace_file) as f:
        lines = [json.loads(l) for l in f if l.strip()]

    child = next(l for l in lines if l["name"] == "llm_call")
    assert child["parent_id"] is not None


def test_set_trace_id():
    set_trace_id("t_custom")
    assert get_trace_id() == "t_custom"


def test_concurrent_writes(trace_file):
    """Multiple spans in sequence should all append without corruption."""
    for i in range(50):
        with span("llm_call", index=i):
            pass

    with open(trace_file) as f:
        lines = f.readlines()

    for line in lines:
        json.loads(line)  # must be valid JSON

    assert len(lines) >= 100  # start + end per span
