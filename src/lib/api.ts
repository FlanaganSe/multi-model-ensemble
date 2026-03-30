import { invoke } from "@tauri-apps/api/core";
import type {
	EvidenceMatrix,
	ProviderProbeResult,
	RunSummary,
	SessionArtifact,
	SessionListEntry,
} from "./types";

export async function probeProviders(): Promise<ProviderProbeResult[]> {
	return invoke<ProviderProbeResult[]>("probe_providers");
}

export async function createSession(label?: string): Promise<SessionListEntry> {
	return invoke<SessionListEntry>("create_session", { label: label ?? null });
}

export async function listSessions(): Promise<SessionListEntry[]> {
	return invoke<SessionListEntry[]>("list_sessions");
}

export async function archiveSession(sessionId: string): Promise<void> {
	return invoke<void>("archive_session", { sessionId });
}

export async function deleteSession(sessionId: string): Promise<void> {
	return invoke<void>("delete_session", { sessionId });
}

export async function runSession(params: {
	prompt: string;
	providers: string[];
	perspectives: string[];
	workingDirectory?: string;
	contextPaths?: string[];
	timeoutSecs?: number;
	label?: string;
	strategy?: string;
}): Promise<RunSummary> {
	return invoke<RunSummary>("run_session", {
		prompt: params.prompt,
		providers: params.providers,
		perspectives: params.perspectives,
		workingDirectory: params.workingDirectory ?? null,
		contextPaths: params.contextPaths ?? null,
		timeoutSecs: params.timeoutSecs ?? null,
		label: params.label ?? null,
		strategy: params.strategy ?? null,
	});
}

export async function getBrief(sessionId: string): Promise<string> {
	return invoke<string>("get_brief", { sessionId });
}

export async function getEvidenceMatrix(sessionId: string): Promise<EvidenceMatrix> {
	return invoke<EvidenceMatrix>("get_evidence_matrix", { sessionId });
}

export async function getNormalizedRuns(sessionId: string): Promise<unknown[]> {
	return invoke<unknown[]>("get_normalized_runs", { sessionId });
}

export async function getSessionArtifacts(sessionId: string): Promise<SessionArtifact[]> {
	return invoke<SessionArtifact[]>("get_session_artifacts", { sessionId });
}

export async function readArtifact(sessionId: string, relativePath: string): Promise<string> {
	return invoke<string>("read_artifact", { sessionId, relativePath });
}
