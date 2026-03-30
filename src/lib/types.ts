export type ProviderName = "claude" | "codex" | "gemini";

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
