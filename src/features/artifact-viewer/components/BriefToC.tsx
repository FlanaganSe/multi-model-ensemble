export interface TocEntry {
	level: 2 | 3;
	text: string;
	id: string;
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
 * Call once per render to get a fresh counter; heading components are
 * called in document order by react-markdown, matching extractHeadings.
 */
export function makeHeadingComponents() {
	const idCounts = new Map<string, number>();

	function getNextId(text: string): string {
		const baseId = slugify(text);
		const count = idCounts.get(baseId) ?? 0;
		idCounts.set(baseId, count + 1);
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
