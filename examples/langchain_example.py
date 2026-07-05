"""
LangChain + griid-trace example (100% local tracing)

Instruments a LangChain agent so every LLM and tool call is written to trace.jsonl.
View live: trace tui --watch trace.jsonl

Requires: pip install trace-py langchain langchain-community langchain-ollama
For local LLM: ollama pull llama3.2
"""

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "adapters", "python"))

from trace_py import init, span, root_span, set_trace_id

init(os.environ.get("TRACE_PATH", "./trace.jsonl"))
set_trace_id("langchain_demo_001")


def run_with_tracing():
    try:
        from langchain_ollama import ChatOllama
        from langchain_core.messages import HumanMessage
    except ImportError as e:
        raise RuntimeError("langchain not installed") from e

    query = "Summarize why local-first observability matters for AI agents in one sentence."

    with root_span(input=query, framework="langchain", model="ollama"):
        with span("llm_call", model="llama3.2", provider="ollama"):
            llm = ChatOllama(model="llama3.2", temperature=0)
            response = llm.invoke([HumanMessage(content=query)])
            text = response.content if hasattr(response, "content") else str(response)
            print("Response:", text)
            return text


def run_mock_without_ollama():
    """Fallback demo when Ollama is not running — still produces a valid trace."""
    query = "Demo query without Ollama"

    with root_span(input=query, framework="langchain", mode="mock"):
        with span("llm_call", model="mock", tokens_in=42, tokens_out=18, cost_usd=0.0):
            print("Mock response: Local tracing works without any cloud API.")
            return "Local tracing works without any cloud API."


if __name__ == "__main__":
    use_mock = os.environ.get("USE_MOCK", "").lower() in ("1", "true", "yes")

    if use_mock:
        run_mock_without_ollama()
    else:
        try:
            run_with_tracing()
        except Exception as e:
            print(f"Falling back to mock demo ({e})...")
            run_mock_without_ollama()

    trace_path = os.environ.get("TRACE_PATH", "./trace.jsonl")
    print(f"\nTrace written to: {trace_path}")
    print("View: cargo run --release -- tui --watch", trace_path)
