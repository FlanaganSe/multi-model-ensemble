import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { App } from "../App";

const mockProviders = [
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
		status: "not_authenticated",
		executable_path: "/usr/local/bin/codex",
		version: "0.117.0",
		auth_ready: false,
		blocked_reason: "Codex is installed but not authenticated. Runs using Codex will be skipped.",
		remediation: "Open a terminal and run: codex login",
	},
	{
		provider: "gemini",
		status: "not_installed",
		executable_path: null,
		version: null,
		auth_ready: false,
		blocked_reason:
			"gemini was not found on your PATH. This provider cannot be used until it is installed.",
		remediation:
			"Install Gemini CLI: see https://google-gemini.github.io/gemini-cli/ for install instructions\nAfter installation, relaunch the app or probe again to detect it.",
	},
];

const mockSessions = [
	{
		id: "sess-001",
		created_at: "2026-03-30T10:00:00Z",
		status: "active",
		label: "Test session",
		path: "/tmp/sessions/sess-001",
	},
	{
		id: "sess-002",
		created_at: "2026-03-29T10:00:00Z",
		status: "archived",
		label: null,
		path: "/tmp/sessions/sess-002",
	},
];

// Mock the Tauri invoke API
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(async (cmd: string) => {
		if (cmd === "probe_providers") return mockProviders;
		if (cmd === "list_sessions") return mockSessions;
		if (cmd === "list_perspectives")
			return [
				{ id: "default", label: "Default", instructions: "Balanced analysis" },
				{ id: "creative", label: "Creative", instructions: "Creative angles" },
				{ id: "adversarial", label: "Adversarial", instructions: "Critical stance" },
			];
		if (cmd === "archive_session") return null;
		if (cmd === "delete_session") return null;
		return null;
	}),
}));

describe("App", () => {
	it("renders the title", () => {
		render(<App />);
		expect(screen.getByText("Multi-Model Synthesizer")).toBeDefined();
	});

	it("renders provider status section", () => {
		render(<App />);
		expect(screen.getByText("Provider Status")).toBeDefined();
	});

	it("renders sessions section", () => {
		render(<App />);
		expect(screen.getByText("Sessions")).toBeDefined();
	});

	it("shows provider probe results after loading", async () => {
		render(<App />);
		expect(await screen.findByText("claude")).toBeDefined();
		expect(await screen.findByText("codex")).toBeDefined();
		expect(await screen.findByText("gemini")).toBeDefined();
	});

	it("shows ready status for Claude", async () => {
		render(<App />);
		expect(await screen.findByText("Ready")).toBeDefined();
	});

	it("shows blocked reason for not-authenticated provider", async () => {
		render(<App />);
		expect(
			await screen.findByText(
				"Codex is installed but not authenticated. Runs using Codex will be skipped.",
			),
		).toBeDefined();
	});

	it("shows remediation for not-installed provider", async () => {
		render(<App />);
		expect(await screen.findByText(/Install Gemini CLI/)).toBeDefined();
	});

	it("shows session list with active and archived sessions", async () => {
		render(<App />);
		expect(await screen.findByText("Test session")).toBeDefined();
		expect(await screen.findByText("archived")).toBeDefined();
	});

	it("shows New Run button", async () => {
		render(<App />);
		expect(await screen.findByText("New Run")).toBeDefined();
	});

	it("shows archive and delete buttons for active sessions", async () => {
		render(<App />);
		const archiveButtons = await screen.findAllByText("Archive");
		expect(archiveButtons.length).toBeGreaterThan(0);
		const deleteButtons = await screen.findAllByText("Delete");
		expect(deleteButtons.length).toBeGreaterThan(0);
	});

	it("navigates to run composer when New Run is clicked", async () => {
		render(<App />);
		const user = userEvent.setup();
		const newRunButton = await screen.findByText("New Run");
		await act(async () => {
			await user.click(newRunButton);
		});
		expect(screen.getByText("Prompt")).toBeDefined();
		expect(screen.getByText("Providers")).toBeDefined();
		expect(screen.getByText("Perspectives")).toBeDefined();
	});
});
