import { defineStore } from "pinia";
import { commitRepo, pushRemote } from "../lib/gitboxCommands";
import { useChangesStore } from "./changes";
import { useRepositoriesStore } from "./repositories";

export const useCommitStore = defineStore("commit", {
  state: () => ({
    message: "",
    amend: false,
    signOff: false,
    gpgSign: false,
    author: "",
    loading: false,
    error: "",
    lastCommit: "",
  }),
  actions: {
    async commit(remoteName?: string, includeWorktree = false) {
      const repos = useRepositoriesStore();
      const changes = useChangesStore();
      if (!repos.path || !this.message.trim()) return;

      this.loading = true;
      this.error = "";
      try {
        const result = await commitRepo(repos.path, this.message, {
          amend: this.amend,
          signOff: this.signOff,
          gpgSign: this.gpgSign,
          author: this.author.trim() || undefined,
          includeWorktree,
        });
        this.lastCommit = result.oid.slice(0, 12);
        this.message = "";
        const commitNotice = this.amend ? `已 amend ${this.lastCommit}` : `已提交 ${this.lastCommit}`;
        if (remoteName) {
          const pushResult = await pushRemote(repos.path, remoteName);
          changes.notice = `${commitNotice}，${pushResult.message}`;
        } else {
          changes.notice = commitNotice;
        }
        await changes.refresh();
      } catch (error) {
        this.error = String(error);
        throw error;
      } finally {
        this.loading = false;
      }
    },
  },
});
