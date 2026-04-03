import { useCallback, useEffect, useState } from "react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import remarkGfm from "remark-gfm";
import { getBrief, getEvidenceMatrix, getSessionArtifacts, readArtifact } from "../../lib/api";
import type { EvidenceMatrix, SessionArtifact } from "../../lib/types";
import { CodeBlock } from "./components/CodeBlock";
import "highlight.js/styles/github-dark.css";
import "./brief-prose.css";

interface ArtifactViewerProps {
	sessionId: string;
	onClose: () => void;
}

type Tab = "brief" | "evidence" | "artifacts";

export function ArtifactViewer({ sessionId, onClose }: ArtifactViewerProps) {
	const [tab, setTab] = useState<Tab>("brief");
	const [brief, setBrief] = useState<string | null>(null);
	const [evidenceMatrix, setEvidenceMatrix] = useState<EvidenceMatrix | null>(null);
	const [artifacts, setArtifacts] = useState<SessionArtifact[]>([]);
	const [selectedArtifact, setSelectedArtifact] = useState<string | null>(null);
	const [artifactContent, setArtifactContent] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(true);

	const loadData = useCallback(async () => {
		setLoading(true);
		try {
			const [briefResult, artifactsResult] = await Promise.allSettled([
				getBrief(sessionId),
				getSessionArtifacts(sessionId),
			]);

			if (briefResult.status === "fulfilled") {
				setBrief(briefResult.value);
			}
			if (artifactsResult.status === "fulfilled") {
				setArtifacts(artifactsResult.value);
			}

			try {
				const matrix = await getEvidenceMatrix(sessionId);
				setEvidenceMatrix(matrix);
			} catch {
				// Evidence matrix may not exist yet
			}
		} catch (e) {
			setError(String(e));
		} finally {
			setLoading(false);
		}
	}, [sessionId]);

	useEffect(() => {
		loadData();
	}, [loadData]);

	const handleArtifactClick = async (path: string) => {
		setSelectedArtifact(path);
		try {
			const content = await readArtifact(sessionId, path);
			setArtifactContent(content);
		} catch (e) {
			setArtifactContent(`Error loading artifact: ${e}`);
		}
	};

	if (loading) {
		return <LoadingSkeleton />;
	}

	const tabStyle = (t: Tab) => ({
		padding: "8px 16px",
		background: tab === t ? "#2563eb" : "transparent",
		color: tab === t ? "white" : "#aaa",
		border: "1px solid #333",
		borderRadius: 4,
		cursor: "pointer" as const,
	});

	return (
		<div style={{ padding: 24, fontFamily: "system-ui, sans-serif", color: "#e0e0e0" }}>
			<div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
				<button
					type="button"
					onClick={onClose}
					style={{
						background: "#333",
						color: "#aaa",
						border: "none",
						borderRadius: 4,
						padding: "4px 12px",
						cursor: "pointer",
					}}
				>
					Back
				</button>
				<h2 style={{ margin: 0, fontSize: 18 }}>Session: {sessionId.slice(0, 8)}...</h2>
			</div>

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

			<div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
				<button type="button" onClick={() => setTab("brief")} style={tabStyle("brief")}>
					Brief
				</button>
				<button type="button" onClick={() => setTab("evidence")} style={tabStyle("evidence")}>
					Evidence Matrix
				</button>
				<button type="button" onClick={() => setTab("artifacts")} style={tabStyle("artifacts")}>
					All Artifacts ({artifacts.length})
				</button>
			</div>

			{tab === "brief" && <BriefView brief={brief} />}
			{tab === "evidence" && <EvidenceView matrix={evidenceMatrix} />}
			{tab === "artifacts" && (
				<ArtifactsListView
					artifacts={artifacts}
					selectedArtifact={selectedArtifact}
					artifactContent={artifactContent}
					onArtifactClick={handleArtifactClick}
				/>
			)}
		</div>
	);
}

