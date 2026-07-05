import React from 'react';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { TraceEvent } from '../types';

interface AnalyticsProps {
  events: TraceEvent[];
}

export const Analytics: React.FC<AnalyticsProps> = ({ events }) => {
  const costData = React.useMemo(() => {
    const costs: Record<string, number> = {};
    events.forEach((event) => {
      const model = event.attrs.model || 'unknown';
      const cost = event.attrs.cost_usd || 0;
      costs[model] = (costs[model] || 0) + cost;
    });
    return Object.entries(costs).map(([model, cost]) => ({ model, cost }));
  }, [events]);

  const latencyData = React.useMemo(() => {
    return events
      .filter((e) => e.attrs.latency_ms)
      .map((event) => ({
        name: event.name,
        latency: event.attrs.latency_ms || 0,
      }));
  }, [events]);

  const tokenData = React.useMemo(() => {
    const tokensIn = events.reduce((sum, e) => sum + (e.attrs.tokens_in || 0), 0);
    const tokensOut = events.reduce((sum, e) => sum + (e.attrs.tokens_out || 0), 0);
    return [
      { name: 'Tokens In', value: tokensIn },
      { name: 'Tokens Out', value: tokensOut },
    ];
  }, [events]);

  const statusData = React.useMemo(() => {
    const statusCount: Record<string, number> = {};
    events.forEach((event) => {
      statusCount[event.status] = (statusCount[event.status] || 0) + 1;
    });
    return Object.entries(statusCount).map(([status, count]) => ({
      name: status,
      value: count,
    }));
  }, [events]);

  const COLORS = {
    ok: '#3fb950',
    error: '#f85149',
    in_progress: '#d29922',
    unknown: '#58a6ff',
  };

  return (
    <div className="space-y-8">
      <div className="grid md:grid-cols-2 gap-8">
        {/* Cost by Model */}
        <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Cost by Model</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={costData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#30363d" />
              <XAxis dataKey="model" stroke="#8b949e" />
              <YAxis stroke="#8b949e" />
              <Tooltip
                contentStyle={{ backgroundColor: '#21262d', border: '1px solid #30363d' }}
              />
              <Bar dataKey="cost" fill="#58a6ff" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Latency Distribution */}
        <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Latency Distribution</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={latencyData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#30363d" />
              <XAxis dataKey="name" stroke="#8b949e" />
              <YAxis stroke="#8b949e" />
              <Tooltip
                contentStyle={{ backgroundColor: '#21262d', border: '1px solid #30363d' }}
              />
              <Legend />
              <Line type="monotone" dataKey="latency" stroke="#3fb950" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="grid md:grid-cols-2 gap-8">
        {/* Token Usage */}
        <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Token Usage</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={tokenData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                <Cell fill="#58a6ff" />
                <Cell fill="#3fb950" />
              </Pie>
              <Tooltip
                contentStyle={{ backgroundColor: '#21262d', border: '1px solid #30363d' }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Status Distribution */}
        <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Status Distribution</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={statusData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {statusData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[entry.name as keyof typeof COLORS] || COLORS.unknown} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{ backgroundColor: '#21262d', border: '1px solid #30363d' }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Summary Stats */}
      <div className="bg-[#161b22] border border-[#30363d] rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Summary</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-[#21262d] rounded-lg p-4">
            <div className="text-2xl font-bold text-[#58a6ff]">{events.length}</div>
            <div className="text-sm text-gray-400">Total Events</div>
          </div>
          <div className="bg-[#21262d] rounded-lg p-4">
            <div className="text-2xl font-bold text-[#3fb950]">
              ${events.reduce((sum, e) => sum + (e.attrs.cost_usd || 0), 0).toFixed(4)}
            </div>
            <div className="text-sm text-gray-400">Total Cost</div>
          </div>
          <div className="bg-[#21262d] rounded-lg p-4">
            <div className="text-2xl font-bold text-[#d29922]">
              {events.reduce((sum, e) => sum + (e.attrs.tokens_in || 0) + (e.attrs.tokens_out || 0), 0).toLocaleString()}
            </div>
            <div className="text-sm text-gray-400">Total Tokens</div>
          </div>
          <div className="bg-[#21262d] rounded-lg p-4">
            <div className="text-2xl font-bold text-[#f85149]">
              {events.filter((e) => e.status === 'error').length}
            </div>
            <div className="text-sm text-gray-400">Errors</div>
          </div>
        </div>
      </div>
    </div>
  );
};
