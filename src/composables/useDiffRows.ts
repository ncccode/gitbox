import type { DiffResponse } from "../types/gitbox";

export type ProjectCodeToken = {
  text: string;
  kind?: "comment" | "string" | "keyword" | "number" | "function" | "property" | "operator";
};

export type SideBySideDiffCell = {
  lineNumber: number | null;
  content: string;
  type: "context" | "add" | "delete" | "empty" | "meta";
  tokens: ProjectCodeToken[];
};

export type SideBySideDiffRow = {
  id: string;
  type: "context" | "add" | "delete" | "modify" | "meta";
  hunkIndex: number | null;
  anchorHunkIndex: number | null;
  old: SideBySideDiffCell;
  new: SideBySideDiffCell;
};

const PROJECT_TOKEN_CACHE_LIMIT = 6000;
const projectKeywordCache = new Map<string, Set<string>>();
const projectLineTokenCache = new Map<string, ProjectCodeToken[]>();

export function projectLanguageForPath(path?: string | null) {
  const extension = path?.split(".").pop()?.toLowerCase() ?? "";
  if (["js", "jsx", "mjs", "cjs"].includes(extension)) return "javascript";
  if (["ts", "tsx"].includes(extension)) return "typescript";
  if (extension === "vue") return "vue";
  if (["json", "jsonc"].includes(extension)) return "json";
  if (["css", "scss", "sass", "less"].includes(extension)) return "css";
  if (["html", "xml", "svg"].includes(extension)) return "markup";
  if (extension === "rs") return "rust";
  if (extension === "toml") return "toml";
  if (["md", "markdown"].includes(extension)) return "markdown";
  if (["sh", "zsh", "bash"].includes(extension)) return "shell";
  return "plain";
}

