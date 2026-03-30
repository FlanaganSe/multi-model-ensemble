use super::normalize;
use super::types::{
    AgreementLevel, Claim, CoverageCell, CoverageMatrix, EvidenceMatrix, EvidenceSource,
    ExtractionStatus, NormalizedRun, SectionType, SourceRef, SourceStatus, Theme,
};
use crate::orchestrator::types::{JobResult, JobState};
use crate::providers::types::ProviderName;
use uuid::Uuid;

/// Build an evidence matrix from a set of job results.
///
/// This normalizes each job result, groups the extracted sections into themes,
/// and tracks coverage across all provider×perspective combinations.
pub fn build_evidence_matrix(
    session_id: &str,
    job_results: &[JobResult],
) -> (EvidenceMatrix, Vec<NormalizedRun>) {
    // Normalize all jobs
    let normalized: Vec<NormalizedRun> = job_results.iter().map(normalize::normalize_job).collect();

    // Build sources and coverage
    let mut sources = Vec::new();
    let mut cells = Vec::new();
    let mut providers_seen = Vec::new();
    let mut perspectives_seen = Vec::new();

    for nr in &normalized {
        let status = match nr.extraction_status {
            ExtractionStatus::Success => SourceStatus::Available,
            ExtractionStatus::ParseFailed => SourceStatus::Available, // still has text
            ExtractionStatus::Empty => SourceStatus::Empty,
            ExtractionStatus::JobNotCompleted => {
                // Check the original job result for the specific failure type
                job_results
                    .iter()
                    .find(|j| j.job_id == nr.job_id)
                    .map(|j| match j.state {
                        JobState::Failed => SourceStatus::Failed,
                        JobState::TimedOut => SourceStatus::TimedOut,
                        JobState::Blocked => SourceStatus::Blocked,
                        _ => SourceStatus::Failed,
                    })
                    .unwrap_or(SourceStatus::Failed)
            }
        };

        sources.push(EvidenceSource {
            job_id: nr.job_id.clone(),
            provider: nr.provider.clone(),
            perspective_id: nr.perspective_id.clone(),
            status: status.clone(),
        });

        cells.push(CoverageCell {
            provider: nr.provider.clone(),
            perspective: nr.perspective_id.clone(),
            status,
            job_id: nr.job_id.clone(),
        });

        if !providers_seen.contains(&nr.provider) {
            providers_seen.push(nr.provider.clone());
        }
        if !perspectives_seen.contains(&nr.perspective_id) {
            perspectives_seen.push(nr.perspective_id.clone());
        }
    }

    // Build themes by grouping sections across sources
    let themes = build_themes(&normalized);

    let matrix = EvidenceMatrix {
        schema_version: 1,
        session_id: session_id.to_string(),
        sources,
        themes,
        coverage: CoverageMatrix {
            providers: providers_seen,
            perspectives: perspectives_seen,
            cells,
        },
    };

    (matrix, normalized)
}

/// Group sections from all normalized runs into themes.
///
/// Grouping strategy: section headings are normalized (lowercased, trimmed) and
/// sections with the same normalized heading are grouped together. Sections
/// without headings go into an "ungrouped" theme per source.
fn build_themes(normalized: &[NormalizedRun]) -> Vec<Theme> {
    use std::collections::BTreeMap;

    let mut heading_groups: BTreeMap<
        String,
        Vec<(&NormalizedRun, &super::types::ResponseSection)>,
    > = BTreeMap::new();
    let mut ungrouped: Vec<(&NormalizedRun, &super::types::ResponseSection)> = Vec::new();

    for nr in normalized {
        if nr.extraction_status == ExtractionStatus::JobNotCompleted {
            continue;
        }
        for section in &nr.sections {
            match &section.heading {
                Some(h) => {
                    let key = normalize_heading(h);
                    heading_groups.entry(key).or_default().push((nr, section));
                }
                None => {
                    ungrouped.push((nr, section));
                }
            }
        }
    }

    let mut themes = Vec::new();

    // Build a theme for each heading group
    for (key, entries) in &heading_groups {
        let label = entries
            .first()
            .and_then(|(_, s)| s.heading.clone())
            .unwrap_or_else(|| key.clone());

        let claims: Vec<Claim> = entries
            .iter()
            .map(|(nr, section)| Claim {
                id: Uuid::new_v4().to_string(),
                text: section.content.clone(),
                section_type: section.section_type.clone(),
                source: SourceRef {
                    job_id: nr.job_id.clone(),
                    provider: nr.provider.clone(),
                    perspective_id: nr.perspective_id.clone(),
                },
            })
            .collect();

        let unique_providers: Vec<&ProviderName> = {
            let mut ps: Vec<&ProviderName> = claims.iter().map(|c| &c.source.provider).collect();
            ps.dedup_by(|a, b| a == b);
            ps.sort_by_key(|p| format!("{p:?}"));
            ps.dedup();
            ps
        };

        let agreement_level = compute_agreement_level(unique_providers.len(), entries.len());

        themes.push(Theme {
            id: Uuid::new_v4().to_string(),
            label,
            claims,
            agreement_level,
            disagreements: vec![], // populated by strategy layer
        });
    }

    // Add ungrouped content as a catch-all theme if non-empty
    if !ungrouped.is_empty() {
        let claims: Vec<Claim> = ungrouped
            .iter()
            .map(|(nr, section)| Claim {
                id: Uuid::new_v4().to_string(),
                text: section.content.clone(),
                section_type: section.section_type.clone(),
                source: SourceRef {
                    job_id: nr.job_id.clone(),
                    provider: nr.provider.clone(),
                    perspective_id: nr.perspective_id.clone(),
                },
            })
            .collect();

        themes.push(Theme {
            id: Uuid::new_v4().to_string(),
            label: "Additional Notes".to_string(),
            claims,
            agreement_level: AgreementLevel::Single,
            disagreements: vec![],
        });
    }

    themes
}

