import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import {
  Play,
  FileText,
  Settings,
  Activity,
  GitCompare,
  BarChart3,
  Flame,
} from "lucide-react";
import { Flamegraph } from "./components/Flamegraph";
import { Analytics } from "./components/Analytics";
import { Replay } from "./components/Replay";
import { Diff } from "./components/Diff";
import { TraceTable } from "./components/TraceTable";
import { TraceEvent } from "./types";
import { parseTraceJsonl, totalCost, totalTokens } from "./lib/trace";

type Tab = "trace" | "flamegraph" | "analytics" | "replay" | "diff";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("trace");
  const [events, setEvents] = useState<TraceEvent[]>([]);
  const [compareEvents, setCompareEvents] = useState<TraceEvent[] | null>(null);
  const [tracePath, setTracePath] = useState<string | null>(null);
  const [comparePath, setComparePath] = useState<string | null>(null);
  const [selectedEvent, setSelectedEvent] = useState<TraceEvent | null>(null);
  const [flameZoom, setFlameZoom] = useState(1);
  const [status, setStatus] = useState("Ready — 100% local, zero cloud");
  const [error, setError] = useState<string | null>(null);

  const loadTraceFile = useCallback(async (forCompare = false) => {
    try {
      setError(null);
      const selected = await open({
        multiple: false,
        filters: [{ name: "Trace", extensions: ["jsonl", "json"] }],
        title: forCompare ? "Select trace to compare" : "Open trace.jsonl",
      });

      if (!selected || typeof selected !== "string") return;

      const content = await invoke<string>("read_trace_file", { path: selected });
      const parsed = parseTraceJsonl(content);

      if (forCompare) {
        setCompareEvents(parsed);
        setComparePath(selected);
        setStatus(`Loaded compare trace: ${parsed.length} events`);
        setActiveTab("diff");
      } else {
        setEvents(parsed);
        setTracePath(selected);
        setCompareEvents(null);
        setComparePath(null);
        setStatus(`Loaded ${parsed.length} events from ${selected.split(/[/\\]/).pop()}`);
      }
    } catch (e) {
      setError(String(e));
      setStatus("Failed to load trace");
    }
  }, []);

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "trace", label: "Trace Events", icon: <FileText className="w-4 h-4" /> },
    { id: "flamegraph", label: "Flamegraph", icon: <Flame className="w-4 h-4" /> },
    { id: "analytics", label: "Analytics", icon: <BarChart3 className="w-4 h-4" /> },
    { id: "replay", label: "Replay", icon: <Play className="w-4 h-4" /> },
    { id: "diff", label: "Diff", icon: <GitCompare className="w-4 h-4" /> },
  ];

  return (
    <div className="h-screen flex flex-col bg-dark-900 text-gray-200">
      <header className="bg-dark-800 border-b border-dark-600 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Activity className="w-8 h-8 text-accent-cyan" />
            <div>
              <h1 className="text-2xl font-bold text-white">griid-trace</h1>
              <p className="text-xs text-gray-500">Local-first agent observability</p>
            </div>
            <span className="px-2 py-1 bg-dark-700 rounded text-xs text-accent-green border border-accent-green/30">
              Open Source
            </span>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => loadTraceFile(false)}
              className="flex items-center gap-2 px-4 py-2 bg-accent-cyan text-dark-900 rounded-lg hover:bg-accent-cyan/90 transition font-medium"
            >
              <FileText className="w-4 h-4" />
              Open Trace
            </button>
            <button className="p-2 hover:bg-dark-700 rounded-lg transition" title="Settings">
              <Settings className="w-5 h-5" />
            </button>
          </div>
        </div>
      </header>

      {error && (
        <div className="bg-accent-red/10 border-b border-accent-red/30 px-6 py-2 text-sm text-accent-red">
          {error}
        </div>
      )}

      <div className="bg-dark-800 border-b border-dark-600 px-6">
        <div className="flex gap-1 overflow-x-auto">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition whitespace-nowrap ${
                activeTab === tab.id
                  ? "border-accent-cyan text-accent-cyan"
                  : "border-transparent text-gray-400 hover:text-gray-200"
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      <main className="flex-1 overflow-auto p-6">
        {events.length > 0 && (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
            <MiniStat label="Events" value={String(events.length)} />
            <MiniStat label="Total Cost" value={`$${totalCost(events).toFixed(4)}`} />
            <MiniStat label="Tokens" value={totalTokens(events).toLocaleString()} />
            <MiniStat label="Errors" value={String(events.filter((e) => e.status === "error").length)} />
          </div>
        )}

        {activeTab === "trace" && (
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-6">
            <h2 className="text-lg font-semibold mb-4">Trace Events</h2>
            {events.length > 0 ? (
              <TraceTable
                events={events}
                selectedId={selectedEvent?.span_id}
                onSelect={setSelectedEvent}
              />
            ) : (
              <EmptyState onOpen={() => loadTraceFile(false)} />
            )}
            {selectedEvent && (
              <div className="mt-4 p-4 bg-dark-700 rounded-lg border border-dark-600">
                <h3 className="font-medium text-white mb-2">Selected: {selectedEvent.name}</h3>
                <pre className="text-xs text-gray-400 overflow-auto">{JSON.stringify(selectedEvent, null, 2)}</pre>
              </div>
            )}
          </div>
        )}

        {activeTab === "flamegraph" && (
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Flamegraph</h2>
              {events.length > 0 && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-gray-400">Zoom</span>
                  <input
                    type="range"
                    min={0.5}
                    max={3}
                    step={0.1}
                    value={flameZoom}
                    onChange={(e) => setFlameZoom(Number(e.target.value))}
                    className="w-24 accent-accent-cyan"
                  />
                </div>
              )}
            </div>
            {events.length > 0 ? (
              <Flamegraph events={events} zoom={flameZoom} onSpanClick={setSelectedEvent} />
            ) : (
              <EmptyState onOpen={() => loadTraceFile(false)} />
            )}
          </div>
        )}

        {activeTab === "analytics" && (
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-6">
            <h2 className="text-lg font-semibold mb-4">Analytics</h2>
            {events.length > 0 ? <Analytics events={events} /> : <EmptyState onOpen={() => loadTraceFile(false)} />}
          </div>
        )}

        {activeTab === "replay" && (
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-6">
            <h2 className="text-lg font-semibold mb-4">Replay Mode</h2>
            <Replay events={events} />
          </div>
        )}

        {activeTab === "diff" && (
          <div className="bg-dark-800 rounded-lg border border-dark-600 p-6">
            <h2 className="text-lg font-semibold mb-4">Run Diff</h2>
            <Diff
              baseline={events}
              compare={compareEvents}
              compareLabel={comparePath?.split(/[/\\]/).pop()}
              onLoadCompare={() => loadTraceFile(true)}
            />
          </div>
        )}
      </main>

      <footer className="bg-dark-800 border-t border-dark-600 px-6 py-2">
        <div className="flex items-center justify-between text-sm text-gray-400">
          <span>{status}{tracePath ? ` • ${tracePath.split(/[/\\]/).pop()}` : ""}</span>
          <span>Local-First • No Cloud • No Telemetry • Your Data Stays on Your Device</span>
        </div>
      </footer>
    </div>
  );
}

function MiniStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-dark-800 border border-dark-600 rounded-lg px-4 py-3">
      <div className="text-lg font-semibold text-white">{value}</div>
      <div className="text-xs text-gray-500">{label}</div>
    </div>
  );
}

function EmptyState({ onOpen }: { onOpen: () => void }) {
  return (
    <div className="text-center py-12">
      <p className="text-gray-400 mb-4">
        Open a trace.jsonl file to get started. Instrument your agent with trace-py or trace-rs.
      </p>
      <button
        onClick={onOpen}
        className="px-6 py-2 bg-accent-cyan text-dark-900 rounded-lg font-medium hover:bg-accent-cyan/90 transition"
      >
        Open Trace File
      </button>
    </div>
  );
}

export default App;
