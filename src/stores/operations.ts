import { defineStore } from "pinia";
import {
  analyzeConflictFile,
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
  revertCommitFiles,
  resolveConflictBlock,
  resolveConflictFile,
  saveConflictResult,
  undoLastCommit,
} from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import type { ConflictAnalysis, ConflictDetails, GitOperationState, MergePreview } from "../types/gitbox";

type OperationAction = "continue" | "abort" | "skip";
type ConflictSide = "ours" | "theirs";
type ConflictBlockSide = "ours" | "base" | "theirs";
type ConflictBlockSelection = ConflictBlockSide | "combined";
type ConflictResultSource = "ours" | "base" | "theirs" | "current";
type LoadConflictOptions = {
  preserveInitial?: boolean;
};
type RefreshOptions = {
  preserveConflictInitial?: boolean;
};
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

function conflictBlockContent(
  block: ConflictDetails["blocks"][number],
  selection: ConflictBlockSide,
) {
  return selection === "base" ? (block.base ?? "") : block[selection];
}

function uniqueConflictBlockCandidates(
  block: ConflictDetails["blocks"][number],
  currentReplacement?: string,
) {
  const values = [
    currentReplacement,
    block.ours,
    block.base ?? "",
    block.theirs,
  ].filter((value): value is string => Boolean(value));
  return [...new Set(values)];
}

function findCandidateRange(draft: string, candidates: string[], startAt: number) {
  let best: { start: number; end: number } | null = null;

  for (const candidate of candidates) {
    const foundAt = draft.indexOf(candidate, startAt);
    if (foundAt < 0) continue;
    if (!best || foundAt < best.start || (foundAt === best.start && candidate.length > best.end - best.start)) {
      best = {
        start: foundAt,
        end: foundAt + candidate.length,
      };
    }
  }

  return best;
}

function appendConflictContent(current: string, addition: string) {
  if (!current) return addition;
  if (!addition) return current;
  if (current.endsWith("\n") || addition.startsWith("\n")) {
    return `${current}${addition}`;
  }
  return `${current}\n${addition}`;
}

function replaceResultDraftBlock(
  draft: string,
  blocks: ConflictDetails["blocks"],
  blockIndex: number,
  replacement: string,
  currentReplacements: Record<number, string>,
) {
  let searchFrom = 0;

  for (const block of blocks) {
    const candidates = uniqueConflictBlockCandidates(block, currentReplacements[block.index]);
    if (block.index === blockIndex) {
      const range = findCandidateRange(draft, candidates, searchFrom);
      if (!range) {
        return `${draft.slice(0, searchFrom)}${replacement}${draft.slice(searchFrom)}`;
      }
      return `${draft.slice(0, range.start)}${replacement}${draft.slice(range.end)}`;
    }

    const range = findCandidateRange(draft, candidates, searchFrom);
    if (range) {
      searchFrom = range.end;
    }
  }

  return draft;
}

function appendResultDraftBlock(
  draft: string,
  blocks: ConflictDetails["blocks"],
  blockIndex: number,
  addition: string,
  currentReplacements: Record<number, string>,
) {
  let searchFrom = 0;

  for (const block of blocks) {
    const candidates = uniqueConflictBlockCandidates(block, currentReplacements[block.index]);
    if (block.index === blockIndex) {
      const range = findCandidateRange(draft, candidates, searchFrom);
      if (!range) {
        return {
          draft: `${draft.slice(0, searchFrom)}${addition}${draft.slice(searchFrom)}`,
          replacement: addition,
        };
      }

      const current = draft.slice(range.start, range.end);
      const replacement = appendConflictContent(current, addition);
      return {
        draft: `${draft.slice(0, range.start)}${replacement}${draft.slice(range.end)}`,
        replacement,
      };
    }

    const range = findCandidateRange(draft, candidates, searchFrom);
    if (range) {
      searchFrom = range.end;
    }
  }

  return {
    draft,
    replacement: currentReplacements[blockIndex] ?? "",
  };
}

