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
  const normalizedRepo = normalizeRepositoryInfo(repo);
  return {
    path: normalizedRepo.path,
    initialized: true,
    repository: normalizedRepo,
  };
}

function normalizeProjectItem(value: unknown): ProjectItem | null {
  if (!isObject(value) || typeof value.path !== "string") return null;

  const path = normalizeProjectPath(value.path);
  if (!path) return null;

  if (isRepositoryInfo(value)) {
    return projectFromRepository(value);
  }

  const repository = isRepositoryInfo(value.repository) ? value.repository : null;
  const error = typeof value.error === "string" ? value.error : undefined;
  const normalizedRepo = repository ? normalizeRepositoryInfo(repository) : null;
  return {
    path: normalizedRepo?.path ?? path,
    initialized: Boolean(value.initialized) && Boolean(normalizedRepo),
    repository: normalizedRepo,
    error,
  };
}

function stripWindowsVerbatimPrefix(path: string) {
  if (path.startsWith("\\\\?\\UNC\\")) {
    return `\\\\${path.slice("\\\\?\\UNC\\".length)}`;
  }
  if (path.startsWith("\\\\?\\")) {
    return path.slice("\\\\?\\".length);
  }
  return path;
}

function normalizeProjectPath(path: string) {
  const trimmed = stripWindowsVerbatimPrefix(path.trim());
  if (!trimmed) return "";
  if (trimmed === "/" || /^[A-Za-z]:[\\/]?$/.test(trimmed)) return trimmed;
  return trimmed.replace(/[\\/]+$/, "");
}

function projectPathKey(path: string) {
  return normalizeProjectPath(path)
    .replace(/\\/g, "/")
    .replace(/^[a-z]:/, (drive) => drive.toUpperCase());
}

function normalizeRepositoryInfo(repo: RepositoryInfo): RepositoryInfo {
  return {
    ...repo,
    path: normalizeProjectPath(repo.path),
    workdir: typeof repo.workdir === "string" ? normalizeProjectPath(repo.workdir) : repo.workdir,
    gitDir: normalizeProjectPath(repo.gitDir),
  };
}

function mergeProjectItems(current: ProjectItem, incoming: ProjectItem) {
  if (incoming.initialized) return incoming;
  if (current.initialized) return current;
  return {
    ...current,
    ...incoming,
    error: incoming.error ?? current.error,
  };
}

function dedupeProjectItems(items: ProjectItem[]) {
  const byPath = new Map<string, ProjectItem>();
  for (const item of items) {
    const key = projectPathKey(item.path);
    const current = byPath.get(key);
    byPath.set(key, current ? mergeProjectItems(current, item) : item);
  }
  return [...byPath.values()];
}

function findProjectItem(items: ProjectItem[], path: string | null | undefined) {
  if (!path) return null;
  const key = projectPathKey(path);
  return items.find((item) => projectPathKey(item.path) === key) ?? null;
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
      activePath: typeof parsed.activePath === "string" ? normalizeProjectPath(parsed.activePath) : null,
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
    const activeItem = findProjectItem(stored.items, stored.activePath) ?? stored.items[0] ?? null;
    const activePath = activeItem?.path ?? null;
    const current = activeItem?.repository ?? null;

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
    selectedItem: (state) => findProjectItem(state.items, state.activePath),
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
      const uniquePaths = dedupeProjectItems(
        paths
          .map((path) => normalizeProjectPath(path))
          .filter(Boolean)
          .map((path) => ({
            path,
            initialized: false,
            repository: null,
          })),
      ).map((item) => item.path);
      const opened: RepositoryInfo[] = [];
      let activePath = "";

      this.loading = true;
      this.error = "";
      try {
        for (const path of uniquePaths) {
          try {
            const repo = await openRepo(path);
            const project = projectFromRepository(repo);
            this.upsert(project);
            opened.push(repo);
            activePath = project.path;
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
          this.current = findProjectItem(this.items, activePath)?.repository ?? null;
        }

        this.persist();
        return opened;
      } finally {
        this.loading = false;
      }
    },
    async select(path: string) {
      const projectPath = normalizeProjectPath(path);
      this.loading = true;
      this.error = "";
      try {
        const repo = await openRepo(projectPath);
        const project = projectFromRepository(repo);
        this.upsert(project);
        this.activePath = project.path;
        this.current = project.repository;
      } catch (error) {
        this.upsert({
          path: projectPath,
          initialized: false,
          repository: null,
          error: String(error),
        });
        this.activePath = projectPath;
        this.current = null;
      } finally {
        this.persist();
        this.loading = false;
      }
    },
    selectKnown(path: string) {
      const project = findProjectItem(this.items, path);
      this.activePath = project?.path ?? normalizeProjectPath(path);
      this.current = project?.repository ?? null;
      this.persist();
    },
    setCurrent(repo: RepositoryInfo) {
      const project = projectFromRepository(repo);
      this.current = project.repository;
      this.activePath = project.path;
      this.upsert(project);
      this.persist();
    },
    remove(path: string) {
      const key = projectPathKey(path);
      const wasActive = projectPathKey(this.activePath ?? "") === key;
      this.items = this.items.filter((item) => projectPathKey(item.path) !== key);
      if (wasActive) {
        this.activePath = this.items[0]?.path ?? null;
        this.current = findProjectItem(this.items, this.activePath)?.repository ?? null;
      } else if (projectPathKey(this.current?.path ?? "") === key) {
        this.current = null;
      }
      this.persist();
    },
    upsert(project: ProjectItem) {
      const normalized = normalizeProjectItem(project);
      if (!normalized) return;
      const key = projectPathKey(normalized.path);
      const index = this.items.findIndex((item) => projectPathKey(item.path) === key);
      if (index >= 0) {
        this.items.splice(index, 1, mergeProjectItems(this.items[index], normalized));
      } else {
        this.items.push(normalized);
      }
    },
    persist() {
      this.items = dedupeProjectItems(this.items);
      const activeItem = findProjectItem(this.items, this.activePath) ?? this.items[0] ?? null;
      this.activePath = activeItem?.path ?? null;
      this.current = activeItem?.repository ?? null;
      saveStoredRepositories({
        items: this.items,
        activePath: this.activePath,
      });
    },
  },
});
