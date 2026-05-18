import { defineStore } from "pinia";
import {
  applyPatch,
  blameFile,
  cleanupMergedBranches,
  clearStashes,
  cloneRepository,
  commitMessageHistory,
  compareRefs,
  createPatch,
  createWorktree,
  dropCommit,
  fileHistory,
  fixupCommit,
  initRepository,
  lfsStatus,
  listStashes,
  listSubmodules,
  listWorktrees,
  pushCommit,
  removeWorktree,
  renameBranch,
  stashAction,
  unshallowRepository,
  updateSubmodules,
  checkoutRevision,
} from "../lib/gitboxCommands";
import { useChangesStore } from "./changes";
import { useRepositoriesStore } from "./repositories";
import type {
  BlameLine,
  FileHistoryEntry,
  RefComparison,
  RepositoryInfo,
  StashInfo,
  SubmoduleInfo,
  WorktreeInfo,
} from "../types/gitbox";

export const useAdvancedStore = defineStore("advanced", {
  state: () => ({
    cloneUrl: "",
    cloneDirectory: "",
    cloneDepth: 0,
    initDirectory: "",
    initBare: false,
    initInitialBranch: "main",
    branchRenameFrom: "",
    branchRenameTo: "",
    compareLeft: "",
    compareRight: "",
    comparison: null as RefComparison | null,
    fileHistory: [] as FileHistoryEntry[],
    blame: [] as BlameLine[],
    generatedPatch: "",
    patchDraft: "",
    applyPatchToIndex: false,
    applyPatchThreeWay: true,
    worktrees: [] as WorktreeInfo[],
    worktreePath: "",
    worktreeBranch: "",
    worktreeStartPoint: "",
    worktreeDetach: false,
    stashes: [] as StashInfo[],
    submodules: [] as SubmoduleInfo[],
    lfsOutput: "",
    commitMessages: [] as string[],
    loading: false,
    error: "",
    notice: "",
  }),
  actions: {
    async cloneInto(): Promise<RepositoryInfo | null> {
      if (!this.cloneUrl.trim() || !this.cloneDirectory.trim()) return null;
      return this.runReturningRepo(() =>
        cloneRepository(
          this.cloneUrl.trim(),
          this.cloneDirectory.trim(),
          this.cloneDepth > 0 ? this.cloneDepth : undefined,
        ),
      );
    },
    async initAt(): Promise<RepositoryInfo | null> {
      if (!this.initDirectory.trim()) return null;
      return this.runReturningRepo(() =>
        initRepository(this.initDirectory.trim(), {
          bare: this.initBare,
          initialBranch: this.initInitialBranch.trim() || undefined,
        }),
      );
    },
    async unshallow(remoteName?: string) {
      await this.runRepoAction((repoPath) => unshallowRepository(repoPath, remoteName));
    },
    async renameSelectedBranch() {
      if (!this.branchRenameFrom.trim() || !this.branchRenameTo.trim()) return;
      await this.runRepoAction((repoPath) =>
        renameBranch(repoPath, this.branchRenameFrom.trim(), this.branchRenameTo.trim()),
      );
      this.branchRenameTo = "";
    },
    async cleanupMerged(target?: string) {
      await this.runRepoAction((repoPath) => cleanupMergedBranches(repoPath, target));
    },
    async checkoutDetached(oid: string) {
      if (!oid) return;
      await this.runRepoAction((repoPath) => checkoutRevision(repoPath, oid));
    },
    async loadComparison() {
      if (!this.compareLeft.trim() || !this.compareRight.trim()) return;
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        this.comparison = await compareRefs(repos.path, this.compareLeft.trim(), this.compareRight.trim());
        this.notice = `已比较 ${this.compareLeft} 与 ${this.compareRight}`;
      });
    },
    async loadFileHistory(filePath?: string | null) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const target = filePath || changes.selectedFile;
      if (!repos.path || !target) return;
      await this.run(async () => {
        this.fileHistory = await fileHistory(repos.path, target, 120);
        this.notice = `已读取 ${target} 的文件历史`;
      });
    },
    async loadBlame(filePath?: string | null) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const target = filePath || changes.selectedFile;
      if (!repos.path || !target) return;
      await this.run(async () => {
        this.blame = await blameFile(repos.path, target);
        this.notice = `已读取 ${target} 的 blame`;
      });
    },
    async generatePatch(staged = false) {
      const changes = useChangesStore();
      await this.runRepoAction(
        (repoPath) => createPatch(repoPath, changes.activePaths, staged),
        (result) => {
          this.generatedPatch = result.output;
        },
      );
    },
    async applyPatchDraft() {
      if (!this.patchDraft.trim()) return;
      await this.runRepoAction((repoPath) =>
        applyPatch(repoPath, this.patchDraft, this.applyPatchToIndex, this.applyPatchThreeWay),
      );
      this.patchDraft = "";
    },
    async refreshWorktrees() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        this.worktrees = await listWorktrees(repos.path);
      });
    },
    async createWorktreeFromDraft() {
      if (!this.worktreePath.trim()) return;
      await this.runRepoAction((repoPath) =>
        createWorktree(repoPath, this.worktreePath.trim(), {
          branch: this.worktreeBranch.trim() || undefined,
          startPoint: this.worktreeStartPoint.trim() || undefined,
          detach: this.worktreeDetach,
        }),
      );
      this.worktreePath = "";
      await this.refreshWorktrees();
    },
    async removeWorktreePath(path: string, force = false) {
      await this.runRepoAction((repoPath) => removeWorktree(repoPath, path, force));
      await this.refreshWorktrees();
    },
    async refreshStashes() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        this.stashes = await listStashes(repos.path);
      });
    },
    async runStashAction(stashRef: string, action: "apply" | "pop" | "drop") {
      await this.runRepoAction((repoPath) => stashAction(repoPath, stashRef, action));
      await this.refreshStashes();
    },
    async clearAllStashes() {
      await this.runRepoAction((repoPath) => clearStashes(repoPath));
      await this.refreshStashes();
    },
    async refreshSubmodules() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        this.submodules = await listSubmodules(repos.path);
      });
    },
    async updateAllSubmodules() {
      await this.runRepoAction((repoPath) => updateSubmodules(repoPath, true, true));
      await this.refreshSubmodules();
    },
    async refreshLfsStatus() {
      await this.runRepoAction(
        (repoPath) => lfsStatus(repoPath),
        (result) => {
          this.lfsOutput = result.output;
        },
      );
    },
    async refreshCommitMessages() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        this.commitMessages = await commitMessageHistory(repos.path, 40);
      });
    },
    async fixupSelectedCommit(oid: string, squash = false) {
      if (!oid) return;
      await this.runRepoAction((repoPath) => fixupCommit(repoPath, oid, squash));
    },
    async dropSelectedCommit(oid: string) {
      if (!oid) return;
      await this.runRepoAction((repoPath) => dropCommit(repoPath, oid));
    },
    async pushSelectedCommit(oid: string, remoteName?: string, targetBranch?: string) {
      if (!oid) return;
      await this.runRepoAction((repoPath) => pushCommit(repoPath, remoteName, oid, targetBranch));
    },
    resetForRepositorySwitch() {
      this.comparison = null;
      this.fileHistory = [];
      this.blame = [];
      this.generatedPatch = "";
      this.worktrees = [];
      this.stashes = [];
      this.submodules = [];
      this.lfsOutput = "";
      this.commitMessages = [];
      this.error = "";
      this.notice = "";
    },
    async runRepoAction(
      action: (repoPath: string) => Promise<{ message: string; output: string }>,
      after?: (result: { message: string; output: string }) => void,
    ) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      await this.run(async () => {
        const result = await action(repos.path);
        after?.(result);
        this.notice = result.message;
      });
    },
    async runReturningRepo(action: () => Promise<RepositoryInfo>) {
      let repo: RepositoryInfo | null = null;
      await this.run(async () => {
        repo = await action();
        this.notice = `已打开 ${repo.path}`;
      });
      return repo;
    },
    async run(action: () => Promise<void>) {
      this.loading = true;
      this.error = "";
      try {
        await action();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
