import { invoke } from "@tauri-apps/api/core";
import type {
  BranchSummary,
  BranchList,
  CommandResult,
  ConflictDetails,
  CommitDetails,
  CommitFileDiffMode,
  CommitResult,
  CommitSummary,
  DiffResponse,
  BlameLine,
  FileHistoryEntry,
  GitOperationState,
  ProjectFileContent,
  ProjectFileEntry,
  ProjectFileMutation,
  RefComparison,
  RepoStatus,
  RepositoryInfo,
  ShelfInfo,
  StashInfo,
  SubmoduleInfo,
  WorktreeInfo,
} from "../types/gitbox";

export function openRepo(path: string) {
  return invoke<RepositoryInfo>("open_repo", { path });
}

export function initRepository(
  path: string,
  options: { bare?: boolean; initialBranch?: string } = {},
) {
  return invoke<RepositoryInfo>("init_repository", {
    path,
    bare: Boolean(options.bare),
    initialBranch: options.initialBranch,
  });
}

export function cloneRepository(url: string, directory: string, shallowDepth?: number) {
  return invoke<RepositoryInfo>("clone_repository", { url, directory, shallowDepth });
}

export function unshallowRepository(path: string, remoteName?: string) {
  return invoke<CommandResult>("unshallow_repository", { path, remoteName });
}

export function repoStatus(path: string, includeIgnored = false) {
  return invoke<RepoStatus>("repo_status", { path, includeIgnored });
}

export function branchSummary(path: string) {
  return invoke<BranchSummary>("branch_summary", { path });
}

export function listProjectFiles(path: string, limit = 1400) {
  return invoke<ProjectFileEntry[]>("list_project_files", { path, limit });
}

export function readProjectFile(path: string, filePath: string) {
  return invoke<ProjectFileContent>("read_project_file", { path, filePath });
}

export function saveProjectFile(path: string, filePath: string, content: string) {
  return invoke<ProjectFileContent>("save_project_file", { path, filePath, content });
}

export function createProjectFile(path: string, directoryPath: string | null | undefined, name: string) {
  return invoke<ProjectFileMutation>("create_project_file", { path, directoryPath, name });
}

export function createProjectDirectory(path: string, directoryPath: string | null | undefined, name: string) {
  return invoke<ProjectFileMutation>("create_project_directory", { path, directoryPath, name });
}

export function renameProjectEntry(path: string, filePath: string, newName: string) {
  return invoke<ProjectFileMutation>("rename_project_entry", { path, filePath, newName });
}

export function deleteProjectEntry(path: string, filePath: string) {
  return invoke<CommandResult>("delete_project_entry", { path, filePath });
}

export function copyProjectEntry(path: string, sourcePath: string, targetDirectoryPath: string | null | undefined) {
  return invoke<ProjectFileMutation>("copy_project_entry", { path, sourcePath, targetDirectoryPath });
}

export function moveProjectEntry(path: string, sourcePath: string, targetDirectoryPath: string | null | undefined) {
  return invoke<ProjectFileMutation>("move_project_entry", { path, sourcePath, targetDirectoryPath });
}

export function getDiff(path: string, filePath?: string | null, staged = false) {
  return invoke<DiffResponse>("get_diff", { path, filePath, staged });
}

export function stagePaths(path: string, paths: string[]) {
  return invoke<CommandResult>("stage_paths", { path, paths });
}

export function unstagePaths(path: string, paths: string[]) {
  return invoke<CommandResult>("unstage_paths", { path, paths });
}

export function stageHunks(path: string, patches: string[], mode: "stage" | "unstage" | "discard") {
  return invoke<CommandResult>("stage_hunks", { path, patches, mode });
}

export function discardChanges(path: string, paths: string[]) {
  return invoke<CommandResult>("discard_changes", { path, paths });
}

export function shelveChanges(path: string, paths: string[], message?: string) {
  return invoke<ShelfInfo>("shelve_changes", { path, paths, message });
}

export function unshelve(path: string, stashRef: string) {
  return invoke<CommandResult>("unshelve", { path, stashRef });
}

export function deleteShelf(path: string, stashRef: string, dropStash = true) {
  return invoke<CommandResult>("delete_shelf", { path, stashRef, dropStash });
}

export function listShelves(path: string) {
  return invoke<ShelfInfo[]>("list_shelves", { path });
}

export function commitRepo(
  path: string,
  message: string,
  options: { amend?: boolean; signOff?: boolean; gpgSign?: boolean; author?: string } = {},
) {
  return invoke<CommitResult>("commit", {
    path,
    message,
    amend: Boolean(options.amend),
    signOff: Boolean(options.signOff),
    gpgSign: Boolean(options.gpgSign),
    author: options.author,
  });
}

export function fetchRemote(path: string, remoteName?: string, options: { prune?: boolean } = {}) {
  return invoke<CommandResult>("fetch", {
    path,
    remoteName,
    prune: Boolean(options.prune),
  });
}

