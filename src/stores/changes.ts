import { defineStore } from "pinia";
import {
  discardChanges,
  deleteShelf,
  listShelves,
  repoStatus,
  shelveChanges,
  stagePaths,
  unstagePaths,
  unshelve,
} from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import { useSettingsStore } from "./settings";
import type { ChangeSide, ChangedFile, RepoStatus, ShelfInfo } from "../types/gitbox";

type RefreshOptions = {
  includeShelves?: boolean;
};

export const useChangesStore = defineStore("changes", {
  state: () => ({
    status: null as RepoStatus | null,
    selectedFile: null as string | null,
    selectedPaths: [] as string[],
    selectedSide: "unstaged" as ChangeSide,
    shelves: [] as ShelfInfo[],
    loading: false,
    error: "",
    notice: "",
  }),
  getters: {
    files: (state) => state.status?.files ?? [],
    counts: (state) => state.status?.counts,
    branch: (state) => state.status?.branch,
    filesForSide: (state) => (side: ChangeSide) =>
      (state.status?.files ?? []).filter((file) =>
        side === "staged" ? file.staged : file.unstaged || file.untracked,
      ),
    activePaths: (state) => {
      if (state.selectedPaths.length > 0) return state.selectedPaths;
      return state.selectedFile ? [state.selectedFile] : [];
    },
    selectedCommitPaths: (state) => {
      const changedPaths = new Set((state.status?.files ?? []).map((file) => file.path));
      return state.selectedPaths.filter((path, index, paths) => changedPaths.has(path) && paths.indexOf(path) === index);
    },
  },
  actions: {
    async refresh(options: RefreshOptions = {}) {
      const repos = useRepositoriesStore();
      const settings = useSettingsStore();
      if (!repos.path) return;

      const includeShelves = options.includeShelves ?? true;
      this.loading = true;
      this.error = "";
      try {
        this.status = await repoStatus(repos.path, settings.includeIgnored);
        repos.setCurrent(this.status.repo);
        const changedPaths = new Set(this.status.files.map((file) => file.path));
        this.selectedPaths = this.selectedPaths.filter((path) => changedPaths.has(path));
        if (this.selectedFile && !changedPaths.has(this.selectedFile)) {
          this.selectedFile = null;
          this.selectedPaths = [];
        }
        if (includeShelves) {
          await this.refreshShelves();
        }
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async refreshShelves() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;
      this.shelves = await listShelves(repos.path);
    },
    selectFile(file: ChangedFile, side: ChangeSide) {
      this.selectedFile = file.path;
      this.selectedSide = side;
      useSettingsStore().setSide(side);
    },
    resetForRepositorySwitch() {
      this.status = null;
      this.selectedFile = null;
      this.selectedPaths = [];
      this.shelves = [];
      this.error = "";
      this.notice = "";
    },
    togglePath(path: string) {
      if (this.selectedPaths.includes(path)) {
        this.selectedPaths = this.selectedPaths.filter((item) => item !== path);
      } else {
        this.selectedPaths = [...this.selectedPaths, path];
      }
      if (!this.selectedFile) this.selectedFile = path;
    },
    async stageSelected() {
      await this.runPathAction((repoPath, paths) => stagePaths(repoPath, paths), "已暂存选中文件");
    },
    async unstageSelected() {
      await this.runPathAction((repoPath, paths) => unstagePaths(repoPath, paths), "已取消暂存");
    },
    async discardSelected() {
      await this.runPathAction((repoPath, paths) => discardChanges(repoPath, paths), "已回滚选中文件");
    },
    async shelveSelected(message: string) {
      const repos = useRepositoriesStore();
      const paths = this.activePaths;
      if (!repos.path || paths.length === 0) return;

      this.loading = true;
      this.error = "";
      try {
        const shelf = await shelveChanges(repos.path, paths, message);
        this.notice = `已搁置：${shelf.message}`;
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async unshelveRecord(record: ShelfInfo) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        await unshelve(repos.path, record.stashRef);
        this.notice = `已恢复：${record.message}`;
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async deleteShelfRecord(record: ShelfInfo) {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        await deleteShelf(repos.path, record.stashRef, true);
        this.notice = `已删除搁置：${record.message}`;
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async runPathAction(
      action: (repoPath: string, paths: string[]) => Promise<{ message: string }>,
      fallbackNotice: string,
    ) {
      const repos = useRepositoriesStore();
      const paths = this.activePaths;
      if (!repos.path || paths.length === 0) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await action(repos.path, paths);
        this.notice = result.message || fallbackNotice;
        await this.refresh({ includeShelves: false });
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
