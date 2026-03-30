use crate::orchestrator::types::{JobResult, JobState};
use crate::providers::types::ProviderName;

use super::types::{
    ExtractionStatus, NormalizedRun, OutputFormat, ProviderOutputMetadata, ResponseSection,
    SectionType, TokenCounts,
};

/// Normalize a single job result into a structured `NormalizedRun`.
pub fn normalize_job(result: &JobResult) -> NormalizedRun {
    let raw_artifact_path = format!(
        "runs/{}/{}/stdout.txt",
        provider_dir_name(&result.provider),
        result.perspective_id
    );

    // Non-completed jobs get a stub with appropriate status
    if result.state != JobState::Completed {
        return NormalizedRun {
            schema_version: 1,
            job_id: result.job_id.clone(),
            provider: result.provider.clone(),
            perspective_id: result.perspective_id.clone(),
            response_text: String::new(),
            provider_metadata: ProviderOutputMetadata::default(),
            sections: vec![],
            extraction_status: ExtractionStatus::JobNotCompleted,
            raw_artifact_path,
        };
    }

    let (response_text, metadata, extraction_status) = match result.provider {
        ProviderName::Claude => parse_claude_output(&result.stdout),
        ProviderName::Codex => parse_codex_output(&result.stdout),
        ProviderName::Gemini => parse_gemini_output(&result.stdout),
    };

    let sections = if extraction_status == ExtractionStatus::Success && !response_text.is_empty() {
        parse_markdown_sections(&response_text)
    } else {
        vec![]
    };

    NormalizedRun {
        schema_version: 1,
        job_id: result.job_id.clone(),
        provider: result.provider.clone(),
        perspective_id: result.perspective_id.clone(),
        response_text,
        provider_metadata: metadata,
        sections,
        extraction_status,
        raw_artifact_path,
    }
}

fn provider_dir_name(p: &ProviderName) -> &'static str {
    match p {
        ProviderName::Claude => "claude",
        ProviderName::Codex => "codex",
        ProviderName::Gemini => "gemini",
    }
}

// ---------------------------------------------------------------------------
// Claude: `--output-format json` produces { type, subtype, result, cost_usd, ... }
// ---------------------------------------------------------------------------

