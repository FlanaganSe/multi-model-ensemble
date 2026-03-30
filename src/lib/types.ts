export type ProviderName = "claude" | "codex" | "gemini";

export interface Perspective {
	id: string;
	label: string;
	instructions: string;
}

export type SynthesisStrategy = "consensus" | "comprehensive" | "executive";

export type ProviderStatus = "ready" | "not_installed" | "not_authenticated" | "error";

export interface ProviderProbeResult {
	provider: ProviderName;
	status: ProviderStatus;
	executable_path: string | null;
	version: string | null;
	auth_ready: boolean;
	blocked_reason: string | null;
	remediation: string | null;
}

export interface SessionMetadata {
	schema_version: number;
	id: string;
	created_at: string;
	status: "active" | "archived";
	label: string | null;
}

export interface SessionListEntry {
	id: string;
	created_at: string;
	status: "active" | "archived";
	label: string | null;
	path: string;
}

// Synthesis types

export interface SessionArtifact {
	relative_path: string;
	artifact_type: string;
	size_bytes: number;
}

export interface JobState {
	state: "queued" | "running" | "completed" | "failed" | "timed_out" | "blocked" | "cancelled";
}

export interface RunSummary {
	session_id: string;
	total_jobs: number;
	completed: number;
	failed: number;
	timed_out: number;
	blocked: number;
	cancelled: number;
}

export interface JobResult {
	job_id: string;
	provider: ProviderName;
	perspective_id: string;
	state: "queued" | "running" | "completed" | "failed" | "timed_out" | "blocked" | "cancelled";
	started_at: string | null;
	ended_at: string | null;
	duration_ms: number | null;
	exit_code: number | null;
	stdout: string;
	stderr: string;
	blocked_reason: string | null;
	blocked_remediation: string | null;
	error: string | null;
}

export interface SourceRef {
	job_id: string;
	provider: ProviderName;
	perspective_id: string;
}

export interface EvidenceMatrix {
	schema_version: number;
	session_id: string;
	sources: Array<{
		job_id: string;
		provider: ProviderName;
		perspective_id: string;
		status: string;
	}>;
	themes: Array<{
		id: string;
		label: string;
		claims: Array<{
			id: string;
			text: string;
			section_type: string;
			source: SourceRef;
		}>;
		agreement_level: string;
		disagreements: Array<{
			description: string;
			positions: Array<{
				stance: string;
				sources: SourceRef[];
			}>;
		}>;
	}>;
	coverage: {
		providers: ProviderName[];
		perspectives: string[];
		cells: Array<{
			provider: ProviderName;
			perspective: string;
			status: string;
			job_id: string;
		}>;
	};
}
