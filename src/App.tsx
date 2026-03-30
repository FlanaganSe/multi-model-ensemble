import { useCallback, useEffect, useState } from "react";
import { ArtifactViewer } from "./features/artifact-viewer/ArtifactViewer";
import { RunComposer } from "./features/run-composer/RunComposer";
import { archiveSession, deleteSession, listSessions, probeProviders } from "./lib/api";
import type { ProviderProbeResult, RunSummary, SessionListEntry } from "./lib/types";

type View = { kind: "dashboard" } | { kind: "composer" } | { kind: "session"; id: string };

function statusColor(status: ProviderProbeResult["status"]): string {
	switch (status) {
		case "ready":
			return "#22c55e";
		case "not_installed":
			return "#ef4444";
		case "not_authenticated":
			return "#f59e0b";
		case "error":
			return "#ef4444";
	}
}

function statusLabel(status: ProviderProbeResult["status"]): string {
	switch (status) {
		case "ready":
			return "Ready";
		case "not_installed":
			return "Not Installed";
		case "not_authenticated":
			return "Not Authenticated";
		case "error":
			return "Error";
	}
}

function ProviderCard({ probe }: { probe: ProviderProbeResult }) {
	return (
		<div
			style={{
				border: "1px solid #333",
				borderRadius: 8,
				padding: 16,
				marginBottom: 8,
				background: "#1a1a1a",
			}}
		>
			<div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
				<span
					style={{
						width: 10,
						height: 10,
						borderRadius: "50%",
						background: statusColor(probe.status),
						display: "inline-block",
					}}
				/>
				<strong style={{ textTransform: "capitalize" }}>{probe.provider}</strong>
				<span style={{ color: "#888", fontSize: 14 }}>{statusLabel(probe.status)}</span>
			</div>
			{probe.version && <div style={{ fontSize: 13, color: "#aaa" }}>Version: {probe.version}</div>}
			{probe.executable_path && (
				<div style={{ fontSize: 13, color: "#aaa" }}>Path: {probe.executable_path}</div>
			)}
			{probe.blocked_reason && (
				<div style={{ fontSize: 13, color: "#f59e0b", marginTop: 4 }}>{probe.blocked_reason}</div>
			)}
			{probe.remediation && (
				<div style={{ fontSize: 13, color: "#888", marginTop: 2 }}>{probe.remediation}</div>
			)}
		</div>
	);
}

