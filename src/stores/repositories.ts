import { defineStore } from "pinia";
import { openRepo } from "../lib/gitboxCommands";
import type { RepositoryInfo } from "../types/gitbox";

const STORAGE_KEY = "gitbox.repositories";

export interface ProjectItem {
  path: string;
  initialized: boolean;
  repository: RepositoryInfo | null;
  error?: string;
}

interface StoredRepositories {
  items: ProjectItem[];
  activePath?: string | null;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isRepositoryInfo(value: unknown): value is RepositoryInfo {
  return (
    isObject(value) &&
    typeof value.path === "string" &&
    typeof value.gitDir === "string" &&
    typeof value.isBare === "boolean" &&
    Array.isArray(value.remotes)
  );
}

function projectFromRepository(repo: RepositoryInfo): ProjectItem {
  return {
    path: repo.path,
    initialized: true,
    repository: repo,
  };
}

function normalizeProjectItem(value: unknown): ProjectItem | null {
  if (!isObject(value) || typeof value.path !== "string" || !value.path.trim()) return null;

  if (isRepositoryInfo(value)) {
    return projectFromRepository(value);
  }

  const repository = isRepositoryInfo(value.repository) ? value.repository : null;
  const error = typeof value.error === "string" ? value.error : undefined;
  return {
    path: value.path,
    initialized: Boolean(value.initialized) && Boolean(repository),
    repository,
    error,
  };
}

function dedupeProjectItems(items: ProjectItem[]) {
  const byPath = new Map<string, ProjectItem>();
  for (const item of items) {
    byPath.set(item.path, item);
  }
  return [...byPath.values()];
}

function readStoredRepositories(): StoredRepositories {
  if (typeof window === "undefined") return { items: [] };

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { items: [] };

    const parsed = JSON.parse(raw) as Partial<StoredRepositories> | RepositoryInfo[];
    if (Array.isArray(parsed)) {
      return {
        items: dedupeProjectItems(
          parsed.map(normalizeProjectItem).filter((item): item is ProjectItem => Boolean(item)),
        ),
      };
    }

    const items = Array.isArray(parsed.items)
      ? parsed.items.map(normalizeProjectItem).filter((item): item is ProjectItem => Boolean(item))
      : [];

    return {
      items: dedupeProjectItems(items),
      activePath: typeof parsed.activePath === "string" ? parsed.activePath : null,
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
    const activePath =
      stored.items.find((item) => item.path === stored.activePath)?.path ?? stored.items[0]?.path ?? null;
    const current = stored.items.find((item) => item.path === activePath)?.repository ?? null;

    return {
      current: current as RepositoryInfo | null,
      activePath: activePath as string | null,
      items: stored.items,
      loading: false,
      error: "",
    };
  },
  getters: {
    path: (state) => state.current?.path ?? "",
    selectedPath: (state) => state.activePath ?? "",
    selectedItem: (state) => state.items.find((item) => item.path === state.activePath) ?? null,
    initializedItems: (state) =>
      state.items
        .map((item) => item.repository)
        .filter((repo): repo is RepositoryInfo => Boolean(repo)),
    name: (state) => {
      const path = state.current?.path ?? state.activePath;
      if (!path) return "未打开仓库";
      return nameFromPath(path);
    },
    projectName: () => (path: string) => nameFromPath(path),
  },
  actions: {
    async open(path: string) {
      const opened = await this.openMany([path]);
      return opened[opened.length - 1] ?? null;
    },
    async openMany(paths: string[]) {
      const uniquePaths = [...new Set(paths.filter(Boolean))];
      const opened: RepositoryInfo[] = [];
      let activePath = "";

      this.loading = true;
      this.error = "";
      try {
        for (const path of uniquePaths) {
          try {
            const repo = await openRepo(path);
            this.upsert(projectFromRepository(repo));
            opened.push(repo);
            activePath = repo.path;
          } catch (error) {
            this.upsert({
              path,
              initialized: false,
              repository: null,
              error: String(error),
            });
            activePath = path;
          }
        }

        if (activePath) {
          this.activePath = activePath;
          this.current = this.items.find((item) => item.path === activePath)?.repository ?? null;
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
        this.upsert(projectFromRepository(repo));
        this.activePath = repo.path;
        this.current = repo;
      } catch (error) {
        this.upsert({
          path,
          initialized: false,
          repository: null,
          error: String(error),
        });
        this.activePath = path;
        this.current = null;
      } finally {
        this.persist();
        this.loading = false;
      }
    },
    selectKnown(path: string) {
      this.activePath = path;
      this.current = this.items.find((item) => item.path === path)?.repository ?? null;
      this.persist();
    },
    setCurrent(repo: RepositoryInfo) {
      this.current = repo;
      this.activePath = repo.path;
      this.upsert(projectFromRepository(repo));
      this.persist();
    },
    remove(path: string) {
      const wasActive = this.activePath === path;
      this.items = this.items.filter((item) => item.path !== path);
      if (wasActive) {
        this.activePath = this.items[0]?.path ?? null;
        this.current = this.items.find((item) => item.path === this.activePath)?.repository ?? null;
      } else if (this.current?.path === path) {
        this.current = null;
      }
      this.persist();
    },
    upsert(project: ProjectItem) {
      const index = this.items.findIndex((item) => item.path === project.path);
      if (index >= 0) {
        this.items.splice(index, 1, project);
      } else {
        this.items.push(project);
      }
    },
    persist() {
      saveStoredRepositories({
        items: this.items,
        activePath: this.activePath,
      });
    },
  },
});
