# RustDash
🦀 Real-time terminal-based monitoring dashboard for Prometheus and Loki, built with Rust.

<img width="1039" height="847" alt="Screenshot 2025-10-06 at 13 10 49" src="https://github.com/user-attachments/assets/968869c3-f153-4974-ad49-d64b7a5ec957" />

## Installation

```bash
# Clone repository
git clone https://github.com/yourusername/rustdash
cd rustdash

# Build
cargo build --release

# Run
cargo run
```

## Configuration

When you start the application, it will prompt you for endpoints:
```
=== RustDash Configuration ===
Press Enter to use default values.

Enter Prometheus URL [default: http://localhost:9090]: 
Enter Loki URL [default: http://localhost:3100]: 
```

Just press Enter to use the default localhost endpoints.

## Quick Example

### Using with Docker services
```bash
# Start Prometheus and Loki using Docker
docker run -d -p 9090:9090 prom/prometheus
docker run -d -p 3100:3100 grafana/loki

# Run the dashboard (press Enter for defaults)
cargo run
```

### Using with custom endpoints
```bash
cargo run
# Enter your Prometheus URL: http://metrics.example.com:9090
# Enter your Loki URL: http://logs.example.com:3100
```

## Features

### Core Features
- **Real-time metrics from Prometheus**
  - HTTP requests per second
  - URI-based average response times
  - Time range selector (1m, 5m, 30m, 1h, 24h, All)
  - Bar chart visualization for response times

- **Live log streaming from Loki**
  - Automatic log fetching every 5 seconds
  - Displays logs in chronological order (newest at bottom)
  - Auto-detects available log streams

- **Responsive Terminal UI**
  - Adaptive layout for different terminal sizes
  - Minimum terminal size: 80x24
  - Dynamic metrics panel sizing based on terminal height

### Panel Navigation
- **Tab Navigation**
  - `Tab` - Switch between Logs and Metrics panels
  - `ESC` - Deactivate current panel (neutral state)
  - Active panel highlighted with cyan border

### Log Navigation & Management
- **Keyboard Navigation (when Logs panel active)**
  - `↑/↓` - Navigate through logs line by line
  - `Enter` - Expand/collapse long log messages
  - `[/]` - Jump 5 lines up/down quickly
  - `Page Up/Down` - Navigate by pages
  - `Home/End` - Go to first/last log
  - Selection highlighting with gray background

- **Log Expansion Feature**
  - Long logs show `▶` indicator when truncated
  - Press `Enter` to expand and see full message across multiple lines
  - Expanded logs show `▼` indicator
  - Press `Enter` again to collapse back to single line
  - Word-wrap for better readability when expanded

- **New Log Highlighting**
  - New logs marked with yellow arrow (→) indicator
  - Highlights persist until newer logs arrive
  - Auto-scroll to show new logs (disabled when selecting)

- **Clipboard Support**
  - `c` - Copy selected log to system clipboard
  - Format: `[LEVEL] message`

### Metrics Navigation
- **Time Range Selection (when Metrics panel active)**
  - `←/→` - Change time range (cycles through 1m → 5m → 30m → 1h → 24h → All)
  - `↑/↓` - Scroll through URI metrics list
  - Loading indicator shows when fetching new data

### Display Information
- **Header Section**
  - Current endpoints (Prometheus & Loki URLs)
  - Last fetch time (when data was retrieved from servers)
  - Last update time (when UI was refreshed)
  - Connection status and new log count

## Controls

### Basic Controls
- `q` - Quit application
- `r` - Manual refresh
- `Tab` - Switch between panels
- `ESC` - Deactivate current panel

### Log Panel (when active)
- `↑/↓` - Navigate logs
- `Enter` - Expand/collapse long log messages (shows ▶ when collapsed, ▼ when expanded)
- `[/]` - Jump 5 lines up/down
- `Page Up/Down` - Navigate by pages
- `Home/End` - Go to first/last log
- `c` - Copy selected log to clipboard

### Metrics Panel (when active)
- `←/→` - Change time range
- `↑/↓` - Scroll metrics (if list is long)

## Configuration

The application prompts for configuration on startup:

**Default Values** (just press Enter to use)
- Prometheus: `http://localhost:9090`
- Loki: `http://localhost:3100`

**Custom Endpoints**
- Enter your custom URLs when prompted
- Example: `http://prometheus.example.com:9090`

## Requirements

- Rust 1.70 or higher
- Terminal with minimum size 80x24
- (Optional) Running Prometheus instance
- (Optional) Running Loki instance

Note: The dashboard will display "No data available" if services are not accessible.

## Troubleshooting

If the dashboard shows no data:
1. Check that Prometheus/Loki are running
2. Verify the URLs are correct when prompted
3. Ensure the services are accessible from your machine
4. Check firewall settings if using remote endpoints
