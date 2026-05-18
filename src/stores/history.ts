import { defineStore } from "pinia";
import { commitDetails, listCommits } from "../lib/gitboxCommands";
import { useRepositoriesStore } from "./repositories";
import type { CommitDetails, CommitSummary } from "../types/gitbox";

export const useHistoryStore = defineStore("history", {
  state: () => ({
    commits: [] as CommitSummary[],
    selectedOid: "",
    details: null as CommitDetails | null,
    branchFilter: "",
    query: "",
    authorFilter: "",
    pathFilter: "",
    loading: false,
    detailLoading: false,
    error: "",
  }),
  actions: {
    async refresh() {
      const repos = useRepositoriesStore();
      if (!repos.path) return;

      this.loading = true;
      this.error = "";
      try {
        this.commits = await listCommits(repos.path, 240, {
          branch: this.branchFilter || undefined,
          query: this.query || undefined,
          author: this.authorFilter || undefined,
          pathFilter: this.pathFilter || undefined,
        });
        const selectedStillExists = this.commits.some((commit) => commit.oid === this.selectedOid);
        const nextOid = selectedStillExists ? this.selectedOid : (this.commits[0]?.oid ?? "");
        if (nextOid) {
          await this.select(nextOid);
        } else {
          this.selectedOid = "";
          this.details = null;
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

      this.selectedOid = oid;
      this.detailLoading = true;
      this.error = "";
      try {
        this.details = await commitDetails(repos.path, oid);
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.detailLoading = false;
      }
    },
    resetForRepositorySwitch() {
      this.commits = [];
      this.selectedOid = "";
      this.details = null;
      this.branchFilter = "";
      this.query = "";
      this.authorFilter = "";
      this.pathFilter = "";
      this.loading = false;
      this.detailLoading = false;
      this.error = "";
    },
  },
});