export function buildSideBySideDiffRows(response: DiffResponse | null, language: string): SideBySideDiffRow[] {
  if (!response) return [];

  const rows: SideBySideDiffRow[] = [];
  let rowIndex = 0;
  const anchoredHunks = new Set<number>();
  const hasCompleteText = hasCompleteDiffText(response);
  if (!response.text.trim() && !hasCompleteText) return [];

  const emptyCell = (): SideBySideDiffCell => ({
    lineNumber: null,
    content: "",
    type: "empty",
    tokens: [{ text: " " }],
  });

  const diffCell = (
    lineNumber: number | null,
    content: string,
    type: SideBySideDiffCell["type"],
  ): SideBySideDiffCell => ({
    lineNumber,
    content,
    type,
    tokens: tokenizeProjectLine(content || " ", language),
  });

  const pushRow = (
    oldCell: SideBySideDiffCell,
    newCell: SideBySideDiffCell,
    type: SideBySideDiffRow["type"],
    hunkIndex: number | null,
  ) => {
    const anchorHunkIndex =
      hunkIndex !== null && type !== "context" && type !== "meta" && !anchoredHunks.has(hunkIndex)
        ? hunkIndex
        : null;
    if (anchorHunkIndex !== null) anchoredHunks.add(anchorHunkIndex);

    rows.push({
      id: `side-diff-${rowIndex}`,
      type,
      hunkIndex,
      anchorHunkIndex,
      old: oldCell,
      new: newCell,
    });
    rowIndex += 1;
  };

  const pushChangeGroup = (
    pendingDeletes: Array<{ lineNumber: number; content: string }>,
    pendingAdds: Array<{ lineNumber: number; content: string }>,
    hunkIndex: number,
  ) => {
    const total = Math.max(pendingDeletes.length, pendingAdds.length);
    for (let index = 0; index < total; index += 1) {
      const deleted = pendingDeletes[index];
      const added = pendingAdds[index];
      const rowType = deleted && added ? "modify" : deleted ? "delete" : "add";
      pushRow(
        deleted ? diffCell(deleted.lineNumber, deleted.content, "delete") : emptyCell(),
        added ? diffCell(added.lineNumber, added.content, "add") : emptyCell(),
        rowType,
        hunkIndex,
      );
    }
    pendingDeletes.length = 0;
    pendingAdds.length = 0;
  };

  const appendPatchHunk = (patch: string, hunkIndex: number, fallbackOldStart: number, fallbackNewStart: number) => {
    const pendingDeletes: Array<{ lineNumber: number; content: string }> = [];
    const pendingAdds: Array<{ lineNumber: number; content: string }> = [];
    let oldLine = fallbackOldStart;
    let newLine = fallbackNewStart;
    let insideHunk = false;

    const flushChanges = () => pushChangeGroup(pendingDeletes, pendingAdds, hunkIndex);

    for (const line of patch.split("\n")) {
      if (!line && !insideHunk) continue;

      if (line.startsWith("@@ ")) {
        flushChanges();
        insideHunk = true;
        const ranges = parseUnifiedHunkRange(line);
        oldLine = ranges.oldStart;
        newLine = ranges.newStart;
        continue;
      }

      if (!insideHunk) continue;

      if (line.startsWith(" ")) {
        flushChanges();
        const content = line.slice(1);
        pushRow(diffCell(oldLine, content, "context"), diffCell(newLine, content, "context"), "context", hunkIndex);
        oldLine += 1;
        newLine += 1;
        continue;
      }

      if (line.startsWith("-")) {
        pendingDeletes.push({ lineNumber: oldLine, content: line.slice(1) });
        oldLine += 1;
        continue;
      }

      if (line.startsWith("+")) {
        pendingAdds.push({ lineNumber: newLine, content: line.slice(1) });
        newLine += 1;
        continue;
      }

      if (line.startsWith("\\")) {
        flushChanges();
        pushRow(emptyCell(), diffCell(null, formatUnifiedDiffMetaLine(line), "meta"), "meta", hunkIndex);
      }
    }

    flushChanges();
    return { oldLine: Math.max(oldLine, 1), newLine: Math.max(newLine, 1) };
  };

  if (hasCompleteText) {
    const oldLines = splitFileContentLines(response.oldText ?? "");
    const newLines = splitFileContentLines(response.newText ?? "");
    let oldCursor = 1;
    let newCursor = 1;

    const pushUnchangedGap = (oldEndExclusive: number, newEndExclusive: number) => {
      while (oldCursor < oldEndExclusive || newCursor < newEndExclusive) {
        if (oldCursor < oldEndExclusive && newCursor < newEndExclusive) {
          pushRow(
            diffCell(oldCursor, oldLines[oldCursor - 1] ?? "", "context"),
            diffCell(newCursor, newLines[newCursor - 1] ?? "", "context"),
            "context",
            null,
          );
          oldCursor += 1;
          newCursor += 1;
        } else if (oldCursor < oldEndExclusive) {
          pushRow(diffCell(oldCursor, oldLines[oldCursor - 1] ?? "", "context"), emptyCell(), "context", null);
          oldCursor += 1;
        } else {
          pushRow(emptyCell(), diffCell(newCursor, newLines[newCursor - 1] ?? "", "context"), "context", null);
          newCursor += 1;
        }
      }
    };

    for (const hunk of response.hunks) {
      const oldStart = hunk.oldStart > 0 ? hunk.oldStart : oldCursor;
      const newStart = hunk.newStart > 0 ? hunk.newStart : newCursor;
      pushUnchangedGap(oldStart, newStart);
      const next = appendPatchHunk(hunk.patch, hunk.index, oldStart, newStart);
      oldCursor = next.oldLine;
      newCursor = next.newLine;
    }

    pushUnchangedGap(oldLines.length + 1, newLines.length + 1);
    return rows;
  }

  for (const hunk of response.hunks) {
    appendPatchHunk(hunk.patch, hunk.index, hunk.oldStart, hunk.newStart);
  }

  return rows;
}

export function hasDisplayableDiffContent(response: DiffResponse | null | undefined) {
  return Boolean(response?.text?.trim()) || hasCompleteDiffText(response);
}

