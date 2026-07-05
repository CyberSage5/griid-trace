import React from 'react';
import { TraceEvent } from '../types';
import { Pause, Play, SkipBack, SkipForward } from 'lucide-react';

interface ReplayProps {
  events: TraceEvent[];
}

export const Replay: React.FC<ReplayProps> = ({ events }) => {
  const sorted = React.useMemo(
    () => [...events].sort((a, b) => new Date(a.ts).getTime() - new Date(b.ts).getTime()),
    [events]
  );

  const [step, setStep] = React.useState(0);
  const [playing, setPlaying] = React.useState(false);

  React.useEffect(() => {
    if (!playing || sorted.length === 0) return;
    const id = setInterval(() => {
      setStep((s) => (s >= sorted.length - 1 ? 0 : s + 1));
    }, 800);
    return () => clearInterval(id);
  }, [playing, sorted.length]);

  if (sorted.length === 0) {
    return (
      <p className="text-gray-400">Open a trace file to replay execution step-by-step.</p>
    );
  }

  const current = sorted[step];
  const visible = sorted.slice(0, step + 1);
  const progress = ((step + 1) / sorted.length) * 100;

  return (
    <div className="space-y-6">
      <div className="bg-dark-700 border border-dark-600 rounded-lg p-4">
        <div className="flex items-center gap-3 mb-4">
          <button
            onClick={() => setStep(0)}
            className="p-2 hover:bg-dark-600 rounded transition"
            title="Restart"
          >
            <SkipBack className="w-5 h-5" />
          </button>
          <button
            onClick={() => setStep((s) => Math.max(0, s - 1))}
            className="p-2 hover:bg-dark-600 rounded transition"
          >
            <SkipBack className="w-4 h-4" />
          </button>
          <button
            onClick={() => setPlaying(!playing)}
            className="p-3 bg-accent-cyan text-dark-900 rounded-full hover:bg-accent-cyan/90 transition"
          >
            {playing ? <Pause className="w-5 h-5" /> : <Play className="w-5 h-5" />}
          </button>
          <button
            onClick={() => setStep((s) => Math.min(sorted.length - 1, s + 1))}
            className="p-2 hover:bg-dark-600 rounded transition"
          >
            <SkipForward className="w-4 h-4" />
          </button>
          <button
            onClick={() => setStep(sorted.length - 1)}
            className="p-2 hover:bg-dark-600 rounded transition"
          >
            <SkipForward className="w-5 h-5" />
          </button>
          <span className="text-sm text-gray-400 ml-2">
            Step {step + 1} / {sorted.length}
          </span>
        </div>

        <input
          type="range"
          min={0}
          max={sorted.length - 1}
          value={step}
          onChange={(e) => {
            setPlaying(false);
            setStep(Number(e.target.value));
          }}
          className="w-full accent-accent-cyan"
        />
        <div className="h-1 bg-dark-600 rounded mt-2 overflow-hidden">
          <div
            className="h-full bg-accent-cyan transition-all"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      <div className="grid md:grid-cols-2 gap-6">
        <div className="bg-dark-700 border border-dark-600 rounded-lg p-4">
          <h3 className="text-white font-semibold mb-3">Current Step</h3>
          <div className="space-y-2 text-sm">
            <p><span className="text-gray-400">Name:</span> <span className="text-white">{current.name}</span></p>
            <p><span className="text-gray-400">Status:</span>{' '}
              <span className={current.status === 'error' ? 'text-accent-red' : 'text-accent-green'}>
                {current.status}
              </span>
            </p>
            <p><span className="text-gray-400">Time:</span> {new Date(current.ts).toLocaleTimeString()}</p>
            {current.attrs.latency_ms != null && (
              <p><span className="text-gray-400">Latency:</span> {current.attrs.latency_ms}ms</p>
            )}
          </div>
        </div>

        <div className="bg-dark-700 border border-dark-600 rounded-lg p-4">
          <h3 className="text-white font-semibold mb-3">Timeline</h3>
          <div className="space-y-1 max-h-64 overflow-y-auto">
            {visible.map((e, i) => (
              <div
                key={`${e.span_id}-${i}`}
                className={`text-xs px-2 py-1 rounded ${
                  i === step ? 'bg-accent-cyan/20 text-accent-cyan' : 'text-gray-400'
                }`}
              >
                {e.name} — {e.status}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