fn normalize_heading(heading: &str) -> String {
    heading.to_lowercase().trim().to_string()
}

fn compute_agreement_level(unique_providers: usize, total_entries: usize) -> AgreementLevel {
    if total_entries <= 1 {
        AgreementLevel::Single
    } else if unique_providers >= 3 {
        AgreementLevel::Full
    } else if unique_providers == 2 {
        AgreementLevel::Strong
    } else {
        // Multiple entries but all from same provider (different perspectives)
        AgreementLevel::Partial
    }
}

/// Classify a section type for filtering purposes.
pub fn is_recommendation_type(st: &SectionType) -> bool {
    matches!(st, SectionType::Recommendation)
}

pub fn is_risk_or_caveat_type(st: &SectionType) -> bool {
    matches!(st, SectionType::Risk | SectionType::Caveat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::types::{JobResult, JobState};
    use crate::synthesis::fixtures::{claude_json, make_job_result};

    #[test]
    fn test_evidence_matrix_basic() {
        let results = vec![
            make_job_result(
                "j1",
                ProviderName::Claude,
                "default",
                JobState::Completed,
                &claude_json("## Summary\n\nGood code.\n\n## Recommendations\n\n- Add tests"),
            ),
            make_job_result(
                "j2",
                ProviderName::Codex,
                "default",
                JobState::Completed,
                "## Summary\n\nSolid implementation.\n\n## Recommendations\n\n- Improve docs",
            ),
        ];

        let (matrix, normalized) = build_evidence_matrix("session-1", &results);

        assert_eq!(matrix.session_id, "session-1");
        assert_eq!(matrix.sources.len(), 2);
        assert_eq!(normalized.len(), 2);
        assert_eq!(matrix.coverage.providers.len(), 2);
        assert_eq!(matrix.coverage.perspectives.len(), 1);
        assert!(!matrix.themes.is_empty());
    }

    #[test]
    fn test_evidence_matrix_with_failures() {
        let results = vec![
            make_job_result(
                "j1",
                ProviderName::Claude,
                "default",
                JobState::Completed,
                &claude_json("## Analysis\n\nLooks good."),
            ),
            make_job_result("j2", ProviderName::Codex, "default", JobState::Failed, ""),
            make_job_result(
                "j3",
                ProviderName::Gemini,
                "default",
                JobState::TimedOut,
                "",
            ),
        ];

        let (matrix, _) = build_evidence_matrix("session-2", &results);

        assert_eq!(matrix.sources.len(), 3);
        let codex_source = matrix
            .sources
            .iter()
            .find(|s| s.provider == ProviderName::Codex)
            .unwrap();
        assert_eq!(codex_source.status, SourceStatus::Failed);
        let gemini_source = matrix
            .sources
            .iter()
            .find(|s| s.provider == ProviderName::Gemini)
            .unwrap();
        assert_eq!(gemini_source.status, SourceStatus::TimedOut);
    }

    #[test]
    fn test_evidence_matrix_theme_grouping() {
        let results = vec![
            make_job_result(
                "j1",
                ProviderName::Claude,
                "default",
                JobState::Completed,
                &claude_json(
                    "## Risks\n\n- Memory leaks\n\n## Recommendations\n\n- Add bounds checking",
                ),
            ),
            make_job_result(
                "j2",
                ProviderName::Codex,
                "default",
                JobState::Completed,
                "## Risks\n\n- Potential overflow\n\n## Recommendations\n\n- Use safe math",
            ),
        ];

        let (matrix, _) = build_evidence_matrix("session-3", &results);

        let risks_theme = matrix.themes.iter().find(|t| t.label == "Risks");
        assert!(risks_theme.is_some());
        let risks = risks_theme.unwrap();
        assert_eq!(risks.claims.len(), 2);
        assert_eq!(risks.agreement_level, AgreementLevel::Strong);
    }

    #[test]
    fn test_evidence_matrix_empty_session() {
        let results: Vec<JobResult> = vec![];
        let (matrix, normalized) = build_evidence_matrix("empty", &results);

        assert_eq!(matrix.sources.len(), 0);
        assert_eq!(matrix.themes.len(), 0);
        assert_eq!(normalized.len(), 0);
    }

    #[test]
    fn test_coverage_matrix() {
        let results = vec![
            make_job_result(
                "j1",
                ProviderName::Claude,
                "default",
                JobState::Completed,
                &claude_json("response text"),
            ),
            make_job_result(
                "j2",
                ProviderName::Claude,
                "adversarial",
                JobState::Completed,
                &claude_json("adversarial response"),
            ),
            make_job_result("j3", ProviderName::Codex, "default", JobState::Blocked, ""),
        ];

        let (matrix, _) = build_evidence_matrix("session-4", &results);

        assert_eq!(matrix.coverage.cells.len(), 3);
        let blocked = matrix
            .coverage
            .cells
            .iter()
            .find(|c| c.provider == ProviderName::Codex)
            .unwrap();
        assert_eq!(blocked.status, SourceStatus::Blocked);
    }
}
