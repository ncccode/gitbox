import { defineStore } from "pinia";
import { commitDetails, commitFileDiff, listCommits } from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import type { CommitDetails, CommitFileDiffMode, CommitSummary, DiffResponse } from "../types/gitbox";

export const useHistoryStore = defineStore("history", {
  state: () => ({
    commits: [] as CommitSummary[],
    authorCandidates: [] as CommitSummary[],
    selectedOid: "",
    details: null as CommitDetails | null,
    fileDiff: null as DiffResponse | null,
    fileDiffPath: "",
    fileDiffMode: "" as CommitFileDiffMode | "",
    branchFilter: "",
    query: "",
    authorFilters: [] as string[],
    pathFilters: [] as string[],
    loading: false,
    detailLoading: false,
    fileDiffLoading: false,
    error: "",
  }),
  actions: {
    async refresh() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        const baseOptions = {
          branch: this.branchFilter || undefined,
          query: this.query || undefined,
          pathFilters: this.pathFilters.length ? this.pathFilters : undefined,
        };
        this.commits = await listCommits(repos.path, 240, {
          ...baseOptions,
          authors: this.authorFilters.length ? this.authorFilters : undefined,
        });
        this.authorCandidates =
          this.authorFilters.length > 0
            ? await listCommits(repos.path, 500, baseOptions)
            : this.commits;
        const selectedStillExists = this.commits.some((commit) => commit.oid === this.selectedOid);
        const nextOid = selectedStillExists ? this.selectedOid : (this.commits[0]?.oid ?? "");
        if (nextOid) {
          await this.select(nextOid);
        } else {
          this.selectedOid = "";
          this.details = null;
          this.fileDiff = null;
          this.fileDiffPath = "";
          this.fileDiffMode = "";
        }
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
    async select(oid: string) {
      const repos = useRepositoriesStore();
      if (!repos.path || !oid) return;

      const changedSelection = this.selectedOid !== oid;
      this.selectedOid = oid;
      this.detailLoading = true;
      this.error = "";
      if (changedSelection) {
        this.fileDiff = null;
        this.fileDiffPath = "";
        this.fileDiffMode = "";
      }
      try {
        this.details = await commitDetails(repos.path, oid);
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.detailLoading = false;
      }
    },
    async loadFileDiff(filePath: string, mode: CommitFileDiffMode = "commit") {
      const repos = useRepositoriesStore();
      if (!repos.path || !this.selectedOid || !filePath) return;

      this.fileDiffLoading = true;
      this.error = "";
      try {
        this.fileDiff = await commitFileDiff(repos.path, this.selectedOid, filePath, mode);
        this.fileDiffPath = filePath;
        this.fileDiffMode = mode;
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.fileDiffLoading = false;
      }
    },
    resetForRepositorySwitch() {
      this.commits = [];
      this.authorCandidates = [];
      this.selectedOid = "";
      this.details = null;
      this.fileDiff = null;
      this.fileDiffPath = "";
      this.fileDiffMode = "";
      this.branchFilter = "";
      this.query = "";
      this.authorFilters = [];
      this.pathFilters = [];
      this.loading = false;
      this.detailLoading = false;
      this.fileDiffLoading = false;
      this.error = "";
    },
  },
});
