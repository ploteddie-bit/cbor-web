import React, { type ReactNode } from "react";

type CBORValue = Record<string, unknown> | unknown[] | string | number | boolean | null | Uint8Array;

interface ContentBlock {
  t: string;
  v?: unknown;
  l?: number;
  src?: string;
  alt?: string;
  caption?: string;
  href?: string;
  attr?: string;
  lang?: string;
  headers?: string[];
  rows?: string[][];
  description?: string;
  level?: string;
}

interface CBORWebContentProps {
  blocks: ContentBlock[];
}

function renderTextWithLinks(text: string): ReactNode {
  const linkPattern = /\[([^\]]+)\]\(([^)]+)\)/g;
  const parts: ReactNode[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = linkPattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    parts.push(
      React.createElement("a", {
        key: parts.length,
        href: match[2],
        target: "_blank",
        rel: "noopener noreferrer",
      }, match[1])
    );
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts.length > 0 ? parts : text;
}

function renderBlock(block: ContentBlock, index: number): ReactNode {
  const { t, v, l } = block;

  switch (t) {
    case "h": {
      const level = l ?? 1;
      const tag = `h${Math.min(Math.max(level, 1), 6)}` as keyof JSX.IntrinsicElements;
      return React.createElement(tag, { key: index }, v as ReactNode);
    }

    case "p":
      return React.createElement("p", { key: index }, renderTextWithLinks(v as string));

    case "ul":
      return React.createElement("ul", { key: index },
        (v as string[]).map((item, i) => React.createElement("li", { key: i }, item))
      );

    case "ol":
      return React.createElement("ol", { key: index },
        (v as string[]).map((item, i) => React.createElement("li", { key: i }, item))
      );

    case "q":
      return React.createElement(
        "blockquote",
        { key: index, cite: block.attr },
        React.createElement("p", null, v as string)
      );

    case "code":
      return React.createElement(
        "pre",
        { key: index },
        React.createElement("code", { className: block.lang ? `language-${block.lang}` : undefined }, v as string)
      );

    case "table":
      return React.createElement("table", { key: index },
        React.createElement("thead", null,
          React.createElement("tr", null,
            (block.headers ?? []).map((h, i) => React.createElement("th", { key: i }, h))
          )
        ),
        React.createElement("tbody", null,
          (block.rows ?? []).map((row, ri) =>
            React.createElement("tr", { key: ri },
              row.map((cell, ci) => React.createElement("td", { key: ci }, cell))
            )
          )
        )
      );

    case "img": {
      const props: Record<string, unknown> = {
        key: index,
        src: block.src,
        alt: block.alt ?? "",
      };
      if (block.caption) {
        return React.createElement("figure", { key: index },
          React.createElement("img", props),
          React.createElement("figcaption", null, block.caption)
        );
      }
      return React.createElement("img", props);
    }

    case "cta":
      return React.createElement("a", {
        key: index,
        href: block.href ?? "#",
        className: "cta",
        target: "_blank",
        rel: "noopener noreferrer",
      }, v as string);

    case "embed":
      return React.createElement("iframe", {
        key: index,
        src: block.src ?? "",
        title: block.description ?? "Embedded content",
        className: "cborweb-embed",
      });

    case "sep":
      return React.createElement("hr", { key: index });

    case "dl":
      return React.createElement("dl", { key: index },
        (v as { term: string; def: string }[]).map((entry, i) => [
          React.createElement("dt", { key: `dt-${i}` }, entry.term),
          React.createElement("dd", { key: `dd-${i}` }, entry.def),
        ])
      );

    case "note":
      return React.createElement("aside", {
        key: index,
        className: `cborweb-note cborweb-note--${block.level ?? "info"}`,
      }, v as string);

    default:
      return null;
  }
}

export function CBORWebContent({ blocks }: CBORWebContentProps): ReactNode {
  if (!blocks || blocks.length === 0) return null;
  return React.createElement(React.Fragment, null, blocks.map(renderBlock));
}

export default CBORWebContent;
