import { defineStore } from "pinia";
import {
  cherryPickCommit,
  cherryPickFiles,
  conflictDetails,
  markConflictResolved,
  mergeBranch,
  operationControl,
  operationState,
  rebaseAdvanced,
  rebaseBranch,
  resetToCommit,
  revertCommit,
  resolveConflictBlock,
  resolveConflictFile,
  saveConflictResult,
  undoLastCommit,
} from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import type { ConflictDetails, GitOperationState } from "../types/gitbox";

type OperationAction = "continue" | "abort" | "skip";
type ConflictSide = "ours" | "theirs";
type ConflictBlockSide = "ours" | "base" | "theirs";
type ConflictResultSource = "ours" | "base" | "theirs" | "current";
export type ResetMode = "soft" | "mixed" | "hard";

function splitPreservingNewlines(content: string) {
  const matches = content.match(/[^\n]*\n|[^\n]+$/g);
  return matches ?? [];
}

function cleanConflictMarkers(content: string, preferred: ConflictBlockSide = "ours") {
  const lines = splitPreservingNewlines(content);
  const result: string[] = [];
  let index = 0;

  while (index < lines.length) {
    if (!lines[index].startsWith("<<<<<<< ")) {
      result.push(lines[index]);
      index += 1;
      continue;
    }

    index += 1;
    const parts: Record<ConflictBlockSide, string[]> = {
      ours: [],
      base: [],
      theirs: [],
    };
    let section: ConflictBlockSide = "ours";

    while (index < lines.length) {
      const line = lines[index];
      if (line.startsWith("||||||| ")) {
        section = "base";
        index += 1;
        continue;
      }
      if (line.startsWith("=======")) {
        section = "theirs";
        index += 1;
        continue;
      }
      if (line.startsWith(">>>>>>> ")) {
        index += 1;
        break;
      }
      parts[section].push(line);
      index += 1;
    }

    result.push(...parts[preferred]);
  }

  return result.join("");
}

