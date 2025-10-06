use crate::loki::LogEntry;
use crate::prometheus::MetricsData;
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct AppState {
    pub metrics: Option<MetricsData>,
    pub logs: Vec<LogEntry>,
    pub last_update: String,
    pub last_fetch: String,  // Track when we last fetched data from server
    pub status: String,
    pub prometheus_url: String,
    pub loki_url: String,
    pub selected_log_index: Option<usize>,
    pub log_scroll_offset: usize,
    pub all_logs: Vec<LogEntry>,  // Store all logs for scrolling
    pub last_terminal_height: u16, // Track terminal height for background updates
    pub last_fetch_count: usize,   // Track how many logs we had in the last fetch
    pub has_initial_fetch: bool,   // Track if we've done the initial fetch
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            metrics: None,
            logs: Vec::new(),
            last_update: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            last_fetch: "Never".to_string(),
            status: "Initializing...".to_string(),
            prometheus_url: String::new(),
            loki_url: String::new(),
            selected_log_index: None,
            log_scroll_offset: 0,
            all_logs: Vec::new(),
            last_terminal_height: 50,
            last_fetch_count: 0,
            has_initial_fetch: false,
        }
    }
}

impl AppState {
    
    pub fn update_visible_logs_with_height(&mut self, terminal_height: u16) {
        // More accurate calculation based on actual layout
        // Fixed UI elements:
        // - Header: 3 lines
        // - Endpoints: 3 lines  
        // - Metrics: 6-12 lines (depends on terminal size)
        // - Footer: 3 lines
        // - Margins: 2 lines (top and bottom)
        // - Borders and padding: ~3 lines
        
        let metrics_height = if terminal_height < 30 {
            6
        } else if terminal_height > 50 {
            12
        } else {
            10
        };
        
        let fixed_lines = 3 + 3 + metrics_height + 3 + 2 + 3; // Total fixed UI
        let visible_height = terminal_height.saturating_sub(fixed_lines) as usize;
        let visible_height = visible_height.max(5); // At least 5 lines
        
        // Ensure scroll offset doesn't go beyond valid range
        if !self.all_logs.is_empty() {
            let max_scroll = self.all_logs.len().saturating_sub(visible_height);
            self.log_scroll_offset = self.log_scroll_offset.min(max_scroll);
            
            let end = (self.log_scroll_offset + visible_height).min(self.all_logs.len());
            self.logs = self.all_logs[self.log_scroll_offset..end].to_vec();
        } else {
            self.logs = Vec::new();
        }
    }
    
    pub fn get_visible_height(&self, terminal_height: u16) -> usize {
        let metrics_height = if terminal_height < 30 {
            6
        } else if terminal_height > 50 {
            12
        } else {
            10
        };
        
        let fixed_lines = 3 + 3 + metrics_height + 3 + 2 + 3;
        terminal_height.saturating_sub(fixed_lines) as usize
    }
}

pub fn draw_ui(frame: &mut Frame, state: &AppState) {
    let size = frame.area();
    
    // Check minimum terminal size
    if size.width < 80 || size.height < 24 {
        draw_size_warning(frame, size);
        return;
    }
    
    // Adjust layout based on terminal size
    let metrics_height = if size.height < 30 {
        6  // Smaller metrics area for small terminals
    } else if size.height > 50 {
        12 // Larger metrics area for big terminals
    } else {
        10 // Default
    };
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Endpoints
            Constraint::Length(metrics_height), // Metrics (dynamic)
            Constraint::Min(10),    // Logs (take remaining space, min 10)
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], state);
    draw_endpoints(frame, chunks[1], state);
    draw_metrics_compact(frame, chunks[2], state, size);
    draw_logs_wide(frame, chunks[3], state, size);
    draw_footer(frame, chunks[4], state);
}

fn draw_size_warning(frame: &mut Frame, size: Rect) {
    let warning = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Terminal Too Small",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Current: {}x{}", size.width, size.height)),
        Line::from("Required: 80x24 minimum"),
        Line::from(""),
        Line::from("Please resize your terminal window"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(" Warning "),
    )
    .alignment(Alignment::Center);
    
    frame.render_widget(warning, size);
}

fn draw_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let header = Paragraph::new(vec![Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "Monitoring Dashboard",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Fetch: {}", state.last_fetch),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Update: {}", state.last_update),
            Style::default().fg(Color::Gray),
        ),
    ])])
    .style(Style::default().bg(Color::Black))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

fn draw_endpoints(frame: &mut Frame, area: Rect, state: &AppState) {
    let endpoints = Paragraph::new(vec![Line::from(vec![
        Span::styled("Prometheus: ", Style::default().fg(Color::Yellow)),
        Span::raw(&state.prometheus_url),
        Span::raw(" | "),
        Span::styled("Loki: ", Style::default().fg(Color::Magenta)),
        Span::raw(&state.loki_url),
    ])])
    .style(Style::default().bg(Color::Black))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)),
    )
    .alignment(Alignment::Center);

    frame.render_widget(endpoints, area);
}

