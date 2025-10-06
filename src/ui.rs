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
    pub active_panel: ActivePanel,  // Which panel is currently active
    pub metrics_scroll_offset: usize, // Scroll offset for metrics
    pub metrics_time_range: TimeRange, // Current time range for metrics
    pub metrics_loading: bool, // Whether metrics are currently loading
    pub expanded_log_index: Option<usize>, // Index of the log that is expanded to show full content
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    None,    // No panel is active
    Logs,
    Metrics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    OneMin,
    FiveMin,   // Default
    ThirtyMin,
    OneHour,
    OneDay,
    All,
}

impl TimeRange {
    pub fn as_str(&self) -> &str {
        match self {
            TimeRange::OneMin => "1m",
            TimeRange::FiveMin => "5m",
            TimeRange::ThirtyMin => "30m",
            TimeRange::OneHour => "1h",
            TimeRange::OneDay => "24h",
            TimeRange::All => "All",
        }
    }
    
    pub fn as_minutes(&self) -> Option<i64> {
        match self {
            TimeRange::OneMin => Some(1),
            TimeRange::FiveMin => Some(5),
            TimeRange::ThirtyMin => Some(30),
            TimeRange::OneHour => Some(60),
            TimeRange::OneDay => Some(1440),
            TimeRange::All => None,  // None means no time limit
        }
    }
    
    pub fn to_prometheus_range(&self) -> String {
        match self {
            TimeRange::OneMin => "1m".to_string(),
            TimeRange::FiveMin => "5m".to_string(),
            TimeRange::ThirtyMin => "30m".to_string(),
            TimeRange::OneHour => "1h".to_string(),
            TimeRange::OneDay => "24h".to_string(),
            TimeRange::All => "all".to_string(),  // Special case for all
        }
    }
    
    pub fn next(&self) -> TimeRange {
        match self {
            TimeRange::OneMin => TimeRange::FiveMin,
            TimeRange::FiveMin => TimeRange::ThirtyMin,
            TimeRange::ThirtyMin => TimeRange::OneHour,
            TimeRange::OneHour => TimeRange::OneDay,
            TimeRange::OneDay => TimeRange::All,
            TimeRange::All => TimeRange::OneMin,
        }
    }
    
    pub fn prev(&self) -> TimeRange {
        match self {
            TimeRange::OneMin => TimeRange::All,
            TimeRange::FiveMin => TimeRange::OneMin,
            TimeRange::ThirtyMin => TimeRange::FiveMin,
            TimeRange::OneHour => TimeRange::ThirtyMin,
            TimeRange::OneDay => TimeRange::OneHour,
            TimeRange::All => TimeRange::OneDay,
        }
    }
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
            active_panel: ActivePanel::None,  // Start with no panel active
            metrics_scroll_offset: 0,
            metrics_time_range: TimeRange::FiveMin,  // Default to 5 minutes
            metrics_loading: false,
            expanded_log_index: None,
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
            "RustDash",
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)),
    )
    .alignment(Alignment::Center);

    frame.render_widget(endpoints, area);
}

