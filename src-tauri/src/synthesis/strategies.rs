use super::types::{
    AgreementLevel, Disagreement, EvidenceMatrix, KeyPoint, Position, Priority, SourceRef,
    SynthesisMeta, SynthesisMethod, SynthesisOutput, SynthesisStrategy, SynthesizedRecommendation,
    SynthesizedTheme, Uncertainty,
};

/// Apply a synthesis strategy to an evidence matrix, producing structured synthesis output.
pub fn synthesize(matrix: &EvidenceMatrix, strategy: SynthesisStrategy) -> SynthesisOutput {
    let start = std::time::Instant::now();

    let (themes, recommendations, uncertainties) = match strategy {
        SynthesisStrategy::Consensus => synthesize_consensus(matrix),
        SynthesisStrategy::Comprehensive => synthesize_comprehensive(matrix),
        SynthesisStrategy::Executive => synthesize_executive(matrix),
    };

    let completed = matrix
        .sources
        .iter()
        .filter(|s| matches!(s.status, super::types::SourceStatus::Available))
        .count();
    let failed = matrix.sources.len() - completed;

    SynthesisOutput {
        schema_version: 1,
        session_id: matrix.session_id.clone(),
        strategy: strategy.clone(),
        synthesis_method: SynthesisMethod::Deterministic,
        themes,
        recommendations,
        uncertainties,
        meta: SynthesisMeta {
            total_sources: matrix.sources.len(),
            completed_sources: completed,
            failed_sources: failed,
            strategy_name: strategy.to_string(),
            synthesis_duration_ms: Some(start.elapsed().as_millis() as u64),
        },
    }
}

// ---------------------------------------------------------------------------
// Consensus strategy: focus on agreement, explicitly flag disagreements
// ---------------------------------------------------------------------------

fn synthesize_consensus(
    matrix: &EvidenceMatrix,
) -> (
    Vec<SynthesizedTheme>,
    Vec<SynthesizedRecommendation>,
    Vec<Uncertainty>,
) {
    let mut themes = Vec::new();
    let mut recommendations = Vec::new();
    let mut uncertainties = Vec::new();

    for theme in &matrix.themes {
        // Group claims by unique provider for cross-provider analysis
        let provider_groups = group_claims_by_provider(theme);
        let num_providers = provider_groups.len();

        // Build key points from claims that appear across multiple providers
        let key_points: Vec<KeyPoint> = theme
            .claims
            .iter()
            .map(|c| KeyPoint {
                text: c.text.clone(),
                support: vec![c.source.clone()],
            })
            .collect();

        // Detect disagreements between providers
        let disagreements = detect_cross_provider_disagreements(theme, &provider_groups);

        // Extract recommendations from recommendation-type sections
        for claim in &theme.claims {
            if super::evidence::is_recommendation_type(&claim.section_type) {
                for point in extract_bullet_points(&claim.text) {
                    recommendations.push(SynthesizedRecommendation {
                        text: point,
                        priority: if num_providers >= 2 {
                            Priority::High
                        } else {
                            Priority::Medium
                        },
                        support: vec![claim.source.clone()],
                    });
                }
            }
        }

        // Record uncertainties from caveat sections
        for claim in &theme.claims {
            if super::evidence::is_risk_or_caveat_type(&claim.section_type) {
                uncertainties.push(Uncertainty {
                    text: claim.text.lines().next().unwrap_or(&claim.text).to_string(),
                    reason: format!(
                        "Flagged by {} ({})",
                        provider_label(&claim.source.provider),
                        claim.source.perspective_id,
                    ),
                });
            }
        }

        let agreement_level = if !disagreements.is_empty() {
            AgreementLevel::Disputed
        } else {
            theme.agreement_level.clone()
        };

        themes.push(SynthesizedTheme {
            label: theme.label.clone(),
            summary: build_consensus_summary(theme, num_providers),
            agreement_level,
            key_points,
            disagreements,
        });
    }

    // Add uncertainty if sources failed
    let failed_sources: Vec<&super::types::EvidenceSource> = matrix
        .sources
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                super::types::SourceStatus::Failed
                    | super::types::SourceStatus::TimedOut
                    | super::types::SourceStatus::Blocked
            )
        })
        .collect();

    if !failed_sources.is_empty() {
        let names: Vec<String> = failed_sources
            .iter()
            .map(|s| format!("{}/{}", provider_label(&s.provider), s.perspective_id))
            .collect();
        uncertainties.push(Uncertainty {
            text: format!(
                "{} source(s) did not complete: {}",
                failed_sources.len(),
                names.join(", ")
            ),
            reason: "Incomplete coverage may hide disagreements or additional insights."
                .to_string(),
        });
    }

    (themes, recommendations, uncertainties)
}

// ---------------------------------------------------------------------------
// Comprehensive strategy: include everything, organized by theme
// ---------------------------------------------------------------------------