export const useOperationsStore = defineStore("operations", {
  state: () => ({
    state: null as GitOperationState | null,
    conflict: null as ConflictDetails | null,
    resultDraft: "",
    resultDirty: false,
    selectedConflictPath: "",
    mergeTarget: "",
    rebaseTarget: "",
    rebaseSourceBranch: "",
    rebaseOnto: "",
    mergeNoFf: false,
    mergeNoCommit: false,
    mergeSquash: false,
    rebaseAutostash: true,
    rebaseInteractive: false,
    rebaseAutosquash: false,
    rebaseMerges: false,
    rebaseKeepEmpty: false,
    rebaseRoot: false,
    rebaseUpdateRefs: false,
    revertNoCommit: false,
    resetMode: "mixed" as ResetMode,
    undoKeepStaged: true,
    loading: false,
    error: "",
    notice: "",
  }),
  getters: {
    activeOperation: (state) => state.state?.operation ?? "",
    conflictedPaths: (state) => state.state?.conflictedPaths ?? [],
  },
  actions: {
    async refresh() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.error = "";
      try {
        this.state = await operationState(repos.path);
        if (!this.selectedConflictPath || !this.state.conflictedPaths.includes(this.selectedConflictPath)) {
          this.selectedConflictPath = this.state.conflictedPaths[0] ?? "";
        }
        if (this.selectedConflictPath) {
          await this.loadConflict(this.selectedConflictPath);
        } else {
          this.conflict = null;
        }
      } catch (error) {
        this.error = String(error);
        throw error;
      }
    },
    async loadConflict(filePath: string) {
      const repos = useRepositoriesStore();
      if (!repos.path || !filePath) return;

      this.selectedConflictPath = filePath;
      this.error = "";
      try {
        this.conflict = await conflictDetails(repos.path, filePath);
        this.resultDraft = this.initialResultFromConflict();
        this.resultDirty = false;
      } catch (error) {
        this.error = String(error);
        throw error;
      }
    },
    async merge() {
      const target = this.mergeTarget.trim();
      if (!target) return;
      await this.runOperation((repoPath) =>
        mergeBranch(repoPath, target, {
          noFf: this.mergeNoFf,
          noCommit: this.mergeNoCommit,
          squash: this.mergeSquash,
        }),
      );
    },
    async rebase() {
      const target = this.rebaseTarget.trim();
      if (!target) return;
      await this.runOperation((repoPath) => rebaseBranch(repoPath, target, this.rebaseAutostash));
    },
    async rebaseWithAdvancedOptions() {
      const target = this.rebaseTarget.trim();
      if (!target && !this.rebaseRoot) return;
      await this.runOperation((repoPath) =>
        rebaseAdvanced(repoPath, {
          target: target || undefined,
          sourceBranch: this.rebaseSourceBranch.trim() || undefined,
          onto: this.rebaseOnto.trim() || undefined,
          autostash: this.rebaseAutostash,
          interactive: this.rebaseInteractive,
          autosquash: this.rebaseAutosquash,
          rebaseMerges: this.rebaseMerges,
          keepEmpty: this.rebaseKeepEmpty,
          root: this.rebaseRoot,
          updateRefs: this.rebaseUpdateRefs,
        }),
      );
    },
    async cherryPick(oid: string) {
      if (!oid) return;
      await this.runOperation((repoPath) => cherryPickCommit(repoPath, oid));
    },
    async cherryPickFiles(oid: string, files: string[]) {
      if (!oid || files.length === 0) return;
      await this.runOperation((repoPath) => cherryPickFiles(repoPath, oid, files));
    },
    async revert(oid: string) {
      if (!oid) return;
      await this.runOperation((repoPath) => revertCommit(repoPath, oid, this.revertNoCommit));
    },
    async resetTo(oid: string) {
      if (!oid) return;
      await this.runOperation((repoPath) => resetToCommit(repoPath, oid, this.resetMode));
    },
    async undoLastCommit() {
      await this.runOperation((repoPath) => undoLastCommit(repoPath, this.undoKeepStaged));
    },
    async control(action: OperationAction) {
      await this.runOperation((repoPath) => operationControl(repoPath, action));
    },
    async resolveFile(side: ConflictSide) {
      if (!this.selectedConflictPath) return;
      await this.runOperation((repoPath) =>
        resolveConflictFile(repoPath, this.selectedConflictPath, side),
      );
    },
    async resolveBlock(index: number, side: ConflictBlockSide) {
      if (!this.selectedConflictPath) return;
      await this.runOperation((repoPath) =>
        resolveConflictBlock(repoPath, this.selectedConflictPath, index, side),
      );
    },
    async markResolved() {
      if (!this.selectedConflictPath) return;
      await this.runOperation((repoPath) =>
        markConflictResolved(repoPath, this.selectedConflictPath),
      );
    },
    useResultSource(source: ConflictResultSource) {
      if (!this.conflict) return;
      const sources: Record<ConflictResultSource, string | null | undefined> = {
        ours: this.conflict.ours,
        base: this.conflict.base,
        theirs: this.conflict.theirs,
        current: this.conflict.current,
      };
      const content = sources[source] ?? "";
      this.resultDraft = source === "current" ? cleanConflictMarkers(content) : content;
      this.resultDirty = true;
    },
    replaceResultBlock(index: number, side: ConflictBlockSide) {
      const block = this.conflict?.blocks.find((item) => item.index === index);
      if (!block) return;
      const replacement = side === "base" ? (block.base ?? "") : block[side];
      this.resultDraft = replacement;
      this.resultDirty = true;
    },
    useAllConflictBlocks(side: ConflictBlockSide) {
      if (!this.conflict?.current) return;
      this.resultDraft = cleanConflictMarkers(this.conflict.current, side);
      this.resultDirty = true;
    },
    setResultDraft(content: string) {
      this.resultDraft = content;
      this.resultDirty = true;
    },
    async saveResult(markResolved = false) {
      if (!this.selectedConflictPath) return;
      await this.runOperation((repoPath) =>
        saveConflictResult(repoPath, this.selectedConflictPath, this.resultDraft, markResolved),
      );
      this.resultDirty = false;
    },
    initialResultFromConflict() {
      if (!this.conflict?.current) return this.conflict?.ours ?? this.conflict?.theirs ?? "";
      return cleanConflictMarkers(this.conflict.current);
    },
    async runOperation(action: (repoPath: string) => Promise<{ ok: boolean; message: string }>) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await action(repos.path);
        this.notice = result.message;
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    resetForRepositorySwitch() {
      this.state = null;
      this.conflict = null;
      this.resultDraft = "";
      this.resultDirty = false;
      this.selectedConflictPath = "";
      this.mergeTarget = "";
      this.rebaseTarget = "";
      this.revertNoCommit = false;
      this.resetMode = "mixed";
      this.undoKeepStaged = true;
      this.loading = false;
      this.error = "";
      this.notice = "";
    },
  },
});
