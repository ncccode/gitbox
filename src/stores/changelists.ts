import { defineStore } from "pinia";
import { useRepositoriesStore } from "./repositories";

export interface LocalChangelist {
  id: string;
  name: string;
  description: string;
  active: boolean;
  paths: string[];
  createdAt: number;
  updatedAt: number;
}

interface StoredChangelists {
  lists: LocalChangelist[];
  activeId: string;
}

function storageKey(repoPath: string) {
  return `gitbox:changelists:${repoPath}`;
}

function nowSeconds() {
  return Math.floor(Date.now() / 1000);
}

function defaultList(): LocalChangelist {
  const now = nowSeconds();
  return {
    id: "default",
    name: "默认变更",
    description: "默认变更列表",
    active: true,
    paths: [],
    createdAt: now,
    updatedAt: now,
  };
}

function normalizeState(value: Partial<StoredChangelists> | null | undefined): StoredChangelists {
  const fallback = defaultList();
  const lists = Array.isArray(value?.lists)
    ? value.lists
        .filter((item) => item && typeof item.id === "string" && typeof item.name === "string")
        .map((item) => ({
          ...item,
          description: item.description ?? "",
          active: false,
          paths: Array.isArray(item.paths)
            ? [...new Set(item.paths.filter((path) => typeof path === "string"))]
            : [],
          createdAt: item.createdAt || nowSeconds(),
          updatedAt: item.updatedAt || nowSeconds(),
        }))
    : [];
  if (!lists.some((item) => item.id === fallback.id)) {
    lists.unshift(fallback);
  }
  const requestedActiveId = value?.activeId;
  const activeId =
    requestedActiveId && lists.some((item) => item.id === requestedActiveId)
      ? requestedActiveId
      : fallback.id;
  return {
    lists: lists.map((item) => ({ ...item, active: item.id === activeId })),
    activeId,
  };
}

function readStored(repoPath: string): StoredChangelists {
  if (typeof localStorage === "undefined" || !repoPath) return normalizeState(null);
  try {
    return normalizeState(JSON.parse(localStorage.getItem(storageKey(repoPath)) || "null"));
  } catch {
    return normalizeState(null);
  }
}

function saveStored(repoPath: string, state: StoredChangelists) {
  if (typeof localStorage === "undefined" || !repoPath) return;
  localStorage.setItem(storageKey(repoPath), JSON.stringify(state));
}

export const useChangelistsStore = defineStore("changelists", {
  state: () => ({
    lists: [defaultList()] as LocalChangelist[],
    activeId: "default",
    newName: "",
    newDescription: "",
    selectedListId: "default",
  }),
  getters: {
    activeList: (state) =>
      state.lists.find((item) => item.id === state.activeId) ?? state.lists[0] ?? defaultList(),
    selectedList: (state) =>
      state.lists.find((item) => item.id === state.selectedListId) ?? state.lists[0] ?? defaultList(),
    pathAssignments: (state) => {
      const assignments = new Map<string, string>();
      for (const list of state.lists) {
        for (const path of list.paths) {
          assignments.set(path, list.id);
        }
      }
      return assignments;
    },
  },
  actions: {
    loadForCurrentRepository() {
      const repos = useRepositoriesStore();
      const stored = readStored(repos.path);
      this.lists = stored.lists;
      this.activeId = stored.activeId;
      this.selectedListId = stored.activeId;
    },
    save() {
      const repos = useRepositoriesStore();
      saveStored(repos.path, {
        lists: this.lists,
        activeId: this.activeId,
      });
    },
    createList() {
      const name = this.newName.trim();
      if (!name) return;
      this.createListFrom(name, this.newDescription.trim(), true);
      this.newName = "";
      this.newDescription = "";
    },
    createListFrom(name: string, description = "", activate = true) {
      name = name.trim();
      if (!name) return null;
      const id = `list-${Date.now().toString(36)}`;
      const now = nowSeconds();
      this.lists = [
        ...this.lists,
        {
          id,
          name,
          description: description.trim(),
          active: false,
          paths: [],
          createdAt: now,
          updatedAt: now,
        },
      ];
      if (activate) {
        this.setActive(id);
      } else {
        this.save();
      }
      return id;
    },
    updateList(id: string, updates: { name?: string; description?: string }) {
      const target = this.lists.find((item) => item.id === id);
      if (!target) return;
      const name = updates.name?.trim();
      const description = updates.description?.trim();
      const now = nowSeconds();
      this.lists = this.lists.map((item) =>
        item.id === id
          ? {
              ...item,
              name: name || item.name,
              description: description ?? item.description,
              updatedAt: now,
            }
          : item,
      );
      this.save();
    },
    setActive(id: string) {
      if (!this.lists.some((item) => item.id === id)) return;
      this.activeId = id;
      this.selectedListId = id;
      this.lists = this.lists.map((item) => ({ ...item, active: item.id === id }));
      this.save();
    },
    movePaths(paths: string[], listId?: string) {
      listId = listId ?? this.selectedListId;
      if (paths.length === 0 || !this.lists.some((item) => item.id === listId)) return;
      const uniquePaths = [...new Set(paths)];
      const now = nowSeconds();
      this.lists = this.lists.map((item) => {
        const remaining = item.paths.filter((path) => !uniquePaths.includes(path));
        if (item.id !== listId) {
          return { ...item, paths: remaining, updatedAt: now };
        }
        return {
          ...item,
          paths: [...new Set([...remaining, ...uniquePaths])].sort(),
          updatedAt: now,
        };
      });
      this.save();
    },
    listForPath(path: string) {
      const id = this.pathAssignments.get(path) ?? "default";
      return this.lists.find((item) => item.id === id) ?? this.lists[0] ?? defaultList();
    },
    pathsForList(id: string) {
      if (id === "default") return null;
      return new Set(this.lists.find((item) => item.id === id)?.paths ?? []);
    },
    deleteList(id: string) {
      if (id === "default") return;
      const target = this.lists.find((item) => item.id === id);
      if (!target) return;
      this.lists = this.lists
        .filter((item) => item.id !== id)
        .map((item) =>
          item.id === "default"
            ? { ...item, paths: [...new Set([...item.paths, ...target.paths])].sort() }
            : item,
        );
      if (this.activeId === id) {
        this.activeId = "default";
      }
      if (this.selectedListId === id) {
        this.selectedListId = this.activeId;
      }
      this.save();
    },
    pruneMissingPaths(existingPaths: string[]) {
      const existing = new Set(existingPaths);
      this.lists = this.lists.map((item) => ({
        ...item,
        paths: item.paths.filter((path) => existing.has(path)),
      }));
      this.save();
    },
    resetForRepositorySwitch() {
      this.lists = [defaultList()];
      this.activeId = "default";
      this.selectedListId = "default";
      this.newName = "";
      this.newDescription = "";
    },
  },
});
