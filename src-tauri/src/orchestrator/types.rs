use crate::providers::types::ProviderName;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Explicit, finite set of job states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    TimedOut,
    Blocked,
    Cancelled,
}

/// Configuration for a single run (the full fan-out).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub session_id: String,
    pub prompt: String,
    pub providers: Vec<ProviderName>,
    pub perspectives: Vec<String>,
    pub working_directory: Option<String>,
    pub context_paths: Vec<String>,
    pub timeout_secs: u64,
    pub max_concurrent: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            prompt: String::new(),
            providers: vec![],
            perspectives: vec![],
            working_directory: None,
            context_paths: vec![],
            timeout_secs: 120,
            max_concurrent: 4,
        }
    }
}

/// Specification for a single job in the matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    pub id: String,
    pub provider: ProviderName,
    pub perspective_id: String,
    pub prompt: String,
    pub perspective_instructions: String,
    pub working_directory: Option<String>,
    pub context_content: Option<String>,
    pub timeout_secs: u64,
    /// Path to a file containing the perspective text (used by Gemini for GEMINI_SYSTEM_MD).
    /// Set by the orchestrator before execution; lives in the session's prompts/ directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perspective_file: Option<String>,
}

impl JobSpec {
    pub fn new(
        provider: ProviderName,
        perspective_id: String,
        prompt: String,
        perspective_instructions: String,
        working_directory: Option<String>,
        context_content: Option<String>,
        timeout_secs: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider,
            perspective_id,
            prompt,
            perspective_instructions,
            working_directory,
            context_content,
            perspective_file: None,
            timeout_secs,
        }
    }
}

/// Result of executing a single job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub provider: ProviderName,
    pub perspective_id: String,
    pub state: JobState,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub blocked_reason: Option<String>,
    pub error: Option<String>,
}

impl JobResult {
    /// Create a new result in the queued state.
    pub fn queued(spec: &JobSpec) -> Self {
        Self {
            job_id: spec.id.clone(),
            provider: spec.provider.clone(),
            perspective_id: spec.perspective_id.clone(),
            state: JobState::Queued,
            started_at: None,
            ended_at: None,
            duration_ms: None,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            blocked_reason: None,
            error: None,
        }
    }

    /// Mark as running with a start timestamp.
    pub fn mark_running(&mut self) {
        self.state = JobState::Running;
        self.started_at = Some(Utc::now().to_rfc3339());
    }

    /// Mark as completed with outputs.
    pub fn mark_completed(&mut self, exit_code: i32, stdout: String, stderr: String) {
        self.state = JobState::Completed;
        self.ended_at = Some(Utc::now().to_rfc3339());
        self.exit_code = Some(exit_code);
        self.stdout = stdout;
        self.stderr = stderr;
        self.compute_duration();
    }

    /// Mark as failed with error info.
    pub fn mark_failed(
        &mut self,
        error: String,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
    ) {
        self.state = JobState::Failed;
        self.ended_at = Some(Utc::now().to_rfc3339());
        self.error = Some(error);
        self.stdout = stdout;
        self.stderr = stderr;
        self.exit_code = exit_code;
        self.compute_duration();
    }

    /// Mark as timed out.
    pub fn mark_timed_out(&mut self, stdout: String, stderr: String) {
        self.state = JobState::TimedOut;
        self.ended_at = Some(Utc::now().to_rfc3339());
        self.stdout = stdout;
        self.stderr = stderr;
        self.compute_duration();
    }

    /// Mark as blocked with a reason.
    pub fn mark_blocked(&mut self, reason: String) {
        self.state = JobState::Blocked;
        self.ended_at = Some(Utc::now().to_rfc3339());
        self.blocked_reason = Some(reason);
        self.compute_duration();
    }

    /// Mark as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.state = JobState::Cancelled;
        self.ended_at = Some(Utc::now().to_rfc3339());
        self.compute_duration();
    }

    fn compute_duration(&mut self) {
        if let (Some(start), Some(end)) = (&self.started_at, &self.ended_at) {
            if let (Ok(s), Ok(e)) = (
                chrono::DateTime::parse_from_rfc3339(start),
                chrono::DateTime::parse_from_rfc3339(end),
            ) {
                let dur = e.signed_duration_since(s);
                self.duration_ms = Some(dur.num_milliseconds().max(0) as u64);
            }
        }
    }
}

/// Invocation metadata persisted alongside raw artifacts.
#[derive(Debug, Clone, Serialize)]
pub struct InvocationMetadata {
    pub job_id: String,
    pub provider: ProviderName,
    pub provider_executable: Option<String>,
    pub provider_version: Option<String>,
    pub perspective_id: String,
    pub prompt: String,
    pub perspective_instructions: String,
    pub working_directory: Option<String>,
    pub timeout_secs: u64,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub terminal_state: JobState,
}

/// Summary of a complete run (all jobs).
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub session_id: String,
    pub total_jobs: usize,
    pub completed: usize,
    pub failed: usize,
    pub timed_out: usize,
    pub blocked: usize,
    pub cancelled: usize,
    pub jobs: Vec<JobResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_state_transitions() {
        let spec = JobSpec::new(
            ProviderName::Claude,
            "default".to_string(),
            "test prompt".to_string(),
            "test instructions".to_string(),
            None,
            None,
            120,
        );

        let mut result = JobResult::queued(&spec);
        assert_eq!(result.state, JobState::Queued);

        result.mark_running();
        assert_eq!(result.state, JobState::Running);
        assert!(result.started_at.is_some());

        result.mark_completed(0, "output".to_string(), String::new());
        assert_eq!(result.state, JobState::Completed);
        assert!(result.ended_at.is_some());
        assert_eq!(result.exit_code, Some(0));
        assert!(result.duration_ms.is_some());
    }

    #[test]
    fn test_job_state_failed() {
        let spec = JobSpec::new(
            ProviderName::Codex,
            "adversarial".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            None,
            60,
        );

        let mut result = JobResult::queued(&spec);
        result.mark_running();
        result.mark_failed(
            "process crashed".to_string(),
            "partial".to_string(),
            "err".to_string(),
            Some(1),
        );
        assert_eq!(result.state, JobState::Failed);
        assert_eq!(result.error, Some("process crashed".to_string()));
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn test_job_state_timed_out() {
        let spec = JobSpec::new(
            ProviderName::Gemini,
            "default".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            None,
            5,
        );

        let mut result = JobResult::queued(&spec);
        result.mark_running();
        result.mark_timed_out("partial output".to_string(), String::new());
        assert_eq!(result.state, JobState::TimedOut);
        assert_eq!(result.stdout, "partial output");
    }

    #[test]
    fn test_job_state_blocked() {
        let spec = JobSpec::new(
            ProviderName::Gemini,
            "default".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            None,
            60,
        );

        let mut result = JobResult::queued(&spec);
        result.mark_blocked("provider not authenticated".to_string());
        assert_eq!(result.state, JobState::Blocked);
        assert_eq!(
            result.blocked_reason,
            Some("provider not authenticated".to_string())
        );
    }

    #[test]
    fn test_job_spec_has_unique_id() {
        let spec1 = JobSpec::new(
            ProviderName::Claude,
            "default".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            None,
            60,
        );
        let spec2 = JobSpec::new(
            ProviderName::Claude,
            "default".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            None,
            60,
        );
        assert_ne!(spec1.id, spec2.id);
    }
}
