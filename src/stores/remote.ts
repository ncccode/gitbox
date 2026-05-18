import { defineStore } from "pinia";
import {
  addRemote,
  deleteRemote,
  fetchRemote,
  pullRemote,
  pushRemote,
  updateRemote,
} from "../lib/gitboxCommands";
import { useChangesStore } from "./changes";
import { useRepositoriesStore } from "./repositories";

type RemoteAction = "fetch" | "pull" | "push";

function wildcardToRegExp(pattern: string) {
  const escaped = pattern
    .trim()
    .replace(/[.+?^${}()|[\]\\]/g, "\\$&")
    .replace(/\*/g, ".*");
  return new RegExp(`^${escaped}$`);
}

function looksLikeRejectedPush(error: string) {
  const value = error.toLowerCase();
  return (
    value.includes("rejected") ||
    value.includes("non-fast-forward") ||
    value.includes("fetch first") ||
    value.includes("failed to push some refs")
  );
}

export const useRemoteStore = defineStore("remote", {
  state: () => ({
    selectedRemote: "origin",
    targetBranch: "",
    fetchPrune: false,
    autoFetchEnabled: false,
    autoFetchAllRepositories: true,
    autoFetchIntervalMinutes: 5,
    setUpstream: false,
    forceWithLease: false,
    pushTags: false,
    protectBranches: true,
    allowProtectedPush: false,
    protectedBranchPatterns: "main,master,production,release/*",
    lastPushRejected: false,
    lastRejectedTarget: "",
    remoteNameDraft: "",
    remoteUrlDraft: "",
    remotePushUrlDraft: "",
    loading: false,
    error: "",
    notice: "",
  }),
  actions: {
    async run(action: RemoteAction) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      if (action === "push") {
        this.lastPushRejected = false;
      }
      try {
        if (action === "push") {
          this.assertPushAllowed();
        }
        const remoteName = this.selectedRemote || undefined;
        const result =
          action === "fetch"
            ? await fetchRemote(repos.path, remoteName, { prune: this.fetchPrune })
            : action === "pull"
              ? await pullRemote(repos.path, remoteName)
              : await pushRemote(repos.path, remoteName, {
                  targetBranch: this.targetBranch,
                  setUpstream: this.setUpstream,
                  forceWithLease: this.forceWithLease,
                  pushTags: this.pushTags,
                });
        this.notice = result.message;
        changes.notice = result.message;
        await repos.select(repos.path);
        await changes.refresh();
      } catch (error) {
        this.error = String(error);
        if (action === "push" && looksLikeRejectedPush(this.error)) {
          this.lastPushRejected = true;
          this.lastRejectedTarget = this.pushTargetRef();
        }
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async fetchAllRepositories() {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const currentPath = repos.path;
      const targets = repos.initializedItems.filter((repo) => repo.remotes.length > 0);
      if (targets.length === 0) return;

      this.loading = true;
      this.error = "";
      try {
        const errors: string[] = [];
        let count = 0;
        for (const repo of targets) {
          const remote =
            repo.remotes.find((item) => item.name === this.selectedRemote) ?? repo.remotes[0];
          try {
            await fetchRemote(repo.path, remote?.name, { prune: this.fetchPrune });
            count += 1;
          } catch (error) {
            errors.push(`${repo.path}: ${String(error)}`);
          }
        }
        if (currentPath) {
          await repos.select(currentPath);
          await changes.refresh();
        }
        this.notice = `已获取 ${count} 个仓库`;
        changes.notice = this.notice;
        if (errors.length > 0) {
          this.error = `部分仓库获取失败：${errors.join("；")}`;
          throw new Error(this.error);
        }
      } finally {
        this.loading = false;
      }
    },
    pushTargetRef() {
      const repos = useRepositoriesStore();
      return this.targetBranch.trim() || repos.current?.branch || "";
    },
    protectedPatterns() {
      return this.protectedBranchPatterns
        .split(",")
        .map((pattern) => pattern.trim())
        .filter(Boolean);
    },
    isProtectedTarget() {
      const target = this.pushTargetRef();
      if (!target || !this.protectBranches) return false;
      return this.protectedPatterns().some((pattern) => wildcardToRegExp(pattern).test(target));
    },
    assertPushAllowed() {
      if (!this.isProtectedTarget() || this.allowProtectedPush) return;
      const target = this.pushTargetRef();
      throw new Error(`受保护分支 ${target} 禁止直接推送，请勾选“允许保护分支推送”后再执行`);
    },
    syncDraftFromSelected() {
      const repos = useRepositoriesStore();
      const remote = repos.current?.remotes.find((item) => item.name === this.selectedRemote);
      this.remoteNameDraft = remote?.name ?? this.selectedRemote;
      this.remoteUrlDraft = remote?.url ?? "";
      this.remotePushUrlDraft = remote?.pushUrl ?? "";
    },
    syncTargetFromBranch(force = false) {
      const repos = useRepositoriesStore();
      if (force || !this.targetBranch) {
        this.targetBranch = repos.current?.branch ?? "";
      }
    },
    async saveRemote() {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      if (!repos.path || !this.remoteNameDraft.trim() || !this.remoteUrlDraft.trim()) return;

      this.loading = true;
      this.error = "";
      try {
        const name = this.remoteNameDraft.trim();
        const exists = repos.current?.remotes.some((item) => item.name === name);
        let result = exists
          ? await updateRemote(
              repos.path,
              name,
              this.remoteUrlDraft.trim(),
              this.remotePushUrlDraft.trim() || undefined,
            )
          : await addRemote(repos.path, name, this.remoteUrlDraft.trim());
        if (!exists && this.remotePushUrlDraft.trim()) {
          result = await updateRemote(
            repos.path,
            name,
            this.remoteUrlDraft.trim(),
            this.remotePushUrlDraft.trim(),
          );
        }
        this.selectedRemote = name;
        this.notice = result.message;
        changes.notice = result.message;
        await repos.select(repos.path);
        await changes.refresh();
        this.syncDraftFromSelected();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async deleteSelectedRemote() {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      if (!repos.path || !this.selectedRemote) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await deleteRemote(repos.path, this.selectedRemote);
        this.notice = result.message;
        changes.notice = result.message;
        await repos.select(repos.path);
        this.selectedRemote = repos.current?.remotes[0]?.name ?? "origin";
        await changes.refresh();
        this.syncDraftFromSelected();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
