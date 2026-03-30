use super::types::{AgreementLevel, Priority, SynthesisOutput, SynthesisStrategy};

/// Render a `brief.md` from structured synthesis output.
/// This is fully deterministic — same input always produces same output.
pub fn render_brief(output: &SynthesisOutput) -> String {
    let mut md = String::new();

    // Header
    md.push_str("# Research Brief\n\n");
    md.push_str(&format!(
        "**Strategy:** {} | **Sources:** {}/{} completed\n\n",
        strategy_label(&output.strategy),
        output.meta.completed_sources,
        output.meta.total_sources,
    ));

    if output.meta.failed_sources > 0 {
        md.push_str(&format!(
            "> **Note:** {} source(s) did not complete. Results may be incomplete.\n\n",
            output.meta.failed_sources
        ));
    }

    md.push_str("---\n\n");

    // Themes
    if !output.themes.is_empty() {
        md.push_str("## Key Themes\n\n");

        for theme in &output.themes {
            md.push_str(&format!("### {}\n\n", theme.label));
            md.push_str(&format!(
                "**Agreement:** {} | {}\n\n",
                agreement_label(&theme.agreement_level),
                theme.summary
            ));

            // Key points with provenance
            if !theme.key_points.is_empty() {
                for point in &theme.key_points {
                    let sources: Vec<String> = point
                        .support
                        .iter()
                        .map(|s| {
                            format!("{}/{}", provider_label_short(&s.provider), s.perspective_id)
                        })
                        .collect();
                    // Indent multi-line content
                    let first_line = point.text.lines().next().unwrap_or(&point.text);
                    md.push_str(&format!(
                        "- {} *({})*\n",
                        first_line.trim(),
                        sources.join(", ")
                    ));
                }
                md.push('\n');
            }

            // Disagreements
            if !theme.disagreements.is_empty() {
                md.push_str("**Disagreements:**\n\n");
                for disagreement in &theme.disagreements {
                    md.push_str(&format!("- {}\n", disagreement.description));
                    for position in &disagreement.positions {
                        let sources: Vec<String> = position
                            .sources
                            .iter()
                            .map(|s| {
                                format!(
                                    "{}/{}",
                                    provider_label_short(&s.provider),
                                    s.perspective_id
                                )
                            })
                            .collect();
                        md.push_str(&format!(
                            "  - **{}** — {}\n",
                            position.stance,
                            sources.join(", ")
                        ));
                    }
                }
                md.push('\n');
            }
        }
    }

    // Recommendations
    if !output.recommendations.is_empty() {
        md.push_str("## Recommendations\n\n");
        // Sort by priority
        let mut recs = output.recommendations.clone();
        recs.sort_by(|a, b| priority_rank(&a.priority).cmp(&priority_rank(&b.priority)));

        for rec in &recs {
            let sources: Vec<String> = rec
                .support
                .iter()
                .map(|s| format!("{}/{}", provider_label_short(&s.provider), s.perspective_id))
                .collect();
            md.push_str(&format!(
                "- **[{}]** {} *({})*\n",
                priority_label(&rec.priority),
                rec.text,
                sources.join(", "),
            ));
        }
        md.push('\n');
    }

    // Disagreements summary
    let total_disagreements: usize = output.themes.iter().map(|t| t.disagreements.len()).sum();
    if total_disagreements > 0 {
        md.push_str("## Disagreements\n\n");
        for theme in &output.themes {
            for d in &theme.disagreements {
                md.push_str(&format!("- **{}:** {}\n", theme.label, d.description));
            }
        }
        md.push('\n');
    }

    // Uncertainties
    if !output.uncertainties.is_empty() {
        md.push_str("## Uncertainties\n\n");
        for u in &output.uncertainties {
            md.push_str(&format!("- {} — *{}*\n", u.text, u.reason));
        }
        md.push('\n');
    }

    // Coverage
    md.push_str("## Source Coverage\n\n");
    md.push_str("| Provider | Perspective | Status |\n");
    md.push_str("|----------|-------------|--------|\n");
    // Use evidence matrix info from meta
    // We don't have coverage cells in SynthesisOutput, so we note the summary
    md.push_str(&format!(
        "| *{} total* | — | *{} completed, {} failed* |\n",
        output.meta.total_sources, output.meta.completed_sources, output.meta.failed_sources
    ));
    md.push('\n');

    // Footer
    md.push_str("---\n\n");
    md.push_str(&format!(
        "*Generated by Multi-Model Synthesizer — {} strategy*\n",
        strategy_label(&output.strategy)
    ));

    md
}

/// Render a more detailed brief that includes coverage cell detail from the evidence matrix.
pub fn render_brief_with_coverage(
    output: &SynthesisOutput,
    coverage_cells: &[(String, String, String)], // (provider, perspective, status)
) -> String {
    let mut md = render_brief(output);

    // Replace the simple coverage table with detailed one
    if !coverage_cells.is_empty() {
        // Find and replace the coverage section
        if let Some(pos) = md.find("## Source Coverage\n") {
            if let Some(end) = md[pos..].find("\n---\n") {
                let replacement = build_coverage_section(coverage_cells);
                md.replace_range(pos..pos + end, &replacement);
            }
        }
    }

    md
}

