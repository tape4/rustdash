use anyhow::Result;
use chrono::DateTime;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct LokiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct LokiResponse {
    #[allow(dead_code)]
    pub status: String,
    pub data: LokiData,
}

#[derive(Debug, Deserialize)]
pub struct LokiData {
    #[serde(rename = "resultType")]
    #[allow(dead_code)]
    pub result_type: String,
    pub result: Vec<LokiStream>,
    #[allow(dead_code)]
    pub stats: Option<LokiStats>,
}

#[derive(Debug, Deserialize)]
pub struct LokiStream {
    #[allow(dead_code)]
    pub stream: serde_json::Value,
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
pub struct LokiStats {
    #[allow(dead_code)]
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    #[allow(dead_code)]
    pub timestamp: String,
    pub message: String,
    pub level: String,
    pub is_new: bool,  // Flag to indicate if this log is newly added in the current update
}

impl LokiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    #[allow(dead_code)]
    pub async fn query_range(
        &self,
        query: &str,
        start: &str,
        end: &str,
        limit: u32,
    ) -> Result<LokiResponse> {
        let url = format!("{}/loki/api/v1/query_range", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", start),
                ("end", end),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?
            .json::<LokiResponse>()
            .await?;
        
        Ok(response)
    }

    pub async fn tail(&self, query: &str, limit: u32) -> Result<Vec<LogEntry>> {
        // Use query_range instead of query for log queries
        let url = format!("{}/loki/api/v1/query_range", self.base_url);
        
        // Get logs up to current time, with a smaller time window for more recent logs
        let end = chrono::Utc::now() + chrono::Duration::minutes(1); // Add buffer for clock skew
        let start = end - chrono::Duration::minutes(30); // Look back only 30 minutes for recent logs
        
        let response = self
            .client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", &start.timestamp_nanos_opt().unwrap_or(0).to_string()),
                ("end", &end.timestamp_nanos_opt().unwrap_or(0).to_string()),
                ("limit", &limit.to_string()),
                ("direction", "backward"),  // Get newest logs first, then reverse
            ])
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Loki query failed: {}", error_text));
        }
        
        let loki_response = response.json::<LokiResponse>().await?;
        
        let mut logs = Vec::new();
        
        for stream in loki_response.data.result {
            for (timestamp_str, message) in stream.values {
                let level = self.extract_log_level(&message);
                let timestamp = timestamp_str.parse::<i64>().unwrap_or(0);
                logs.push((timestamp, LogEntry {
                    timestamp: self.format_timestamp(&timestamp_str),
                    message: message.clone(),
                    level,
                    is_new: false,  // Will be set properly when comparing with previous logs
                }));
            }
        }
        
        // Sort by timestamp (oldest first, newest last)
        logs.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Extract just the LogEntry values
        let sorted_logs: Vec<LogEntry> = logs.into_iter().map(|(_, entry)| entry).collect();
        
        Ok(sorted_logs)
    }

    pub async fn get_recent_logs(&self, limit: u32) -> Result<Vec<LogEntry>> {
        // Try different queries in order of preference
        let queries = vec![
            "{service_name=\"fontory\"}",  // fontory service specifically
            "{service_name=~\".+\"}",      // Any service_name label
            "{app=~\".+\"}",               // Any app label
            "{host=~\".+\"}",              // Any host label
            "{level=~\".+\"}",             // Any level label
            "{}",                          // Any logs (might not work on all Loki configs)
        ];
        
        for query in queries {
            match self.tail(query, limit).await {
                Ok(logs) if !logs.is_empty() => {
                    return Ok(logs);
                }
                Ok(_) => continue,  // Empty result, try next query
                Err(e) => {
                    eprintln!("Query '{}' failed: {}", query, e);
                    continue;
                }
            }
        }
        
        // Return dummy logs for testing if no real logs found
        Ok(vec![
            LogEntry {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: "No logs found in Loki. Check your Loki configuration.".to_string(),
                level: "WARN".to_string(),
                is_new: false,
            },
            LogEntry {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                message: "RustDash is running in demo mode".to_string(),
                level: "INFO".to_string(),
                is_new: false,
            },
        ])
    }

    #[allow(dead_code)]
    pub async fn get_error_logs(&self, limit: u32) -> Result<Vec<LogEntry>> {
        self.tail("{} |= \"error\" or \"ERROR\"", limit).await
    }

    fn extract_log_level(&self, message: &str) -> String {
        // Check for common log level patterns
        if message.contains("l=ERROR") || message.contains("[ERROR]") || message.contains(" ERROR ") {
            return "ERROR".to_string();
        }
        if message.contains("l=WARN") || message.contains("[WARN]") || message.contains(" WARN ") {
            return "WARN".to_string();
        }
        if message.contains("l=INFO") || message.contains("[INFO]") || message.contains(" INFO ") {
            return "INFO".to_string();
        }
        if message.contains("l=DEBUG") || message.contains("[DEBUG]") || message.contains(" DEBUG ") {
            return "DEBUG".to_string();
        }
        
        // Fallback to case-insensitive search
        let message_lower = message.to_lowercase();
        if message_lower.contains("error") || message_lower.contains("fatal") {
            "ERROR".to_string()
        } else if message_lower.contains("warn") {
            "WARN".to_string()
        } else if message_lower.contains("info") {
            "INFO".to_string()
        } else if message_lower.contains("debug") {
            "DEBUG".to_string()
        } else {
            "INFO".to_string()
        }
    }

    fn format_timestamp(&self, timestamp: &str) -> String {
        if let Ok(nanos) = timestamp.parse::<i64>() {
            let seconds = nanos / 1_000_000_000;
            let nanos_remainder = (nanos % 1_000_000_000) as u32;
            
            if let Some(datetime) = DateTime::from_timestamp(seconds, nanos_remainder) {
                return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            }
        }
        timestamp.to_string()
    }
}

