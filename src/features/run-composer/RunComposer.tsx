import { useCallback, useEffect, useState } from "react";
import { getRunResults, listPerspectives, runSession } from "../../lib/api";
import type {
	JobResult,
	Perspective,
	ProviderName,
	ProviderProbeResult,
	RunSummary,
	SynthesisStrategy,
} from "../../lib/types";

interface RunComposerProps {
	providers: ProviderProbeResult[];
	onRunComplete: (summary: RunSummary) => void;
	onCancel: () => void;
}

const STRATEGIES: { value: SynthesisStrategy; label: string }[] = [
	{ value: "consensus", label: "Consensus — shared conclusions and next actions" },
	{ value: "comprehensive", label: "Comprehensive — full union of findings" },
	{ value: "executive", label: "Executive — concise decision-oriented TL;DR" },
];

export function RunComposer({ providers, onRunComplete, onCancel }: RunComposerProps) {
	const [prompt, setPrompt] = useState("");
	const [selectedProviders, setSelectedProviders] = useState<ProviderName[]>([]);
	const [selectedPerspectives, setSelectedPerspectives] = useState<string[]>(["default"]);
	const [strategy, setStrategy] = useState<SynthesisStrategy>("consensus");
	const [workingDirectory, setWorkingDirectory] = useState("");
	const [perspectives, setPerspectives] = useState<Perspective[]>([]);
	const [running, setRunning] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [result, setResult] = useState<RunSummary | null>(null);
	const [jobResults, setJobResults] = useState<JobResult[]>([]);

	useEffect(() => {
		listPerspectives()
			.then(setPerspectives)
			.catch(() => {});
	}, []);

	// Auto-select ready providers on mount
	useEffect(() => {
		const ready = providers.filter((p) => p.status === "ready").map((p) => p.provider);
		setSelectedProviders(ready);
	}, [providers]);

	const toggleProvider = useCallback((name: ProviderName) => {
		setSelectedProviders((prev) =>
			prev.includes(name) ? prev.filter((p) => p !== name) : [...prev, name],
		);
	}, []);

	const togglePerspective = useCallback((id: string) => {
		setSelectedPerspectives((prev) =>
			prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
		);
	}, []);

	const canRun =
		prompt.trim().length > 0 && selectedProviders.length > 0 && selectedPerspectives.length > 0;

	const jobCount = selectedProviders.length * selectedPerspectives.length;

	const handleRun = async () => {
		if (!canRun || running) return;
		setRunning(true);
		setError(null);
		setResult(null);
		setJobResults([]);

		try {
			const summary = await runSession({
				prompt: prompt.trim(),
				providers: selectedProviders,
				perspectives: selectedPerspectives,
				strategy,
				workingDirectory: workingDirectory.trim() || undefined,
			});
			setResult(summary);
			onRunComplete(summary);

			// Fetch detailed job results
			try {
				const jobs = await getRunResults(summary.session_id);
				setJobResults(jobs);
			} catch {
				// Non-fatal: summary is still useful
			}
		} catch (e) {
			setError(String(e));
		} finally {
			setRunning(false);
		}
	};

	return (
		<div style={{ padding: 24, fontFamily: "system-ui, sans-serif", color: "#e0e0e0" }}>
			<div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 16 }}>
				<button
					type="button"
					onClick={onCancel}
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
				<h2 style={{ margin: 0, fontSize: 18 }}>New Run</h2>
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

			{/* Prompt */}
			<div style={{ marginBottom: 16 }}>
				<label
					htmlFor="prompt-input"
					style={{ display: "block", marginBottom: 4, fontWeight: 600, fontSize: 14 }}
				>
					Prompt
				</label>
				<textarea
					id="prompt-input"
					value={prompt}
					onChange={(e) => setPrompt(e.target.value)}
					placeholder="Enter your research prompt..."
					disabled={running}
					style={{
						width: "100%",
						minHeight: 120,
						background: "#111",
						border: "1px solid #333",
						borderRadius: 8,
						padding: 12,
						color: "#e0e0e0",
						fontSize: 14,
						fontFamily: "system-ui, sans-serif",
						resize: "vertical",
						boxSizing: "border-box",
					}}
				/>
			</div>

			{/* Providers */}
			<div style={{ marginBottom: 16 }}>
				<div style={{ fontWeight: 600, fontSize: 14, marginBottom: 8 }}>Providers</div>
				<div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
					{providers.map((p) => {
						const isReady = p.status === "ready";
						const isSelected = selectedProviders.includes(p.provider);
						return (
							<button
								type="button"
								key={p.provider}
								disabled={!isReady || running}
								onClick={() => toggleProvider(p.provider)}
								title={!isReady ? (p.blocked_reason ?? "Not available") : undefined}
								style={{
									padding: "8px 16px",
									border: `1px solid ${isSelected ? "#2563eb" : "#333"}`,
									borderRadius: 8,
									background: isSelected ? "#1e3a5f" : "#1a1a1a",
									color: isReady ? "#e0e0e0" : "#666",
									cursor: isReady && !running ? "pointer" : "not-allowed",
									opacity: isReady ? 1 : 0.6,
									textTransform: "capitalize",
									fontSize: 14,
								}}
							>
								<span
									style={{
										display: "inline-block",
										width: 8,
										height: 8,
										borderRadius: "50%",
										background: isReady ? "#22c55e" : "#ef4444",
										marginRight: 8,
									}}
								/>
								{p.provider}
								{p.version && (
									<span style={{ fontSize: 11, color: "#888", marginLeft: 6 }}>{p.version}</span>
								)}
							</button>
						);
					})}
				</div>
				{providers.some((p) => p.status !== "ready") && (
					<div style={{ fontSize: 12, color: "#888", marginTop: 4 }}>
						Unavailable providers are disabled.{" "}
						{providers
							.filter((p) => p.status !== "ready" && p.remediation)
							.map((p) => `${p.provider}: ${p.remediation}`)
							.join(" | ")}
					</div>
				)}
			</div>

			{/* Perspectives */}
			<div style={{ marginBottom: 16 }}>
				<div style={{ fontWeight: 600, fontSize: 14, marginBottom: 8 }}>Perspectives</div>
				<div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
					{perspectives.map((p) => {
						const isSelected = selectedPerspectives.includes(p.id);
						return (
							<button
								type="button"
								key={p.id}
								disabled={running}
								onClick={() => togglePerspective(p.id)}
								title={p.instructions.slice(0, 120)}
								style={{
									padding: "6px 14px",
									border: `1px solid ${isSelected ? "#2563eb" : "#333"}`,
									borderRadius: 6,
									background: isSelected ? "#1e3a5f" : "#1a1a1a",
									color: "#e0e0e0",
									cursor: running ? "not-allowed" : "pointer",
									fontSize: 13,
								}}
							>
								{p.label}
							</button>
						);
					})}
				</div>
			</div>

			{/* Strategy */}
			<div style={{ marginBottom: 16 }}>
				<label
					htmlFor="strategy-select"
					style={{ display: "block", fontWeight: 600, fontSize: 14, marginBottom: 4 }}
				>
					Synthesis Strategy
				</label>
				<select
					id="strategy-select"
					value={strategy}
					onChange={(e) => setStrategy(e.target.value as SynthesisStrategy)}
					disabled={running}
					style={{
						background: "#111",
						border: "1px solid #333",
						borderRadius: 6,
						padding: "8px 12px",
						color: "#e0e0e0",
						fontSize: 14,
						width: "100%",
						boxSizing: "border-box",
					}}
				>
					{STRATEGIES.map((s) => (
						<option key={s.value} value={s.value}>
							{s.label}
						</option>
					))}
				</select>
			</div>

			{/* Working directory */}
			<div style={{ marginBottom: 20 }}>
				<label
					htmlFor="workdir-input"
					style={{ display: "block", fontWeight: 600, fontSize: 14, marginBottom: 4 }}
				>
					Working Directory <span style={{ fontWeight: 400, color: "#888" }}>(optional)</span>
				</label>
				<input
					id="workdir-input"
					type="text"
					value={workingDirectory}
					onChange={(e) => setWorkingDirectory(e.target.value)}
					placeholder="/path/to/project"
					disabled={running}
					style={{
						width: "100%",
						background: "#111",
						border: "1px solid #333",
						borderRadius: 6,
						padding: "8px 12px",
						color: "#e0e0e0",
						fontSize: 14,
						boxSizing: "border-box",
					}}
				/>
			</div>

			{/* Run button */}
			<div style={{ display: "flex", alignItems: "center", gap: 12 }}>
				<button
					type="button"
					onClick={handleRun}
					disabled={!canRun || running}
					style={{
						background: canRun && !running ? "#2563eb" : "#333",
						color: canRun && !running ? "white" : "#888",
						border: "none",
						borderRadius: 6,
						padding: "10px 24px",
						fontSize: 15,
						fontWeight: 600,
						cursor: canRun && !running ? "pointer" : "not-allowed",
					}}
				>
					{running ? "Running..." : `Run (${jobCount} job${jobCount !== 1 ? "s" : ""})`}
				</button>
				{running && (
					<span style={{ color: "#888", fontSize: 13 }}>This may take a few minutes...</span>
				)}
			</div>

			{/* Result summary + per-job details */}
			{result && <RunResultSummary summary={result} jobs={jobResults} />}
		</div>
	);
}