fn synthesize_comprehensive(
    matrix: &EvidenceMatrix,
) -> (
    Vec<SynthesizedTheme>,
    Vec<SynthesizedRecommendation>,
    Vec<Uncertainty>,
) {
    // Comprehensive includes all themes, all claims — no filtering
    let mut themes = Vec::new();
    let mut recommendations = Vec::new();
    let mut uncertainties = Vec::new();

    for theme in &matrix.themes {
        let key_points: Vec<KeyPoint> = theme
            .claims
            .iter()
            .map(|c| KeyPoint {
                text: c.text.clone(),
                support: vec![c.source.clone()],
            })
            .collect();

        let provider_groups = group_claims_by_provider(theme);
        let disagreements = detect_cross_provider_disagreements(theme, &provider_groups);

        for claim in &theme.claims {
            if super::evidence::is_recommendation_type(&claim.section_type) {
                for point in extract_bullet_points(&claim.text) {
                    recommendations.push(SynthesizedRecommendation {
                        text: point,
                        priority: Priority::Medium,
                        support: vec![claim.source.clone()],
                    });
                }
            }
            if super::evidence::is_risk_or_caveat_type(&claim.section_type) {
                uncertainties.push(Uncertainty {
                    text: first_line(&claim.text),
                    reason: format!(
                        "Flagged by {} ({})",
                        provider_label(&claim.source.provider),
                        claim.source.perspective_id,
                    ),
                });
            }
        }

        themes.push(SynthesizedTheme {
            label: theme.label.clone(),
            summary: format!(
                "Addressed by {} source(s) across {} claim(s).",
                provider_groups.len(),
                theme.claims.len()
            ),
            agreement_level: theme.agreement_level.clone(),
            key_points,
            disagreements,
        });
    }

    (themes, recommendations, uncertainties)
}

// ---------------------------------------------------------------------------
// Executive strategy: concise, action-focused
// ---------------------------------------------------------------------------

fn synthesize_executive(
    matrix: &EvidenceMatrix,
) -> (
    Vec<SynthesizedTheme>,
    Vec<SynthesizedRecommendation>,
    Vec<Uncertainty>,
) {
    let mut themes = Vec::new();
    let mut recommendations = Vec::new();
    let mut uncertainties = Vec::new();

    // Only include themes that have multi-provider coverage or are summaries
    for theme in &matrix.themes {
        let provider_groups = group_claims_by_provider(theme);

        // Executive only shows themes with broad coverage or explicit disagreements
        let is_summary = theme.claims.iter().any(|c| {
            matches!(
                c.section_type,
                super::types::SectionType::Summary | super::types::SectionType::Recommendation
            )
        });

        if provider_groups.len() < 2 && !is_summary {
            continue;
        }

        let key_points: Vec<KeyPoint> = theme
            .claims
            .iter()
            .take(3) // Executive: limit depth
            .map(|c| KeyPoint {
                text: first_line(&c.text),
                support: vec![c.source.clone()],
            })
            .collect();

        let disagreements = detect_cross_provider_disagreements(theme, &provider_groups);

        themes.push(SynthesizedTheme {
            label: theme.label.clone(),
            summary: build_consensus_summary(theme, provider_groups.len()),
            agreement_level: theme.agreement_level.clone(),
            key_points,
            disagreements,
        });
    }

    // Executive: only high-priority recommendations (multi-provider)
    for theme in &matrix.themes {
        let provider_groups = group_claims_by_provider(theme);
        for claim in &theme.claims {
            if super::evidence::is_recommendation_type(&claim.section_type)
                && provider_groups.len() >= 2
            {
                for point in extract_bullet_points(&claim.text) {
                    recommendations.push(SynthesizedRecommendation {
                        text: point,
                        priority: Priority::High,
                        support: vec![claim.source.clone()],
                    });
                }
            }
        }
    }

    // Executive: only critical uncertainties
    let failed_count = matrix
        .sources
        .iter()
        .filter(|s| !matches!(s.status, super::types::SourceStatus::Available))
        .count();
    if failed_count > 0 {
        uncertainties.push(Uncertainty {
            text: format!("{failed_count} source(s) did not complete."),
            reason: "Synthesis may be incomplete.".to_string(),
        });
    }

    (themes, recommendations, uncertainties)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use std::collections::HashMap;

fn group_claims_by_provider(
    theme: &super::types::Theme,
) -> HashMap<String, Vec<&super::types::Claim>> {
    let mut groups: HashMap<String, Vec<&super::types::Claim>> = HashMap::new();
    for claim in &theme.claims {
        let key = format!("{:?}", claim.source.provider);
        groups.entry(key).or_default().push(claim);
    }
    groups
}

/// Detect disagreements by comparing claims from different providers.
/// Heuristic: if providers give different section types for the same theme heading,
/// that's a structural disagreement worth flagging.
fn detect_cross_provider_disagreements(
    theme: &super::types::Theme,
    provider_groups: &HashMap<String, Vec<&super::types::Claim>>,
) -> Vec<Disagreement> {
    if provider_groups.len() < 2 {
        return vec![];
    }

    let mut disagreements = Vec::new();

    // Check for mixed section types within the same theme (one says risk, another says recommendation)
    let section_types: Vec<(&str, &super::types::SectionType, &SourceRef)> = theme
        .claims
        .iter()
        .map(|c| {
            (
                provider_label(&c.source.provider),
                &c.section_type,
                &c.source,
            )
        })
        .collect();

    let has_risks = section_types
        .iter()
        .any(|(_, st, _)| matches!(st, super::types::SectionType::Risk));
    let has_recommendations = section_types
        .iter()
        .any(|(_, st, _)| matches!(st, super::types::SectionType::Recommendation));

    if has_risks && has_recommendations {
        let risk_sources: Vec<SourceRef> = section_types
            .iter()
            .filter(|(_, st, _)| matches!(st, super::types::SectionType::Risk))
            .map(|(_, _, sr)| (*sr).clone())
            .collect();
        let rec_sources: Vec<SourceRef> = section_types
            .iter()
            .filter(|(_, st, _)| matches!(st, super::types::SectionType::Recommendation))
            .map(|(_, _, sr)| (*sr).clone())
            .collect();

        disagreements.push(Disagreement {
            description: format!(
                "Mixed assessment in '{}': some sources flag risks while others recommend action.",
                theme.label
            ),
            positions: vec![
                Position {
                    stance: "Flags risks/concerns".to_string(),
                    sources: risk_sources,
                },
                Position {
                    stance: "Recommends action".to_string(),
                    sources: rec_sources,
                },
            ],
        });
    }

    disagreements
}

fn provider_label(p: &super::super::providers::types::ProviderName) -> &'static str {
    match p {
        super::super::providers::types::ProviderName::Claude => "Claude",
        super::super::providers::types::ProviderName::Codex => "Codex",
        super::super::providers::types::ProviderName::Gemini => "Gemini",
    }
}

