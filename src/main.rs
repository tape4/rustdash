mod config;
mod loki;
mod prometheus;
mod ui;

use anyhow::Result;
use chrono::Local;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;
use config::Settings;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use loki::LokiClient;
use prometheus::PrometheusClient;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io::{self, stdout},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, time};
use ui::{draw_ui, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let settings = Settings::new().unwrap_or_else(|_| {
        eprintln!("Warning: Could not load config file, using defaults and environment variables");
        Settings::from_env().unwrap_or_default()
    });

    let prometheus_client = PrometheusClient::new(settings.prometheus.base_url.clone());
    let loki_client = LokiClient::new(settings.loki.base_url.clone());

    let mut initial_state = AppState::default();
    initial_state.prometheus_url = settings.prometheus.base_url.clone();
    initial_state.loki_url = settings.loki.base_url.clone();
    
    let app_state = Arc::new(Mutex::new(initial_state));

    setup_terminal()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let app_state_clone = app_state.clone();
    let settings_clone = settings.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(
            settings_clone.ui.refresh_interval_seconds,
        ));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        
        let mut first_run = true;
        
        loop {
            // Skip the first tick to execute immediately
            if !first_run {
                interval.tick().await;
            } else {
                first_run = false;
            }
            
            // Fetch metrics
            let metrics = prometheus_client.get_metrics().await.ok();
            
            // Fetch logs directly (get all logs)
            let all_logs = if let Ok(fetched_logs) = loki_client
                .get_recent_logs(settings_clone.loki.log_limit)
                .await
            {
                fetched_logs
            } else {
                Vec::new()
            };
            
            // Update state while preserving scroll position
            let mut state = app_state_clone.lock().await;
            state.metrics = metrics;
            
            // Preserve scroll position and selection when updating logs
            let old_scroll_offset = state.log_scroll_offset;
            let old_selected_index = state.selected_log_index;
            let terminal_height = state.last_terminal_height;
            
            // Keep track of previous state before processing
            let old_logs = state.all_logs.clone();
            let old_fetch_count = state.last_fetch_count;
            
            // Process new logs
            let mut marked_logs = all_logs;
            let new_count = marked_logs.len();
            
            // Initialize all logs as not new
            for i in 0..new_count {
                marked_logs[i].is_new = false;
            }
            
            // Handle different cases
            if !state.has_initial_fetch {
                // First fetch - don't highlight anything
                state.has_initial_fetch = true;
                state.last_fetch_count = new_count;
                state.status = format!("Connected - Initial: {} logs", new_count);
            } else if new_count > old_fetch_count {
                // New logs detected! Highlight only the new ones
                let new_log_count = new_count - old_fetch_count;
                
                // Mark only the NEW logs (at the end of the list)
                for i in old_fetch_count..new_count {
                    marked_logs[i].is_new = true;
                }
                
                // Clear old highlights when we get truly new logs
                state.last_fetch_count = new_count;
                state.status = format!("Connected - {} new logs!", new_log_count);
            } else {
                // Same count - preserve existing highlights
                let mut preserved = 0;
                
                // Copy highlight status from old logs if they match
                for i in 0..new_count.min(old_logs.len()) {
                    if i < old_logs.len() && old_logs[i].is_new {
                        // Same position, same message = preserve highlight
                        if marked_logs[i].message == old_logs[i].message {
                            marked_logs[i].is_new = true;
                            preserved += 1;
                        }
                    }
                }
                
                state.last_fetch_count = new_count;
                if preserved > 0 {
                    state.status = format!("Connected - {} logs highlighted", preserved);
                } else {
                    state.status = "Connected".to_string();
                }
            }
            
            // Check if we had new logs
            let had_new_logs = new_count > old_fetch_count;
            
            state.all_logs = marked_logs;
            state.last_fetch = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            
            // Handle scrolling
            let is_first_load = old_scroll_offset == 0 && old_selected_index.is_none() && state.logs.is_empty();
            
            if is_first_load && !state.all_logs.is_empty() {
                // First load - scroll to bottom to show latest logs
                let visible_height = state.get_visible_height(terminal_height);
                let last_idx = state.all_logs.len().saturating_sub(1);
                state.log_scroll_offset = last_idx.saturating_sub(visible_height - 1);
            } else if had_new_logs && old_selected_index.is_none() {
                // New logs arrived and user isn't selecting - auto-scroll to show them
                let visible_height = state.get_visible_height(terminal_height);
                if state.all_logs.len() > visible_height {
                    state.log_scroll_offset = state.all_logs.len() - visible_height;
                } else {
                    state.log_scroll_offset = 0;
                }
            } else if old_selected_index.is_some() {
                // User has selected something, preserve their position
                state.log_scroll_offset = old_scroll_offset;
                state.selected_log_index = old_selected_index;
                
                // Validate that the selection is still in bounds
                if let Some(idx) = state.selected_log_index {
                    if idx >= state.all_logs.len() {
                        state.selected_log_index = Some(state.all_logs.len().saturating_sub(1));
                    }
                }
            } else {
                // No new logs, no selection - keep current position
                state.log_scroll_offset = old_scroll_offset;
            }
            
            // Update visible logs with the last known terminal height
            state.update_visible_logs_with_height(terminal_height);
        }
    });

    let res = run_app(&mut terminal, app_state.clone(), settings).await;

    restore_terminal()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app_state: Arc<Mutex<AppState>>,
    settings: Settings,
) -> io::Result<()> {
    let _prometheus_client = PrometheusClient::new(settings.prometheus.base_url.clone());
    let _loki_client = LokiClient::new(settings.loki.base_url.clone());

    loop {
        // Get current terminal size
        let terminal_size = terminal.size()?;
        
        // Update terminal height in state for background task
        {
            let mut state = app_state.lock().await;
            state.last_terminal_height = terminal_size.height;
        }
        
        let state = app_state.lock().await;
        terminal.draw(|f| draw_ui(f, &state))?;
        drop(state);

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut state = app_state.lock().await;
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('r') => {
                            state.status = "Manual refresh triggered".to_string();
                            state.last_update = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        }
                        KeyCode::Up => {
                            // Move selection up
                            if state.all_logs.is_empty() {
                                continue;
                            }
                            
                            match state.selected_log_index {
                                None => {
                                    // Start selection from bottom (newest log)
                                    let last_idx = state.all_logs.len() - 1;
                                    state.selected_log_index = Some(last_idx);
                                    // Scroll to show the last log
                                    let visible_height = state.get_visible_height(terminal_size.height);
                                    if state.all_logs.len() > visible_height {
                                        state.log_scroll_offset = state.all_logs.len() - visible_height;
                                    } else {
                                        state.log_scroll_offset = 0;
                                    }
                                }
                                Some(idx) if idx > 0 => {
                                    state.selected_log_index = Some(idx - 1);
                                    // Adjust scroll if needed
                                    if idx - 1 < state.log_scroll_offset {
                                        state.log_scroll_offset = idx - 1;
                                    }
                                }
                                _ => {}
                            }
                            state.update_visible_logs_with_height(terminal_size.height);
                        }
                        KeyCode::Down => {
                            // Move selection down
                            if state.all_logs.is_empty() {
                                continue;
                            }
                            
                            match state.selected_log_index {
                                None => {
                                    // Start selection from bottom (newest log)
                                    let last_idx = state.all_logs.len() - 1;
                                    state.selected_log_index = Some(last_idx);
                                    // Scroll to show the last log
                                    let visible_height = state.get_visible_height(terminal_size.height);
                                    if state.all_logs.len() > visible_height {
                                        state.log_scroll_offset = state.all_logs.len() - visible_height;
                                    } else {
                                        state.log_scroll_offset = 0;
                                    }
                                }
                                Some(idx) if idx < state.all_logs.len() - 1 => {
                                    state.selected_log_index = Some(idx + 1);
                                    // Adjust scroll if needed
                                    let visible_height = state.get_visible_height(terminal_size.height);
                                    // Check if the new selection is below the visible area
                                    if idx + 1 >= state.log_scroll_offset + visible_height {
                                        // Scroll down to show the selected item at the bottom of the visible area
                                        state.log_scroll_offset = (idx + 2).saturating_sub(visible_height);
                                    }
                                }
                                _ => {}
                            }
                            state.update_visible_logs_with_height(terminal_size.height);
                        }
                        KeyCode::Char('[') => {
                            // Move up 5 lines
                            if let Some(idx) = state.selected_log_index {
                                let new_idx = idx.saturating_sub(5);
                                state.selected_log_index = Some(new_idx);
                                if new_idx < state.log_scroll_offset {
                                    state.log_scroll_offset = new_idx;
                                }
                                state.update_visible_logs_with_height(terminal_size.height);
                            } else if !state.all_logs.is_empty() {
                                // If no selection, start from bottom (newest)
                                let start_idx = state.all_logs.len().saturating_sub(1);
                                state.selected_log_index = Some(start_idx);
                                // Scroll to show the last log
                                let visible_height = state.get_visible_height(terminal_size.height);
                                if state.all_logs.len() > visible_height {
                                    state.log_scroll_offset = state.all_logs.len() - visible_height;
                                } else {
                                    state.log_scroll_offset = 0;
                                }
                                state.update_visible_logs_with_height(terminal_size.height);
                            }
                        }
                        KeyCode::Char(']') => {
                            // Move down 5 lines
                            if let Some(idx) = state.selected_log_index {
                                let new_idx = (idx + 5).min(state.all_logs.len().saturating_sub(1));
                                state.selected_log_index = Some(new_idx);
                                let visible_height = state.get_visible_height(terminal_size.height);
                                if new_idx >= state.log_scroll_offset + visible_height {
                                    state.log_scroll_offset = new_idx.saturating_sub(visible_height - 1);
                                }
                                state.update_visible_logs_with_height(terminal_size.height);
                            } else if !state.all_logs.is_empty() {
                                // If no selection, start from bottom (newest)
                                let start_idx = state.all_logs.len().saturating_sub(1);
                                state.selected_log_index = Some(start_idx);
                                // Scroll to show the last log
                                let visible_height = state.get_visible_height(terminal_size.height);
                                if state.all_logs.len() > visible_height {
                                    state.log_scroll_offset = state.all_logs.len() - visible_height;
                                } else {
                                    state.log_scroll_offset = 0;
                                }
                                state.update_visible_logs_with_height(terminal_size.height);
                            }
                        }
                        KeyCode::Esc => {
                            // Deselect
                            state.selected_log_index = None;
                        }
                        KeyCode::Char('c') => {
                            // Copy selected log to clipboard
                            if let Some(idx) = state.selected_log_index {
                                if let Some(log) = state.all_logs.get(idx) {
                                    let log_text = format!("[{}] {}", 
                                        log.level, log.message);
                                    
                                    // Actually copy to system clipboard
                                    match ClipboardContext::new() {
                                        Ok(mut ctx) => {
                                            match ctx.set_contents(log_text.clone()) {
                                                Ok(_) => {
                                                    state.status = format!("Log copied to clipboard ({}...)", 
                                                        &log_text.chars().take(30).collect::<String>());
                                                }
                                                Err(e) => {
                                                    state.status = format!("Failed to copy: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            state.status = format!("Clipboard unavailable: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
