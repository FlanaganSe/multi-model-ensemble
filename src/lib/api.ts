import { invoke } from "@tauri-apps/api/core";
import type { ProviderProbeResult, SessionListEntry } from "./types";

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
