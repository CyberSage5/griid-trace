import React from 'react';
import { TraceEvent } from '../types';

interface TraceTableProps {
  events: TraceEvent[];
  onSelect?: (event: TraceEvent) => void;
  selectedId?: string | null;
}

export const TraceTable: React.FC<TraceTableProps> = ({ events, onSelect, selectedId }) => {
  const [filter, setFilter] = React.useState('');
  const [statusFilter, setStatusFilter] = React.useState<string>('all');

  const filtered = React.useMemo(() => {
    return events.filter((e) => {
      const matchText =
        !filter ||
        e.name.toLowerCase().includes(filter.toLowerCase()) ||
        e.span_id.includes(filter);
      const matchStatus = statusFilter === 'all' || e.status === statusFilter;
      return matchText && matchStatus;
    });
  }, [events, filter, statusFilter]);

  if (events.length === 0) {
    return <p className="text-gray-400">No events loaded.</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-3">
        <input
          type="search"
          placeholder="Filter by name or span ID..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="flex-1 min-w-[200px] px-3 py-2 bg-dark-700 border border-dark-600 rounded-lg text-sm focus:border-accent-cyan outline-none"
        />
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="px-3 py-2 bg-dark-700 border border-dark-600 rounded-lg text-sm"
        >
          <option value="all">All statuses</option>
          <option value="ok">ok</option>
          <option value="error">error</option>
          <option value="in_progress">in_progress</option>
        </select>
      </div>

      <div className="overflow-x-auto border border-dark-600 rounded-lg">
        <table className="w-full text-sm">
          <thead className="bg-dark-700 text-gray-400 text-left">
            <tr>
              <th className="px-4 py-3">Time</th>
              <th className="px-4 py-3">Name</th>
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Model</th>
              <th className="px-4 py-3">Latency</th>
              <th className="px-4 py-3">Cost</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((event) => (
              <tr
                key={`${event.span_id}-${event.ts}`}
                onClick={() => onSelect?.(event)}
                className={`border-t border-dark-600 cursor-pointer hover:bg-dark-700/50 ${
                  selectedId === event.span_id ? 'bg-accent-cyan/10' : ''
                }`}
              >
                <td className="px-4 py-2 text-gray-400 font-mono text-xs">
                  {new Date(event.ts).toLocaleTimeString()}
                </td>
                <td className="px-4 py-2 text-white">{event.name}</td>
                <td className="px-4 py-2">
                  <StatusBadge status={event.status} />
                </td>
                <td className="px-4 py-2 text-gray-300">{event.attrs.model ?? '—'}</td>
                <td className="px-4 py-2 text-gray-300">
                  {event.attrs.latency_ms != null ? `${event.attrs.latency_ms}ms` : '—'}
                </td>
                <td className="px-4 py-2 text-gray-300">
                  {event.attrs.cost_usd != null ? `$${event.attrs.cost_usd.toFixed(4)}` : '—'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="text-xs text-gray-500">
        Showing {filtered.length} of {events.length} events
      </p>
    </div>
  );
};

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    ok: 'bg-accent-green/20 text-accent-green',
    error: 'bg-accent-red/20 text-accent-red',
    in_progress: 'bg-accent-yellow/20 text-accent-yellow',
  };
  return (
    <span className={`px-2 py-0.5 rounded text-xs ${colors[status] ?? 'bg-dark-600 text-gray-300'}`}>
      {status}
    </span>
  );
}