function LoadingSkeleton() {
	return (
		<div style={{ padding: 24 }} className="skeleton-container">
			<div className="skeleton-bar" style={{ width: "40%", height: 24, marginBottom: 16 }} />
			<div className="skeleton-bar" style={{ width: "60%", height: 14, marginBottom: 8 }} />
			<div className="skeleton-bar" style={{ width: "100%", height: 1, marginBottom: 16 }} />
			<div className="skeleton-bar" style={{ width: "30%", height: 18, marginBottom: 12 }} />
			<div className="skeleton-bar" style={{ width: "90%", height: 14, marginBottom: 6 }} />
			<div className="skeleton-bar" style={{ width: "85%", height: 14, marginBottom: 6 }} />
			<div className="skeleton-bar" style={{ width: "70%", height: 14, marginBottom: 16 }} />
			<div className="skeleton-bar" style={{ width: "25%", height: 18, marginBottom: 12 }} />
			<div className="skeleton-bar" style={{ width: "80%", height: 14, marginBottom: 6 }} />
			<div className="skeleton-bar" style={{ width: "95%", height: 14, marginBottom: 6 }} />
		</div>
	);
}

const remarkPlugins = [remarkGfm];
const rehypePlugins = [rehypeHighlight];
const markdownComponents = { pre: CodeBlock };

export function BriefView({ brief }: { brief: string | null }) {
	if (!brief) {
		return <div style={{ color: "#888" }}>No brief available. Run synthesis first.</div>;
	}

	return (
		<div className="brief-prose">
			<Markdown
				remarkPlugins={remarkPlugins}
				rehypePlugins={rehypePlugins}
				components={markdownComponents}
			>
				{brief}
			</Markdown>
		</div>
	);
}

function EvidenceView({ matrix }: { matrix: EvidenceMatrix | null }) {
	if (!matrix) {
		return <div style={{ color: "#888" }}>No evidence matrix available.</div>;
	}

	return (
		<div>
			<h3 style={{ fontSize: 16, marginBottom: 12 }}>Coverage</h3>
			<table
				style={{
					borderCollapse: "collapse",
					fontSize: 13,
					marginBottom: 24,
					width: "100%",
				}}
			>
				<thead>
					<tr>
						<th style={thStyle}>Provider</th>
						<th style={thStyle}>Perspective</th>
						<th style={thStyle}>Status</th>
					</tr>
				</thead>
				<tbody>
					{matrix.coverage.cells.map((cell) => (
						<tr key={`${cell.provider}-${cell.perspective}`}>
							<td style={tdStyle}>{cell.provider}</td>
							<td style={tdStyle}>{cell.perspective}</td>
							<td style={tdStyle}>
								<StatusBadge status={cell.status} />
							</td>
						</tr>
					))}
				</tbody>
			</table>

			<h3 style={{ fontSize: 16, marginBottom: 12 }}>Themes ({matrix.themes.length})</h3>
			{matrix.themes.map((theme) => (
				<div
					key={theme.id}
					style={{
						border: "1px solid #333",
						borderRadius: 8,
						padding: 12,
						marginBottom: 8,
						background: "#1a1a1a",
					}}
				>
					<div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 8 }}>
						<strong>{theme.label}</strong>
						<AgreementBadge level={theme.agreement_level} />
					</div>
					<div style={{ fontSize: 13, color: "#aaa" }}>
						{theme.claims.length} claim(s) from{" "}
						{new Set(theme.claims.map((c) => c.source.provider)).size} provider(s)
					</div>
					{theme.disagreements.length > 0 && (
						<div style={{ fontSize: 13, color: "#f59e0b", marginTop: 4 }}>
							{theme.disagreements.length} disagreement(s)
						</div>
					)}
				</div>
			))}
		</div>
	);
}