fn build_coverage_section(cells: &[(String, String, String)]) -> String {
    let mut s = String::from("## Source Coverage\n\n");
    s.push_str("| Provider | Perspective | Status |\n");
    s.push_str("|----------|-------------|--------|\n");
    for (provider, perspective, status) in cells {
        let status_icon = match status.as_str() {
            "available" => "completed",
            "failed" => "FAILED",
            "blocked" => "BLOCKED",
            "timed_out" => "TIMED OUT",
            "empty" => "empty",
            other => other,
        };
        s.push_str(&format!(
            "| {} | {} | {} |\n",
            provider, perspective, status_icon
        ));
    }
    s.push('\n');
    s
}

fn strategy_label(s: &SynthesisStrategy) -> &'static str {
    match s {
        SynthesisStrategy::Consensus => "Consensus",
        SynthesisStrategy::Comprehensive => "Comprehensive",
        SynthesisStrategy::Executive => "Executive",
    }
}

fn agreement_label(a: &AgreementLevel) -> &'static str {
    match a {
        AgreementLevel::Full => "Full agreement",
        AgreementLevel::Strong => "Strong agreement",
        AgreementLevel::Partial => "Partial agreement",
        AgreementLevel::Disputed => "DISPUTED",
        AgreementLevel::Single => "Single source",
    }
}

fn priority_label(p: &Priority) -> &'static str {
    match p {
        Priority::High => "High",
        Priority::Medium => "Medium",
        Priority::Low => "Low",
    }
}

fn priority_rank(p: &Priority) -> u8 {
    match p {
        Priority::High => 0,
        Priority::Medium => 1,
        Priority::Low => 2,
    }
}

fn provider_label_short(p: &crate::providers::types::ProviderName) -> &'static str {
    match p {
        crate::providers::types::ProviderName::Claude => "claude",
        crate::providers::types::ProviderName::Codex => "codex",
        crate::providers::types::ProviderName::Gemini => "gemini",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::types::JobState;
    use crate::providers::types::ProviderName;
    use crate::synthesis::evidence::build_evidence_matrix;
    use crate::synthesis::fixtures::{
        claude_json, make_job_result, three_provider_one_failed, two_provider_results,
    };
    use crate::synthesis::strategies::synthesize;

    #[test]
    fn test_render_brief_consensus() {
        let results = two_provider_results();
        let (matrix, _) = build_evidence_matrix("s1", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Consensus);
        let brief = render_brief(&output);

        assert!(brief.contains("# Research Brief"));
        assert!(brief.contains("Consensus"));
        assert!(brief.contains("2/2 completed"));
        assert!(brief.contains("## Key Themes"));
        assert!(brief.contains("## Recommendations"));
    }

    #[test]
    fn test_render_brief_with_failures() {
        let results = three_provider_one_failed();
        let (matrix, _) = build_evidence_matrix("s2", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Consensus);
        let brief = render_brief(&output);

        assert!(brief.contains("1 source(s) did not complete"));
        assert!(brief.contains("2/3 completed"));
    }

    #[test]
    fn test_render_brief_deterministic() {
        let results = vec![make_job_result(
            "j1",
            ProviderName::Claude,
            "default",
            JobState::Completed,
            &claude_json("## Summary\n\nGood."),
        )];

        let (matrix, _) = build_evidence_matrix("s1", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Consensus);

        let brief1 = render_brief(&output);
        let brief2 = render_brief(&output);
        assert_eq!(brief1, brief2, "Brief rendering must be deterministic");
    }

    #[test]
    fn test_render_brief_comprehensive() {
        let results = vec![
            make_job_result(
                "j1",
                ProviderName::Claude,
                "default",
                JobState::Completed,
                &claude_json(
                    "## Analysis\n\nDetailed findings here.\n\n## Risks\n\n- Memory leak potential",
                ),
            ),
            make_job_result(
                "j2",
                ProviderName::Codex,
                "adversarial",
                JobState::Completed,
                "## Analysis\n\nAlternative view.\n\n## Caveats\n\n- Limited sample size",
            ),
        ];

        let (matrix, _) = build_evidence_matrix("s3", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Comprehensive);
        let brief = render_brief(&output);

        assert!(brief.contains("Comprehensive"));
        assert!(brief.contains("## Uncertainties"));
    }

    #[test]
    fn test_render_brief_executive() {
        let results = two_provider_results();
        let (matrix, _) = build_evidence_matrix("s4", &results);
        let output = synthesize(&matrix, SynthesisStrategy::Executive);
        let brief = render_brief(&output);

        assert!(brief.contains("Executive"));
    }

    #[test]
    fn test_render_empty_synthesis() {
        let output = SynthesisOutput {
            schema_version: 1,
            session_id: "empty".to_string(),
            strategy: SynthesisStrategy::Consensus,
            synthesis_method: super::super::types::SynthesisMethod::Deterministic,
            themes: vec![],
            recommendations: vec![],
            uncertainties: vec![],
            meta: super::super::types::SynthesisMeta {
                total_sources: 0,
                completed_sources: 0,
                failed_sources: 0,
                strategy_name: "consensus".to_string(),
                synthesis_duration_ms: Some(1),
            },
        };

        let brief = render_brief(&output);
        assert!(brief.contains("# Research Brief"));
        assert!(brief.contains("0/0 completed"));
    }
}