fn parse_claude_output(stdout: &str) -> (String, ProviderOutputMetadata, ExtractionStatus) {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return (
            String::new(),
            ProviderOutputMetadata {
                output_format: OutputFormat::Json,
                ..Default::default()
            },
            ExtractionStatus::Empty,
        );
    }

    match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(val) => {
            let response_text = val
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let cost_usd = val.get("cost_usd").and_then(|v| v.as_f64());
            let duration_ms = val.get("duration_ms").and_then(|v| v.as_u64());
            let model_id = val
                .get("model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let status = if response_text.is_empty() {
                ExtractionStatus::Empty
            } else {
                ExtractionStatus::Success
            };

            (
                response_text,
                ProviderOutputMetadata {
                    output_format: OutputFormat::Json,
                    parse_success: true,
                    cost_usd,
                    duration_ms,
                    model_id,
                    ..Default::default()
                },
                status,
            )
        }
        Err(_) => {
            // JSON parse failed — use raw text as response
            (
                trimmed.to_string(),
                ProviderOutputMetadata {
                    output_format: OutputFormat::Json,
                    parse_success: false,
                    ..Default::default()
                },
                ExtractionStatus::ParseFailed,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Codex: `codex exec` produces plain text on stdout
// ---------------------------------------------------------------------------

fn parse_codex_output(stdout: &str) -> (String, ProviderOutputMetadata, ExtractionStatus) {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return (
            String::new(),
            ProviderOutputMetadata {
                output_format: OutputFormat::Text,
                parse_success: true,
                ..Default::default()
            },
            ExtractionStatus::Empty,
        );
    }

    (
        trimmed.to_string(),
        ProviderOutputMetadata {
            output_format: OutputFormat::Text,
            parse_success: true,
            ..Default::default()
        },
        ExtractionStatus::Success,
    )
}

// ---------------------------------------------------------------------------
// Gemini: `--output-format json` produces { response, stats, error? }
// ---------------------------------------------------------------------------

fn parse_gemini_output(stdout: &str) -> (String, ProviderOutputMetadata, ExtractionStatus) {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return (
            String::new(),
            ProviderOutputMetadata {
                output_format: OutputFormat::Json,
                ..Default::default()
            },
            ExtractionStatus::Empty,
        );
    }

    match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(val) => {
            // Check for error field first
            if val.get("error").is_some() && val.get("response").is_none() {
                return (
                    String::new(),
                    ProviderOutputMetadata {
                        output_format: OutputFormat::Json,
                        parse_success: true,
                        ..Default::default()
                    },
                    ExtractionStatus::Empty,
                );
            }

            let response_text = val
                .get("response")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Extract stats from the nested structure
            let mut token_counts = None;
            let mut model_id = None;

            if let Some(stats) = val.get("stats") {
                if let Some(models) = stats.get("models").and_then(|m| m.as_object()) {
                    // First model entry has the stats
                    if let Some((model_name, model_stats)) = models.iter().next() {
                        model_id = Some(model_name.clone());
                        if let Some(tokens) = model_stats.get("tokens") {
                            token_counts = Some(TokenCounts {
                                prompt: tokens.get("prompt").and_then(|v| v.as_u64()),
                                completion: tokens.get("candidates").and_then(|v| v.as_u64()),
                                cached: tokens.get("cached").and_then(|v| v.as_u64()),
                            });
                        }
                    }
                }
            }

            let status = if response_text.is_empty() {
                ExtractionStatus::Empty
            } else {
                ExtractionStatus::Success
            };

            (
                response_text,
                ProviderOutputMetadata {
                    output_format: OutputFormat::Json,
                    parse_success: true,
                    token_counts,
                    model_id,
                    ..Default::default()
                },
                status,
            )
        }
        Err(_) => {
            // JSON parse failed — use raw text
            (
                trimmed.to_string(),
                ProviderOutputMetadata {
                    output_format: OutputFormat::Json,
                    parse_success: false,
                    ..Default::default()
                },
                ExtractionStatus::ParseFailed,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Markdown section parser
// ---------------------------------------------------------------------------

/// Parse markdown text into sections based on headings.
/// Each heading starts a new section. Content before the first heading goes
/// into a section with `heading: None`.
pub fn parse_markdown_sections(text: &str) -> Vec<ResponseSection> {
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(heading_text) = extract_heading(trimmed) {
            // Flush previous section
            flush_section(&mut sections, &current_heading, &current_content);
            current_heading = Some(heading_text);
            current_content.clear();
        } else {
            if !current_content.is_empty() || !trimmed.is_empty() {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }
    }

    // Flush final section
    flush_section(&mut sections, &current_heading, &current_content);

    sections
}

fn flush_section(sections: &mut Vec<ResponseSection>, heading: &Option<String>, content: &str) {
    let trimmed = content.trim();
    if heading.is_some() || !trimmed.is_empty() {
        let section_type = heading
            .as_deref()
            .map(classify_section)
            .unwrap_or(SectionType::Other);

        sections.push(ResponseSection {
            heading: heading.clone(),
            content: trimmed.to_string(),
            section_type,
        });
    }
}

fn extract_heading(line: &str) -> Option<String> {
    if line.starts_with('#') {
        let text = line.trim_start_matches('#').trim();
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }
    None
}

/// Classify a section heading into a semantic type.
fn classify_section(heading: &str) -> SectionType {
    let lower = heading.to_lowercase();

    if lower.contains("recommend")
        || lower.contains("action")
        || lower.contains("next step")
        || lower.contains("suggestion")
    {
        return SectionType::Recommendation;
    }
    if lower.contains("risk")
        || lower.contains("threat")
        || lower.contains("vulnerabilit")
        || lower.contains("concern")
        || lower.contains("warning")
    {
        return SectionType::Risk;
    }
    if lower.contains("caveat")
        || lower.contains("limitation")
        || lower.contains("uncertaint")
        || lower.contains("assumption")
    {
        return SectionType::Caveat;
    }
    if lower.contains("summary")
        || lower.contains("overview")
        || lower.contains("conclusion")
        || lower.contains("tldr")
        || lower.contains("tl;dr")
    {
        return SectionType::Summary;
    }
    if lower.contains("finding")
        || lower.contains("analysis")
        || lower.contains("observation")
        || lower.contains("result")
        || lower.contains("detail")
        || lower.contains("assessment")
    {
        return SectionType::Finding;
    }

    SectionType::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::types::JobResult;
    use crate::providers::types::ProviderName;

    fn make_completed_result(provider: ProviderName, stdout: &str) -> JobResult {
        JobResult {
            job_id: "test-job-1".to_string(),
            provider,
            perspective_id: "default".to_string(),
            state: JobState::Completed,
            started_at: Some("2026-03-30T10:00:00Z".to_string()),
            ended_at: Some("2026-03-30T10:00:05Z".to_string()),
            duration_ms: Some(5000),
            exit_code: Some(0),
            stdout: stdout.to_string(),
            stderr: String::new(),
            blocked_reason: None,
            error: None,
        }
    }

    fn claude_json(result_text: &str) -> String {
        serde_json::json!({
            "type": "result",
            "subtype": "success",
            "result": result_text,
            "cost_usd": 0.05,
            "duration_ms": 3500,
            "is_error": false,
            "num_turns": 1
        })
        .to_string()
    }

    fn gemini_json(response_text: &str) -> String {
        serde_json::json!({
            "response": response_text,
            "stats": {
                "models": {
                    "gemini-2.0-flash": {
                        "requests": 1,
                        "errors": 0,
                        "latency": 2500,
                        "tokens": {
                            "prompt": 100,
                            "candidates": 500,
                            "cached": 0,
                            "thoughts": 0,
                            "tool": 0
                        }
                    }
                },
                "tools": { "totalCalls": 0, "success": 0, "fail": 0, "decisions": {} },
                "files": { "additions": 0, "removals": 0 }
            }
        })
        .to_string()
    }

    #[test]
    fn test_normalize_claude_json() {
        let stdout = claude_json("## Summary\n\nThis is a good approach.\n\n## Recommendations\n\n- Use Rust for safety\n- Add tests early");
        let result = make_completed_result(ProviderName::Claude, &stdout);
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::Success);
        assert!(normalized.response_text.contains("This is a good approach"));
        assert!(normalized.provider_metadata.parse_success);
        assert_eq!(normalized.provider_metadata.cost_usd, Some(0.05));
        assert_eq!(normalized.provider_metadata.duration_ms, Some(3500));
        assert_eq!(
            normalized.provider_metadata.output_format,
            OutputFormat::Json
        );
        assert!(!normalized.sections.is_empty());
    }

    #[test]
    fn test_normalize_claude_malformed_json() {
        let stdout = "This is not JSON, just raw text output";
        let result = make_completed_result(ProviderName::Claude, stdout);
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::ParseFailed);
        assert_eq!(normalized.response_text, stdout);
        assert!(!normalized.provider_metadata.parse_success);
    }

    #[test]
    fn test_normalize_codex_text() {
        let stdout = "## Analysis\n\nThe code looks solid.\n\n## Risks\n\n- Memory leaks possible\n- No input validation";
        let result = make_completed_result(ProviderName::Codex, stdout);
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::Success);
        assert_eq!(normalized.response_text, stdout);
        assert!(normalized.provider_metadata.parse_success);
        assert_eq!(
            normalized.provider_metadata.output_format,
            OutputFormat::Text
        );
        assert_eq!(normalized.sections.len(), 2);
        assert_eq!(normalized.sections[0].section_type, SectionType::Finding);
        assert_eq!(normalized.sections[1].section_type, SectionType::Risk);
    }

    #[test]
    fn test_normalize_gemini_json() {
        let stdout = gemini_json(
            "## Key Findings\n\nGood architecture.\n\n## Recommendations\n\n- Add logging",
        );
        let result = make_completed_result(ProviderName::Gemini, &stdout);
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::Success);
        assert!(normalized.response_text.contains("Good architecture"));
        assert!(normalized.provider_metadata.parse_success);
        assert_eq!(
            normalized.provider_metadata.model_id.as_deref(),
            Some("gemini-2.0-flash")
        );
        let tokens = normalized.provider_metadata.token_counts.as_ref().unwrap();
        assert_eq!(tokens.prompt, Some(100));
        assert_eq!(tokens.completion, Some(500));
    }

    #[test]
    fn test_normalize_gemini_error() {
        let stdout = serde_json::json!({"error":{"type":"auth","message":"Authentication failed","code":"FATAL_AUTH"}}).to_string();
        let result = make_completed_result(ProviderName::Gemini, &stdout);
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::Empty);
        assert!(normalized.response_text.is_empty());
    }

    #[test]
    fn test_normalize_empty_output() {
        let result = make_completed_result(ProviderName::Claude, "");
        let normalized = normalize_job(&result);

        assert_eq!(normalized.extraction_status, ExtractionStatus::Empty);
        assert!(normalized.response_text.is_empty());
    }

    #[test]
    fn test_normalize_failed_job() {
        let mut result = make_completed_result(ProviderName::Codex, "partial output");
        result.state = JobState::Failed;
        result.error = Some("process crashed".to_string());

        let normalized = normalize_job(&result);
        assert_eq!(
            normalized.extraction_status,
            ExtractionStatus::JobNotCompleted
        );
        assert!(normalized.response_text.is_empty());
    }

    #[test]
    fn test_normalize_timed_out_job() {
        let mut result = make_completed_result(ProviderName::Gemini, "");
        result.state = JobState::TimedOut;

        let normalized = normalize_job(&result);
        assert_eq!(
            normalized.extraction_status,
            ExtractionStatus::JobNotCompleted
        );
    }

    #[test]
    fn test_parse_markdown_sections() {
        let text = "## Summary\n\nThis is the summary.\n\n## Key Findings\n\n- Finding A\n- Finding B\n\n## Recommendations\n\n1. Do X\n2. Do Y";
        let sections = parse_markdown_sections(text);

        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].heading.as_deref(), Some("Summary"));
        assert_eq!(sections[0].section_type, SectionType::Summary);
        assert!(sections[0].content.contains("This is the summary"));

        assert_eq!(sections[1].heading.as_deref(), Some("Key Findings"));
        assert_eq!(sections[1].section_type, SectionType::Finding);

        assert_eq!(sections[2].heading.as_deref(), Some("Recommendations"));
        assert_eq!(sections[2].section_type, SectionType::Recommendation);
    }

    #[test]
    fn test_parse_markdown_no_headings() {
        let text = "Just plain text with no headings.\nLine two.";
        let sections = parse_markdown_sections(text);

        assert_eq!(sections.len(), 1);
        assert!(sections[0].heading.is_none());
        assert_eq!(sections[0].section_type, SectionType::Other);
    }

    #[test]
    fn test_parse_markdown_content_before_first_heading() {
        let text = "Intro text here.\n\n## Analysis\n\nDetails.";
        let sections = parse_markdown_sections(text);

        assert_eq!(sections.len(), 2);
        assert!(sections[0].heading.is_none());
        assert_eq!(sections[0].content, "Intro text here.");
        assert_eq!(sections[1].heading.as_deref(), Some("Analysis"));
    }

    #[test]
    fn test_classify_section_types() {
        assert_eq!(
            classify_section("Recommendations"),
            SectionType::Recommendation
        );
        assert_eq!(
            classify_section("Action Items"),
            SectionType::Recommendation
        );
        assert_eq!(classify_section("Risks and Threats"), SectionType::Risk);
        assert_eq!(
            classify_section("Security Vulnerabilities"),
            SectionType::Risk
        );
        assert_eq!(classify_section("Caveats"), SectionType::Caveat);
        assert_eq!(classify_section("Limitations"), SectionType::Caveat);
        assert_eq!(classify_section("Executive Summary"), SectionType::Summary);
        assert_eq!(classify_section("TL;DR"), SectionType::Summary);
        assert_eq!(classify_section("Detailed Analysis"), SectionType::Finding);
        assert_eq!(classify_section("Key Findings"), SectionType::Finding);
        assert_eq!(classify_section("Introduction"), SectionType::Other);
    }
}
