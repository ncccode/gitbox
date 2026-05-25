import type { CommitFileChange } from "../types/gitbox";

export type LogFileTreeRow = {
  id: string;
  name: string;
  path: string;
  parent: string | null;
  depth: number;
  directory: boolean;
  fileCount?: number;
  status?: string;
  oldPath?: string | null;
};

export function buildCommitFileTreeRows(files: CommitFileChange[]): LogFileTreeRow[] {
  type LogFileTreeNode = LogFileTreeRow & {
    children: Map<string, LogFileTreeNode>;
  };

  const root: LogFileTreeNode = {
    id: "root",
    name: "",
    path: "",
    parent: null,
    depth: -1,
    directory: true,
    fileCount: files.length,
    children: new Map(),
  };
  const rows: LogFileTreeRow[] = [];
  const sortedFiles = [...files].sort((left, right) => left.path.localeCompare(right.path));

  for (const file of sortedFiles) {
    const parts = file.path.split("/").filter(Boolean);
    let parent = root;
    let currentPath = "";
    for (let index = 0; index < parts.length - 1; index += 1) {
      currentPath = currentPath ? `${currentPath}/${parts[index]}` : parts[index];
      const key = `dir:${parts[index]}`;
      let directory = parent.children.get(key);
      if (!directory) {
        directory = {
          id: `dir:${currentPath}`,
          name: parts[index],
          path: currentPath,
          parent: parent.path || null,
          depth: index,
          directory: true,
          fileCount: 0,
          children: new Map(),
        };
        parent.children.set(key, directory);
      }
      directory.fileCount = (directory.fileCount ?? 0) + 1;
      parent = directory;
    }

    const fileName = parts.length > 0 ? parts[parts.length - 1] : file.path;
    parent.children.set(`file:${file.path}`, {
      id: `file:${file.status}:${file.oldPath ?? ""}:${file.path}`,
      name: fileName,
      path: file.path,
      parent: parent.path || null,
      depth: Math.max(0, parts.length - 1),
      directory: false,
      status: file.status,
      oldPath: file.oldPath,
      children: new Map(),
    });
  }

  const appendRows = (parent: LogFileTreeNode) => {
    const children = [...parent.children.values()].sort(compareLogFileTreeNodes);
    for (const child of children) {
      rows.push({
        id: child.id,
        name: child.name,
        path: child.path,
        parent: child.parent,
        depth: child.depth,
        directory: child.directory,
        fileCount: child.fileCount,
        status: child.status,
        oldPath: child.oldPath,
      });
      if (child.directory) appendRows(child);
    }
  };

  appendRows(root);
  return rows;
}

function compareLogFileTreeNodes(left: LogFileTreeRow, right: LogFileTreeRow) {
  if (left.directory !== right.directory) {
    return left.directory ? -1 : 1;
  }
  return left.name.toLocaleLowerCase().localeCompare(right.name.toLocaleLowerCase());
}