fn draw_metrics_compact(frame: &mut Frame, area: Rect, state: &AppState, _terminal_size: Rect) {
    let metrics_block = Block::default()
        .title(" Prometheus Metrics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(metrics) = &state.metrics {
        // Create horizontal layout for metrics
        let metrics_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .margin(1)
            .split(area);

        // Requests per second
        let req_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Requests/s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{:.2}", metrics.http_requests_total), Style::default().fg(Color::Green)),
            ]),
        ])
        .block(Block::default().borders(Borders::RIGHT))
        .alignment(Alignment::Center);
        
        frame.render_widget(req_widget, metrics_chunks[0]);

        // P50 Latency
        let p50_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("P50 Latency", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{:.1}ms", metrics.http_request_duration_p50 * 1000.0), Style::default().fg(Color::Green)),
            ]),
        ])
        .block(Block::default().borders(Borders::RIGHT))
        .alignment(Alignment::Center);
        
        frame.render_widget(p50_widget, metrics_chunks[1]);

        // P95 Latency
        let p95_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("P95 Latency", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{:.1}ms", metrics.http_request_duration_p95 * 1000.0), Style::default().fg(Color::Yellow)),
            ]),
        ])
        .block(Block::default().borders(Borders::RIGHT))
        .alignment(Alignment::Center);
        
        frame.render_widget(p95_widget, metrics_chunks[2]);

        // P99 Latency
        let p99_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("P99 Latency", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{:.1}ms", metrics.http_request_duration_p99 * 1000.0), Style::default().fg(Color::Red)),
            ]),
        ])
        .alignment(Alignment::Center);
        
        frame.render_widget(p99_widget, metrics_chunks[3]);
    } else {
        let no_data = Paragraph::new("No metrics data available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        frame.render_widget(no_data, area);
    }

    // Render the outer block last to draw borders over content
    frame.render_widget(metrics_block, area);
}

fn draw_logs_wide(frame: &mut Frame, area: Rect, state: &AppState, _terminal_size: Rect) {
    let help_text = if state.selected_log_index.is_some() {
        " ↑/↓: navigate | [/]: 5 lines | c: copy | ESC: deselect "
    } else {
        " ↑/↓: select & navigate | [/]: jump 5 lines "
    };
    
    // Count how many logs are marked as new
    let new_count = state.logs.iter().filter(|log| log.is_new).count();
    let title = if new_count > 0 {
        format!(" Loki Logs [{} entries, {} new] {} ", state.all_logs.len(), new_count, help_text)
    } else {
        format!(" Loki Logs [{} entries] {} ", state.all_logs.len(), help_text)
    };
    let logs_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if !state.logs.is_empty() {
        // Get the available width for log messages
        let available_width = area.width.saturating_sub(4) as usize; // Account for borders
        
        let log_items: Vec<ListItem> = state
            .logs
            .iter()
            .enumerate()
            .map(|(index, log)| {
                // Determine color based on whether log is new or not
                let level_color = match log.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Gray,
                    _ => Color::White,
                };

                // Calculate the width needed for level only (no timestamp)
                let level_str = format!("[{:5}]", log.level); // Fixed width for alignment
                let prefix_len = level_str.len() + 1; // +1 for space
                
                // Calculate available width for message
                let message_width = available_width.saturating_sub(prefix_len);
                
                // Truncate or wrap the message if needed
                let message = if log.message.len() > message_width && message_width > 20 {
                    // Add ellipsis if message is too long
                    format!("{}...", &log.message[..message_width.saturating_sub(3)])
                } else {
                    log.message.clone()
                };

                // Check if this log is selected
                let is_selected = state.selected_log_index
                    .map(|selected| selected == state.log_scroll_offset + index)
                    .unwrap_or(false);
                
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else if log.is_new {
                    // Make the entire new log line stand out with brighter text
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let content = if log.is_new {
                    // New logs: Add a special marker and highlight
                    vec![Line::from(vec![
                        Span::styled(
                            "→ ",  // Arrow indicator for new logs
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            level_str,
                            if is_selected {
                                Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            },
                        ),
                        Span::raw(" "),
                        Span::styled(
                            message,
                            if is_selected {
                                Style::default().bg(Color::DarkGray).fg(Color::White)
                            } else {
                                Style::default().fg(Color::Yellow)
                            },
                        ),
                    ])]
                } else {
                    // Normal logs
                    vec![Line::from(vec![
                        Span::raw("  "),  // Spacing to align with new logs
                        Span::styled(
                            level_str,
                            if is_selected {
                                Style::default().bg(Color::DarkGray).fg(level_color).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(level_color).add_modifier(Modifier::BOLD)
                            },
                        ),
                        Span::raw(" "),
                        Span::styled(
                            message,
                            style,
                        ),
                    ])]
                };

                ListItem::new(content)
            })
            .collect();

        let logs_list = List::new(log_items)
            .block(logs_block)
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(logs_list, area);
    } else {
        let no_logs = Paragraph::new("No logs available")
            .style(Style::default().fg(Color::Gray))
            .block(logs_block)
            .alignment(Alignment::Center);
        
        frame.render_widget(no_logs, area);
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let footer_text = vec![Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &state.status,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled("'q'", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled(" to quit | ", Style::default().fg(Color::Gray)),
        Span::styled("'r'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" to refresh", Style::default().fg(Color::Gray)),
    ])];

    let footer = Paragraph::new(footer_text)
        .style(Style::default().bg(Color::Black))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}