fn draw_metrics_compact(frame: &mut Frame, area: Rect, state: &AppState, _terminal_size: Rect) {
    // Calculate time range based on current setting
    let now = chrono::Local::now();
    let time_range_display = if let Some(minutes) = state.metrics_time_range.as_minutes() {
        let from_time = (now - chrono::TimeDelta::try_minutes(minutes).unwrap()).format("%H:%M:%S");
        let to_time = now.format("%H:%M:%S");
        format!("[{}] ({} → {})", state.metrics_time_range.as_str(), from_time, to_time)
    } else {
        format!("[{}] (All time)", state.metrics_time_range.as_str())
    };
    
    let (border_color, base_title, help_text) = match state.active_panel {
        ActivePanel::Metrics => (
            Color::Cyan,
            "API Response Times",
            " [↑/↓: scroll, ←/→: time range, ESC: exit] "
        ),
        ActivePanel::None => (
            Color::Gray,
            "API Response Times",
            " [TAB to activate] "
        ),
        _ => (
            Color::Yellow,
            "API Response Times",
            " [TAB to switch here] "
        ),
    };
    
    let title = format!(" {} {} {} ", base_title, time_range_display, help_text);
    
    let metrics_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    // Draw the block first
    frame.render_widget(metrics_block, area);
    
    // Create inner area for content
    let inner = ratatui::layout::Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    
    if state.metrics_loading {
        // Show loading indicator
        let loading_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "⏳ Loading metrics...",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Please wait while fetching data from Prometheus",
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ];
        
        let loading_widget = Paragraph::new(loading_text)
            .alignment(Alignment::Center);
            
        frame.render_widget(loading_widget, inner);
    } else if let Some(metrics) = &state.metrics {
        
        // Calculate dynamic column widths based on terminal width
        let available_width = inner.width as usize;
        
        // Find the longest URI to determine minimum needed width
        let max_uri_len = metrics.uri_metrics.iter()
            .map(|m| m.uri.len())
            .max()
            .unwrap_or(20)
            .min(available_width / 3); // Cap at 1/3 of terminal width
        
        // Fixed edge columns
        let uri_column_width = max_uri_len + 2; // URI on left edge with small padding
        let req_width = 8; // Reqs/min on right edge (shortened)
        let ms_width = 7;  // ms value (shortened)
        let spacing = 2;   // Small spacing between bar and numbers
        
        // Calculate middle space for bar chart
        let middle_space = available_width.saturating_sub(uri_column_width + ms_width + req_width + spacing);
        let bar_width = middle_space.max(20); // Bar chart takes all middle space, minimum 20 chars
        
        // Build lines for each URI metric
        let mut lines = Vec::new();
        
        // Add header with proper alignment matching data lines
        let uri_header = format!("{:<width$}", "URI", width = uri_column_width);
        let response_header = format!("{:^width$}", "Response Time (ms)", width = bar_width + ms_width + 1);
        let req_header = format!("{:>width$}", "Req/min", width = req_width);
        
        // Create header line with separate spans to match data line structure
        lines.push(Line::from(vec![
            Span::styled(
                uri_header,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                response_header,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                req_header,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]));
        
        // Calculate visible metrics based on area height
        let visible_count = (inner.height as usize).saturating_sub(3).min(10); // Header + footer space
        let start_idx = state.metrics_scroll_offset;
        let end_idx = (start_idx + visible_count).min(metrics.uri_metrics.len());
        
        // Find the max duration for scaling the bars
        let max_duration = metrics.uri_metrics.iter()
            .map(|m| m.avg_duration_ms)
            .fold(0.0_f64, f64::max)
            .max(1.0); // Avoid division by zero
        
        // Add each visible URI metric
        for uri_metric in &metrics.uri_metrics[start_idx..end_idx] {
            // Truncate URI if too long for the calculated width
            let display_uri = if uri_metric.uri.len() > uri_column_width - 2 {
                format!("{}...", &uri_metric.uri[..uri_column_width.saturating_sub(5)])
            } else {
                uri_metric.uri.clone()
            };
            
            // Color code based on response time
            let duration_color = if uri_metric.avg_duration_ms < 100.0 {
                Color::Green
            } else if uri_metric.avg_duration_ms < 500.0 {
                Color::Yellow
            } else {
                Color::Red
            };
            
            // Create the bar visualization
            let bar_filled = ((uri_metric.avg_duration_ms / max_duration) * bar_width as f64) as usize;
            let bar_filled = bar_filled.min(bar_width);
            
            let bar_char = match duration_color {
                Color::Green => "█",
                Color::Yellow => "█",
                Color::Red => "█",
                _ => "█",
            };
            
            let bar_string = format!("{:█<width$}", "", width = bar_filled)
                .replace(' ', bar_char);
            let bar_empty = " ".repeat(bar_width.saturating_sub(bar_filled));
            
            // Build the line with proper spacing
            let uri_part = format!("{:<width$}", display_uri, width = uri_column_width);
            let ms_part = format!("{:>width$.1}", uri_metric.avg_duration_ms, width = ms_width);
            let req_part = format!("{:>width$.0}", uri_metric.request_count, width = req_width);
            
            let line_spans = vec![
                Span::styled(
                    uri_part,
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    bar_string,
                    Style::default().fg(duration_color),
                ),
                Span::styled(
                    bar_empty,
                    Style::default(),
                ),
                Span::styled(
                    format!(" {}", ms_part),
                    Style::default().fg(duration_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    req_part,
                    Style::default().fg(Color::Cyan),
                ),
            ];
            
            lines.push(Line::from(line_spans));
        }
        
        // Add total requests/sec and scale info at the bottom
        if !lines.is_empty() {
            lines.push(Line::from(""));
            
            // Add scale and period info
            let scale_text = format!("Scale: █ = {:.0}ms", max_duration);
            let period_text = match state.metrics_time_range {
                TimeRange::OneMin => "1-minute average",
                TimeRange::FiveMin => "5-minute average",
                TimeRange::ThirtyMin => "30-minute average",
                TimeRange::OneHour => "1-hour average",
                TimeRange::OneDay => "24-hour average",
                TimeRange::All => "All-time average",
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("Total Req/s: {:.2}  |  {}  |  {}", 
                        metrics.http_requests_total, scale_text, period_text),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }
        
        // If no URI metrics but we have total
        if metrics.uri_metrics.is_empty() && metrics.http_requests_total > 0.0 {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "No per-URI metrics available",
                    Style::default().fg(Color::Gray),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("Total Requests/sec: {:.2}", metrics.http_requests_total),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
        
        let metrics_content = Paragraph::new(lines)
            .alignment(Alignment::Left);
        
        frame.render_widget(metrics_content, inner);
    } else {
        let no_data = Paragraph::new("No metrics data available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        frame.render_widget(no_data, inner);
    }
}

fn draw_logs_wide(frame: &mut Frame, area: Rect, state: &AppState, _terminal_size: Rect) {
    let (border_color, help_text) = match state.active_panel {
        ActivePanel::Logs => {
            let text = if state.selected_log_index.is_some() {
                " ↑/↓: navigate | Enter: expand/collapse | [/]: 5 lines | c: copy | ESC: exit "
            } else {
                " ↑/↓: select & navigate | [/]: jump 5 lines | ESC: deactivate panel "
            };
            (Color::Cyan, text)
        },
        ActivePanel::None => (
            Color::Gray,
            " TAB: activate this panel "
        ),
        _ => (
            Color::Magenta,
            " TAB: switch to this panel "
        ),
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
        .border_style(Style::default().fg(border_color));

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
                
                // Check if this log is selected
                let is_selected = state.selected_log_index
                    .map(|selected| selected == state.log_scroll_offset + index)
                    .unwrap_or(false);
                
                // Check if this log is expanded
                let is_expanded = state.expanded_log_index
                    .map(|expanded| expanded == state.log_scroll_offset + index)
                    .unwrap_or(false);
                
                // Handle message display based on expanded state
                let (message_lines, is_truncated) = if is_expanded {
                    // Show full message, wrapped across multiple lines
                    let mut lines = Vec::new();
                    let mut remaining = log.message.as_str();
                    
                    while !remaining.is_empty() {
                        if remaining.len() <= message_width {
                            lines.push(remaining.to_string());
                            break;
                        } else {
                            // Find a good break point (space, if possible)
                            let mut break_point = message_width;
                            if let Some(space_pos) = remaining[..message_width].rfind(' ') {
                                if space_pos > message_width / 2 {
                                    break_point = space_pos;
                                }
                            }
                            lines.push(remaining[..break_point].to_string());
                            remaining = remaining[break_point..].trim_start();
                        }
                    }
                    (lines, false)
                } else {
                    // Normal display - truncate if needed
                    if log.message.len() > message_width && message_width > 20 {
                        // Add ellipsis if message is too long
                        (vec![format!("{}...", &log.message[..message_width.saturating_sub(3)])], true)
                    } else {
                        (vec![log.message.clone()], false)
                    }
                };
                
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else if log.is_new {
                    // Make the entire new log line stand out with brighter text
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let mut content = Vec::new();
                
                // Build the first line with level and first message line
                if let Some(first_message) = message_lines.first() {
                    if log.is_new {
                        // New logs: Add a special marker and highlight
                        let mut line_spans = vec![
                            Span::styled(
                                "→ ",  // Arrow indicator for new logs
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                level_str.clone(),
                                if is_selected {
                                    Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                },
                            ),
                            Span::raw(" "),
                            Span::styled(
                                first_message.clone(),
                                if is_selected {
                                    Style::default().bg(Color::DarkGray).fg(Color::White)
                                } else {
                                    Style::default().fg(Color::Yellow)
                                },
                            ),
                        ];
                        
                        // Add expand/collapse indicator if truncated or expanded
                        if is_expanded {
                            line_spans.push(Span::styled(" ▼", Style::default().fg(Color::Cyan)));
                        } else if is_truncated {
                            line_spans.push(Span::styled(" ▶", Style::default().fg(Color::Cyan)));
                        }
                        
                        content.push(Line::from(line_spans));
                    } else {
                        // Normal logs
                        let mut line_spans = vec![
                            Span::raw("  "),  // Spacing to align with new logs
                            Span::styled(
                                level_str.clone(),
                                if is_selected {
                                    Style::default().bg(Color::DarkGray).fg(level_color).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(level_color).add_modifier(Modifier::BOLD)
                                },
                            ),
                            Span::raw(" "),
                            Span::styled(
                                first_message.clone(),
                                style,
                            ),
                        ];
                        
                        // Add expand/collapse indicator if truncated or expanded
                        if is_expanded {
                            line_spans.push(Span::styled(" ▼", Style::default().fg(Color::Cyan)));
                        } else if is_truncated {
                            line_spans.push(Span::styled(" ▶", Style::default().fg(Color::Cyan)));
                        }
                        
                        content.push(Line::from(line_spans));
                    }
                    
                    // Add continuation lines if expanded
                    if is_expanded && message_lines.len() > 1 {
                        for continuation_line in &message_lines[1..] {
                            // Add indentation to align with the message part
                            let indent = " ".repeat(prefix_len + 2); // +2 for the arrow/spacing
                            content.push(Line::from(vec![
                                Span::raw(indent),
                                Span::styled(
                                    continuation_line.clone(),
                                    if is_selected {
                                        Style::default().bg(Color::DarkGray).fg(Color::White)
                                    } else if log.is_new {
                                        Style::default().fg(Color::Yellow)
                                    } else {
                                        style
                                    },
                                ),
                            ]));
                        }
                    }
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}