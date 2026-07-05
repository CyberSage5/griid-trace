"""
trace-py: Python adapter for griid-trace

Lightweight instrumentation library that appends to trace.jsonl.
Zero dependency on the griid-trace viewer.
"""

import os
import json
import uuid
import time
import tempfile
from datetime import datetime, timezone
from contextlib import contextmanager
from typing import Optional, Dict, Any

try:
    import fcntl
    HAS_FCNTL = True
except ImportError:
    HAS_FCNTL = False

# Global trace file path
_trace_path: Optional[str] = None


def init(path: Optional[str] = None) -> None:
    """
    Initialize the tracer.
    
    Args:
        path: Path to trace.jsonl file. If None, reads TRACE_PATH env var
              or defaults to ./trace.jsonl
    """
    global _trace_path
    _trace_path = path or os.environ.get("TRACE_PATH", "./trace.jsonl")
    
    parent = os.path.dirname(os.path.abspath(_trace_path))
    if parent:
        os.makedirs(parent, exist_ok=True)


def _get_trace_path() -> str:
    """Get the current trace path, initializing if needed."""
    global _trace_path
    if _trace_path is None:
        init()
    return _trace_path


def _write_event(event: Dict[str, Any]) -> None:
    """Atomically append a trace event with file locking when available."""
    path = _get_trace_path()
    line = json.dumps(event, separators=(",", ":")) + "\n"

    with open(path, "a", encoding="utf-8") as f:
        if HAS_FCNTL:
            fcntl.flock(f.fileno(), fcntl.LOCK_EX)
        try:
            f.write(line)
            f.flush()
            os.fsync(f.fileno())
        finally:
            if HAS_FCNTL:
                fcntl.flock(f.fileno(), fcntl.LOCK_UN)


@contextmanager
def span(name: str, **attrs):
    """
    Context manager for creating spans.
    
    Args:
        name: Span name (e.g., "llm_call", "tool_call")
        **attrs: Arbitrary attributes (model, tokens, cost, etc.)
    
    Example:
        with span("llm_call", model="claude-3.5"):
            response = call_llm("Hello")
    """
    span_id = str(uuid.uuid4())
    trace_id = os.environ.get("TRACE_ID", str(uuid.uuid4()))
    parent_id = os.environ.get("PARENT_SPAN_ID")
    
    # Set current span as parent for nested spans
    old_parent = os.environ.get("PARENT_SPAN_ID")
    os.environ["PARENT_SPAN_ID"] = span_id
    
    start_time = time.time()
    start_ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z"
    
    # Write start event
    _write_event({
        "ts": start_ts,
        "trace_id": trace_id,
        "span_id": span_id,
        "parent_id": parent_id,
        "name": name,
        "status": "in_progress",
        "attrs": attrs
    })
    
    try:
        yield
        
        # Write completion event
        end_time = time.time()
        latency_ms = (end_time - start_time) * 1000
        
        attrs_with_latency = {**attrs, "latency_ms": round(latency_ms, 2)}
        
        _write_event({
            "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z",
            "trace_id": trace_id,
            "span_id": span_id,
            "parent_id": parent_id,
            "name": name,
            "status": "ok",
            "attrs": attrs_with_latency
        })
        
    except Exception as e:
        # Write error event
        end_time = time.time()
        latency_ms = (end_time - start_time) * 1000
        
        attrs_with_error = {**attrs, "latency_ms": round(latency_ms, 2), "error": str(e)}
        
        _write_event({
            "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z",
            "trace_id": trace_id,
            "span_id": span_id,
            "parent_id": parent_id,
            "name": name,
            "status": "error",
            "attrs": attrs_with_error
        })
        
        raise
    finally:
        # Restore parent span
        if old_parent:
            os.environ["PARENT_SPAN_ID"] = old_parent
        else:
            os.environ.pop("PARENT_SPAN_ID", None)


def llm_call(model: str, prompt: str, response: str, tokens_in: int, tokens_out: int) -> None:
    """
    Convenience function for LLM calls.
    
    Args:
        model: Model name (e.g., "claude-3.5")
        prompt: Input prompt
        response: Model response
        tokens_in: Input token count
        tokens_out: Output token count
    """
    with span("llm_call", 
              model=model,
              prompt=prompt,
              response=response,
              tokens_in=tokens_in,
              tokens_out=tokens_out):
        pass


def tool_call(tool: str, **attrs) -> contextmanager:
    """
    Convenience context manager for tool calls.
    
    Args:
        tool: Tool name
        **attrs: Additional tool attributes
    """
    return span("tool_call", tool=tool, **attrs)


def root_span(input: str, **attrs) -> contextmanager:
    """
    Convenience context manager for root spans (agent execution).
    
    Args:
        input: Input query/prompt
        **attrs: Additional attributes
    """
    return span("agent_execution", input=input, **attrs)


def set_trace_id(trace_id: str) -> None:
    """
    Set a custom trace ID for the current execution.
    
    Args:
        trace_id: Custom trace ID
    """
    os.environ["TRACE_ID"] = trace_id


def get_trace_id() -> str:
    """Get the current trace ID."""
    return os.environ.get("TRACE_ID", str(uuid.uuid4()))
