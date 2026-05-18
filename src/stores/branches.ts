import { defineStore } from "pinia";
import {
  checkoutBranch,
  checkoutRemoteBranch,
  createBranch,
  createTag,
  deleteBranch,
  deleteRemoteBranch,
  deleteRemoteTag,
  deleteTag,
  listBranches,
  pushTag,
  setBranchUpstream,
} from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import type { BranchList } from "../types/gitbox";

function favoriteStorageKey(repoPath: string) {
  return `gitbox:favorite-refs:${repoPath}`;
}

function readFavoriteRefs(repoPath: string) {
  if (typeof localStorage === "undefined" || !repoPath) return [];
  try {
    const raw = localStorage.getItem(favoriteStorageKey(repoPath));
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((item) => typeof item === "string") : [];
  } catch {
    return [];
  }
}

function saveFavoriteRefs(repoPath: string, refs: string[]) {
  if (typeof localStorage === "undefined" || !repoPath) return;
  localStorage.setItem(favoriteStorageKey(repoPath), JSON.stringify(refs));
}

export const useBranchesStore = defineStore("branches", {
  state: () => ({
    list: null as BranchList | null,
    favoriteRefs: [] as string[],
    loading: false,
    error: "",
    notice: "",
    selectedLocalBranch: "",
    upstreamTarget: "",
  }),
  getters: {
    localBranches: (state) => state.list?.branches.filter((branch) => branch.branchType === "local") ?? [],
    remoteBranches: (state) =>
      state.list?.branches.filter((branch) => branch.branchType === "remote") ?? [],
    sortedLocalBranches(): BranchList["branches"] {
      return [...this.localBranches].sort((left, right) => {
        const favoriteDelta =
          Number(this.favoriteRefs.includes(right.fullName)) -
          Number(this.favoriteRefs.includes(left.fullName));
        if (favoriteDelta !== 0) return favoriteDelta;
        return left.name.localeCompare(right.name);
      });
    },
    sortedRemoteBranches(): BranchList["branches"] {
      return [...this.remoteBranches].sort((left, right) => {
        const favoriteDelta =
          Number(this.favoriteRefs.includes(right.fullName)) -
          Number(this.favoriteRefs.includes(left.fullName));
        if (favoriteDelta !== 0) return favoriteDelta;
        return left.name.localeCompare(right.name);
      });
    },
  },
  actions: {
    async refresh() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        this.favoriteRefs = readFavoriteRefs(repos.path);
        this.list = await listBranches(repos.path);
        this.syncUpstreamDraft();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    isFavorite(refName: string) {
      return this.favoriteRefs.includes(refName);
    },
    toggleFavorite(refName: string) {
      const repos = useRepositoriesStore();
      if (!repos.path || !refName) return;
      if (this.favoriteRefs.includes(refName)) {
        this.favoriteRefs = this.favoriteRefs.filter((item) => item !== refName);
      } else {
        this.favoriteRefs = [...this.favoriteRefs, refName].sort();
      }
      saveFavoriteRefs(repos.path, this.favoriteRefs);
    },
    async checkout(name: string) {
      await this.runBranchAction((repoPath) => checkoutBranch(repoPath, name), `已切换到 ${name}`);
    },
    async checkoutRemote(remoteBranch: string, localName?: string) {
      await this.runBranchAction(
        (repoPath) => checkoutRemoteBranch(repoPath, remoteBranch, localName),
        `已检出 ${remoteBranch}`,
      );
    },
    async create(name: string, checkout = true, startPoint?: string) {
      await this.runBranchAction(
        (repoPath) => createBranch(repoPath, name, checkout, startPoint),
        checkout ? `已创建并切换到 ${name}` : `已创建分支 ${name}`,
      );
    },
    async delete(name: string, force = false) {
      await this.runBranchAction((repoPath) => deleteBranch(repoPath, name, force), `已删除分支 ${name}`);
    },
    async deleteRemote(remoteBranch: string) {
      await this.runBranchAction(
        (repoPath) => deleteRemoteBranch(repoPath, remoteBranch),
        `已删除远程分支 ${remoteBranch}`,
      );
    },
    async setUpstream(branchName: string, upstream?: string) {
      await this.runBranchAction(
        (repoPath) => setBranchUpstream(repoPath, branchName, upstream),
        upstream ? `已设置 ${branchName} 的上游分支` : `已取消 ${branchName} 的上游分支`,
      );
    },
    async createTag(name: string, target?: string, annotated = false, message?: string) {
      await this.runBranchAction(
        (repoPath) => createTag(repoPath, name, { target, annotated, message }),
        `已创建标签 ${name}`,
      );
    },
    async deleteTag(name: string) {
      await this.runBranchAction((repoPath) => deleteTag(repoPath, name), `已删除本地标签 ${name}`);
    },
    async pushTag(name: string, remoteName?: string) {
      await this.runBranchAction(
        (repoPath) => pushTag(repoPath, remoteName, name),
        `已推送标签 ${name}`,
      );
    },
    async deleteRemoteTag(name: string, remoteName?: string) {
      await this.runBranchAction(
        (repoPath) => deleteRemoteTag(repoPath, remoteName, name),
        `已删除远程标签 ${name}`,
      );
    },
    syncUpstreamDraft(forceTarget = false) {
      const local = this.localBranches;
      const remote = this.remoteBranches;
      if (!local.some((branch) => branch.name === this.selectedLocalBranch)) {
        this.selectedLocalBranch = local.find((branch) => branch.current)?.name ?? local[0]?.name ?? "";
      }

      const selected = local.find((branch) => branch.name === this.selectedLocalBranch);
      const remoteNames = remote.map((branch) => branch.name);
      const suggested =
        selected?.upstream && remoteNames.includes(selected.upstream)
          ? selected.upstream
          : selected && remoteNames.includes(`origin/${selected.name}`)
            ? `origin/${selected.name}`
            : remoteNames[0] ?? "";

      if (forceTarget || !remoteNames.includes(this.upstreamTarget)) {
        this.upstreamTarget = suggested;
      }
    },
    async runBranchAction(
      action: (repoPath: string) => Promise<{ message: string }>,
      fallbackNotice: string,
    ) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await action(repos.path);
        this.notice = result.message || fallbackNotice;
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    resetForRepositorySwitch() {
      this.list = null;
      this.favoriteRefs = [];
      this.loading = false;
      this.error = "";
      this.notice = "";
      this.selectedLocalBranch = "";
      this.upstreamTarget = "";
    },
  },
});