export function pullRemote(path: string, remoteName?: string) {
  return invoke<CommandResult>("pull", { path, remoteName });
}

export function pushRemote(
  path: string,
  remoteName?: string,
  options: {
    targetBranch?: string;
    setUpstream?: boolean;
    forceWithLease?: boolean;
    pushTags?: boolean;
  } = {},
) {
  return invoke<CommandResult>("push", {
    path,
    remoteName,
    targetBranch: options.targetBranch,
    setUpstream: Boolean(options.setUpstream),
    forceWithLease: Boolean(options.forceWithLease),
    pushTags: Boolean(options.pushTags),
  });
}

export function addRemote(path: string, name: string, url: string) {
  return invoke<CommandResult>("add_remote", { path, name, url });
}

export function updateRemote(path: string, name: string, url: string, pushUrl?: string) {
  return invoke<CommandResult>("update_remote", { path, name, url, pushUrl });
}

export function deleteRemote(path: string, name: string) {
  return invoke<CommandResult>("delete_remote", { path, name });
}

export function listCommits(
  path: string,
  limit = 80,
  options: {
    branch?: string;
    query?: string;
    author?: string;
    authors?: string[];
    pathFilter?: string;
    pathFilters?: string[];
  } = {},
) {
  return invoke<CommitSummary[]>("list_commits", {
    path,
    limit,
    branch: options.branch,
    query: options.query,
    author: options.author,
    authors: options.authors,
    pathFilter: options.pathFilter,
    pathFilters: options.pathFilters,
  });
}

export function commitDetails(path: string, oid: string) {
  return invoke<CommitDetails>("commit_details", { path, oid });
}

export function commitFileDiff(path: string, oid: string, filePath: string, mode: CommitFileDiffMode = "commit") {
  return invoke<DiffResponse>("commit_file_diff", { path, oid, filePath, mode });
}

export function fileHistory(path: string, filePath: string, limit = 80) {
  return invoke<FileHistoryEntry[]>("file_history", { path, filePath, limit });
}

export function blameFile(path: string, filePath: string) {
  return invoke<BlameLine[]>("blame_file", { path, filePath });
}

export function compareRefs(path: string, left: string, right: string) {
  return invoke<RefComparison>("compare_refs", { path, left, right });
}

export function listBranches(path: string) {
  return invoke<BranchList>("list_branches", { path });
}

export function checkoutBranch(path: string, name: string) {
  return invoke<CommandResult>("checkout_branch", { path, name });
}

export function checkoutRevision(path: string, revision: string) {
  return invoke<CommandResult>("checkout_revision", { path, revision });
}

export function checkoutRemoteBranch(path: string, remoteBranch: string, localName?: string) {
  return invoke<CommandResult>("checkout_remote_branch", { path, remoteBranch, localName });
}

export function createBranch(path: string, name: string, checkout = true, startPoint?: string) {
  return invoke<CommandResult>("create_branch", { path, name, checkout, startPoint });
}

export function renameBranch(path: string, oldName: string, newName: string) {
  return invoke<CommandResult>("rename_branch", { path, oldName, newName });
}

export function cleanupMergedBranches(path: string, target?: string) {
  return invoke<CommandResult>("cleanup_merged_branches", { path, target });
}

export function deleteBranch(path: string, name: string, force = false) {
  return invoke<CommandResult>("delete_branch", { path, name, force });
}

export function deleteRemoteBranch(path: string, remoteBranch: string) {
  return invoke<CommandResult>("delete_remote_branch", { path, remoteBranch });
}

export function setBranchUpstream(path: string, branchName: string, upstream?: string) {
  return invoke<CommandResult>("set_branch_upstream", { path, branchName, upstream });
}

export function createTag(
  path: string,
  name: string,
  options: { target?: string; annotated?: boolean; message?: string } = {},
) {
  return invoke<CommandResult>("create_tag", {
    path,
    name,
    target: options.target,
    annotated: Boolean(options.annotated),
    message: options.message,
  });
}

export function deleteTag(path: string, name: string) {
  return invoke<CommandResult>("delete_tag", { path, name });
}

export function pushTag(path: string, remoteName: string | undefined, name: string) {
  return invoke<CommandResult>("push_tag", { path, remoteName, name });
}

export function deleteRemoteTag(path: string, remoteName: string | undefined, name: string) {
  return invoke<CommandResult>("delete_remote_tag", { path, remoteName, name });
}

export function mergeBranch(
  path: string,
  target: string,
  options: { noFf?: boolean; noCommit?: boolean; squash?: boolean } = {},
) {
  return invoke<CommandResult>("merge_branch", {
    path,
    target,
    noFf: Boolean(options.noFf),
    noCommit: Boolean(options.noCommit),
    squash: Boolean(options.squash),
  });
}

export function rebaseBranch(path: string, target: string, autostash = true) {
  return invoke<CommandResult>("rebase_branch", { path, target, autostash });
}

