use crate::providers::types::ProviderName;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Normalized run — one per job (provider × perspective)
// ---------------------------------------------------------------------------

/// Parsed and normalized output from a single provider run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRun {
    pub schema_version: u32,
    pub job_id: String,
    pub provider: ProviderName,
    pub perspective_id: String,
    /// The extracted response text from the provider output.
    pub response_text: String,
    /// Provider-specific metadata extracted during normalization.
    pub provider_metadata: ProviderOutputMetadata,
    /// Sections parsed from the response text (markdown heading-based).
    pub sections: Vec<ResponseSection>,
    pub extraction_status: ExtractionStatus,
    /// Relative path to the raw stdout.txt artifact within the session dir.
    pub raw_artifact_path: String,
}

/// Metadata extracted from the provider-specific output envelope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderOutputMetadata {
    pub output_format: OutputFormat,
    pub parse_success: bool,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
    pub token_counts: Option<TokenCounts>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCounts {
    pub prompt: Option<u64>,
    pub completion: Option<u64>,
    pub cached: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
    #[default]
    Text,
}

/// A section parsed from the response markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSection {
    pub heading: Option<String>,
    pub content: String,
    pub section_type: SectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    Finding,
    Recommendation,
    Risk,
    Caveat,
    Summary,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStatus {
    /// Provider output parsed successfully; response text and sections extracted.
    Success,
    /// Provider output was not valid JSON (or wrong shape); raw text used as-is.
    ParseFailed,
    /// Provider returned empty output.
    Empty,
    /// Job did not complete (failed, timed out, blocked).
    JobNotCompleted,
}

// ---------------------------------------------------------------------------
// Evidence matrix — one per session
// ---------------------------------------------------------------------------

/// A grid of all provider×perspective responses with cross-cutting theme tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMatrix {
    pub schema_version: u32,
    pub session_id: String,
    pub sources: Vec<EvidenceSource>,
    pub themes: Vec<Theme>,
    pub coverage: CoverageMatrix,
}

/// One source in the evidence grid (one provider×perspective job).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSource {
    pub job_id: String,
    pub provider: ProviderName,
    pub perspective_id: String,
    pub status: SourceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Available,
    Failed,
    Blocked,
    TimedOut,
    Empty,
}

/// A theme groups related content found across multiple sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: String,
    pub label: String,
    pub claims: Vec<Claim>,
    pub agreement_level: AgreementLevel,
    pub disagreements: Vec<Disagreement>,
}

/// A single claim extracted from a provider response section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub section_type: SectionType,
    pub source: SourceRef,
}

/// Reference back to a specific provider×perspective source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRef {
    pub job_id: String,
    pub provider: ProviderName,
    pub perspective_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgreementLevel {
    /// All available sources addressed this theme similarly.
    Full,
    /// Most sources agree.
    Strong,
    /// Some agreement, some disagreement.
    Partial,
    /// Significant disagreement between sources.
    Disputed,
    /// Only one source addressed this theme.
    Single,
}

/// An explicit disagreement between sources on a theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disagreement {
    pub description: String,
    pub positions: Vec<Position>,
}

/// One side of a disagreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub stance: String,
    pub sources: Vec<SourceRef>,
}

/// Grid showing which provider×perspective combinations completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMatrix {
    pub providers: Vec<ProviderName>,
    pub perspectives: Vec<String>,
    pub cells: Vec<CoverageCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageCell {
    pub provider: ProviderName,
    pub perspective: String,
    pub status: SourceStatus,
    pub job_id: String,
}

// ---------------------------------------------------------------------------
// Synthesis output — one per session
// ---------------------------------------------------------------------------

/// The structured synthesis produced by applying a strategy to the evidence matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisOutput {
    pub schema_version: u32,
    pub session_id: String,
    pub strategy: SynthesisStrategy,
    pub synthesis_method: SynthesisMethod,
    pub themes: Vec<SynthesizedTheme>,
    pub recommendations: Vec<SynthesizedRecommendation>,
    pub uncertainties: Vec<Uncertainty>,
    pub meta: SynthesisMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisStrategy {
    /// Focus on points of agreement; explicitly flag disagreements.
    Consensus,
    /// Include all themes and claims from all providers.
    Comprehensive,
    /// Concise action-oriented summary.
    Executive,
}

impl std::fmt::Display for SynthesisStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Consensus => write!(f, "consensus"),
            Self::Comprehensive => write!(f, "comprehensive"),
            Self::Executive => write!(f, "executive"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisMethod {
    /// Deterministic code-based synthesis (no model call).
    Deterministic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedTheme {
    pub label: String,
    pub summary: String,
    pub agreement_level: AgreementLevel,
    pub key_points: Vec<KeyPoint>,
    pub disagreements: Vec<Disagreement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPoint {
    pub text: String,
    pub support: Vec<SourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedRecommendation {
    pub text: String,
    pub priority: Priority,
    pub support: Vec<SourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub text: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisMeta {
    pub total_sources: usize,
    pub completed_sources: usize,
    pub failed_sources: usize,
    pub strategy_name: String,
    pub synthesis_duration_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Session artifact listing — for the artifact viewer
// ---------------------------------------------------------------------------

/// An entry in the session artifact listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    pub relative_path: String,
    pub artifact_type: String,
    pub size_bytes: u64,
}