function RunResultSummary({ summary, jobs }: { summary: RunSummary; jobs: JobResult[] }) {
	const allCompleted = summary.completed === summary.total_jobs;
	const hasIssues = summary.failed > 0 || summary.timed_out > 0 || summary.blocked > 0;

	return (
		<div
			style={{
				marginTop: 20,
				border: `1px solid ${allCompleted ? "#22c55e" : "#f59e0b"}`,
				borderRadius: 8,
				padding: 16,
				background: allCompleted ? "#0a2e0a" : "#2e2a0a",
			}}
		>
			<div style={{ fontWeight: 600, marginBottom: 8 }}>
				{allCompleted ? "Run completed successfully" : "Run completed with issues"}
			</div>
			<div style={{ display: "flex", gap: 16, fontSize: 13, marginBottom: hasIssues ? 12 : 0 }}>
				<span style={{ color: "#22c55e" }}>{summary.completed} completed</span>
				{summary.failed > 0 && <span style={{ color: "#ef4444" }}>{summary.failed} failed</span>}
				{summary.timed_out > 0 && (
					<span style={{ color: "#f59e0b" }}>{summary.timed_out} timed out</span>
				)}
				{summary.blocked > 0 && <span style={{ color: "#ef4444" }}>{summary.blocked} blocked</span>}
			</div>

			{/* Per-job details for non-completed jobs */}
			{jobs
				.filter((j) => j.state !== "completed")
				.map((job) => (
					<JobIssueCard key={job.job_id} job={job} />
				))}
		</div>
	);
}