export function rebaseAdvanced(
  path: string,
  options: {
    target?: string;
    sourceBranch?: string;
    onto?: string;
    autostash?: boolean;
    interactive?: boolean;
    autosquash?: boolean;
    rebaseMerges?: boolean;
    keepEmpty?: boolean;
    root?: boolean;
    updateRefs?: boolean;
  },
) {
  return invoke<CommandResult>("rebase_advanced", {
    path,
    target: options.target,
    sourceBranch: options.sourceBranch,
    onto: options.onto,
    autostash: options.autostash ?? true,
    interactive: Boolean(options.interactive),
    autosquash: Boolean(options.autosquash),
    rebaseMerges: Boolean(options.rebaseMerges),
    keepEmpty: Boolean(options.keepEmpty),
    root: Boolean(options.root),
    updateRefs: Boolean(options.updateRefs),
  });
}

export function cherryPickCommit(path: string, oid: string) {
  return invoke<CommandResult>("cherry_pick_commit", { path, oid });
}

export function cherryPickFiles(path: string, oid: string, files: string[]) {
  return invoke<CommandResult>("cherry_pick_files", { path, oid, files });
}

export function revertCommitFiles(path: string, oid: string, files: string[]) {
  return invoke<CommandResult>("revert_commit_files", { path, oid, files });
}

export function revertCommit(path: string, oid: string, noCommit = false) {
  return invoke<CommandResult>("revert_commit", { path, oid, noCommit });
}

export function resetToCommit(path: string, oid: string, mode: "soft" | "mixed" | "hard") {
  return invoke<CommandResult>("reset_to_commit", { path, oid, mode });
}

export function undoLastCommit(path: string, keepStaged = true) {
  return invoke<CommandResult>("undo_last_commit", { path, keepStaged });
}

export function fixupCommit(path: string, oid: string, squash = false) {
  return invoke<CommandResult>("fixup_commit", { path, oid, squash });
}

export function dropCommit(path: string, oid: string) {
  return invoke<CommandResult>("drop_commit", { path, oid });
}

export function pushCommit(
  path: string,
  remoteName: string | undefined,
  oid: string,
  targetBranch?: string,
) {
  return invoke<CommandResult>("push_commit", { path, remoteName, oid, targetBranch });
}

export function createPatch(path: string, paths: string[], staged = false) {
  return invoke<CommandResult>("create_patch", { path, paths, staged });
}

export function applyPatch(path: string, patch: string, index = false, threeWay = true) {
  return invoke<CommandResult>("apply_patch", { path, patch, index, threeWay });
}

export function listWorktrees(path: string) {
  return invoke<WorktreeInfo[]>("list_worktrees", { path });
}

export function createWorktree(
  path: string,
  worktreePath: string,
  options: { branch?: string; startPoint?: string; detach?: boolean } = {},
) {
  return invoke<CommandResult>("create_worktree", {
    path,
    worktreePath,
    branch: options.branch,
    startPoint: options.startPoint,
    detach: Boolean(options.detach),
  });
}

export function removeWorktree(path: string, worktreePath: string, force = false) {
  return invoke<CommandResult>("remove_worktree", { path, worktreePath, force });
}

export function listStashes(path: string) {
  return invoke<StashInfo[]>("list_stashes", { path });
}

export function stashAction(path: string, stashRef: string, action: "apply" | "pop" | "drop") {
  return invoke<CommandResult>("stash_action", { path, stashRef, action });
}

export function clearStashes(path: string) {
  return invoke<CommandResult>("clear_stashes", { path });
}

export function listSubmodules(path: string) {
  return invoke<SubmoduleInfo[]>("list_submodules", { path });
}

export function updateSubmodules(path: string, init = true, recursive = true) {
  return invoke<CommandResult>("update_submodules", { path, init, recursive });
}

export function lfsStatus(path: string) {
  return invoke<CommandResult>("lfs_status", { path });
}

export function commitMessageHistory(path: string, limit = 30) {
  return invoke<string[]>("commit_message_history", { path, limit });
}

export function operationState(path: string) {
  return invoke<GitOperationState>("operation_state", { path });
}

export function operationControl(path: string, action: "continue" | "abort" | "skip") {
  return invoke<CommandResult>("operation_control", { path, action });
}

export function conflictDetails(path: string, filePath: string) {
  return invoke<ConflictDetails>("conflict_details", { path, filePath });
}

export function resolveConflictFile(path: string, filePath: string, side: "ours" | "theirs") {
  return invoke<CommandResult>("resolve_conflict_file", { path, filePath, side });
}

export function resolveConflictBlock(
  path: string,
  filePath: string,
  blockIndex: number,
  side: "ours" | "base" | "theirs",
) {
  return invoke<CommandResult>("resolve_conflict_block", { path, filePath, blockIndex, side });
}

export function markConflictResolved(path: string, filePath: string) {
  return invoke<CommandResult>("mark_conflict_resolved", { path, filePath });
}

export function saveConflictResult(
  path: string,
  filePath: string,
  content: string,
  markResolved = false,
) {
  return invoke<CommandResult>("save_conflict_result", { path, filePath, content, markResolved });
}
