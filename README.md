# Monitoring Dashboard

Real-time terminal-based monitoring dashboard for Prometheus and Loki.

## Quick Start

```bash
# Test if Prometheus and Loki are running
./test_endpoints.sh

# Build
cargo build --release

# Run with defaults (http://localhost:9090 for Prometheus, http://localhost:3100 for Loki)
cargo run

# Run with custom endpoints
MONITOR_PROMETHEUS_BASE_URL=http://prometheus:9090 \
MONITOR_LOKI_BASE_URL=http://loki:3100 \
cargo run
```

## Features

### Core Features
- **Real-time metrics from Prometheus**
  - HTTP requests per second
  - P50, P95, P99 latency percentiles
  - Auto-fallback to test data if unavailable

- **Live log streaming from Loki**
  - Automatic log fetching every 5 seconds
  - Displays logs in chronological order (newest at bottom)
  - Smart label detection for fontory service

- **Responsive Terminal UI**
  - Adaptive layout for different terminal sizes
  - Minimum terminal size: 80x24
  - Dynamic metrics panel sizing based on terminal height

### Log Navigation & Management
- **Keyboard Navigation**
  - `↑/↓` - Navigate through logs line by line
  - `[/]` - Jump 5 lines up/down quickly
  - `ESC` - Deselect current log
  - Selection highlighting with gray background

- **New Log Highlighting**
  - New logs marked with yellow arrow (→) indicator
  - Highlights persist until newer logs arrive
  - Auto-scroll to show new logs (disabled when selecting)

- **Clipboard Support**
  - `c` - Copy selected log to system clipboard
  - Format: `[LEVEL] message`

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

### Log Navigation
- `↑` - Move selection up / Start selection at newest log
- `↓` - Move selection down / Start selection at newest log  
- `[` - Jump 5 lines up
- `]` - Jump 5 lines down
- `ESC` - Clear selection
- `c` - Copy selected log to clipboard

## Configuration

Three ways to configure:

1. **Environment Variables** (recommended for Docker)
```bash
export MONITOR_PROMETHEUS_BASE_URL=http://localhost:9090
export MONITOR_LOKI_BASE_URL=http://localhost:3100
```

2. **Config File** (`config.toml`)
```toml
[prometheus]
base_url = "http://localhost:9090"

[loki]  
base_url = "http://localhost:3100"
```

3. **Default Values**
- Prometheus: `http://localhost:9090`
- Loki: `http://localhost:3100`

## Requirements

- Rust 1.70+
- Running Prometheus instance (optional - will show test data if unavailable)
- Running Loki instance (optional - will show test data if unavailable)

## Troubleshooting

If the dashboard shows no data:
1. Run `./test_endpoints.sh` to check if services are accessible
2. Check that Prometheus/Loki are running: `docker ps`
3. Verify the URLs are correct in your configuration
4. The dashboard will show test data if services are unavailable