function JobIssueCard({ job }: { job: JobResult }) {
	const stateColor =
		job.state === "failed"
			? "#ef4444"
			: job.state === "timed_out"
				? "#f59e0b"
				: job.state === "blocked"
					? "#ef4444"
					: "#888";

	return (
		<div
			style={{
				background: "#111",
				border: `1px solid ${stateColor}33`,
				borderRadius: 6,
				padding: 10,
				marginBottom: 6,
				fontSize: 13,
			}}
		>
			<div style={{ display: "flex", gap: 8, alignItems: "center", marginBottom: 4 }}>
				<span style={{ textTransform: "capitalize", fontWeight: 600 }}>{job.provider}</span>
				<span style={{ color: "#888" }}>{job.perspective_id}</span>
				<span
					style={{
						color: stateColor,
						fontSize: 11,
						textTransform: "uppercase",
						padding: "1px 6px",
						border: `1px solid ${stateColor}44`,
						borderRadius: 3,
					}}
				>
					{job.state.replace("_", " ")}
				</span>
			</div>
			{job.blocked_reason && <div style={{ color: "#fca5a5" }}>{job.blocked_reason}</div>}
			{job.blocked_remediation && (
				<div style={{ color: "#888", marginTop: 2, whiteSpace: "pre-line" }}>
					{job.blocked_remediation}
				</div>
			)}
			{job.error && <div style={{ color: "#fca5a5" }}>{job.error}</div>}
			{job.state === "timed_out" && (
				<div style={{ color: "#fbbf24" }}>
					The provider did not respond within the timeout window. Try increasing the timeout or
					simplifying the prompt.
				</div>
			)}
		</div>
	);
}