fn build_consensus_summary(theme: &super::types::Theme, num_providers: usize) -> String {
    let claim_count = theme.claims.len();
    match num_providers {
        0 => "No sources available.".to_string(),
        1 => format!("Based on {claim_count} claim(s) from a single provider."),
        n => format!("Based on {claim_count} claim(s) across {n} providers."),
    }
}

fn extract_bullet_points(text: &str) -> Vec<String> {
    let mut points = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            points.push(rest.to_string());
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            points.push(rest.to_string());
        } else if trimmed.len() > 2
            && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            && trimmed.contains(". ")
        {
            if let Some(pos) = trimmed.find(". ") {
                points.push(trimmed[pos + 2..].to_string());
            }
        }
    }
    // If no bullets found, use the full text as one point
    if points.is_empty() && !text.trim().is_empty() {
        points.push(first_line(text));
    }
    points
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or(text).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthesis::evidence::build_evidence_matrix;
    use crate::synthesis::fixtures::{three_provider_one_failed, two_provider_results};

    #[test]
    fn test_consensus_strategy() {
        let results = two_provider_results();
        let (matrix, _) = build_evidence_matrix("s1", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Consensus);

        assert_eq!(output.strategy, SynthesisStrategy::Consensus);
        assert_eq!(output.synthesis_method, SynthesisMethod::Deterministic);
        assert!(!output.themes.is_empty());
        assert!(!output.recommendations.is_empty());
        assert_eq!(output.meta.total_sources, 2);
        assert_eq!(output.meta.completed_sources, 2);
    }

    #[test]
    fn test_comprehensive_strategy() {
        let results = two_provider_results();
        let (matrix, _) = build_evidence_matrix("s1", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Comprehensive);

        assert_eq!(output.strategy, SynthesisStrategy::Comprehensive);
        assert!(output.themes.len() >= 2);
    }

    #[test]
    fn test_executive_strategy() {
        let results = two_provider_results();
        let (matrix, _) = build_evidence_matrix("s1", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Executive);

        assert_eq!(output.strategy, SynthesisStrategy::Executive);
        assert!(output.themes.len() <= matrix.themes.len());
    }

    #[test]
    fn test_synthesis_with_failures() {
        let results = three_provider_one_failed();
        let (matrix, _) = build_evidence_matrix("s2", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Consensus);

        assert_eq!(output.meta.failed_sources, 1);
        assert!(output
            .uncertainties
            .iter()
            .any(|u| u.text.contains("did not complete")));
    }

    #[test]
    fn test_extract_bullet_points() {
        let text = "- Point A\n- Point B\n- Point C";
        let points = extract_bullet_points(text);
        assert_eq!(points, vec!["Point A", "Point B", "Point C"]);
    }

    #[test]
    fn test_extract_numbered_points() {
        let text = "1. First thing\n2. Second thing";
        let points = extract_bullet_points(text);
        assert_eq!(points, vec!["First thing", "Second thing"]);
    }

    #[test]
    fn test_extract_no_bullets() {
        let text = "Just plain text description.";
        let points = extract_bullet_points(text);
        assert_eq!(points, vec!["Just plain text description."]);
    }
}
