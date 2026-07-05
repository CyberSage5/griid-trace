use crate::index::MemoryIndex;
use crate::trace::TraceEvent;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Table,
    Flamegraph,
    Detail,
}

pub struct App {
    pub index: MemoryIndex,
    pub selected_index: usize,
    pub view_mode: ViewMode,
    pub search_query: String,
    pub filter_status: Option<String>,
    pub should_quit: bool,
    pub show_help: bool,
    pub scroll_offset: usize,
    pub search_mode: bool,
    pub input_mode: bool,
    pub watch_mode: bool,
    pub trace_path: Option<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        Self {
            index: MemoryIndex::new(),
            selected_index: 0,
            view_mode: ViewMode::Table,
            search_query: String::new(),
            filter_status: None,
            should_quit: false,
            show_help: false,
            scroll_offset: 0,
            search_mode: false,
            input_mode: false,
            watch_mode: false,
            trace_path: None,
        }
    }
    
    pub fn with_events(events: Vec<TraceEvent>) -> Self {
        let mut app = Self::new();
        app.index.add_events(events);
        app
    }
    
    pub fn with_events_paged(events: Vec<TraceEvent>, page_size: usize) -> Self {
        let mut app = Self::new();
        app.index = MemoryIndex::with_page_size(page_size);
        app.index.add_events(events);
        app
    }
    
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        self.run_with_watch(terminal, None)
    }

    pub fn run_with_watch<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        watch_rx: Option<Receiver<TraceEvent>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tick_rate = Duration::from_millis(16);

        loop {
            if let Some(rx) = &watch_rx {
                while let Ok(event) = rx.try_recv() {
                    self.index.add_event(event);
                }
            }

            terminal.draw(|f| self.draw(f))?;

            if self.should_quit {
                return Ok(());
            }

            if crossterm::event::poll(tick_rate)? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    self.handle_key(key);
                }
            }
        }
    }

    fn reload_trace(&mut self) {
        if let Some(path) = &self.trace_path {
            if let Ok(trace) = TraceEvent::stream_parse(path) {
                self.index.clear();
                self.index.add_events(trace.events);
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
        }
    }
    
    pub fn draw(&self, f: &mut Frame) {
        let size = f.size();
        
        if self.show_help {
            self.draw_help(f, size);
        } else {
            match self.view_mode {
                ViewMode::Table => self.draw_table(f, size),
                ViewMode::Flamegraph => self.draw_flamegraph(f, size),
                ViewMode::Detail => self.draw_detail(f, size),
            }
        }
        
        self.draw_status_bar(f, size);
    }
    
    fn draw_table(&self, f: &mut Frame, size: Rect) {
        let events = self.get_filtered_events();
        
        let title = if self.search_mode {
            format!("Search: {}", self.search_query)
        } else {
            "Spans".to_string()
        };
        
        let block = Block::default()
            .title(Span::styled(title, Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL);
        
        let mut lines = Vec::new();
        
        if self.search_mode {
            lines.push(Line::from("Type to search, Enter to apply, Esc to cancel"));
            lines.push(Line::from(""));
        }
        
        for (i, event) in events.iter().enumerate() {
            let style = if i == self.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                match event.status.as_str() {
                    "ok" => Style::default().fg(Color::Green),
                    "error" => Style::default().fg(Color::Red),
                    "in_progress" => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                }
            };
            
            let line = Line::from(vec![
                Span::styled(format!("{:3} ", i), style),
                Span::styled(&event.name, style),
                Span::styled(" | ", Style::default()),
                Span::styled(&event.status, style),
                Span::styled(" | ", Style::default()),
                Span::styled(
                    format!("{:.2}ms", event.latency_ms().unwrap_or(0.0)),
                    style,
                ),
            ]);
            lines.push(line);
        }
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((self.scroll_offset as u16, 0));
        
        f.render_widget(paragraph, size);
    }
    
    fn draw_flamegraph(&self, f: &mut Frame, size: Rect) {
        let block = Block::default()
            .title(Span::styled("Flamegraph", Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL);
        
        let events = self.get_filtered_events();
        if events.is_empty() {
            let text = vec![Line::from("No events to display")];
            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, size);
            return;
        }
        
        // Build a simple flamegraph visualization
        let mut lines = Vec::new();
        
        // Calculate time range
        let start_time = events.first().unwrap().ts;
        let end_time = events.last().unwrap().ts;
        let total_duration = (end_time - start_time).num_milliseconds() as f64;
        
        if total_duration == 0.0 {
            let text = vec![Line::from("Events have no time span")];
            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, size);
            return;
        }
        
        // Header
        lines.push(Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}ms", total_duration), Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(""));
        
        // Draw spans as horizontal bars
        let available_width = (size.width as usize).saturating(4);
        
        for event in events.iter().take(20) { // Limit to 20 for TUI
            let event_start = (event.ts - start_time).num_milliseconds() as f64;
            let event_duration = event.latency_ms().unwrap_or(10.0);
            
            let x_start = ((event_start / total_duration) * available_width as f64) as usize;
            let width = ((event_duration / total_duration) * available_width as f64) as usize;
            
            let color = match event.status.as_str() {
                "ok" => Color::Green,
                "error" => Color::Red,
                "in_progress" => Color::Yellow,
                _ => Color::Blue,
            };
            
            let mut bar = String::new();
            for i in 0..available_width {
                if i >= x_start && i < x_start + width {
                    bar.push('█');
                } else {
                    bar.push(' ');
                }
            }
            
            lines.push(Line::from(vec![
                Span::styled(&event.name, Style::default().fg(color)),
                Span::styled(" ", Style::default()),
                Span::styled(bar, Style::default().fg(color)),
            ]));
        }
        
        if events.len() > 20 {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("... and {} more events", events.len() - 20)));
        }
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((self.scroll_offset as u16, 0));
        
        f.render_widget(paragraph, size);
    }
    
    fn draw_detail(&self, f: &mut Frame, size: Rect) {
        let events = self.get_filtered_events();
        
        if let Some(event) = events.get(self.selected_index) {
            let block = Block::default()
                .title(Span::styled("Details", Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL);
            
            let json = serde_json::to_string_pretty(event).unwrap_or_else(|_| "Invalid JSON".to_string());
            let lines: Vec<Line> = json.lines().map(Line::from).collect();
            
            let paragraph = Paragraph::new(lines)
                .block(block)
                .scroll((self.scroll_offset as u16, 0));
            
            f.render_widget(paragraph, size);
        } else {
            let block = Block::default()
                .title(Span::styled("Details", Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL);
            
            let paragraph = Paragraph::new("No event selected").block(block);
            f.render_widget(paragraph, size);
        }
    }
    
    fn draw_help(&self, f: &mut Frame, size: Rect) {
        let block = Block::default()
            .title(Span::styled("Help", Style::default().fg(Color::Cyan)))
            .borders(Borders::ALL);
        
        let help_text = vec![
            Line::from("Global Keys:"),
            Line::from("  q - Quit"),
            Line::from("  ? - Toggle help"),
            Line::from("  Tab - Cycle views"),
            Line::from("  / - Search"),
            Line::from("  r - Refresh / re-index"),
            Line::from("  Ctrl+W - Toggle watch indicator"),
            Line::from(""),
            Line::from("Table View:"),
            Line::from("  ↑/↓ - Navigate"),
            Line::from("  Enter - View details"),
            Line::from(""),
            Line::from("Press ? to close"),
        ];
        
        let paragraph = Paragraph::new(help_text).block(block);
        f.render_widget(paragraph, size);
    }
    
    fn draw_status_bar(&self, f: &mut Frame, size: Rect) {
        let watch_label = if self.watch_mode { " | LIVE" } else { "" };
        let status = format!(
            "Events: {} | Selected: {} | View: {:?} | Filter: {}{}",
            self.index.len(),
            self.selected_index,
            self.view_mode,
            self.filter_status.as_deref().unwrap_or("None"),
            watch_label,
        );
        
        let block = Block::default()
            .title(Span::styled(status, Style::default().fg(Color::Gray)))
            .borders(Borders::TOP);
        
        let paragraph = Paragraph::new("").block(block);
        let status_size = Rect {
            x: size.x,
            y: size.bottom() - 3,
            width: size.width,
            height: 3,
        };
        
        f.render_widget(paragraph, status_size);
    }
    
    fn get_filtered_events(&self) -> Vec<&TraceEvent> {
        let mut events = self.index.get_events();
        
        if let Some(status) = &self.filter_status {
            events = events.iter().filter(|e| e.status == *status).copied().collect();
        }
        
        if !self.search_query.is_empty() {
            events = events
                .iter()
                .filter(|e| {
                    e.name.contains(&self.search_query)
                        || e.span_id.contains(&self.search_query)
                })
                .copied()
                .collect();
        }
        
        events
    }
    
    pub fn select_next(&mut self) {
        let events = self.get_filtered_events();
        if !events.is_empty() && self.selected_index < events.len() - 1 {
            self.selected_index += 1;
        }
    }
    
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
    
    pub fn cycle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Table => ViewMode::Flamegraph,
            ViewMode::Flamegraph => ViewMode::Detail,
            ViewMode::Detail => ViewMode::Table,
        };
    }
    
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.search_mode {
            self.handle_search_key(key);
            return;
        }
        
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit();
            }
            KeyCode::Char('?') => {
                self.toggle_help();
            }
            KeyCode::Tab => {
                self.cycle_view();
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
            }
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                self.show_help = false;
            }
            KeyCode::Char('r') => {
                self.reload_trace();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.watch_mode = !self.watch_mode;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
            }
            KeyCode::Char('g') => {
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                let events = self.get_filtered_events();
                if !events.is_empty() {
                    self.selected_index = events.len() - 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.view_mode == ViewMode::Table {
                    self.view_mode = ViewMode::Detail;
                }
            }
            KeyCode::Char('f') => {
                // Toggle filter - cycle through statuses
                self.filter_status = match self.filter_status.as_deref() {
                    None => Some("ok".to_string()),
                    Some("ok") => Some("error".to_string()),
                    Some("error") => Some("in_progress".to_string()),
                    _ => None,
                };
                self.selected_index = 0;
            }
            KeyCode::Char('c') => {
                // Collapse/expand - placeholder
            }
            _ => {}
        }
    }
    
    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.search_mode = false;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
