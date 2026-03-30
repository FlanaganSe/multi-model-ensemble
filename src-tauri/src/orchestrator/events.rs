use crate::orchestrator::types::JobState;
use crate::providers::types::ProviderName;
use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Event types for the JSONL event log.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    RunStarted {
        session_id: String,
        total_jobs: usize,
    },
    JobQueued {
        job_id: String,
        provider: ProviderName,
        perspective_id: String,
    },
    JobStarted {
        job_id: String,
        provider: ProviderName,
    },
    JobCompleted {
        job_id: String,
        provider: ProviderName,
        exit_code: i32,
        duration_ms: u64,
    },
    JobFailed {
        job_id: String,
        provider: ProviderName,
        error: String,
    },
    JobTimedOut {
        job_id: String,
        provider: ProviderName,
        timeout_secs: u64,
    },
    JobBlocked {
        job_id: String,
        provider: ProviderName,
        reason: String,
    },
    JobCancelled {
        job_id: String,
        provider: ProviderName,
    },
    RunCompleted {
        session_id: String,
        total_jobs: usize,
        completed: usize,
        failed: usize,
        timed_out: usize,
        blocked: usize,
        cancelled: usize,
    },
}

/// Wrapper for a timestamped JSONL event.
#[derive(Debug, Serialize)]
struct TimestampedEvent<'a> {
    timestamp: String,
    #[serde(flatten)]
    event: &'a Event,
}

/// Append-only JSONL event logger.
pub struct EventLogger {
    path: PathBuf,
}

impl EventLogger {
    /// Create a new event logger targeting the given file path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Log an event. Appends one JSON line to the file.
    pub fn log(&self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        let stamped = TimestampedEvent {
            timestamp: Utc::now().to_rfc3339(),
            event,
        };

        let line = serde_json::to_string(&stamped)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        writeln!(file, "{line}")?;
        Ok(())
    }

    /// Log an event, ignoring errors (best-effort logging).
    pub fn log_best_effort(&self, event: &Event) {
        if let Err(e) = self.log(event) {
            log::warn!("Failed to log event: {e}");
        }
    }
}

/// Build a terminal event from a JobResult.
pub fn terminal_event(result: &super::types::JobResult, timeout_secs: u64) -> Event {
    match result.state {
        JobState::Completed => Event::JobCompleted {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            exit_code: result.exit_code.unwrap_or(-1),
            duration_ms: result.duration_ms.unwrap_or(0),
        },
        JobState::Failed => Event::JobFailed {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            error: result
                .error
                .clone()
                .unwrap_or_else(|| "unknown error".to_string()),
        },
        JobState::TimedOut => Event::JobTimedOut {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            timeout_secs,
        },
        JobState::Blocked => Event::JobBlocked {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            reason: result
                .blocked_reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
        },
        JobState::Cancelled => Event::JobCancelled {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
        },
        // Queued/Running shouldn't produce terminal events, but handle gracefully
        _ => Event::JobFailed {
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            error: format!("unexpected terminal state: {:?}", result.state),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_event_logger_creates_file_and_appends() {
        let tmp = tempfile::tempdir().unwrap();
        let log_path = tmp.path().join("events.jsonl");
        let logger = EventLogger::new(log_path.clone());

        logger
            .log(&Event::RunStarted {
                session_id: "test-session".to_string(),
                total_jobs: 4,
            })
            .unwrap();

        logger
            .log(&Event::JobQueued {
                job_id: "job-1".to_string(),
                provider: ProviderName::Claude,
                perspective_id: "default".to_string(),
            })
            .unwrap();

        let contents = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);

        // Verify each line is valid JSON
        for line in &lines {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }

        // Verify first event has expected structure
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["event"], "run_started");
        assert_eq!(first["total_jobs"], 4);
        assert!(first["timestamp"].is_string());
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::JobCompleted {
            job_id: "j1".to_string(),
            provider: ProviderName::Codex,
            exit_code: 0,
            duration_ms: 5000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event\":\"job_completed\""));
        assert!(json.contains("\"exit_code\":0"));
    }
}