export const useOperationsStore = defineStore("operations", {
  state: () => ({
    state: null as GitOperationState | null,
    conflict: null as ConflictDetails | null,
    conflictAnalysis: null as ConflictAnalysis | null,
    mergePreview: null as MergePreview | null,
    resultDraft: "",
    resultInitialDraft: "",
    resultSavedDraft: "",
    resultBlockReplacements: {} as Record<number, string>,
    resultBlockSelections: {} as Record<number, ConflictBlockSelection>,
    resultInitialBlockReplacements: {} as Record<number, string>,
    resultInitialBlockSelections: {} as Record<number, ConflictBlockSelection>,
    resultInitialConflictPath: "",
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
    async refresh(options: RefreshOptions = {}) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.error = "";
      try {
        this.state = await operationState(repos.path);
        if (!this.selectedConflictPath || !this.state.conflictedPaths.includes(this.selectedConflictPath)) {
          this.selectedConflictPath = this.state.conflictedPaths[0] ?? "";
        }
        if (this.selectedConflictPath) {
          await this.loadConflict(this.selectedConflictPath, {
            preserveInitial: options.preserveConflictInitial,
          });
        } else {
          this.conflict = null;
          this.conflictAnalysis = null;
        }
      } catch (error) {
        this.error = String(error);
        throw error;
      }
    },
    async loadConflict(filePath: string, options: LoadConflictOptions = {}) {
      const repos = useRepositoriesStore();
      if (!repos.path || !filePath) return;

      this.selectedConflictPath = filePath;
      this.error = "";
      try {
        this.conflict = await conflictDetails(repos.path, filePath);
        this.conflictAnalysis = await analyzeConflictFile(repos.path, filePath).catch(() => null);
        const initialDraft = this.initialResultFromConflict();
        const shouldPreserveInitial =
          options.preserveInitial && this.resultInitialConflictPath === filePath;
        this.resultDraft = initialDraft;
        this.resultSavedDraft = initialDraft;
        this.resultBlockReplacements = {};
        this.resultBlockSelections = {};
        if (!shouldPreserveInitial) {
          this.resultInitialConflictPath = filePath;
          this.resultInitialDraft = initialDraft;
          this.resultInitialBlockReplacements = {};
          this.resultInitialBlockSelections = {};
        }
        this.resultDirty = false;
      } catch (error) {
        this.conflictAnalysis = null;
        this.error = String(error);
        throw error;
      }
    },
    setMergePreview(preview: MergePreview | null) {
      this.mergePreview = preview;
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
    async revertFiles(oid: string, files: string[]) {
      if (!oid || files.length === 0) return;
      await this.runOperation((repoPath) => revertCommitFiles(repoPath, oid, files));
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
      this.resultBlockReplacements = {};
      this.resultBlockSelections =
        source === "ours" || source === "base" || source === "theirs"
          ? Object.fromEntries((this.conflict?.blocks ?? []).map((block) => [block.index, source]))
          : {};
      this.resultDirty = true;
    },
    replaceResultBlock(index: number, side: ConflictBlockSide) {
      const block = this.conflict?.blocks.find((item) => item.index === index);
      if (!block) return;
      const replacement = conflictBlockContent(block, side);
      this.resultDraft = replaceResultDraftBlock(
        this.resultDraft,
        this.conflict?.blocks ?? [],
        index,
        replacement,
        this.resultBlockReplacements,
      );
      this.resultBlockReplacements = {
        ...this.resultBlockReplacements,
        [index]: replacement,
      };
      this.resultBlockSelections = {
        ...this.resultBlockSelections,
        [index]: side,
      };
      this.resultDirty = true;
    },
    appendResultBlock(index: number, side: ConflictBlockSide) {
      const block = this.conflict?.blocks.find((item) => item.index === index);
      if (!block) return;
      const result = appendResultDraftBlock(
        this.resultDraft,
        this.conflict?.blocks ?? [],
        index,
        conflictBlockContent(block, side),
        this.resultBlockReplacements,
      );
      this.resultDraft = result.draft;
      this.resultBlockReplacements = {
        ...this.resultBlockReplacements,
        [index]: result.replacement,
      };
      this.resultBlockSelections = {
        ...this.resultBlockSelections,
        [index]: "combined",
      };
      this.resultDirty = true;
    },
    useAllConflictBlocks(side: ConflictBlockSide) {
      if (!this.conflict?.current) return;
      this.resultDraft = cleanConflictMarkers(this.conflict.current, side);
      this.resultBlockReplacements = Object.fromEntries(
        this.conflict.blocks.map((block) => [block.index, conflictBlockContent(block, side)]),
      );
      this.resultBlockSelections = Object.fromEntries(
        this.conflict.blocks.map((block) => [block.index, side]),
      );
      this.resultDirty = true;
    },
    setResultDraft(content: string) {
      this.resultDraft = content;
      this.resultDirty = content !== this.resultSavedDraft;
    },
    async saveResult(markResolved = false) {
      if (!this.selectedConflictPath) return;
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      const filePath = this.selectedConflictPath;
      const content = this.resultDraft;
      this.loading = true;
      this.error = "";
      try {
        const result = await saveConflictResult(repos.path, filePath, content, markResolved);
        this.notice = result.message;
        this.resultSavedDraft = content;
        this.resultDirty = false;
        await this.refresh({ preserveConflictInitial: !markResolved });
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async restoreInitialResult() {
      if (!this.selectedConflictPath) return;
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      const filePath = this.selectedConflictPath;
      const initialDraft = this.resultInitialDraft;
      this.resultDraft = initialDraft;
      this.resultBlockReplacements = { ...this.resultInitialBlockReplacements };
      this.resultBlockSelections = { ...this.resultInitialBlockSelections };
      this.resultDirty = initialDraft !== this.resultSavedDraft;
      if (!this.resultDirty) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await saveConflictResult(repos.path, filePath, initialDraft, false);
        this.notice = result.message;
        this.resultSavedDraft = initialDraft;
        this.resultDirty = false;
        await this.loadConflict(filePath, { preserveInitial: true });
        this.resultDraft = initialDraft;
        this.resultBlockReplacements = { ...this.resultInitialBlockReplacements };
        this.resultBlockSelections = { ...this.resultInitialBlockSelections };
        this.resultDirty = false;
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
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
      this.conflictAnalysis = null;
      this.mergePreview = null;
      this.resultDraft = "";
      this.resultInitialDraft = "";
      this.resultSavedDraft = "";
      this.resultDirty = false;
      this.resultBlockReplacements = {};
      this.resultBlockSelections = {};
      this.resultInitialBlockReplacements = {};
      this.resultInitialBlockSelections = {};
      this.resultInitialConflictPath = "";
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
