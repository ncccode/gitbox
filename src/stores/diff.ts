import { defineStore } from "pinia";
import { getDiff, stageHunks } from "../lib/gitboxCommands";
import { useChangesStore } from "./changes";
import { useRepositoriesStore } from "./repositories";
import type { DiffResponse } from "../types/gitbox";

export const useDiffStore = defineStore("diff", {
  state: () => ({
    current: null as DiffResponse | null,
    loading: false,
    error: "",
  }),
  getters: {
    lines: (state) =>
      (state.current?.text ?? "").split("\n").map((content, index) => ({
        index,
        content,
        type: content.startsWith("+")
          ? "add"
          : content.startsWith("-")
            ? "delete"
            : content.startsWith("@@")
              ? "hunk"
              : content.startsWith("diff --git")
                ? "file"
                : "context",
      })),
  },
  actions: {
    async loadSelected() {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      if (!repos.path || !changes.selectedFile) {
        this.current = null;
        return;
      }

      this.loading = true;
      this.error = "";
      try {
        this.current = await getDiff(
          repos.path,
          changes.selectedFile,
          changes.selectedSide === "staged",
        );
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async applyHunk(index: number) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      const hunk = this.current?.hunks.find((item) => item.index === index);
      if (!repos.path || !hunk) return;

      this.loading = true;
      this.error = "";
      try {
        const mode = changes.selectedSide === "staged" ? "unstage" : "stage";
        await stageHunks(repos.path, [hunk.patch], mode);
        changes.notice = mode === "stage" ? "已暂存选中块" : "已取消暂存选中块";
        await changes.refresh();
        await this.loadSelected();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
