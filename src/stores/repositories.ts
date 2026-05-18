import { defineStore } from "pinia";
import { openRepo } from "../lib/gitboxCommands";
import type { RepositoryInfo } from "../types/gitbox";

const STORAGE_KEY = "gitbox.repositories";

interface StoredRepositories {
  items: RepositoryInfo[];
  activePath?: string | null;
}

function readStoredRepositories(): StoredRepositories {
  if (typeof window === "undefined") return { items: [] };

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { items: [] };

    const parsed = JSON.parse(raw) as Partial<StoredRepositories> | RepositoryInfo[];
    if (Array.isArray(parsed)) return { items: parsed };
    return {
      items: Array.isArray(parsed.items) ? parsed.items : [],
      activePath: parsed.activePath,
    };
  } catch {
    return { items: [] };
  }
}

function saveStoredRepositories(state: StoredRepositories) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function nameFromPath(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

export const useRepositoriesStore = defineStore("repositories", {
  state: () => {
    const stored = readStoredRepositories();
    const current =
      stored.items.find((item) => item.path === stored.activePath) ?? stored.items[0] ?? null;

    return {
      current: current as RepositoryInfo | null,
      items: stored.items,
      loading: false,
      error: "",
    };
  },
  getters: {
    path: (state) => state.current?.path ?? "",
    name: (state) => {
      if (!state.current?.path) return "未打开仓库";
      return nameFromPath(state.current.path);
    },
    projectName: () => (path: string) => nameFromPath(path),
  },
  actions: {
    async open(path: string) {
      const opened = await this.openMany([path]);
      return opened[opened.length - 1];
    },
    async openMany(paths: string[]) {
      const uniquePaths = [...new Set(paths.filter(Boolean))];
      const opened: RepositoryInfo[] = [];
      const errors: string[] = [];

      this.loading = true;
      this.error = "";
      try {
        for (const path of uniquePaths) {
          try {
            const repo = await openRepo(path);
            this.upsert(repo);
            opened.push(repo);
          } catch (error) {
            errors.push(`${path}: ${String(error)}`);
          }
        }

        if (opened.length > 0) {
          this.current = opened[opened.length - 1];
        }

        if (errors.length > 0) {
          this.error = `部分项目添加失败：${errors.join("；")}`;
          if (opened.length === 0) throw new Error(this.error);
        }

        this.persist();
        return opened;
      } finally {
        this.loading = false;
      }
    },
    async select(path: string) {
      this.loading = true;
      this.error = "";
      try {
        const repo = await openRepo(path);
        this.upsert(repo);
        this.current = repo;
        this.persist();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    setCurrent(repo: RepositoryInfo) {
      this.current = repo;
      this.upsert(repo);
      this.persist();
    },
    remove(path: string) {
      const wasCurrent = this.current?.path === path;
      this.items = this.items.filter((item) => item.path !== path);
      if (wasCurrent) {
        this.current = this.items[0] ?? null;
      }
      this.persist();
    },
    upsert(repo: RepositoryInfo) {
      const index = this.items.findIndex((item) => item.path === repo.path);
      if (index >= 0) {
        this.items.splice(index, 1, repo);
      } else {
        this.items.push(repo);
      }
    },
    persist() {
      saveStoredRepositories({
        items: this.items,
        activePath: this.current?.path ?? null,
      });
    },
  },
});