export function tokenizeProjectLine(content: string, language: string): ProjectCodeToken[] {
  const cacheKey = `${language}\u0000${content}`;
  const cached = projectLineTokenCache.get(cacheKey);
  if (cached) return cached;

  const tokens: ProjectCodeToken[] = [];
  const keywords = projectKeywords(language);
  let index = 0;

  const push = (text: string, kind?: ProjectCodeToken["kind"]) => {
    if (text) tokens.push(kind ? { text, kind } : { text });
  };

  while (index < content.length) {
    const rest = content.slice(index);
    const char = content[index];

    if (rest.startsWith("//") || (rest.startsWith("#") && ["shell", "toml"].includes(language))) {
      push(rest, "comment");
      break;
    }

    if (rest.startsWith("/*")) {
      push(rest, "comment");
      break;
    }

    if (char === "\"" || char === "'" || char === "`") {
      let cursor = index + 1;
      while (cursor < content.length) {
        if (content[cursor] === "\\") {
          cursor += 2;
          continue;
        }
        if (content[cursor] === char) {
          cursor += 1;
          break;
        }
        cursor += 1;
      }
      push(content.slice(index, cursor), "string");
      index = cursor;
      continue;
    }

    const numberMatch = rest.match(/^\b\d+(?:\.\d+)?\b/);
    if (numberMatch) {
      push(numberMatch[0], "number");
      index += numberMatch[0].length;
      continue;
    }

    const wordMatch = rest.match(/^[A-Za-z_$][\w$-]*/);
    if (wordMatch) {
      const word = wordMatch[0];
      const before = content.slice(0, index).trimEnd();
      const after = content.slice(index + word.length).trimStart();
      if (keywords.has(word)) {
        push(word, "keyword");
      } else if (before.endsWith(".")) {
        push(word, "property");
      } else if (after.startsWith("(")) {
        push(word, "function");
      } else {
        push(word);
      }
      index += word.length;
      continue;
    }

    if (/^[{}()[\].,;:+\-*/%=!&|<>?@]/.test(char)) {
      push(char, "operator");
      index += 1;
      continue;
    }

    push(char);
    index += 1;
  }

  return rememberProjectLineTokens(cacheKey, tokens.length ? tokens : [{ text: " " }]);
}

function hasCompleteDiffText(response: DiffResponse | null | undefined) {
  return (
    response?.oldText !== undefined &&
    response?.newText !== undefined &&
    (response.oldText !== null || response.newText !== null)
  );
}

function formatUnifiedDiffMetaLine(line: string) {
  if (line.startsWith("\\ No newline")) return "文件末尾缺少换行符";
  return line;
}

function splitFileContentLines(content: string) {
  if (!content) return [];
  const lines = content.split("\n");
  if (lines[lines.length - 1] === "") lines.pop();
  return lines;
}

function parseUnifiedHunkRange(header: string) {
  const match = header.match(/^@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@/);
  return {
    oldStart: Number(match?.[1] ?? 0),
    newStart: Number(match?.[2] ?? 0),
  };
}

function rememberProjectLineTokens(key: string, tokens: ProjectCodeToken[]) {
  if (projectLineTokenCache.size >= PROJECT_TOKEN_CACHE_LIMIT) {
    const oldestKey = projectLineTokenCache.keys().next().value;
    if (oldestKey) {
      projectLineTokenCache.delete(oldestKey);
    }
  }
  projectLineTokenCache.set(key, tokens);
  return tokens;
}

function projectKeywords(language: string) {
  const cached = projectKeywordCache.get(language);
  if (cached) return cached;

  const shared = [
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "let",
    "new",
    "null",
    "return",
    "static",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "while",
  ];
  const byLanguage: Record<string, string[]> = {
    rust: ["fn", "impl", "let", "match", "mod", "mut", "pub", "self", "struct", "trait", "use", "where"],
    css: ["important", "media", "supports"],
    json: ["false", "null", "true"],
    markup: ["DOCTYPE"],
    markdown: [],
    shell: ["do", "done", "elif", "fi", "for", "function", "if", "in", "then"],
    toml: ["false", "true"],
  };
  const keywords = new Set([...(byLanguage[language] ?? shared), ...(language === "plain" ? [] : shared)]);
  projectKeywordCache.set(language, keywords);
  return keywords;
}
