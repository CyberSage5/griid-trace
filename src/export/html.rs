use crate::trace::{Trace, TraceError};
use handlebars::Handlebars;
use std::fs;
use std::path::Path;

/// Generate a self-contained HTML report from a trace
pub fn export_html<P: AsRef<Path>>(trace: &Trace, output_path: P) -> Result<(), TraceError> {
    let html = generate_html(trace)?;
    fs::write(output_path, html)?;
    Ok(())
}

fn generate_html(trace: &Trace) -> Result<String, TraceError> {
    let mut handlebars = Handlebars::new();
    
    let template = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>griid-trace Report - {{trace_id}}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117;
            color: #c9d1d9;
            line-height: 1.6;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }
        .header {
            background: #161b22;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            border: 1px solid #30363d;
        }
        .header h1 {
            color: #58a6ff;
            margin-bottom: 10px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }
        .stat-card {
            background: #161b22;
            padding: 15px;
            border-radius: 6px;
            border: 1px solid #30363d;
        }
        .stat-card h3 {
            color: #8b949e;
            font-size: 0.9em;
            margin-bottom: 5px;
        }
        .stat-card .value {
            color: #c9d1d9;
            font-size: 1.5em;
            font-weight: bold;
        }
        .spans-table {
            width: 100%;
            border-collapse: collapse;
            background: #161b22;
            border-radius: 8px;
            overflow: hidden;
        }
        .spans-table th, .spans-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #30363d;
        }
        .spans-table th {
            background: #21262d;
            color: #c9d1d9;
            font-weight: 600;
        }
        .spans-table tr:hover {
            background: #21262d;
        }
        .status-ok { color: #3fb950; }
        .status-error { color: #f85149; }
        .status-in_progress { color: #d29922; }
        .details {
            background: #161b22;
            padding: 15px;
            border-radius: 6px;
            border: 1px solid #30363d;
            margin-top: 20px;
        }
        .details pre {
            background: #0d1117;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
            color: #8b949e;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>griid-trace Report</h1>
            <p>Trace ID: {{trace_id}}</p>
        </div>
        
        <div class="stats">
            <div class="stat-card">
                <h3>Total Events</h3>
                <div class="value">{{event_count}}</div>
            </div>
            <div class="stat-card">
                <h3>Total Cost</h3>
                <div class="value">${{total_cost}}</div>
            </div>
            <div class="stat-card">
                <h3>Total Latency</h3>
                <div class="value">{{total_latency}}ms</div>
            </div>
        </div>
        
        <table class="spans-table">
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Latency</th>
                    <th>Cost</th>
                </tr>
            </thead>
            <tbody>
                {{#each events}}
                <tr>
                    <td>{{name}}</td>
                    <td class="status-{{status}}">{{status}}</td>
                    <td>{{#if latency_ms}}{{latency_ms}}ms{{else}}-{{/if}}</td>
                    <td>{{#if cost_usd}}${{cost_usd}}{{else}}-{{/if}}</td>
                </tr>
                {{/each}}
            </tbody>
        </table>
        
        <div class="details">
            <h2>Raw Data</h2>
            <pre>{{{raw_json}}}</pre>
        </div>
    </div>
</body>
</html>
"#;
    
    handlebars.register_template_string("report", template)?;
    
    let data = serde_json::json!({
        "trace_id": trace.trace_id,
        "event_count": trace.events.len(),
        "total_cost": format!("{:.4}", trace.total_cost()),
        "total_latency": format!("{:.2}", trace.total_latency()),
        "events": trace.events,
        "raw_json": serde_json::to_string_pretty(trace)?,
    });
    
    handlebars.render("report", &data).map_err(|e| TraceError::InvalidJson(serde_json::Error::custom(e.to_string())))
}