export function App() {
	const [providers, setProviders] = useState<ProviderProbeResult[]>([]);
	const [sessions, setSessions] = useState<SessionListEntry[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [view, setView] = useState<View>({ kind: "dashboard" });

	const refresh = useCallback(async () => {
		try {
			const [providerResults, sessionResults] = await Promise.all([
				probeProviders(),
				listSessions(),
			]);
			setProviders(providerResults);
			setSessions(sessionResults);
			setError(null);
		} catch (e) {
			setError(String(e));
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		refresh();
	}, [refresh]);

	const handleDeleteSession = async (id: string) => {
		try {
			await deleteSession(id);
			await refresh();
		} catch (e) {
			setError(String(e));
		}
	};

	const handleArchiveSession = async (id: string) => {
		try {
			await archiveSession(id);
			await refresh();
		} catch (e) {
			setError(String(e));
		}
	};

	const handleRunComplete = async (_summary: RunSummary) => {
		await refresh();
	};

	if (view.kind === "session") {
		return (
			<ArtifactViewer
				sessionId={view.id}
				onClose={() => {
					setView({ kind: "dashboard" });
					refresh();
				}}
			/>
		);
	}

	if (view.kind === "composer") {
		return (
			<RunComposer
				providers={providers}
				onRunComplete={handleRunComplete}
				onCancel={() => setView({ kind: "dashboard" })}
			/>
		);
	}

	const readyProviders = providers.filter((p) => p.status === "ready").length;

	return (
		<div style={{ padding: 24, fontFamily: "system-ui, sans-serif", color: "#e0e0e0" }}>
			<h1 style={{ fontSize: 24, marginBottom: 16 }}>Multi-Model Synthesizer</h1>

			{error && (
				<div
					style={{
						background: "#3b1111",
						border: "1px solid #ef4444",
						borderRadius: 8,
						padding: 12,
						marginBottom: 16,
						color: "#fca5a5",
					}}
				>
					{error}
				</div>
			)}

			<section style={{ marginBottom: 32 }}>
				<h2 style={{ fontSize: 18, marginBottom: 12 }}>Provider Status</h2>
				{loading ? (
					<div style={{ color: "#888" }}>Probing providers...</div>
				) : (
					providers.map((p) => <ProviderCard key={p.provider} probe={p} />)
				)}
			</section>

			<section>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						gap: 12,
						marginBottom: 12,
					}}
				>
					<h2 style={{ fontSize: 18, margin: 0 }}>Sessions</h2>
					<button
						type="button"
						onClick={() => setView({ kind: "composer" })}
						disabled={readyProviders === 0}
						title={readyProviders === 0 ? "No providers are ready" : undefined}
						style={{
							background: readyProviders > 0 ? "#2563eb" : "#333",
							color: readyProviders > 0 ? "white" : "#888",
							border: "none",
							borderRadius: 4,
							padding: "4px 12px",
							cursor: readyProviders > 0 ? "pointer" : "not-allowed",
						}}
					>
						New Run
					</button>
				</div>
				{sessions.length === 0 ? (
					<div style={{ color: "#888" }}>No sessions yet. Start a new run to begin.</div>
				) : (
					sessions.map((s) => (
						<SessionCard
							key={s.id}
							session={s}
							onView={() => setView({ kind: "session", id: s.id })}
							onArchive={() => handleArchiveSession(s.id)}
							onDelete={() => handleDeleteSession(s.id)}
						/>
					))
				)}
			</section>
		</div>
	);
}

function SessionCard({
	session,
	onView,
	onArchive,
	onDelete,
}: {
	session: SessionListEntry;
	onView: () => void;
	onArchive: () => void;
	onDelete: () => void;
}) {
	const isActive = session.status === "active";
	const isArchived = session.status === "archived";

	return (
		<div
			style={{
				border: "1px solid #333",
				borderRadius: 8,
				padding: 12,
				marginBottom: 8,
				background: isArchived ? "#111" : "#1a1a1a",
				display: "flex",
				justifyContent: "space-between",
				alignItems: "center",
				opacity: isArchived ? 0.7 : 1,
			}}
		>
			<button
				type="button"
				onClick={onView}
				style={{
					background: "none",
					border: "none",
					color: "#e0e0e0",
					cursor: "pointer",
					textAlign: "left",
					padding: 0,
					flex: 1,
				}}
			>
				<div style={{ fontWeight: 600 }}>{session.label ?? session.id.slice(0, 8)}</div>
				<div style={{ fontSize: 13, color: "#888" }}>
					{formatDate(session.created_at)}
					{isArchived && (
						<span
							style={{
								marginLeft: 8,
								color: "#666",
								fontSize: 11,
								textTransform: "uppercase",
							}}
						>
							archived
						</span>
					)}
				</div>
			</button>
			<div style={{ display: "flex", gap: 6 }}>
				{isActive && (
					<button
						type="button"
						onClick={onArchive}
						title="Archive session"
						style={{
							background: "#333",
							color: "#aaa",
							border: "none",
							borderRadius: 4,
							padding: "4px 10px",
							cursor: "pointer",
							fontSize: 12,
						}}
					>
						Archive
					</button>
				)}
				<button
					type="button"
					onClick={onDelete}
					title="Delete session permanently"
					style={{
						background: "#991b1b",
						color: "white",
						border: "none",
						borderRadius: 4,
						padding: "4px 10px",
						cursor: "pointer",
						fontSize: 12,
					}}
				>
					Delete
				</button>
			</div>
		</div>
	);
}

function formatDate(iso: string): string {
	try {
		const d = new Date(iso);
		return d.toLocaleString();
	} catch {
		return iso;
	}
}
