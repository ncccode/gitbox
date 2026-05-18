export type ChangeSide = "unstaged" | "staged";

export interface RemoteInfo {
  name: string;
  url?: string | null;
  pushUrl?: string | null;
}

export interface RepositoryInfo {
  path: string;
  workdir?: string | null;
  gitDir: string;
  isBare: boolean;
  branch?: string | null;
  head?: string | null;
  remotes: RemoteInfo[];
}

export interface BranchSummary {
  currentBranch?: string | null;
  upstream?: string | null;
  head?: string | null;
  ahead: number;
  behind: number;
  detached: boolean;
  clean: boolean;
  remotes: RemoteInfo[];
}

export interface StatusCounts {
  staged: number;
  unstaged: number;
  untracked: number;
  conflicted: number;
  ignored: number;
}

export interface ChangedFile {
  path: string;
  oldPath?: string | null;
  kind: string;
  staged: boolean;
  unstaged: boolean;
  untracked: boolean;
  ignored: boolean;
  conflicted: boolean;
}

export interface RepoStatus {
  repo: RepositoryInfo;
  branch: BranchSummary;
  files: ChangedFile[];
  counts: StatusCounts;
}

export interface DiffHunk {
  index: number;
  header: string;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  patch: string;
}

export interface DiffResponse {
  path?: string | null;
  staged: boolean;
  text: string;
  hunks: DiffHunk[];
}

export interface CommandResult {
  ok: boolean;
  message: string;
  output: string;
}

export interface CommitResult {
  oid: string;
  summary: BranchSummary;
}

export interface ShelfInfo {
  id?: number | null;
  repoPath: string;
  message: string;
  stashRef: string;
  stashOid: string;
  createdAt: number;
  appliedAt?: number | null;
}

export interface CommitSummary {
  oid: string;
  shortOid: string;
  summary: string;
  body: string;
  authorName: string;
  authorEmail: string;
  authorTime: number;
  committerTime: number;
  parents: string[];
  refs: string[];
}

export interface CommitFileChange {
  path: string;
  oldPath?: string | null;
  status: string;
}

export interface CommitDetails {
  commit: CommitSummary;
  files: CommitFileChange[];
  diff: string;
}

export type CommitFileDiffMode = "commit" | "worktree" | "parent-worktree";

export interface BranchInfo {
  name: string;
  fullName: string;
  branchType: "local" | "remote";
  current: boolean;
  upstream?: string | null;
  target?: string | null;
  ahead: number;
  behind: number;
}

export interface TagInfo {
  name: string;
  target?: string | null;
}

export interface FileHistoryEntry {
  oid: string;
  shortOid: string;
  summary: string;
  authorName: string;
  authorEmail: string;
  authorTime: number;
}

export interface BlameLine {
  lineNumber: number;
  oid: string;
  shortOid: string;
  authorName: string;
  authorEmail: string;
  authorTime: number;
  summary: string;
  content: string;
}

export interface RefComparisonCommit {
  side: "left" | "right" | "both";
  oid: string;
  shortOid: string;
  summary: string;
  authorName: string;
  authorEmail: string;
  authorTime: number;
}

export interface RefComparison {
  left: string;
  right: string;
  commits: RefComparisonCommit[];
  files: CommitFileChange[];
  diff: string;
}

export interface WorktreeInfo {
  path: string;
  head?: string | null;
  branch?: string | null;
  detached: boolean;
  bare: boolean;
  prunable: boolean;
}

export interface StashInfo {
  stashRef: string;
  oid: string;
  message: string;
  createdAt: number;
}

export interface SubmoduleInfo {
  path: string;
  oid: string;
  status: string;
  branch?: string | null;
}

export interface BranchList {
  current?: string | null;
  branches: BranchInfo[];
  tags: TagInfo[];
}

export interface GitOperationState {
  active: boolean;
  operation?: string | null;
  conflictedPaths: string[];
}

export interface ConflictBlock {
  index: number;
  ours: string;
  base?: string | null;
  theirs: string;
}

export interface ConflictDetails {
  path: string;
  base?: string | null;
  ours?: string | null;
  theirs?: string | null;
  current?: string | null;
  blocks: ConflictBlock[];
}

export interface ProjectFileEntry {
  path: string;
  name: string;
  parent?: string | null;
  depth: number;
  directory: boolean;
  size?: number | null;
}

export interface ProjectFileMutation {
  path: string;
  directory: boolean;
  message: string;
}

export interface ProjectFileContent {
  path: string;
  content?: string | null;
  binary: boolean;
  size: number;
}
