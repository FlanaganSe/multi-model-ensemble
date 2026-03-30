import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RunComposer } from "../features/run-composer/RunComposer";
import type { ProviderProbeResult, RunSummary } from "../lib/types";

const readyProviders: ProviderProbeResult[] = [
	{
		provider: "claude",
		status: "ready",
		executable_path: "/usr/local/bin/claude",
		version: "2.1.81",
		auth_ready: true,
		blocked_reason: null,
		remediation: null,
	},
	{
		provider: "codex",
		status: "ready",
		executable_path: "/usr/local/bin/codex",
		version: "0.117.0",
		auth_ready: true,
		blocked_reason: null,
		remediation: null,
	},
	{
		provider: "gemini",
		status: "not_installed",
		executable_path: null,
		version: null,
		auth_ready: false,
		blocked_reason: "gemini was not found on your PATH",
		remediation: "Install gemini",
	},
];

const mockRunSummary: RunSummary = {
	session_id: "test-session",
	total_jobs: 2,
	completed: 2,
	failed: 0,
	timed_out: 0,
	blocked: 0,
	cancelled: 0,
};

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(async (cmd: string) => {
		if (cmd === "list_perspectives")
			return [
				{ id: "default", label: "Default", instructions: "Balanced analysis" },
				{ id: "creative", label: "Creative", instructions: "Creative angles" },
				{ id: "adversarial", label: "Adversarial", instructions: "Critical stance" },
			];
		if (cmd === "run_session") return mockRunSummary;
		if (cmd === "get_run_results") return [];
		return null;
	}),
}));

describe("RunComposer", () => {
	const onRunComplete = vi.fn();
	const onCancel = vi.fn();

	it("renders all form sections", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		expect(screen.getByText("Prompt")).toBeDefined();
		expect(screen.getByText("Providers")).toBeDefined();
		expect(screen.getByText("Synthesis Strategy")).toBeDefined();
		expect(await screen.findByText("Perspectives")).toBeDefined();
	});

	it("auto-selects ready providers", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		// The run button should show 2 jobs (2 ready providers * 1 default perspective)
		expect(await screen.findByText("Run (2 jobs)")).toBeDefined();
	});

	it("disables not-installed providers", () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const geminiButton = screen.getByText("gemini").closest("button");
		expect(geminiButton?.disabled).toBe(true);
	});

	it("disables run button when prompt is empty", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		await screen.findByText("Run (2 jobs)");
		const runButton = screen.getByText("Run (2 jobs)").closest("button");
		expect(runButton?.disabled).toBe(true);
	});

	it("enables run button when prompt is entered", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const user = userEvent.setup();
		const textarea = screen.getByPlaceholderText("Enter your research prompt...");
		await act(async () => {
			await user.type(textarea, "What is Rust?");
		});
		const runButton = screen.getByText("Run (2 jobs)").closest("button");
		expect(runButton?.disabled).toBe(false);
	});

	it("toggles perspective selection", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const user = userEvent.setup();
		const creativeBtn = await screen.findByText("Creative");
		await act(async () => {
			await user.click(creativeBtn);
		});
		// Should now show 4 jobs (2 providers * 2 perspectives)
		expect(screen.getByText("Run (4 jobs)")).toBeDefined();
	});

	it("calls onCancel when back button is clicked", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const user = userEvent.setup();
		await act(async () => {
			await user.click(screen.getByText("Back"));
		});
		expect(onCancel).toHaveBeenCalled();
	});

	it("shows strategy dropdown with three options", () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const select = screen.getByRole("combobox");
		expect(select).toBeDefined();
		const options = select.querySelectorAll("option");
		expect(options.length).toBe(3);
	});

	it("shows unavailable provider remediation text", () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		expect(screen.getByText(/Install gemini/)).toBeDefined();
	});

	it("submits run and shows success result", async () => {
		render(
			<RunComposer providers={readyProviders} onRunComplete={onRunComplete} onCancel={onCancel} />,
		);
		const user = userEvent.setup();
		const textarea = screen.getByPlaceholderText("Enter your research prompt...");
		await act(async () => {
			await user.type(textarea, "Analyze this code");
		});
		const runButton = screen.getByText("Run (2 jobs)");
		await act(async () => {
			await user.click(runButton);
		});
		expect(await screen.findByText("Run completed successfully")).toBeDefined();
		expect(await screen.findByText("2 completed")).toBeDefined();
		expect(onRunComplete).toHaveBeenCalledWith(mockRunSummary);
	});
});
