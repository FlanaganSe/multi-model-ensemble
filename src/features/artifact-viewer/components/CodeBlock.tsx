import { useEffect, useRef, useState } from "react";

interface CodeBlockProps extends React.ComponentPropsWithoutRef<"pre"> {
	node?: unknown;
}

export function CodeBlock({ node: _node, children, ...rest }: CodeBlockProps) {
	const preRef = useRef<HTMLPreElement>(null);
	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const [copied, setCopied] = useState(false);

	useEffect(() => {
		return () => {
			if (timeoutRef.current !== null) clearTimeout(timeoutRef.current);
		};
	}, []);

	const handleCopy = () => {
		const text = preRef.current?.textContent ?? "";
		navigator.clipboard
			.writeText(text)
			.then(() => {
				setCopied(true);
				if (timeoutRef.current !== null) clearTimeout(timeoutRef.current);
				timeoutRef.current = setTimeout(() => setCopied(false), 2000);
			})
			.catch(() => {});
	};

	return (
		<div style={{ position: "relative" }}>
			<pre ref={preRef} {...rest}>
				{children}
			</pre>
			<button
				type="button"
				onClick={handleCopy}
				style={{
					position: "absolute",
					top: 8,
					right: 8,
					background: copied ? "#22c55e33" : "#333",
					color: copied ? "#22c55e" : "#aaa",
					border: "1px solid #444",
					borderRadius: 4,
					padding: "2px 8px",
					fontSize: 11,
					cursor: "pointer",
					opacity: 0.8,
					transition: "opacity 150ms, background 150ms, color 150ms",
				}}
				onMouseEnter={(e) => {
					e.currentTarget.style.opacity = "1";
				}}
				onMouseLeave={(e) => {
					e.currentTarget.style.opacity = "0.8";
				}}
			>
				{copied ? "Copied!" : "Copy"}
			</button>
		</div>
	);
}
