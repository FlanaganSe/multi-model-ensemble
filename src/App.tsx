import { useCallback, useEffect, useState } from "react";
import { ArtifactViewer } from "./features/artifact-viewer/ArtifactViewer";
import { createSession, deleteSession, listSessions, probeProviders } from "./lib/api";
import type { ProviderProbeResult, SessionListEntry } from "./lib/types";

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
	const [viewingSession, setViewingSession] = useState<string | null>(null);

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

	const handleCreateSession = async () => {
		try {
			await createSession();
			await refresh();
		} catch (e) {
			setError(String(e));
		}
	};

	const handleDeleteSession = async (id: string) => {
		try {
			await deleteSession(id);
			await refresh();
		} catch (e) {
			setError(String(e));
		}
	};

	if (viewingSession) {
		return <ArtifactViewer sessionId={viewingSession} onClose={() => setViewingSession(null)} />;
	}

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
						onClick={handleCreateSession}
						style={{
							background: "#2563eb",
							color: "white",
							border: "none",
							borderRadius: 4,
							padding: "4px 12px",
							cursor: "pointer",
						}}
					>
						New Session
					</button>
				</div>
				{sessions.length === 0 ? (
					<div style={{ color: "#888" }}>No sessions yet.</div>
				) : (
					sessions.map((s) => (
						<div
							key={s.id}
							style={{
								border: "1px solid #333",
								borderRadius: 8,
								padding: 12,
								marginBottom: 8,
								background: "#1a1a1a",
								display: "flex",
								justifyContent: "space-between",
								alignItems: "center",
							}}
						>
							<button
								type="button"
								onClick={() => setViewingSession(s.id)}
								style={{
									background: "none",
									border: "none",
									color: "#e0e0e0",
									cursor: "pointer",
									textAlign: "left",
									padding: 0,
								}}
							>
								<div style={{ fontWeight: 600 }}>{s.label ?? s.id}</div>
								<div style={{ fontSize: 13, color: "#888" }}>
									{s.created_at} &middot; {s.status}
								</div>
							</button>
							{s.status === "active" && (
								<button
									type="button"
									onClick={() => handleDeleteSession(s.id)}
									style={{
										background: "#991b1b",
										color: "white",
										border: "none",
										borderRadius: 4,
										padding: "4px 12px",
										cursor: "pointer",
									}}
								>
									Delete
								</button>
							)}
						</div>
					))
				)}
			</section>
		</div>
	);
}
