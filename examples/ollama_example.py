"""
Example: Using griid-trace with Ollama (local LLM)

This demonstrates instrumenting a local AI agent using Ollama with griid-trace.
"""

import sys
import os

# Add the adapter to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'adapters', 'python'))

from trace_py import init, span, root_span, llm_call, tool_call
import requests
import json

# Initialize griid-trace
init()

OLLAMA_BASE_URL = "http://localhost:11434"


def call_ollama(model: str, prompt: str) -> dict:
    """Call Ollama API"""
    response = requests.post(
        f"{OLLAMA_BASE_URL}/api/generate",
        json={
            "model": model,
            "prompt": prompt,
            "stream": False
        }
    )
    return response.json()


def search_web(query: str) -> str:
    """Simulated web search tool"""
    with tool_call("web_search", query=query):
        # In a real implementation, this would call a search API
        # For demo purposes, we'll simulate it
        return f"Search results for: {query}"


def main():
    """Main agent execution"""
    query = "What is the capital of France?"
    
    with root_span(input=query, agent="ollama_agent"):
        print(f"Query: {query}")
        
        # Step 1: Generate initial response
        with span("reasoning", step="initial_generation"):
            response = call_ollama("llama2", query)
            print(f"Initial response: {response.get('response', '')}")
        
        # Step 2: Search for additional information
        search_result = search_web("capital of France")
        print(f"Search result: {search_result}")
        
        # Step 3: Generate final answer with context
        with span("reasoning", step="final_generation"):
            final_prompt = f"{query}\n\nContext: {search_result}"
            final_response = call_ollama("llama2", final_prompt)
            print(f"Final answer: {final_response.get('response', '')}")


if __name__ == "__main__":
    main()
    print("\nTrace saved to trace.jsonl")
    print("View with: trace tui --watch trace.jsonl")
