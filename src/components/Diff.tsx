import React from 'react';
import { TraceEvent } from '../types';
import { diffTraces, totalCost } from '../lib/trace';
import { GitCompare, Upload } from 'lucide-react';

interface DiffProps {
  baseline: TraceEvent[];
  onLoadCompare: () => void;
  compare: TraceEvent[] | null;
  compareLabel?: string;
}

export const Diff: React.FC<DiffProps> = ({
  baseline,
  onLoadCompare,
  compare,
  compareLabel,
}) => {
  const diff = React.useMemo(() => {
    if (!compare) return null;
    return diffTraces(baseline, compare);
  }, [baseline, compare]);

  if (baseline.length === 0) {
    return <p className="text-gray-400">Open a baseline trace first, then load a second trace to compare.</p>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-white font-semibold flex items-center gap-2">
            <GitCompare className="w-5 h-5 text-accent-cyan" />
            Run Comparison
          </h3>
          <p className="text-sm text-gray-400 mt-1">
            Baseline: {baseline.length} events
            {compare && ` • Compare: ${compare.length} events (${compareLabel ?? 'trace B'})`}
          </p>
        </div>
        <button
          onClick={onLoadCompare}
          className="flex items-center gap-2 px-4 py-2 bg-dark-700 border border-dark-600 rounded-lg hover:border-accent-cyan transition text-sm"
        >
          <Upload className="w-4 h-4" />
          Load Compare Trace
        </button>
      </div>

      {!compare ? (
        <div className="bg-dark-700 border border-dashed border-dark-600 rounded-lg p-12 text-center text-gray-400">
          Load a second trace.jsonl to see side-by-side differences in spans, cost, and latency.
        </div>
      ) : diff && (
        <>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard label="Added spans" value={diff.added.length} color="text-accent-green" />
            <StatCard label="Removed spans" value={diff.removed.length} color="text-accent-red" />
            <StatCard label="Changed spans" value={diff.changed.length} color="text-accent-yellow" />
            <StatCard
              label="Cost delta"
              value={`${diff.costDelta >= 0 ? '+' : ''}$${diff.costDelta.toFixed(4)}`}
              color={diff.costDelta > 0 ? 'text-accent-red' : 'text-accent-green'}
            />
          </div>

          <div className="grid md:grid-cols-2 gap-4 text-sm">
            <DiffList title="Added" items={diff.added} empty="No new spans" variant="added" />
            <DiffList title="Removed" items={diff.removed} empty="No removed spans" variant="removed" />
          </div>

          {diff.changed.length > 0 && (
            <div className="bg-dark-700 border border-dark-600 rounded-lg p-4">
              <h4 className="text-white font-medium mb-3">Changed Attributes</h4>
              <div className="space-y-3 max-h-64 overflow-y-auto">
                {diff.changed.map(({ before, after }) => (
                  <div key={before.span_id} className="border-b border-dark-600 pb-2">
                    <span className="text-accent-cyan">{before.name}</span>
                    <span className="text-gray-500 ml-2">({before.span_id})</span>
                    <div className="grid grid-cols-2 gap-2 mt-1 text-xs">
                      <pre className="text-gray-400">{JSON.stringify(before.attrs, null, 2)}</pre>
                      <pre className="text-accent-green">{JSON.stringify(after.attrs, null, 2)}</pre>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="text-xs text-gray-500">
            Baseline cost: ${totalCost(baseline).toFixed(4)} • Compare cost: ${totalCost(compare).toFixed(4)} •
            Latency delta: {diff.latencyDelta >= 0 ? '+' : ''}{diff.latencyDelta.toFixed(0)}ms
          </div>
        </>
      )}
    </div>
  );
};

function StatCard({ label, value, color }: { label: string; value: string | number; color: string }) {
  return (
    <div className="bg-dark-700 border border-dark-600 rounded-lg p-4">
      <div className={`text-2xl font-bold ${color}`}>{value}</div>
      <div className="text-sm text-gray-400">{label}</div>
    </div>
  );
}

function DiffList({
  title,
  items,
  empty,
  variant,
}: {
  title: string;
  items: TraceEvent[];
  empty: string;
  variant: 'added' | 'removed';
}) {
  const border = variant === 'added' ? 'border-accent-green/30' : 'border-accent-red/30';
  return (
    <div className={`bg-dark-700 border ${border} rounded-lg p-4`}>
      <h4 className="text-white font-medium mb-2">{title}</h4>
      {items.length === 0 ? (
        <p className="text-gray-500 text-sm">{empty}</p>
      ) : (
        <ul className="space-y-1 max-h-48 overflow-y-auto text-sm">
          {items.map((e) => (
            <li key={e.span_id} className="text-gray-300">
              {e.name} <span className="text-gray-500">({e.status})</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
