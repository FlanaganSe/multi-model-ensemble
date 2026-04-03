export interface TocEntry {
	level: 2 | 3;
	text: string;
	id: string;
}

export interface BriefSection {
	heading: string;
	id: string;
	body: string;
}

export interface SplitResult {
	preamble: string;
	sections: BriefSection[];
	h2IdCounts: Map<string, number>;
}

export function slugify(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^\w]+/g, "-")
		.replace(/^-+|-+$/g, "");
}

/** Strip markdown link syntax: `[text](url)` → `text` */
function stripLinks(text: string): string {
	return text.replace(/\[([^\]]+)\]\([^)]*\)/g, "$1");
}

export function extractHeadings(markdown: string): TocEntry[] {
	const lines = markdown.split("\n");
	const headings: TocEntry[] = [];
	const idCounts = new Map<string, number>();
	let inCodeBlock = false;

	for (const line of lines) {
		if (line.startsWith("```") || line.startsWith("~~~")) {
			inCodeBlock = !inCodeBlock;
			continue;
		}
		if (inCodeBlock) continue;

		let text: string | undefined;
		let level: 2 | 3 | undefined;

		if (line.startsWith("### ")) {
			text = line.slice(4).trim();
			level = 3;
		} else if (line.startsWith("## ")) {
			text = line.slice(3).trim();
			level = 2;
		}

		if (text !== undefined && level !== undefined) {
			const baseId = slugify(stripLinks(text));
			const count = idCounts.get(baseId) ?? 0;
			idCounts.set(baseId, count + 1);
			const id = count === 0 ? baseId : `${baseId}-${count + 1}`;
			headings.push({ level, text, id });
		}
	}

	return headings;
}

/**
 * Split markdown at h2 boundaries into a preamble (before first h2)
 * and sections (each starting with an h2). Respects fenced code blocks.
 * Returns h2IdCounts so heading components can be pre-seeded for correct dedup.
 */
export function splitSections(markdown: string): SplitResult {
	const lines = markdown.split("\n");
	const h2IdCounts = new Map<string, number>();
	const preambleLines: string[] = [];
	const sections: BriefSection[] = [];
	let current: { heading: string; id: string; bodyLines: string[] } | null = null;
	let inCodeBlock = false;

	for (const line of lines) {
		if (line.startsWith("```") || line.startsWith("~~~")) {
			inCodeBlock = !inCodeBlock;
		}

		if (!inCodeBlock && line.startsWith("## ") && !line.startsWith("### ")) {
			if (current) {
				sections.push({
					heading: current.heading,
					id: current.id,
					body: current.bodyLines.join("\n"),
				});
			}
			const text = line.slice(3).trim();
			const baseId = slugify(stripLinks(text));
			const count = h2IdCounts.get(baseId) ?? 0;
			h2IdCounts.set(baseId, count + 1);
			const id = count === 0 ? baseId : `${baseId}-${count + 1}`;
			current = { heading: text, id, bodyLines: [] };
		} else if (current) {
			current.bodyLines.push(line);
		} else {
			preambleLines.push(line);
		}
	}

	if (current) {
		sections.push({ heading: current.heading, id: current.id, body: current.bodyLines.join("\n") });
	}

	return { preamble: preambleLines.join("\n"), sections, h2IdCounts };
}

function extractText(node: React.ReactNode): string {
	if (typeof node === "string") return node;
	if (typeof node === "number") return String(node);
	if (Array.isArray(node)) return node.map(extractText).join("");
	if (node !== null && node !== undefined && typeof node === "object" && "props" in node) {
		const props = (node as { props: { children?: React.ReactNode } }).props;
		return extractText(props.children);
	}
	return "";
}

interface HeadingProps extends React.ComponentPropsWithoutRef<"h2"> {
	node?: unknown;
}

/**
 * Creates h2/h3 component overrides that add deduped id attributes.
 * Accepts an external counter ref so the caller can reset it at the top
 * of each render pass, making this safe in React Strict Mode (which
 * double-invokes the render function — a stale internal Map would
 * produce wrong IDs on the second pass).
 */
export function makeHeadingComponents(counterRef: { current: Map<string, number> }) {
	function getNextId(text: string): string {
		const baseId = slugify(text);
		const count = counterRef.current.get(baseId) ?? 0;
		counterRef.current.set(baseId, count + 1);
		return count === 0 ? baseId : `${baseId}-${count + 1}`;
	}

	return {
		h2({ node: _node, children, ...rest }: HeadingProps) {
			const text = extractText(children);
			return (
				<h2 id={getNextId(text)} {...rest}>
					{children}
				</h2>
			);
		},
		h3({
			node: _node,
			children,
			...rest
		}: React.ComponentPropsWithoutRef<"h3"> & { node?: unknown }) {
			const text = extractText(children);
			return (
				<h3 id={getNextId(text)} {...rest}>
					{children}
				</h3>
			);
		},
	};
}

interface BriefToCProps {
	headings: TocEntry[];
}

export function BriefToC({ headings }: BriefToCProps) {
	if (headings.length === 0) return null;

	const handleClick = (id: string) => {
		const el = document.getElementById(id);
		el?.scrollIntoView({ behavior: "smooth", block: "start" });
	};

	return (
		<nav className="brief-toc" aria-label="Table of contents">
			<div className="brief-toc-title">Contents</div>
			<ul className="brief-toc-list">
				{headings.map((h, i) => (
					<li key={`${h.id}-${i}`} className={h.level === 3 ? "brief-toc-indent" : ""}>
						<button type="button" className="brief-toc-link" onClick={() => handleClick(h.id)}>
							{h.text}
						</button>
					</li>
				))}
			</ul>
		</nav>
	);
}
