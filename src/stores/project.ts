import { defineStore } from "pinia";
import { getDiff, listProjectFiles, readProjectFile, saveProjectFile, stageHunks } from "../lib/gitboxCommands";
import type { DiffResponse, ProjectFileContent, ProjectFileEntry } from "../types/gitbox";
import { useChangesStore } from "./changes";
import { useRepositoriesStore } from "./repositories";

export const PROJECT_ROOT_PATH = "__gitbox_project_root__";

export const useProjectStore = defineStore("project", {
  state: () => ({
    files: [] as ProjectFileEntry[],
    expandedPaths: [] as string[],
    openPaths: [] as string[],
    selectedPath: null as string | null,
    contents: {} as Record<string, ProjectFileContent>,
    drafts: {} as Record<string, string>,
    diffs: {} as Record<string, DiffResponse | null>,
    loadingContentPath: null as string | null,
    savingContentPath: null as string | null,
    loading: false,
    error: "",
  }),
  getters: {
    selectedFile: (state) =>
      state.files.find((file) => file.path === state.selectedPath && !file.directory) ?? null,
    openTabs: (state) =>
      state.openPaths
        .map((path) => state.files.find((file) => file.path === path && !file.directory))
        .filter((file): file is ProjectFileEntry => Boolean(file)),
    content: (state) => (state.selectedPath ? state.contents[state.selectedPath] ?? null : null),
    diff: (state) => (state.selectedPath ? state.diffs[state.selectedPath] ?? null : null),
    editorText: (state) => {
      if (!state.selectedPath) return "";
      return state.drafts[state.selectedPath] ?? state.contents[state.selectedPath]?.content ?? "";
    },
    editorDirty: (state) => {
      if (!state.selectedPath) return false;
      const content = state.contents[state.selectedPath]?.content;
      return content !== undefined && state.drafts[state.selectedPath] !== undefined && state.drafts[state.selectedPath] !== content;
    },
    isPathDirty: (state) => (path: string) => {
      const content = state.contents[path]?.content;
      return content !== undefined && state.drafts[path] !== undefined && state.drafts[path] !== content;
    },
    contentLoading: (state) => state.loadingContentPath === state.selectedPath,
    contentSaving: (state) => state.savingContentPath === state.selectedPath,
    isExpanded: (state) => (path: string) => state.expandedPaths.includes(path),
  },
  actions: {
    async refresh() {
      const repos = useRepositoriesStore();
      if (!repos.path) {
        this.resetForRepositorySwitch();
        return;
      }

      this.loading = true;
      this.error = "";
      try {
        const hadFileTree = this.files.length > 0;
        const files = await listProjectFiles(repos.path);
        const filePaths = new Set(files.filter((file) => !file.directory).map((file) => file.path));
        const directoryPaths = new Set([
          PROJECT_ROOT_PATH,
          ...files.filter((file) => file.directory).map((file) => file.path),
        ]);

        this.files = files;
        this.expandedPaths = hadFileTree
          ? this.expandedPaths.filter((path) => directoryPaths.has(path))
          : [PROJECT_ROOT_PATH];
        this.openPaths = this.openPaths.filter((path) => filePaths.has(path));
        this.contents = Object.fromEntries(
          Object.entries(this.contents).filter(([path]) => filePaths.has(path)),
        );
        this.drafts = Object.fromEntries(
          Object.entries(this.drafts).filter(([path]) => filePaths.has(path)),
        );
        this.diffs = Object.fromEntries(
          Object.entries(this.diffs).filter(([path]) => filePaths.has(path)),
        );

        if (!this.selectedPath || !this.openPaths.includes(this.selectedPath)) {
          this.selectedPath = this.openPaths[0] ?? null;
        }
        if (this.selectedPath) {
          if (!this.editorDirty) {
            delete this.contents[this.selectedPath];
            delete this.diffs[this.selectedPath];
            delete this.drafts[this.selectedPath];
            await this.loadFileContent(this.selectedPath);
          }
        }
      } catch (error) {
        this.error = String(error);
      } finally {
        this.loading = false;
      }
    },
    toggleDirectory(path: string) {
      const file = this.files.find((item) => item.path === path);
      if (path !== PROJECT_ROOT_PATH && !file?.directory) return;

      if (this.expandedPaths.includes(path)) {
        this.expandedPaths = this.expandedPaths.filter((item) => item !== path);
      } else {
        this.expandedPaths = [...this.expandedPaths, path];
      }
    },
    async openFile(path: string) {
      const repos = useRepositoriesStore();
      const file = this.files.find((item) => item.path === path);
      if (!repos.path || !file || file.directory) return;

      if (!this.openPaths.includes(path)) {
        this.openPaths = [...this.openPaths, path];
      }
      this.selectedPath = path;
      await this.loadFileContent(path);
    },
    async selectTab(path: string) {
      if (!this.openPaths.includes(path)) return;
      this.selectedPath = path;
      await this.loadFileContent(path);
    },
    async closeTab(path: string) {
      const index = this.openPaths.indexOf(path);
      if (index < 0) return;

      this.openPaths = this.openPaths.filter((item) => item !== path);
      delete this.contents[path];
      delete this.drafts[path];
      delete this.diffs[path];

      if (this.selectedPath !== path) return;

      const nextPath = this.openPaths[index] ?? this.openPaths[index - 1] ?? null;
      this.selectedPath = nextPath;
      if (nextPath) {
        await this.loadFileContent(nextPath);
      }
    },
    async loadFileContent(path: string) {
      const repos = useRepositoriesStore();
      const file = this.files.find((item) => item.path === path);
      if (!repos.path || !file || file.directory) return;
      if (this.contents[path] && Object.prototype.hasOwnProperty.call(this.diffs, path)) return;

      this.loadingContentPath = path;
      this.error = "";
      try {
        if (!this.contents[path]) {
          this.contents[path] = await readProjectFile(repos.path, path);
        }
        this.diffs[path] = this.contents[path].binary ? null : await getDiff(repos.path, path, false);
      } catch (error) {
        if (!this.contents[path]) {
          delete this.contents[path];
        }
        delete this.diffs[path];
        this.error = String(error);
      } finally {
        if (this.loadingContentPath === path) {
          this.loadingContentPath = null;
        }
      }
    },
    setEditorText(value: string) {
      const path = this.selectedPath;
      if (!path) return;
      this.drafts[path] = value;
    },
    async saveSelectedContent() {
      if (!this.selectedPath) return;
      await this.saveContent(this.selectedPath);
    },
    async saveContent(path: string) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const content = this.contents[path] ?? null;
      if (!repos.path || !path || !content || content.binary) return;

      const text = this.drafts[path] ?? content.content ?? "";
      this.savingContentPath = path;
      this.error = "";
      try {
        const saved = await saveProjectFile(repos.path, path, text);
        this.contents[path] = saved;
        this.drafts[path] = saved.content ?? "";
        this.diffs[path] = await getDiff(repos.path, path, false);
        changes.notice = "已保存文件";
        await changes.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        if (this.savingContentPath === path) {
          this.savingContentPath = null;
        }
      }
    },
    async discardHunk(index: number) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const path = this.selectedPath;
      const hunk = this.diff?.hunks.find((item) => item.index === index);
      if (!repos.path || !path || !hunk) return;

      this.loadingContentPath = path;
      this.error = "";
      try {
        await stageHunks(repos.path, [hunk.patch], "discard");
        delete this.contents[path];
        delete this.diffs[path];
        changes.notice = "已撤回选中块";
        await changes.refresh();
        await this.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        if (this.loadingContentPath === path) {
          this.loadingContentPath = null;
        }
      }
    },
    async stageHunk(index: number) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const path = this.selectedPath;
      const hunk = this.diff?.hunks.find((item) => item.index === index);
      if (!repos.path || !path || !hunk) return;

      this.loadingContentPath = path;
      this.error = "";
      try {
        await stageHunks(repos.path, [hunk.patch], "stage");
        delete this.diffs[path];
        changes.notice = "已暂存选中块";
        await changes.refresh();
        await this.loadFileContent(path);
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        if (this.loadingContentPath === path) {
          this.loadingContentPath = null;
        }
      }
    },
    resetForRepositorySwitch() {
      this.files = [];
      this.expandedPaths = [];
      this.openPaths = [];
      this.selectedPath = null;
      this.contents = {};
      this.drafts = {};
      this.diffs = {};
      this.loading = false;
      this.loadingContentPath = null;
      this.savingContentPath = null;
      this.error = "";
    },
  },
});
