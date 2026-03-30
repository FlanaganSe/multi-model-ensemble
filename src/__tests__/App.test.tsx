import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "../App";

// Mock the Tauri invoke API
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(async (cmd: string) => {
		if (cmd === "probe_providers") {
			return [
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
					blocked_reason: "Codex auth not active",
					remediation: "Run `codex login` in your terminal",
				},
				{
					provider: "gemini",
					status: "not_installed",
					executable_path: null,
					version: null,
					auth_ready: false,
					blocked_reason: "gemini binary not found in PATH",
					remediation: "Install gemini and ensure it is on your PATH",
				},
			];
		}
		if (cmd === "list_sessions") {
			return [];
		}
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
		// Provider names are lowercase in the DOM; CSS text-transform: capitalize styles them visually
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
		expect(await screen.findByText("Codex auth not active")).toBeDefined();
	});

	it("shows remediation for not-installed provider", async () => {
		render(<App />);
		expect(await screen.findByText("Install gemini and ensure it is on your PATH")).toBeDefined();
	});
});