function ArtifactsListView({
	artifacts,
	selectedArtifact,
	artifactContent,
	onArtifactClick,
}: {
	artifacts: SessionArtifact[];
	selectedArtifact: string | null;
	artifactContent: string | null;
	onArtifactClick: (path: string) => void;
}) {
	const grouped = groupArtifacts(artifacts);

	return (
		<div style={{ display: "flex", gap: 16 }}>
			<div style={{ minWidth: 280, maxHeight: "70vh", overflow: "auto" }}>
				{Object.entries(grouped).map(([group, items]) => (
					<div key={group} style={{ marginBottom: 16 }}>
						<div
							style={{ fontSize: 12, color: "#888", marginBottom: 4, textTransform: "uppercase" }}
						>
							{group}
						</div>
						{items.map((a) => (
							<button
								key={a.relative_path}
								type="button"
								onClick={() => onArtifactClick(a.relative_path)}
								style={{
									display: "block",
									width: "100%",
									textAlign: "left",
									padding: "6px 8px",
									background: selectedArtifact === a.relative_path ? "#2563eb" : "transparent",
									color: selectedArtifact === a.relative_path ? "white" : "#ccc",
									border: "1px solid #333",
									borderRadius: 4,
									marginBottom: 2,
									cursor: "pointer",
									fontSize: 13,
								}}
							>
								<div>{a.relative_path.split("/").pop()}</div>
								<div style={{ fontSize: 11, color: "#888" }}>
									{formatBytes(a.size_bytes)} &middot; {a.artifact_type}
								</div>
							</button>
						))}
					</div>
				))}
			</div>

			<div style={{ flex: 1 }}>
				{selectedArtifact ? (
					<div>
						<div style={{ fontSize: 13, color: "#888", marginBottom: 8 }}>{selectedArtifact}</div>
						<pre
							style={{
								background: "#111",
								border: "1px solid #333",
								borderRadius: 8,
								padding: 12,
								whiteSpace: "pre-wrap",
								wordBreak: "break-word",
								fontSize: 12,
								maxHeight: "65vh",
								overflow: "auto",
							}}
						>
							{artifactContent ?? "Loading..."}
						</pre>
					</div>
				) : (
					<div style={{ color: "#888", padding: 24 }}>Select an artifact to view.</div>
				)}
			</div>
		</div>
	);
}

function StatusBadge({ status }: { status: string }) {
	const color =
		status === "available"
			? "#22c55e"
			: status === "failed"
				? "#ef4444"
				: status === "timed_out"
					? "#f59e0b"
					: status === "blocked"
						? "#ef4444"
						: "#888";

	return (
		<span
			style={{
				background: `${color}22`,
				color,
				padding: "2px 8px",
				borderRadius: 4,
				fontSize: 12,
			}}
		>
			{status}
		</span>
	);
}

function AgreementBadge({ level }: { level: string }) {
	const color =
		level === "full"
			? "#22c55e"
			: level === "strong"
				? "#22c55e"
				: level === "partial"
					? "#f59e0b"
					: level === "disputed"
						? "#ef4444"
						: "#888";

	return (
		<span
			style={{
				background: `${color}22`,
				color,
				padding: "2px 8px",
				borderRadius: 4,
				fontSize: 11,
				textTransform: "uppercase",
			}}
		>
			{level}
		</span>
	);
}

const thStyle: React.CSSProperties = {
	textAlign: "left",
	padding: "8px 12px",
	borderBottom: "1px solid #333",
	color: "#888",
	fontSize: 12,
	textTransform: "uppercase",
};

const tdStyle: React.CSSProperties = {
	padding: "8px 12px",
	borderBottom: "1px solid #222",
};

function groupArtifacts(artifacts: SessionArtifact[]): Record<string, SessionArtifact[]> {
	const groups: Record<string, SessionArtifact[]> = {};
	for (const a of artifacts) {
		const parts = a.relative_path.split("/");
		const group = parts.length > 1 ? parts.slice(0, -1).join("/") : "root";
		if (!groups[group]) groups[group] = [];
		groups[group].push(a);
	}
	return groups;
}

function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
