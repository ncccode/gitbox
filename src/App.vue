<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Archive,
  ArchiveRestore,
  Check,
  Columns3,
  Download,
  FolderOpen,
  GitBranch,
  GitCommitVertical,
  ListChecks,
  Minus,
  Plus,
  RefreshCw,
  RotateCcw,
  Search,
  Star,
  Trash2,
  Upload,
  X,
} from "@lucide/vue";
import AppTopbar from "./components/AppTopbar.vue";
import ProjectPane from "./components/ProjectPane.vue";
import WorkbenchRail from "./components/WorkbenchRail.vue";
import { useAdvancedStore } from "./stores/advanced";
import { useBranchesStore } from "./stores/branches";
import { useChangelistsStore } from "./stores/changelists";
import { useChangesStore } from "./stores/changes";
import { useCommitStore } from "./stores/commit";
import { useDiffStore } from "./stores/diff";
import { useHistoryStore } from "./stores/history";
import { useOperationsStore } from "./stores/operations";
import { useRemoteStore } from "./stores/remote";
import { useRepositoriesStore } from "./stores/repositories";
import { useSettingsStore } from "./stores/settings";
import type { LayoutPanelKey, ThemeMode } from "./stores/settings";
import type {
  BranchInfo,
  ChangeSide,
  ChangedFile,
  CommitFileChange,
  ShelfInfo,
  TagInfo,
} from "./types/gitbox";

const repos = useRepositoriesStore();
const advanced = useAdvancedStore();
const branches = useBranchesStore();
const changelists = useChangelistsStore();
const changes = useChangesStore();
const commit = useCommitStore();
const diff = useDiffStore();
const history = useHistoryStore();
const operations = useOperationsStore();
const remote = useRemoteStore();
const settings = useSettingsStore();
const shelveMessage = ref("");
const newBranchName = ref("");
const newTagName = ref("");
const newTagTarget = ref("");
const annotatedTag = ref(false);
const tagMessage = ref("");
const selectedCommitFilePaths = ref<string[]>([]);
type WorkbenchMode = "changes" | "log" | "branches" | "remote" | "operations" | "advanced";

const workbenchMode = ref<WorkbenchMode>("changes");
const activeResizePanel = ref<LayoutPanelKey | null>(null);
const systemPrefersDark = ref(
  typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches,
);
let stopSystemThemeWatch: (() => void) | null = null;
let autoFetchTimer: number | null = null;
const repositoryContextModes = new Set<WorkbenchMode>(["branches", "remote", "operations"]);
const workbenchContextModes = new Set<WorkbenchMode>(["changes", "log", "advanced"]);

const activeFiles = computed(() => {
  const files = changes.filesForSide(settings.selectedSide);
  return files.filter((file) => changelists.listForPath(file.path).id === changelists.selectedListId);
});
const usesRepositoryContext = computed(() => repositoryContextModes.has(workbenchMode.value));
const usesWorkbenchContext = computed(() => workbenchContextModes.has(workbenchMode.value));
const counts = computed(() => changes.status?.counts);
const branch = computed(() => changes.branch);
const brandSubtitle = computed(() =>
  repos.current ? repos.name : `${repos.items.length} 个项目`,
);
const activeError = computed(
  () =>
    repos.error ||
    branches.error ||
    changes.error ||
    diff.error ||
    commit.error ||
    history.error ||
    operations.error ||
    remote.error ||
    advanced.error,
);
const activeNotice = computed(
  () =>
    operations.notice ||
    advanced.notice ||
    branches.notice ||
    changes.notice ||
    remote.notice ||
    (commit.lastCommit ? `最近提交 ${commit.lastCommit}` : ""),
);
const selectedDiffTitle = computed(() => {
  if (!changes.selectedFile) return "未选择文件";
  return `${changes.selectedSide === "staged" ? "暂存区" : "工作区"} · ${changes.selectedFile}`;
});
const canCommit = computed(() =>
  Boolean(commit.message.trim() && ((counts.value?.staged ?? 0) > 0 || (commit.amend && branch.value?.head))),
);
const selectedCommitTitle = computed(() => {
  if (!history.details) return "未选择提交";
  return `${history.details.commit.shortOid} · ${history.details.commit.summary}`;
});
const commitDiffLines = computed(() =>
  (history.details?.diff ?? "").split("\n").map((content, index) => ({
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
);
const selectableBranchTargets = computed(() =>
  (branches.list?.branches ?? []).filter((item) => !item.current).map((item) => item.name),
);
const allRefTargets = computed(() => [
  ...(branches.list?.branches ?? []).map((item) => item.name),
  ...(branches.list?.tags ?? []).map((item) => item.name),
]);
const conflictedFiles = computed(() => changes.files.filter((file) => file.conflicted));
const canSkipOperation = computed(() =>
  ["rebase", "cherry-pick", "revert"].includes(operations.activeOperation),
);
const resultHasConflictMarkers = computed(
  () =>
    operations.resultDraft.includes("<<<<<<< ") ||
    operations.resultDraft.includes("=======") ||
    operations.resultDraft.includes(">>>>>>> "),
);
const effectiveTheme = computed<Exclude<ThemeMode, "system">>(() => {
  if (settings.themeMode === "system") {
    return systemPrefersDark.value ? "dark" : "light";
  }
  return settings.themeMode;
});
const panelLabels: Record<LayoutPanelKey, string> = {
  project: "项目栏",
  repo: "仓库上下文",
  changes: "工作区上下文",
};
const workspaceGridStyle = computed(() => {
  const columns: string[] = [];

  if (settings.panelVisibility.project) {
    columns.push(`${settings.panelWidths.project}px`, "6px");
  }

  if (repos.current) {
    columns.push("68px");
  }

  if (repos.current && usesRepositoryContext.value && settings.panelVisibility.repo) {
    columns.push(`${settings.panelWidths.repo}px`, "6px");
  }

  if (repos.current && usesWorkbenchContext.value && settings.panelVisibility.changes) {
    columns.push(`${settings.panelWidths.changes}px`, "6px");
  }

  columns.push("minmax(0, 1fr)");
  return {
    gridTemplateColumns: columns.join(" "),
  };
});
const statusKindLabels: Record<string, string> = {
  added: "新增",
  modified: "修改",
  deleted: "删除",
  renamed: "重命名",
  typechange: "类型变更",
  conflicted: "冲突",
  ignored: "忽略",
  unknown: "未知",
};
const operationKindLabels: Record<string, string> = {
  merge: "合并",
  rebase: "变基",
  "cherry-pick": "挑选提交",
  revert: "反向提交",
};
const resetModeLabels: Record<string, string> = {
  soft: "软重置",
  mixed: "混合重置",
  hard: "硬重置",
};
const hostedRemoteLinks = computed(() =>
  (repos.current?.remotes ?? [])
    .map((item) => {
      const url = item.url || item.pushUrl || "";
      const parsed = parseHostedRemote(url);
      if (!parsed) return null;
      return {
        name: item.name,
        ...parsed,
      };
    })
    .filter((item): item is { name: string; provider: string; repo: string; webUrl: string; compareUrl: string } =>
      Boolean(item),
    ),
);

watch(
  () => [changes.selectedFile, changes.selectedSide],
  () => {
    diff.loadSelected().catch(() => undefined);
  },
);

watch(
  effectiveTheme,
  (theme) => {
    if (typeof document === "undefined") return;
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  },
  { immediate: true },
);

watch(
  () => [
    remote.autoFetchEnabled,
    remote.autoFetchIntervalMinutes,
    remote.autoFetchAllRepositories,
    repos.items.length,
  ],
  () => {
    scheduleAutoFetch();
  },
);

watch(workbenchMode, (mode) => {
  if (mode === "advanced") {
    loadAdvancedSnapshots().catch(() => undefined);
  }
});

onMounted(() => {
  if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const updateSystemTheme = () => {
      systemPrefersDark.value = mediaQuery.matches;
    };

    updateSystemTheme();
    mediaQuery.addEventListener("change", updateSystemTheme);
    stopSystemThemeWatch = () => mediaQuery.removeEventListener("change", updateSystemTheme);
  }

  if (repos.current) {
    loadCurrentRepository().catch(() => undefined);
  }
  scheduleAutoFetch();
});

onUnmounted(() => {
  stopSystemThemeWatch?.();
  clearAutoFetchTimer();
});

function normalizeSelectedPaths(selected: string | string[] | null) {
  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}

function parseHostedRemote(rawUrl: string) {
  const value = rawUrl.trim().replace(/\.git$/, "");
  if (!value) return null;

  const sshLike = value.match(/^git@([^:]+):(.+)$/);
  const normalized = sshLike ? `ssh://git@${sshLike[1]}/${sshLike[2]}` : value;
  try {
    const url = new URL(normalized);
    const host = url.hostname.toLowerCase();
    if (!["github.com", "gitlab.com", "bitbucket.org"].includes(host)) return null;
    const repo = url.pathname.replace(/^\/+/, "").replace(/\.git$/, "");
    const provider =
      host === "github.com" ? "GitHub" : host === "gitlab.com" ? "GitLab" : "Bitbucket";
    const webUrl = `https://${host}/${repo}`;
    return {
      provider,
      repo,
      webUrl,
      compareUrl: `${webUrl}/compare`,
    };
  } catch {
    return null;
  }
}

async function chooseRepository() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "添加 Git 仓库",
  });

  const paths = normalizeSelectedPaths(selected);
  if (paths.length === 0) return;

  const opened = await repos.openMany(paths);
  if (opened.length > 0) {
    await loadCurrentRepository();
  }
}

async function cloneRepositoryFromInput() {
  const repo = await advanced.cloneInto();
  if (!repo) return;
  repos.setCurrent(repo);
  advanced.cloneUrl = "";
  advanced.cloneDirectory = "";
  await loadCurrentRepository();
}

async function initRepositoryFromInput() {
  const repo = await advanced.initAt();
  if (!repo) return;
  repos.setCurrent(repo);
  await loadCurrentRepository();
}

async function switchRepository(path: string) {
  if (repos.path === path) return;
  await repos.select(path);
  await loadCurrentRepository();
}

async function removeRepository(path: string) {
  const wasCurrent = repos.path === path;
  repos.remove(path);
  if (!wasCurrent) return;

  if (repos.current) {
    await loadCurrentRepository();
  } else {
    clearProjectView();
  }
}

async function loadCurrentRepository() {
  clearProjectView();
  if (!repos.current) return;
  changelists.loadForCurrentRepository();
  syncSelectedRemote(true);
  await changes.refresh();
  changelists.pruneMissingPaths(changes.files.map((file) => file.path));
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  syncOperationTargets();
  syncSelectedRemote(true);
  pickFirstAvailable(settings.selectedSide);
  await diff.loadSelected();
}

function clearProjectView() {
  advanced.resetForRepositorySwitch();
  branches.resetForRepositorySwitch();
  changelists.resetForRepositorySwitch();
  changes.resetForRepositorySwitch();
  history.resetForRepositorySwitch();
  operations.resetForRepositorySwitch();
  diff.current = null;
  diff.error = "";
  remote.error = "";
  remote.notice = "";
  commit.error = "";
  commit.lastCommit = "";
}

function syncSelectedRemote(forceTarget = false) {
  const names = repos.current?.remotes.map((item) => item.name) ?? [];
  if (!names.includes(remote.selectedRemote)) {
    remote.selectedRemote = names[0] ?? "origin";
  }
  remote.syncTargetFromBranch(forceTarget);
  remote.syncDraftFromSelected();
}

function syncOperationTargets() {
  const targets = selectableBranchTargets.value;
  if (!targets.includes(operations.mergeTarget)) {
    operations.mergeTarget = targets[0] ?? "";
  }
  if (!targets.includes(operations.rebaseTarget)) {
    operations.rebaseTarget = targets[0] ?? "";
  }
  const refs = allRefTargets.value;
  const current = branch.value?.currentBranch ?? "HEAD";
  if (!refs.includes(advanced.compareLeft) && advanced.compareLeft !== "HEAD") {
    advanced.compareLeft = current;
  }
  if (!refs.includes(advanced.compareRight) && advanced.compareRight !== "HEAD") {
    advanced.compareRight = targets[0] ?? "HEAD";
  }
  if (!branches.localBranches.some((item) => item.name === advanced.branchRenameFrom)) {
    advanced.branchRenameFrom = branches.localBranches.find((item) => !item.current)?.name ?? "";
  }
}

async function refreshAll() {
  await changes.refresh();
  changelists.pruneMissingPaths(changes.files.map((file) => file.path));
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft();
  syncOperationTargets();
  syncSelectedRemote();
  pickFirstAvailable(settings.selectedSide);
  await diff.loadSelected();
}

async function reloadAfterGitOperation() {
  await changes.refresh();
  changelists.pruneMissingPaths(changes.files.map((file) => file.path));
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft();
  syncOperationTargets();
  syncSelectedRemote();
  pickFirstAvailable(settings.selectedSide);
  await diff.loadSelected();
  if (workbenchMode.value === "advanced") {
    await loadAdvancedSnapshots();
  }
}

async function loadAdvancedSnapshots() {
  if (!repos.current) return;
  await Promise.allSettled([
    advanced.refreshWorktrees(),
    advanced.refreshStashes(),
    advanced.refreshSubmodules(),
    advanced.refreshCommitMessages(),
  ]);
}

async function runRemoteAction(action: "fetch" | "pull" | "push") {
  await remote.run(action);
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft();
  syncOperationTargets();
  syncSelectedRemote();
}

async function fetchAllRepositories() {
  await remote.fetchAllRepositories();
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft();
  syncOperationTargets();
  syncSelectedRemote();
}

function clearAutoFetchTimer() {
  if (autoFetchTimer !== null && typeof window !== "undefined") {
    window.clearInterval(autoFetchTimer);
  }
  autoFetchTimer = null;
}

function scheduleAutoFetch() {
  clearAutoFetchTimer();
  if (!remote.autoFetchEnabled || typeof window === "undefined") return;
  const interval = Math.max(1, Number(remote.autoFetchIntervalMinutes) || 5) * 60 * 1000;
  autoFetchTimer = window.setInterval(() => {
    runAutoFetch().catch(() => undefined);
  }, interval);
}

async function runAutoFetch() {
  if (!remote.autoFetchEnabled || remote.loading) return;
  if (remote.autoFetchAllRepositories) {
    await fetchAllRepositories();
  } else if (repos.current) {
    await runRemoteAction("fetch");
  }
}

async function resolveRejectedPush(strategy: "merge" | "rebase") {
  const target = remote.lastRejectedTarget || remote.pushTargetRef();
  const remoteName = remote.selectedRemote || "origin";
  const upstream = `${remoteName}/${target}`;
  await remote.run("fetch");
  if (strategy === "merge") {
    operations.mergeTarget = upstream;
    await operations.merge();
  } else {
    operations.rebaseTarget = upstream;
    await operations.rebase();
  }
  await reloadAfterGitOperation();
}

function syncRemoteDraft() {
  remote.syncDraftFromSelected();
  remote.syncTargetFromBranch();
}

async function saveRemoteConfig() {
  await remote.saveRemote();
  syncSelectedRemote(true);
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft();
  syncOperationTargets();
}

async function deleteSelectedRemote() {
  if (!remote.selectedRemote) return;
  if (!window.confirm(`删除远程 ${remote.selectedRemote}？`)) return;
  await remote.deleteSelectedRemote();
  syncSelectedRemote(true);
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  branches.syncUpstreamDraft(true);
  syncOperationTargets();
}

async function unshallowCurrentRepository() {
  await advanced.unshallow(remote.selectedRemote || undefined);
  await reloadAfterGitOperation();
}

async function renameSelectedBranch() {
  if (!advanced.branchRenameFrom || !advanced.branchRenameTo.trim()) return;
  if (!window.confirm(`将分支 ${advanced.branchRenameFrom} 重命名为 ${advanced.branchRenameTo.trim()}？`)) return;
  await advanced.renameSelectedBranch();
  await loadCurrentRepository();
}

async function cleanupMergedBranches() {
  const target = branch.value?.currentBranch || "HEAD";
  if (!window.confirm(`清理已合并到 ${target} 的本地分支？`)) return;
  await advanced.cleanupMerged(target);
  await loadCurrentRepository();
}

async function runRefComparison() {
  await advanced.loadComparison();
}

async function generatePatch(staged = false) {
  await advanced.generatePatch(staged);
}

async function applyPatchDraft() {
  await advanced.applyPatchDraft();
  await reloadAfterGitOperation();
}

async function createWorktreeFromDraft() {
  await advanced.createWorktreeFromDraft();
  await loadAdvancedSnapshots();
}

async function removeWorktree(path: string) {
  if (!window.confirm(`移除工作树 ${path}？`)) return;
  await advanced.removeWorktreePath(path, true);
  await loadAdvancedSnapshots();
}

async function runStashAction(stashRef: string, action: "apply" | "pop" | "drop") {
  if (action === "drop" && !window.confirm(`删除 ${stashRef}？`)) return;
  await advanced.runStashAction(stashRef, action);
  await reloadAfterGitOperation();
}

async function clearAllStashes() {
  if (!window.confirm("清空所有贮藏记录？")) return;
  await advanced.clearAllStashes();
  await loadAdvancedSnapshots();
}

async function updateAllSubmodules() {
  await advanced.updateAllSubmodules();
  await reloadAfterGitOperation();
}

async function loadLfsStatus() {
  await advanced.refreshLfsStatus();
}

function selectSide(side: ChangeSide) {
  settings.setSide(side);
  changes.selectedSide = side;
  pickFirstAvailable(side);
}

function setIncludeIgnored(event: Event) {
  settings.setIncludeIgnored((event.target as HTMLInputElement).checked);
  refreshAll().catch(() => undefined);
}

function nudgePanelWidth(panel: LayoutPanelKey, delta: number) {
  settings.setPanelWidth(panel, settings.panelWidths[panel] + delta);
}

function resizeLabel(panel: LayoutPanelKey) {
  return `${panelLabels[panel]}宽度`;
}

function startPanelResize(panel: LayoutPanelKey, event: PointerEvent) {
  event.preventDefault();

  const startX = event.clientX;
  const startWidth = settings.panelWidths[panel];
  const target = event.currentTarget as HTMLElement;
  activeResizePanel.value = panel;
  target.setPointerCapture?.(event.pointerId);

  const handlePointerMove = (moveEvent: PointerEvent) => {
    settings.setPanelWidth(panel, startWidth + moveEvent.clientX - startX);
  };

  const stopResize = (upEvent: PointerEvent) => {
    target.releasePointerCapture?.(upEvent.pointerId);
    window.removeEventListener("pointermove", handlePointerMove);
    window.removeEventListener("pointerup", stopResize);
    window.removeEventListener("pointercancel", stopResize);
    activeResizePanel.value = null;
  };

  window.addEventListener("pointermove", handlePointerMove);
  window.addEventListener("pointerup", stopResize);
  window.addEventListener("pointercancel", stopResize);
}

function selectFile(file: ChangedFile, side: ChangeSide) {
  changes.selectFile(file, side);
}

function pickFirstAvailable(side: ChangeSide) {
  const preferred = changes.filesForSide(side);
  const fallbackSide: ChangeSide = side === "staged" ? "unstaged" : "staged";
  const fallback = changes.filesForSide(fallbackSide);
  const nextSide = preferred.length > 0 ? side : fallbackSide;
  const file = preferred[0] ?? fallback[0];
  if (file) {
    selectFile(file, nextSide);
  } else {
    changes.selectedFile = null;
    changes.selectedPaths = [];
    diff.current = null;
  }
}

async function runAndReload(action: () => Promise<unknown>) {
  await action();
  await diff.loadSelected();
}

async function discardSelected() {
  if (changes.activePaths.length === 0) return;
  if (!window.confirm("回滚选中的本地变更？")) return;
  await runAndReload(() => changes.discardSelected());
}

async function shelveSelected() {
  await runAndReload(() => changes.shelveSelected(shelveMessage.value));
  shelveMessage.value = "";
}

async function deleteShelfRecord(record: ShelfInfo) {
  if (!window.confirm(`删除搁置 ${record.message}？`)) return;
  await runAndReload(() => changes.deleteShelfRecord(record));
}

async function commitCurrent(pushAfter = false) {
  await commit.commit(pushAfter ? remote.selectedRemote || undefined : undefined);
  await Promise.all([branches.refresh(), history.refresh(), operations.refresh()]);
  syncOperationTargets();
}

function shortHash(hash?: string | null) {
  return hash ? hash.slice(0, 10) : "无提交";
}

function formatTime(seconds: number) {
  return new Date(seconds * 1000).toLocaleString();
}

function branchNameLabel(name?: string | null) {
  return name || "游离 HEAD";
}

function formatStatusKind(kind: string) {
  return kind
    .split("|")
    .map((part) => statusKindLabels[part] ?? part)
    .join(" / ");
}

function formatCommitTime(seconds: number) {
  return new Date(seconds * 1000).toLocaleString();
}

function formatRefName(ref: string) {
  return ref.replace(/^refs\/heads\//, "").replace(/^refs\/remotes\//, "").replace(/^refs\/tags\//, "");
}

function formatCommitFileStatus(file: CommitFileChange) {
  const code = file.status.charAt(0);
  const labels: Record<string, string> = {
    A: "新增",
    C: "复制",
    D: "删除",
    M: "修改",
    R: "重命名",
    T: "类型变更",
    U: "冲突",
  };
  return labels[code] ?? file.status;
}

function formatOperationName(name?: string | null) {
  return name ? (operationKindLabels[name] ?? name) : "冲突";
}

function formatResetMode(mode: string) {
  return resetModeLabels[mode] ?? mode;
}

function formatWorktreeLabel(item: { branch?: string | null; detached?: boolean }) {
  return item.branch || (item.detached ? "游离状态" : "工作树");
}

async function selectCommit(oid: string) {
  workbenchMode.value = "log";
  await history.select(oid);
  selectedCommitFilePaths.value = history.details?.files.map((file) => file.path) ?? [];
}

async function checkoutSelectedBranch(branch: BranchInfo) {
  if (branch.current) return;
  if (!changes.branch?.clean && !window.confirm("当前有未提交变更，仍然尝试切换分支？")) return;
  if (branch.branchType === "remote") {
    await branches.checkoutRemote(branch.name);
  } else {
    await branches.checkout(branch.name);
  }
  await loadCurrentRepository();
}

async function createBranchFromHead() {
  const name = newBranchName.value.trim();
  if (!name) return;
  await branches.create(name, true);
  newBranchName.value = "";
  await loadCurrentRepository();
}

async function deleteLocalBranch(branch: BranchInfo) {
  if (branch.current) return;
  if (!window.confirm(`删除本地分支 ${branch.name}？`)) return;
  await branches.delete(branch.name, false);
  await loadCurrentRepository();
}

async function deleteRemoteBranchItem(branch: BranchInfo) {
  if (branch.branchType !== "remote") return;
  if (!window.confirm(`删除远程分支 ${branch.name}？这会推送删除到远程仓库。`)) return;
  await branches.deleteRemote(branch.name);
  await loadCurrentRepository();
}

async function createTagFromInput() {
  const name = newTagName.value.trim();
  if (!name) return;
  await branches.createTag(
    name,
    newTagTarget.value.trim() || undefined,
    annotatedTag.value,
    tagMessage.value.trim() || undefined,
  );
  newTagName.value = "";
  newTagTarget.value = "";
  tagMessage.value = "";
  await loadCurrentRepository();
}

async function deleteLocalTag(tag: TagInfo) {
  if (!window.confirm(`删除本地标签 ${tag.name}？`)) return;
  await branches.deleteTag(tag.name);
  await loadCurrentRepository();
}

async function pushSelectedTag(tag: TagInfo) {
  await branches.pushTag(tag.name, remote.selectedRemote || undefined);
  await loadCurrentRepository();
}

async function deleteSelectedRemoteTag(tag: TagInfo) {
  const remoteName = remote.selectedRemote || "origin";
  if (!window.confirm(`删除 ${remoteName} 上的标签 ${tag.name}？`)) return;
  await branches.deleteRemoteTag(tag.name, remote.selectedRemote || undefined);
  await loadCurrentRepository();
}

async function setSelectedUpstream() {
  if (!branches.selectedLocalBranch || !branches.upstreamTarget) return;
  await branches.setUpstream(branches.selectedLocalBranch, branches.upstreamTarget);
  await loadCurrentRepository();
}

async function unsetSelectedUpstream() {
  if (!branches.selectedLocalBranch) return;
  await branches.setUpstream(branches.selectedLocalBranch);
  await loadCurrentRepository();
}

async function mergeSelectedTarget() {
  if (!operations.mergeTarget) return;
  if (!window.confirm(`将 ${operations.mergeTarget} 合并到当前分支？`)) return;
  await operations.merge();
  await reloadAfterGitOperation();
}

async function rebaseOntoSelectedTarget() {
  if (!operations.rebaseTarget) return;
  if (!window.confirm(`将当前分支变基到 ${operations.rebaseTarget}？`)) return;
  await operations.rebase();
  await reloadAfterGitOperation();
}

async function rebaseWithAdvancedOptions() {
  if (!operations.rebaseTarget && !operations.rebaseRoot) return;
  if (!window.confirm("按当前高级参数执行变基？")) return;
  await operations.rebaseWithAdvancedOptions();
  await reloadAfterGitOperation();
}

async function runOperationControl(action: "continue" | "abort" | "skip") {
  if (action === "abort" && !window.confirm("终止当前 Git 操作？")) return;
  await operations.control(action);
  await reloadAfterGitOperation();
}

async function cherryPickSelectedCommit() {
  const oid = history.selectedOid;
  if (!oid) return;
  if (!window.confirm(`cherry-pick ${oid.slice(0, 10)} 到当前分支？`)) return;
  await operations.cherryPick(oid);
  await reloadAfterGitOperation();
}

async function cherryPickSelectedFiles() {
  const oid = history.selectedOid;
  const files = selectedCommitFilePaths.value;
  if (!oid || files.length === 0) return;
  await operations.cherryPickFiles(oid, files);
  workbenchMode.value = "changes";
  await reloadAfterGitOperation();
}

async function revertSelectedCommit() {
  const oid = history.selectedOid;
  if (!oid) return;
  const mode = operations.revertNoCommit ? "不自动提交" : "自动创建反向提交";
  if (!window.confirm(`反向提交 ${oid.slice(0, 10)}？模式：${mode}`)) return;
  await operations.revert(oid);
  workbenchMode.value = "changes";
  await reloadAfterGitOperation();
}

async function resetToSelectedCommit() {
  const oid = history.selectedOid;
  if (!oid) return;
  const mode = operations.resetMode;
  const warning =
    mode === "hard"
      ? "硬重置会丢弃工作区和暂存区里未保存到目标提交的改动。"
      : mode === "soft"
        ? "软重置会保留变更为已暂存状态。"
        : "混合重置会保留变更到工作区。";
  if (!window.confirm(`将当前分支${formatResetMode(mode)}到 ${oid.slice(0, 10)}？\n${warning}`)) return;
  await operations.resetTo(oid);
  await reloadAfterGitOperation();
}

async function undoLastCommit() {
  const target = branch.value?.head?.slice(0, 10) ?? "HEAD";
  const mode = operations.undoKeepStaged ? "暂存区" : "工作区";
  if (!window.confirm(`撤销最后一次提交 ${target}？变更将保留在${mode}。`)) return;
  await operations.undoLastCommit();
  workbenchMode.value = "changes";
  await reloadAfterGitOperation();
}

async function checkoutSelectedRevision() {
  const oid = history.selectedOid;
  if (!oid) return;
  if (!window.confirm(`检出 ${oid.slice(0, 10)} 为游离 HEAD？`)) return;
  await advanced.checkoutDetached(oid);
  await loadCurrentRepository();
}

async function fixupSelectedCommit(squash = false) {
  const oid = history.selectedOid;
  if (!oid) return;
  const label = squash ? "压缩" : "修正";
  if (!window.confirm(`用当前暂存区创建${label}提交到 ${oid.slice(0, 10)}？`)) return;
  await advanced.fixupSelectedCommit(oid, squash);
  await reloadAfterGitOperation();
}

async function dropSelectedCommit() {
  const oid = history.selectedOid;
  if (!oid) return;
  if (!window.confirm(`从当前分支丢弃提交 ${oid.slice(0, 10)}？这会改写提交历史。`)) return;
  await advanced.dropSelectedCommit(oid);
  await reloadAfterGitOperation();
}

async function pushSelectedCommit() {
  const oid = history.selectedOid;
  if (!oid) return;
  await advanced.pushSelectedCommit(oid, remote.selectedRemote || undefined, remote.targetBranch || undefined);
}

async function loadSelectedFileHistory() {
  await advanced.loadFileHistory(changes.selectedFile);
  workbenchMode.value = "advanced";
}

async function loadSelectedFileBlame() {
  await advanced.loadBlame(changes.selectedFile);
  workbenchMode.value = "advanced";
}

function toggleCommitFile(path: string) {
  if (selectedCommitFilePaths.value.includes(path)) {
    selectedCommitFilePaths.value = selectedCommitFilePaths.value.filter((item) => item !== path);
  } else {
    selectedCommitFilePaths.value = [...selectedCommitFilePaths.value, path];
  }
}

function setConflictResultFromEvent(event: Event) {
  operations.setResultDraft((event.target as HTMLTextAreaElement).value);
}

async function selectConflict(path: string) {
  await operations.loadConflict(path);
}

async function resolveConflictFile(side: "ours" | "theirs") {
  await operations.resolveFile(side);
  await reloadAfterGitOperation();
}

async function resolveConflictBlock(index: number, side: "ours" | "base" | "theirs") {
  await operations.resolveBlock(index, side);
  await reloadAfterGitOperation();
}

async function markConflictResolved() {
  await operations.markResolved();
  await reloadAfterGitOperation();
}

async function saveConflictResult(markResolved = false) {
  await operations.saveResult(markResolved);
  if (markResolved) {
    await reloadAfterGitOperation();
  } else if (operations.selectedConflictPath) {
    await operations.loadConflict(operations.selectedConflictPath);
  }
}

async function applyAllConflictBlocks(side: "ours" | "base" | "theirs") {
  operations.useAllConflictBlocks(side);
  await saveConflictResult(false);
}

function createChangelist() {
  changelists.createList();
}

function moveSelectedToChangelist() {
  changelists.movePaths(changes.activePaths);
}

function setActiveChangelist(id: string) {
  changelists.setActive(id);
  pickFirstAvailable(settings.selectedSide);
}

function deleteChangelist(id: string) {
  changelists.deleteList(id);
  pickFirstAvailable(settings.selectedSide);
}
</script>

<template>
  <div class="app-shell" :data-theme="effectiveTheme">
    <AppTopbar
      :brand-subtitle="brandSubtitle"
      :has-repository="Boolean(repos.current)"
      :current-branch="branchNameLabel(branch?.currentBranch)"
      :ahead="branch?.ahead ?? 0"
      :behind="branch?.behind ?? 0"
      @add-repository="chooseRepository"
      @refresh="refreshAll"
    />

    <section
      class="workspace"
      :class="{ 'workspace-empty': !repos.current, 'is-resizing': activeResizePanel }"
      :style="workspaceGridStyle"
    >
      <ProjectPane
        v-if="settings.panelVisibility.project"
        @choose-repository="chooseRepository"
        @clone-repository="cloneRepositoryFromInput"
        @init-repository="initRepositoryFromInput"
        @remove-repository="removeRepository"
        @switch-repository="switchRepository"
      />

      <div
        v-if="settings.panelVisibility.project"
        class="pane-resizer"
        :class="{ active: activeResizePanel === 'project' }"
        role="separator"
        tabindex="0"
        aria-orientation="vertical"
        :aria-label="resizeLabel('project')"
        @pointerdown="startPanelResize('project', $event)"
        @keydown.left.prevent="nudgePanelWidth('project', -20)"
        @keydown.right.prevent="nudgePanelWidth('project', 20)"
      />

      <WorkbenchRail
        v-if="repos.current"
        v-model:mode="workbenchMode"
        :conflict-count="conflictedFiles.length"
      />

      <aside
        v-if="repos.current && usesRepositoryContext && settings.panelVisibility.repo"
        class="repo-pane context-pane"
      >
        <section class="pane-section">
          <div class="section-title">
            <GitBranch :size="16" />
            <span>仓库</span>
          </div>
          <div class="repo-name">{{ repos.name }}</div>
          <div class="repo-path">{{ repos.path }}</div>
          <div class="branch-line">
            <span>{{ branchNameLabel(branch?.currentBranch) }}</span>
            <small>{{ shortHash(branch?.head) }}</small>
          </div>
          <div class="sync-line">
            <span>领先 {{ branch?.ahead ?? 0 }}</span>
            <span>落后 {{ branch?.behind ?? 0 }}</span>
          </div>
        </section>

        <section v-if="workbenchMode === 'branches'" class="pane-section branch-manager">
          <div class="section-title">
            <GitBranch :size="16" />
            <span>分支</span>
          </div>
          <form class="branch-create" @submit.prevent="createBranchFromHead">
            <input v-model="newBranchName" placeholder="新分支名称" />
            <button class="icon-only-button" title="从当前 HEAD 创建并切换分支" :disabled="!newBranchName.trim()">
              <Plus :size="14" />
            </button>
          </form>

          <div v-if="branches.localBranches.length && branches.remoteBranches.length" class="upstream-manager">
            <select v-model="branches.selectedLocalBranch" class="remote-select" @change="branches.syncUpstreamDraft(true)">
              <option v-for="branchItem in branches.localBranches" :key="`local-${branchItem.fullName}`" :value="branchItem.name">
                {{ branchItem.name }}
              </option>
            </select>
            <select v-model="branches.upstreamTarget" class="remote-select">
              <option v-for="branchItem in branches.remoteBranches" :key="`upstream-${branchItem.fullName}`" :value="branchItem.name">
                {{ branchItem.name }}
              </option>
            </select>
            <div class="remote-editor-actions">
              <button class="icon-button" :disabled="branches.loading || !branches.upstreamTarget" @click="setSelectedUpstream">
                <Check :size="14" />
                <span>设置上游</span>
              </button>
              <button class="icon-button danger" :disabled="branches.loading || !branches.selectedLocalBranch" @click="unsetSelectedUpstream">
                <X :size="14" />
                <span>取消上游</span>
              </button>
            </div>
          </div>

          <div class="branch-list">
            <div class="branch-group-label">本地</div>
            <div
              v-for="branchItem in branches.sortedLocalBranches"
              :key="branchItem.fullName"
              class="branch-row"
              :class="{ active: branchItem.current }"
            >
              <button class="branch-checkout" :title="branchItem.fullName" @click="checkoutSelectedBranch(branchItem)">
                <span class="branch-dot" />
                <span class="branch-copy">
                  <strong>{{ branchItem.name }}</strong>
                  <small v-if="branchItem.upstream">
                    {{ formatRefName(branchItem.upstream) }} · 领先 {{ branchItem.ahead }} / 落后 {{ branchItem.behind }}
                  </small>
                  <small v-else>未设置上游</small>
                </span>
              </button>
              <button
                class="icon-only-button"
                :title="branches.isFavorite(branchItem.fullName) ? '取消收藏分支' : '收藏分支'"
                @click="branches.toggleFavorite(branchItem.fullName)"
              >
                <Star :size="13" :fill="branches.isFavorite(branchItem.fullName) ? 'currentColor' : 'none'" />
              </button>
              <button
                class="project-remove"
                title="删除本地分支"
                :disabled="branchItem.current"
                @click="deleteLocalBranch(branchItem)"
              >
                <Trash2 :size="13" />
              </button>
            </div>

            <div v-if="branches.remoteBranches.length" class="branch-group-label">远程</div>
            <div
              v-for="branchItem in branches.sortedRemoteBranches"
              :key="branchItem.fullName"
              class="remote-branch-item"
            >
              <button
                class="remote-branch-row"
                :title="`${branchItem.fullName} · 检出成本地跟踪分支`"
                @click="checkoutSelectedBranch(branchItem)"
              >
                {{ branchItem.name }}
              </button>
              <button
                class="icon-only-button"
                :title="branches.isFavorite(branchItem.fullName) ? '取消收藏分支' : '收藏分支'"
                @click="branches.toggleFavorite(branchItem.fullName)"
              >
                <Star :size="13" :fill="branches.isFavorite(branchItem.fullName) ? 'currentColor' : 'none'" />
              </button>
              <button
                class="project-remove"
                title="删除远程分支"
                @click="deleteRemoteBranchItem(branchItem)"
              >
                <Trash2 :size="13" />
              </button>
            </div>

            <div class="branch-group-label">标签</div>
            <form class="tag-create" @submit.prevent="createTagFromInput">
              <input v-model="newTagName" placeholder="新标签名称" />
              <input v-model="newTagTarget" placeholder="目标：HEAD 或提交，可空" />
              <label class="tag-option">
                <input v-model="annotatedTag" type="checkbox" />
                附注标签
              </label>
              <input v-if="annotatedTag" v-model="tagMessage" placeholder="标签说明，可空" />
              <button class="icon-button" :disabled="branches.loading || !newTagName.trim()">
                <Plus :size="14" />
                <span>创建标签</span>
              </button>
            </form>
            <div v-if="branches.list?.tags.length" class="tag-list">
              <div v-for="tag in branches.list.tags" :key="tag.name" class="tag-row">
                <span class="tag-copy">
                  <strong>{{ tag.name }}</strong>
                  <small>{{ shortHash(tag.target) }}</small>
                </span>
                <button class="icon-only-button" title="推送标签" :disabled="branches.loading || !remote.selectedRemote" @click="pushSelectedTag(tag)">
                  <Upload :size="13" />
                </button>
                <button class="icon-only-button danger" title="删除远程标签" :disabled="branches.loading || !remote.selectedRemote" @click="deleteSelectedRemoteTag(tag)">
                  <X :size="13" />
                </button>
                <button class="project-remove" title="删除本地标签" :disabled="branches.loading" @click="deleteLocalTag(tag)">
                  <Trash2 :size="13" />
                </button>
              </div>
            </div>
          </div>
        </section>

        <section v-if="workbenchMode === 'operations'" class="pane-section git-operation-panel">
          <div class="section-title">
            <GitBranch :size="16" />
            <span>合并 / 变基</span>
          </div>

          <div v-if="operations.state?.active" class="operation-state">
            <strong>{{ formatOperationName(operations.activeOperation) }}</strong>
            <span>{{ operations.conflictedPaths.length }} 个冲突文件</span>
            <div class="operation-actions">
              <button class="icon-button" :disabled="operations.loading" @click="runOperationControl('continue')">
                <Check :size="14" />
                <span>继续</span>
              </button>
              <button class="icon-button" :disabled="operations.loading || !canSkipOperation" @click="runOperationControl('skip')">
                <Minus :size="14" />
                <span>跳过</span>
              </button>
              <button class="icon-button danger" :disabled="operations.loading" @click="runOperationControl('abort')">
                <X :size="14" />
                <span>终止</span>
              </button>
            </div>
          </div>

          <div class="operation-form">
            <select v-model="operations.mergeTarget" class="remote-select">
              <option v-for="target in selectableBranchTargets" :key="`merge-${target}`" :value="target">
                {{ target }}
              </option>
            </select>
            <div class="operation-options">
              <label title="--no-ff"><input v-model="operations.mergeNoFf" type="checkbox" /> 禁用快进</label>
              <label title="--no-commit"><input v-model="operations.mergeNoCommit" type="checkbox" /> 不自动提交</label>
              <label title="--squash"><input v-model="operations.mergeSquash" type="checkbox" /> 压缩合并</label>
            </div>
            <button class="tool-button" :disabled="!operations.mergeTarget || operations.loading" @click="mergeSelectedTarget">
              <GitBranch :size="14" />
              <span>合并</span>
            </button>
          </div>

          <div class="operation-form">
            <select v-model="operations.rebaseTarget" class="remote-select">
              <option v-for="target in selectableBranchTargets" :key="`rebase-${target}`" :value="target">
                {{ target }}
              </option>
            </select>
            <div class="operation-options">
              <label title="--autostash"><input v-model="operations.rebaseAutostash" type="checkbox" /> 自动贮藏</label>
            </div>
            <button class="tool-button" :disabled="!operations.rebaseTarget || operations.loading" @click="rebaseOntoSelectedTarget">
              <RotateCcw :size="14" />
              <span>变基</span>
            </button>
          </div>

          <details class="advanced-rebase">
            <summary>高级变基</summary>
            <input v-model="operations.rebaseSourceBranch" list="git-refs" placeholder="源分支，可空" />
            <input v-model="operations.rebaseOnto" list="git-refs" placeholder="--onto 新基线，可空" />
            <div class="operation-options">
              <label title="--interactive"><input v-model="operations.rebaseInteractive" type="checkbox" /> 交互式</label>
              <label title="--autosquash"><input v-model="operations.rebaseAutosquash" type="checkbox" /> 自动压缩</label>
              <label title="--rebase-merges"><input v-model="operations.rebaseMerges" type="checkbox" /> 保留合并</label>
              <label title="--keep-empty"><input v-model="operations.rebaseKeepEmpty" type="checkbox" /> 保留空提交</label>
              <label title="--update-refs"><input v-model="operations.rebaseUpdateRefs" type="checkbox" /> 更新引用</label>
              <label title="--root"><input v-model="operations.rebaseRoot" type="checkbox" /> 从根提交</label>
            </div>
            <button
              class="tool-button"
              :disabled="operations.loading || (!operations.rebaseTarget && !operations.rebaseRoot)"
              @click="rebaseWithAdvancedOptions"
            >
              <RotateCcw :size="14" />
              <span>执行高级变基</span>
            </button>
          </details>
        </section>

        <section v-if="workbenchMode === 'remote'" class="pane-section">
          <div class="section-title">
            <Download :size="16" />
            <span>远程</span>
          </div>
          <select v-model="remote.selectedRemote" class="remote-select" @change="syncRemoteDraft">
            <option
              v-for="item in repos.current?.remotes ?? []"
              :key="item.name"
              :value="item.name"
            >
              {{ item.name }}
            </option>
            <option v-if="!(repos.current?.remotes.length)" value="origin">origin</option>
          </select>

          <div class="remote-editor">
            <input v-model="remote.remoteNameDraft" type="text" placeholder="远程名称，例如 origin" />
            <input v-model="remote.remoteUrlDraft" type="text" placeholder="获取地址" />
            <input v-model="remote.remotePushUrlDraft" type="text" placeholder="推送地址（可选）" />
            <div class="remote-editor-actions">
              <button
                class="icon-button"
                :disabled="remote.loading || !remote.remoteNameDraft.trim() || !remote.remoteUrlDraft.trim()"
                @click="saveRemoteConfig"
              >
                <Check :size="14" />
                <span>保存</span>
              </button>
              <button
                class="icon-button danger"
                :disabled="remote.loading || !(repos.current?.remotes.length)"
                @click="deleteSelectedRemote"
              >
                <Trash2 :size="14" />
                <span>删除</span>
              </button>
            </div>
          </div>

          <div class="push-options">
            <div class="operation-options">
              <label title="fetch --prune"><input v-model="remote.fetchPrune" type="checkbox" /> 获取时清理失效引用</label>
            </div>
            <label>
              <span>推送到分支</span>
              <input v-model="remote.targetBranch" type="text" placeholder="目标分支" />
            </label>
            <div class="operation-options">
              <label title="-u"><input v-model="remote.setUpstream" type="checkbox" /> 设置上游</label>
              <label title="--force-with-lease"><input v-model="remote.forceWithLease" type="checkbox" /> 安全强推</label>
              <label title="--tags"><input v-model="remote.pushTags" type="checkbox" /> 同步标签</label>
            </div>
            <div class="operation-options">
              <label><input v-model="remote.protectBranches" type="checkbox" /> 启用保护分支</label>
              <label><input v-model="remote.allowProtectedPush" type="checkbox" /> 允许保护分支推送</label>
            </div>
            <label>
              <span>保护分支规则</span>
              <input v-model="remote.protectedBranchPatterns" type="text" placeholder="main,master,production,release/*" />
            </label>
            <div class="operation-options">
              <label><input v-model="remote.autoFetchEnabled" type="checkbox" /> 自动获取</label>
              <label><input v-model="remote.autoFetchAllRepositories" type="checkbox" /> 所有仓库</label>
            </div>
            <label>
              <span>自动获取间隔（分钟）</span>
              <input v-model.number="remote.autoFetchIntervalMinutes" type="number" min="1" />
            </label>
          </div>

          <div v-if="remote.lastPushRejected" class="push-rejected-panel">
            <strong>推送被远程拒绝</strong>
            <span>可以先获取远程更新，再选择合并或变基到 {{ remote.lastRejectedTarget }}</span>
            <div class="remote-editor-actions">
              <button class="icon-button" :disabled="remote.loading || operations.loading" @click="resolveRejectedPush('merge')">
                <GitBranch :size="14" />
                <span>获取后合并</span>
              </button>
              <button class="icon-button" :disabled="remote.loading || operations.loading" @click="resolveRejectedPush('rebase')">
                <RotateCcw :size="14" />
                <span>获取后变基</span>
              </button>
            </div>
          </div>

          <div
            v-for="item in repos.current?.remotes ?? []"
            :key="item.name"
            class="remote-row"
          >
            <strong>{{ item.name }}</strong>
            <span>{{ item.url || "未配置地址" }}</span>
            <span v-if="item.pushUrl">推送地址：{{ item.pushUrl }}</span>
          </div>

          <div v-if="hostedRemoteLinks.length" class="hosted-panel">
            <div v-for="item in hostedRemoteLinks" :key="`${item.name}-${item.webUrl}`" class="hosted-row">
              <span>
                <strong>{{ item.provider }}</strong>
                <small>{{ item.repo }}</small>
              </span>
              <a :href="item.webUrl" target="_blank" rel="noreferrer">仓库</a>
              <a :href="item.compareUrl" target="_blank" rel="noreferrer">对比</a>
            </div>
          </div>
        </section>

        <section v-if="workbenchMode === 'operations'" class="pane-section shelves">
          <div class="section-title">
            <ArchiveRestore :size="16" />
            <span>搁置</span>
          </div>
          <div
            v-for="record in changes.shelves"
            :key="record.id ?? record.stashRef"
            class="shelf-row"
          >
            <button
              class="shelf-restore"
              :disabled="Boolean(record.appliedAt)"
              @click="changes.unshelveRecord(record)"
            >
              <span>{{ record.message }}</span>
              <small>{{ record.appliedAt ? "已恢复" : formatTime(record.createdAt) }}</small>
            </button>
            <button class="project-remove" title="删除搁置" @click="deleteShelfRecord(record)">
              <Trash2 :size="13" />
            </button>
          </div>
        </section>
      </aside>

      <div
        v-if="repos.current && usesRepositoryContext && settings.panelVisibility.repo"
        class="pane-resizer"
        :class="{ active: activeResizePanel === 'repo' }"
        role="separator"
        tabindex="0"
        aria-orientation="vertical"
        :aria-label="resizeLabel('repo')"
        @pointerdown="startPanelResize('repo', $event)"
        @keydown.left.prevent="nudgePanelWidth('repo', -20)"
        @keydown.right.prevent="nudgePanelWidth('repo', 20)"
      />

      <section v-if="!repos.current" class="empty-workbench">
        <div class="empty-panel">
          <ListChecks :size="40" />
          <h1>选择本地 Git 仓库</h1>
          <button class="tool-button primary large" @click="chooseRepository">
            <FolderOpen :size="18" />
            <span>添加项目</span>
          </button>
        </div>
      </section>

      <template v-else>
      <section v-if="settings.panelVisibility.changes && workbenchMode === 'changes'" class="changes-pane">
        <div class="segmented">
          <button
            :class="{ active: settings.selectedSide === 'unstaged' }"
            @click="selectSide('unstaged')"
          >
            工作区 {{ counts?.unstaged ?? 0 }}
          </button>
          <button
            :class="{ active: settings.selectedSide === 'staged' }"
            @click="selectSide('staged')"
          >
            暂存区 {{ counts?.staged ?? 0 }}
          </button>
        </div>

        <section class="changelist-panel">
          <div class="changelist-tabs">
            <button
              v-for="list in changelists.lists"
              :key="list.id"
              :class="{ active: changelists.selectedListId === list.id }"
              :title="list.description || list.name"
              @click="setActiveChangelist(list.id)"
            >
              <span>{{ list.name }}</span>
              <small>{{ list.id === 'default' ? changes.files.filter((file) => changelists.listForPath(file.path).id === list.id).length : list.paths.length }}</small>
            </button>
          </div>
          <form class="changelist-create" @submit.prevent="createChangelist">
            <input v-model="changelists.newName" placeholder="新变更列表" />
            <input v-model="changelists.newDescription" placeholder="描述，可空" />
            <button class="icon-only-button" title="创建变更列表" :disabled="!changelists.newName.trim()">
              <Plus :size="13" />
            </button>
          </form>
          <div class="changelist-actions">
            <select v-model="changelists.selectedListId" class="remote-select">
              <option v-for="list in changelists.lists" :key="`move-${list.id}`" :value="list.id">
                {{ list.name }}
              </option>
            </select>
            <button class="icon-button" :disabled="changes.activePaths.length === 0" @click="moveSelectedToChangelist">
              <ListChecks :size="14" />
              <span>移动</span>
            </button>
            <button
              class="icon-button danger"
              :disabled="changelists.selectedListId === 'default'"
              @click="deleteChangelist(changelists.selectedListId)"
            >
              <Trash2 :size="14" />
              <span>删除</span>
            </button>
          </div>
        </section>

        <section v-if="conflictedFiles.length" class="conflict-panel">
          <div class="conflict-header">
            <strong>冲突 {{ conflictedFiles.length }}</strong>
            <span>{{ operations.activeOperation ? formatOperationName(operations.activeOperation) : "待处理" }}</span>
          </div>

          <div class="conflict-file-tabs">
            <button
              v-for="file in conflictedFiles"
              :key="file.path"
              :class="{ active: operations.selectedConflictPath === file.path }"
              @click="selectConflict(file.path)"
            >
              {{ file.path }}
            </button>
          </div>

          <div v-if="operations.conflict" class="conflict-actions">
            <button class="icon-button" @click="resolveConflictFile('ours')">
              <Check :size="14" />
              <span>整文件当前</span>
            </button>
            <button class="icon-button" @click="resolveConflictFile('theirs')">
              <Download :size="14" />
              <span>整文件传入</span>
            </button>
            <button class="icon-button" @click="markConflictResolved">
              <ListChecks :size="14" />
              <span>标记解决</span>
            </button>
            <button class="icon-button" @click="applyAllConflictBlocks('ours')">
              <Check :size="14" />
              <span>全部当前</span>
            </button>
            <button class="icon-button" @click="applyAllConflictBlocks('theirs')">
              <Download :size="14" />
              <span>全部传入</span>
            </button>
          </div>

          <div v-if="operations.conflict?.blocks.length" class="conflict-blocks">
            <div v-for="block in operations.conflict.blocks" :key="block.index" class="conflict-block">
              <div class="conflict-block-title">冲突块 {{ block.index + 1 }}</div>
              <div class="conflict-block-actions">
                <button class="mini-button" @click="resolveConflictBlock(block.index, 'ours')">当前</button>
                <button v-if="block.base" class="mini-button" @click="resolveConflictBlock(block.index, 'base')">基线</button>
                <button class="mini-button" @click="resolveConflictBlock(block.index, 'theirs')">传入</button>
              </div>
            </div>
          </div>
        </section>

        <div class="file-actions">
          <button
            class="icon-button"
            title="暂存选中文件"
            :disabled="changes.activePaths.length === 0 || settings.selectedSide === 'staged'"
            @click="runAndReload(() => changes.stageSelected())"
          >
            <Check :size="15" />
            <span>暂存</span>
          </button>
          <button
            class="icon-button"
            title="取消暂存"
            :disabled="changes.activePaths.length === 0 || settings.selectedSide === 'unstaged'"
            @click="runAndReload(() => changes.unstageSelected())"
          >
            <Minus :size="15" />
            <span>移出</span>
          </button>
          <button
            class="icon-button danger"
            title="回滚变更"
            :disabled="changes.activePaths.length === 0"
            @click="discardSelected"
          >
            <Trash2 :size="15" />
            <span>回滚</span>
          </button>
          <button
            class="icon-button"
            title="查看文件历史"
            :disabled="!changes.selectedFile"
            @click="loadSelectedFileHistory"
          >
            <GitCommitVertical :size="15" />
            <span>历史</span>
          </button>
          <button
            class="icon-button"
            title="查看文件追溯"
            :disabled="!changes.selectedFile"
            @click="loadSelectedFileBlame"
          >
            <ListChecks :size="15" />
            <span>追溯</span>
          </button>
        </div>

        <div class="file-list">
          <button
            v-for="file in activeFiles"
            :key="`${settings.selectedSide}-${file.path}`"
            class="file-row"
            :class="{ active: changes.selectedFile === file.path }"
            @click="selectFile(file, settings.selectedSide)"
          >
            <input
              type="checkbox"
              :checked="changes.selectedPaths.includes(file.path)"
              @click.stop
              @change="changes.togglePath(file.path)"
            />
            <span class="status-dot" :class="file.kind.split('|')[0]" />
            <span class="file-main">
              <strong>{{ file.path }}</strong>
              <small v-if="file.oldPath">{{ file.oldPath }}</small>
            </span>
            <span class="kind-badge">{{ formatStatusKind(file.kind) }}</span>
          </button>
        </div>

        <div class="shelve-box">
          <input v-model="shelveMessage" placeholder="搁置说明" />
          <button
            class="icon-button"
            :disabled="changes.activePaths.length === 0"
            title="搁置选中变更"
            @click="shelveSelected"
          >
            <Archive :size="15" />
            <span>搁置</span>
          </button>
        </div>

        <form class="commit-box" @submit.prevent="commitCurrent(false)">
          <div class="commit-title">
            <GitCommitVertical :size="16" />
            <span>提交</span>
          </div>
          <textarea v-model="commit.message" rows="5" placeholder="提交信息" />
          <div class="commit-options">
            <label title="替换上一次提交">
              <input v-model="commit.amend" type="checkbox" />
              <span>修正上次提交</span>
            </label>
            <label title="追加提交签署行">
              <input v-model="commit.signOff" type="checkbox" />
              <span>追加签署</span>
            </label>
            <label title="使用系统 Git 执行 GPG 签名提交">
              <input v-model="commit.gpgSign" type="checkbox" />
              <span>GPG 签名</span>
            </label>
          </div>
          <input
            v-model="commit.author"
            class="commit-author"
            placeholder="覆盖作者：姓名 <email@example.com>"
          />
          <div class="commit-actions">
            <button class="commit-button" :disabled="!canCommit || commit.loading">
              {{ commit.amend ? "修正提交" : "提交" }} {{ counts?.staged ?? 0 }} 个文件
            </button>
            <button
              class="commit-button secondary"
              type="button"
              :disabled="!canCommit || commit.loading || !remote.selectedRemote"
              @click="commitCurrent(true)"
            >
              提交并推送
            </button>
          </div>
        </form>
      </section>

      <section v-else-if="settings.panelVisibility.changes && workbenchMode === 'log'" class="history-pane">
        <div class="history-header">
          <div class="section-title">
            <GitCommitVertical :size="16" />
            <span>提交日志</span>
          </div>
          <span>{{ history.commits.length }} 条</span>
        </div>

        <div class="log-filter-panel">
          <label>
            <Search :size="13" />
            <input v-model="history.query" placeholder="搜索提交、哈希、引用" @keydown.enter="history.refresh" />
          </label>
          <select v-model="history.branchFilter" class="remote-select" @change="history.refresh">
            <option value="">全部引用</option>
            <option v-for="target in allRefTargets" :key="`log-filter-${target}`" :value="target">
              {{ target }}
            </option>
          </select>
          <input v-model="history.authorFilter" placeholder="作者" @keydown.enter="history.refresh" />
          <input v-model="history.pathFilter" placeholder="路径" @keydown.enter="history.refresh" />
          <button class="icon-button" :disabled="history.loading" @click="history.refresh">
            <RefreshCw :size="14" />
            <span>过滤</span>
          </button>
        </div>

        <div class="commit-list">
          <button
            v-for="item in history.commits"
            :key="item.oid"
            class="commit-row"
            :class="{ active: history.selectedOid === item.oid }"
            @click="selectCommit(item.oid)"
          >
            <span class="commit-node" />
            <span class="commit-copy">
              <strong>{{ item.summary }}</strong>
              <small>{{ item.authorName }} · {{ formatCommitTime(item.authorTime) }}</small>
              <span v-if="item.refs.length" class="commit-refs">
                <em v-for="refName in item.refs" :key="refName">{{ formatRefName(refName) }}</em>
              </span>
            </span>
            <code>{{ item.shortOid }}</code>
          </button>
        </div>
      </section>

      <section v-else-if="settings.panelVisibility.changes && workbenchMode === 'advanced'" class="advanced-sidebar">
        <div class="history-header">
          <div class="section-title">
            <GitBranch :size="16" />
            <span>高级工具</span>
          </div>
          <button class="icon-only-button" title="刷新高级状态" @click="loadAdvancedSnapshots">
            <RefreshCw :size="14" />
          </button>
        </div>
        <div class="advanced-nav">
          <button @click="advanced.refreshWorktrees">工作树 {{ advanced.worktrees.length }}</button>
          <button @click="advanced.refreshStashes">贮藏 {{ advanced.stashes.length }}</button>
          <button @click="advanced.refreshSubmodules">子模块 {{ advanced.submodules.length }}</button>
          <button @click="advanced.refreshCommitMessages">提交信息 {{ advanced.commitMessages.length }}</button>
        </div>
      </section>

      <div
        v-if="settings.panelVisibility.changes && usesWorkbenchContext"
        class="pane-resizer"
        :class="{ active: activeResizePanel === 'changes' }"
        role="separator"
        tabindex="0"
        aria-orientation="vertical"
        :aria-label="resizeLabel('changes')"
        @pointerdown="startPanelResize('changes', $event)"
        @keydown.left.prevent="nudgePanelWidth('changes', -20)"
        @keydown.right.prevent="nudgePanelWidth('changes', 20)"
      />

      <main class="diff-pane">
        <template v-if="operations.conflict">
        <div class="diff-header merge-header">
          <div>
            <span class="eyebrow">三方合并</span>
            <h2>{{ operations.conflict.path }}</h2>
          </div>
          <div class="merge-header-actions">
            <button class="tool-button" @click="operations.useResultSource('ours')">
              <Check :size="14" />
              <span>结果=当前</span>
            </button>
            <button class="tool-button" :disabled="!operations.conflict.base" @click="operations.useResultSource('base')">
              <ListChecks :size="14" />
              <span>结果=基线</span>
            </button>
            <button class="tool-button" @click="operations.useResultSource('theirs')">
              <Download :size="14" />
              <span>结果=传入</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div class="merge-editor-toolbar">
          <span :class="{ warning: resultHasConflictMarkers }">
            {{ resultHasConflictMarkers ? "结果仍包含冲突标记" : "结果可标记为解决" }}
          </span>
          <div class="merge-save-actions">
            <button class="tool-button" :disabled="operations.loading || !operations.resultDirty" @click="saveConflictResult(false)">
              <RefreshCw :size="14" />
              <span>保存结果</span>
            </button>
            <button
              class="tool-button primary"
              :disabled="operations.loading || resultHasConflictMarkers"
              @click="saveConflictResult(true)"
            >
              <Check :size="14" />
              <span>保存并标记解决</span>
            </button>
          </div>
        </div>

        <div class="merge-editor">
          <section class="merge-column">
            <div class="merge-column-title">
              <strong>当前</strong>
              <span>当前版本</span>
            </div>
            <pre>{{ operations.conflict.ours || "" }}</pre>
          </section>

          <section class="merge-column result">
            <div class="merge-column-title">
              <strong>结果</strong>
              <span>{{ operations.resultDirty ? "已修改" : "未修改" }}</span>
            </div>
            <textarea
              :value="operations.resultDraft"
              spellcheck="false"
              @input="setConflictResultFromEvent"
            />
          </section>

          <section class="merge-column">
            <div class="merge-column-title">
              <strong>传入</strong>
              <span>传入版本</span>
            </div>
            <pre>{{ operations.conflict.theirs || "" }}</pre>
          </section>
        </div>

        <details v-if="operations.conflict.base" class="merge-base-panel">
          <summary>基线版本</summary>
          <pre>{{ operations.conflict.base }}</pre>
        </details>
        </template>

        <template v-else-if="workbenchMode === 'changes'">
        <div class="diff-header">
          <div>
            <span class="eyebrow">差异</span>
            <h2>{{ selectedDiffTitle }}</h2>
          </div>
          <label class="toggle-row">
            <input
              :checked="settings.includeIgnored"
              type="checkbox"
              @change="setIncludeIgnored"
            />
            <span>显示忽略文件</span>
          </label>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div v-if="diff.current?.hunks.length" class="hunk-strip">
          <button
            v-for="hunk in diff.current.hunks"
            :key="hunk.index"
            class="hunk-button"
            @click="diff.applyHunk(hunk.index)"
          >
            <Check v-if="changes.selectedSide === 'unstaged'" :size="14" />
            <Minus v-else :size="14" />
            <span>{{ changes.selectedSide === "unstaged" ? "暂存此块" : "取消此块" }}</span>
            <small>{{ hunk.header }}</small>
          </button>
        </div>

        <div class="diff-scroller">
          <div v-if="diff.loading" class="diff-empty">加载中</div>
          <div v-else-if="!diff.current || !diff.current.text" class="diff-empty">没有差异</div>
          <pre v-else class="diff-lines"><code
            v-for="line in diff.lines"
            :key="line.index"
            class="diff-line"
            :class="line.type"
          ><span class="line-number">{{ line.index + 1 }}</span><span class="line-content">{{ line.content || " " }}</span>
</code></pre>
        </div>
        </template>

        <template v-else-if="workbenchMode === 'advanced'">
        <div class="diff-header">
          <div>
            <span class="eyebrow">高级</span>
            <h2>Git 工具箱</h2>
          </div>
          <div class="log-actions">
            <button class="tool-button" :disabled="advanced.loading" @click="loadAdvancedSnapshots">
              <RefreshCw :size="14" />
              <span>刷新</span>
            </button>
            <button class="tool-button" :disabled="advanced.loading" @click="unshallowCurrentRepository">
              <Download :size="14" />
              <span>补全历史</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div class="advanced-workbench">
          <section class="advanced-card">
            <div class="section-title">
              <GitBranch :size="16" />
              <span>分支增强</span>
            </div>
            <div class="advanced-form two">
              <select v-model="advanced.branchRenameFrom">
                <option v-for="branchItem in branches.localBranches" :key="branchItem.name" :value="branchItem.name">
                  {{ branchItem.name }}
                </option>
              </select>
              <input v-model="advanced.branchRenameTo" placeholder="新分支名称" />
              <button class="icon-button" :disabled="advanced.loading || !advanced.branchRenameFrom || !advanced.branchRenameTo.trim()" @click="renameSelectedBranch">
                <Check :size="14" />
                <span>重命名</span>
              </button>
              <button class="icon-button danger" :disabled="advanced.loading" @click="cleanupMergedBranches">
                <Trash2 :size="14" />
                <span>清理已合并</span>
              </button>
            </div>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <Columns3 :size="16" />
              <span>引用对比</span>
            </div>
            <div class="advanced-form three">
              <input v-model="advanced.compareLeft" list="git-refs" placeholder="左侧引用，例如 main" />
              <input v-model="advanced.compareRight" list="git-refs" placeholder="右侧引用，例如 feature" />
              <button class="icon-button" :disabled="advanced.loading || !advanced.compareLeft.trim() || !advanced.compareRight.trim()" @click="runRefComparison">
                <Columns3 :size="14" />
                <span>对比</span>
              </button>
            </div>
            <datalist id="git-refs">
              <option value="HEAD" />
              <option v-for="target in allRefTargets" :key="target" :value="target" />
            </datalist>
            <div v-if="advanced.comparison" class="comparison-summary">
              <span>{{ advanced.comparison.commits.filter((item) => item.side === "left").length }} 仅左侧</span>
              <span>{{ advanced.comparison.commits.filter((item) => item.side === "right").length }} 仅右侧</span>
              <span>{{ advanced.comparison.files.length }} 个文件</span>
            </div>
          </section>

          <section class="advanced-card wide">
            <div class="section-title">
              <ListChecks :size="16" />
              <span>补丁</span>
            </div>
            <div class="advanced-actions">
              <button class="icon-button" :disabled="advanced.loading" @click="generatePatch(false)">
                <Download :size="14" />
                <span>工作区补丁</span>
              </button>
              <button class="icon-button" :disabled="advanced.loading" @click="generatePatch(true)">
                <Download :size="14" />
                <span>暂存区补丁</span>
              </button>
              <label class="log-option" title="--index"><input v-model="advanced.applyPatchToIndex" type="checkbox" /> 更新索引</label>
              <label class="log-option" title="--3way"><input v-model="advanced.applyPatchThreeWay" type="checkbox" /> 三方应用</label>
              <button class="icon-button" :disabled="advanced.loading || !advanced.patchDraft.trim()" @click="applyPatchDraft">
                <Upload :size="14" />
                <span>应用补丁</span>
              </button>
            </div>
            <textarea v-model="advanced.generatedPatch" class="advanced-textarea" readonly placeholder="生成的补丁会出现在这里" />
            <textarea v-model="advanced.patchDraft" class="advanced-textarea" placeholder="粘贴补丁后应用" />
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <GitCommitVertical :size="16" />
              <span>文件历史</span>
            </div>
            <div class="advanced-actions">
              <button class="icon-button" :disabled="!changes.selectedFile || advanced.loading" @click="advanced.loadFileHistory(changes.selectedFile)">
                <GitCommitVertical :size="14" />
                <span>读取历史</span>
              </button>
              <button class="icon-button" :disabled="!changes.selectedFile || advanced.loading" @click="advanced.loadBlame(changes.selectedFile)">
                <ListChecks :size="14" />
                <span>读取追溯</span>
              </button>
            </div>
            <div class="advanced-list compact">
              <div v-for="item in advanced.fileHistory" :key="item.oid" class="advanced-row">
                <strong>{{ item.summary }}</strong>
                <small>{{ item.shortOid }} · {{ item.authorName }} · {{ formatCommitTime(item.authorTime) }}</small>
              </div>
            </div>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <ListChecks :size="16" />
              <span>文件追溯</span>
            </div>
            <div class="advanced-list blame-list">
              <div v-for="line in advanced.blame.slice(0, 80)" :key="`${line.lineNumber}-${line.oid}`" class="blame-row">
                <code>{{ line.lineNumber }}</code>
                <span>{{ line.shortOid }}</span>
                <strong>{{ line.authorName }}</strong>
                <pre>{{ line.content }}</pre>
              </div>
            </div>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <GitBranch :size="16" />
              <span>工作树</span>
            </div>
            <div class="advanced-form">
              <input v-model="advanced.worktreePath" placeholder="工作树目录" />
              <input v-model="advanced.worktreeBranch" placeholder="新分支，可空" />
              <input v-model="advanced.worktreeStartPoint" list="git-refs" placeholder="起点，可空" />
              <label class="log-option" title="--detach"><input v-model="advanced.worktreeDetach" type="checkbox" /> 游离状态</label>
              <button class="icon-button" :disabled="advanced.loading || !advanced.worktreePath.trim()" @click="createWorktreeFromDraft">
                <Plus :size="14" />
                <span>创建</span>
              </button>
            </div>
            <div class="advanced-list compact">
              <div v-for="item in advanced.worktrees" :key="item.path" class="advanced-row with-action">
                <span>
                  <strong>{{ formatWorktreeLabel(item) }}</strong>
                  <small>{{ item.path }}</small>
                </span>
                <button class="project-remove" title="移除工作树" @click="removeWorktree(item.path)">
                  <Trash2 :size="13" />
                </button>
              </div>
            </div>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <ArchiveRestore :size="16" />
              <span>贮藏</span>
            </div>
            <div class="advanced-actions">
              <button class="icon-button" :disabled="advanced.loading" @click="advanced.refreshStashes">
                <RefreshCw :size="14" />
                <span>刷新</span>
              </button>
              <button class="icon-button danger" :disabled="advanced.loading || !advanced.stashes.length" @click="clearAllStashes">
                <Trash2 :size="14" />
                <span>清空</span>
              </button>
            </div>
            <div class="advanced-list compact">
              <div v-for="item in advanced.stashes" :key="item.stashRef" class="stash-row">
                <span>
                  <strong>{{ item.stashRef }}</strong>
                  <small>{{ item.message }} · {{ formatTime(item.createdAt) }}</small>
                </span>
                <button class="mini-button" @click="runStashAction(item.stashRef, 'apply')">应用</button>
                <button class="mini-button" @click="runStashAction(item.stashRef, 'pop')">弹出</button>
                <button class="mini-button danger" @click="runStashAction(item.stashRef, 'drop')">删除</button>
              </div>
            </div>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <Columns3 :size="16" />
              <span>子模块 / LFS</span>
            </div>
            <div class="advanced-actions">
              <button class="icon-button" :disabled="advanced.loading" @click="updateAllSubmodules">
                <RefreshCw :size="14" />
                <span>更新子模块</span>
              </button>
              <button class="icon-button" :disabled="advanced.loading" @click="loadLfsStatus">
                <ListChecks :size="14" />
                <span>LFS 状态</span>
              </button>
            </div>
            <div class="advanced-list compact">
              <div v-for="item in advanced.submodules" :key="item.path" class="advanced-row">
                <strong>{{ item.path }}</strong>
                <small>{{ item.status }} · {{ item.oid.slice(0, 10) }} {{ item.branch || "" }}</small>
              </div>
            </div>
            <pre v-if="advanced.lfsOutput" class="advanced-output">{{ advanced.lfsOutput }}</pre>
          </section>

          <section class="advanced-card">
            <div class="section-title">
              <GitCommitVertical :size="16" />
              <span>提交信息历史</span>
            </div>
            <div class="advanced-list compact">
              <button v-for="message in advanced.commitMessages" :key="message" class="message-history-row" @click="commit.message = message">
                {{ message }}
              </button>
            </div>
          </section>
        </div>
        </template>

        <template v-else-if="workbenchMode === 'log'">
        <div class="diff-header">
          <div>
            <span class="eyebrow">提交详情</span>
            <h2>{{ selectedCommitTitle }}</h2>
          </div>
          <div class="log-actions">
            <button class="tool-button" :disabled="!history.selectedOid || operations.loading" @click="cherryPickSelectedCommit">
              <GitCommitVertical :size="14" />
              <span>挑选提交</span>
            </button>
            <button class="tool-button" :disabled="!history.selectedOid || advanced.loading" @click="checkoutSelectedRevision">
              <GitBranch :size="14" />
              <span>检出</span>
            </button>
            <button class="tool-button" :disabled="!history.selectedOid || advanced.loading" @click="fixupSelectedCommit(false)">
              <Plus :size="14" />
              <span>修正</span>
            </button>
            <button class="tool-button" :disabled="!history.selectedOid || advanced.loading" @click="fixupSelectedCommit(true)">
              <Plus :size="14" />
              <span>压缩</span>
            </button>
            <button
              class="tool-button"
              :disabled="!history.selectedOid || selectedCommitFilePaths.length === 0 || operations.loading"
              @click="cherryPickSelectedFiles"
            >
              <Download :size="14" />
              <span>应用文件</span>
            </button>
            <button class="tool-button" :disabled="!history.selectedOid || operations.loading" @click="revertSelectedCommit">
              <RotateCcw :size="14" />
              <span>反向提交</span>
            </button>
            <label class="log-option" title="只应用反向变更，不自动创建 revert 提交">
              <input v-model="operations.revertNoCommit" type="checkbox" />
              <span>不自动提交</span>
            </label>
            <select v-model="operations.resetMode" class="reset-select" title="重置模式">
              <option value="soft">软重置</option>
              <option value="mixed">混合重置</option>
              <option value="hard">硬重置</option>
            </select>
            <button class="tool-button danger" :disabled="!history.selectedOid || operations.loading" @click="resetToSelectedCommit">
              <Trash2 :size="14" />
              <span>重置</span>
            </button>
            <button class="tool-button danger" :disabled="!branch?.head || operations.loading" @click="undoLastCommit">
              <RotateCcw :size="14" />
              <span>撤销提交</span>
            </button>
            <button class="tool-button danger" :disabled="!history.selectedOid || advanced.loading" @click="dropSelectedCommit">
              <Trash2 :size="14" />
              <span>丢弃提交</span>
            </button>
            <button class="tool-button" :disabled="!history.selectedOid || advanced.loading || !remote.selectedRemote" @click="pushSelectedCommit">
              <Upload :size="14" />
              <span>推送提交</span>
            </button>
            <label class="log-option" title="撤销提交后保留为已暂存变更">
              <input v-model="operations.undoKeepStaged" type="checkbox" />
              <span>保留暂存</span>
            </label>
            <button class="tool-button" :disabled="history.loading" @click="history.refresh()">
              <RefreshCw :size="14" />
              <span>刷新日志</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div v-if="history.details" class="commit-detail-strip">
          <div>
            <span>作者</span>
            <strong>{{ history.details.commit.authorName }}</strong>
          </div>
          <div>
            <span>时间</span>
            <strong>{{ formatCommitTime(history.details.commit.authorTime) }}</strong>
          </div>
          <div>
            <span>父提交</span>
            <strong>{{ history.details.commit.parents.length || "无" }}</strong>
          </div>
        </div>

        <div v-if="history.details?.files.length" class="commit-files">
          <div
            v-for="file in history.details.files"
            :key="`${file.status}-${file.oldPath ?? ''}-${file.path}`"
            class="commit-file-row"
          >
            <input
              type="checkbox"
              :checked="selectedCommitFilePaths.includes(file.path)"
              @change="toggleCommitFile(file.path)"
            />
            <span class="kind-badge">{{ formatCommitFileStatus(file) }}</span>
            <strong>{{ file.path }}</strong>
            <small v-if="file.oldPath">{{ file.oldPath }}</small>
          </div>
        </div>

        <div class="diff-scroller">
          <div v-if="history.detailLoading || history.loading" class="diff-empty">加载中</div>
          <div v-else-if="!history.details" class="diff-empty">没有提交历史</div>
          <pre v-else class="diff-lines"><code
            v-for="line in commitDiffLines"
            :key="line.index"
            class="diff-line"
            :class="line.type"
          ><span class="line-number">{{ line.index + 1 }}</span><span class="line-content">{{ line.content || " " }}</span>
</code></pre>
        </div>
        </template>

        <template v-else-if="workbenchMode === 'branches'">
        <div class="diff-header">
          <div>
            <span class="eyebrow">分支</span>
            <h2>{{ branchNameLabel(branch?.currentBranch) }}</h2>
          </div>
          <div class="log-actions">
            <button class="tool-button" :disabled="branches.loading" @click="branches.refresh()">
              <RefreshCw :size="14" />
              <span>刷新分支</span>
            </button>
            <button class="tool-button" :disabled="!newBranchName.trim()" @click="createBranchFromHead">
              <Plus :size="14" />
              <span>创建分支</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div class="context-dashboard">
          <section class="dashboard-card">
            <div class="section-title">
              <GitBranch :size="16" />
              <span>当前分支</span>
            </div>
            <strong>{{ branchNameLabel(branch?.currentBranch) }}</strong>
            <small>{{ shortHash(branch?.head) || "HEAD" }}</small>
            <div class="metric-row">
              <span>领先 {{ branch?.ahead ?? 0 }}</span>
              <span>落后 {{ branch?.behind ?? 0 }}</span>
            </div>
          </section>

          <section class="dashboard-card">
            <div class="section-title">
              <ListChecks :size="16" />
              <span>引用</span>
            </div>
            <div class="metric-grid">
              <span><strong>{{ branches.localBranches.length }}</strong> 本地分支</span>
              <span><strong>{{ branches.remoteBranches.length }}</strong> 远程分支</span>
              <span><strong>{{ branches.list?.tags.length ?? 0 }}</strong> 标签</span>
              <span><strong>{{ branches.favoriteRefs.length }}</strong> 收藏</span>
            </div>
          </section>

          <section class="dashboard-card wide">
            <div class="section-title">
              <Star :size="16" />
              <span>收藏分支</span>
            </div>
            <div class="chip-list">
              <span v-for="name in branches.favoriteRefs" :key="name">{{ formatRefName(name) }}</span>
              <span v-if="branches.favoriteRefs.length === 0">无收藏分支</span>
            </div>
          </section>
        </div>
        </template>

        <template v-else-if="workbenchMode === 'remote'">
        <div class="diff-header">
          <div>
            <span class="eyebrow">远程</span>
            <h2>{{ remote.selectedRemote || "origin" }}</h2>
          </div>
          <div class="log-actions">
            <button class="tool-button" :disabled="!repos.current || remote.loading" @click="runRemoteAction('fetch')">
              <Download :size="14" />
              <span>获取</span>
            </button>
            <button class="tool-button" :disabled="repos.items.length === 0 || remote.loading" @click="fetchAllRepositories">
              <Download :size="14" />
              <span>全部获取</span>
            </button>
            <button class="tool-button" :disabled="!repos.current || remote.loading" @click="runRemoteAction('pull')">
              <RotateCcw :size="14" />
              <span>拉取</span>
            </button>
            <button class="tool-button primary" :disabled="!repos.current || remote.loading" @click="runRemoteAction('push')">
              <Upload :size="14" />
              <span>推送</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div class="context-dashboard">
          <section class="dashboard-card">
            <div class="section-title">
              <Download :size="16" />
              <span>同步状态</span>
            </div>
            <strong>{{ remote.selectedRemote || "origin" }}</strong>
            <small>{{ remote.targetBranch || branch?.currentBranch || "HEAD" }}</small>
            <div class="metric-row">
              <span>领先 {{ branch?.ahead ?? 0 }}</span>
              <span>落后 {{ branch?.behind ?? 0 }}</span>
            </div>
          </section>

          <section class="dashboard-card">
            <div class="section-title">
              <RefreshCw :size="16" />
              <span>自动获取</span>
            </div>
            <strong>{{ remote.autoFetchEnabled ? "已开启" : "已关闭" }}</strong>
            <small>{{ remote.autoFetchAllRepositories ? "所有仓库" : "当前仓库" }} · {{ remote.autoFetchIntervalMinutes }} 分钟</small>
          </section>

          <section class="dashboard-card wide">
            <div class="section-title">
              <Upload :size="16" />
              <span>远程地址</span>
            </div>
            <div class="remote-dashboard-list">
              <div v-for="item in repos.current?.remotes ?? []" :key="`dashboard-${item.name}`">
                <strong>{{ item.name }}</strong>
                <span>{{ item.url || "未配置地址" }}</span>
              </div>
            </div>
          </section>
        </div>
        </template>

        <template v-else>
        <div class="diff-header">
          <div>
            <span class="eyebrow">操作</span>
            <h2>{{ operations.state?.active ? formatOperationName(operations.activeOperation) : "合并 / 变基" }}</h2>
          </div>
          <div class="log-actions">
            <button class="tool-button" :disabled="operations.loading" @click="operations.refresh()">
              <RefreshCw :size="14" />
              <span>刷新操作</span>
            </button>
            <button class="tool-button" :disabled="operations.loading" @click="workbenchMode = 'changes'">
              <ListChecks :size="14" />
              <span>查看变更</span>
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>
        <div v-else-if="activeNotice" class="message notice">{{ activeNotice }}</div>

        <div class="context-dashboard">
          <section class="dashboard-card">
            <div class="section-title">
              <RotateCcw :size="16" />
              <span>当前操作</span>
            </div>
            <strong>{{ operations.state?.active ? formatOperationName(operations.activeOperation) : "无进行中操作" }}</strong>
            <small>{{ operations.conflictedPaths.length }} 个冲突文件</small>
            <div v-if="operations.state?.active" class="operation-actions">
              <button class="icon-button" :disabled="operations.loading" @click="runOperationControl('continue')">
                <Check :size="14" />
                <span>继续</span>
              </button>
              <button class="icon-button" :disabled="operations.loading || !canSkipOperation" @click="runOperationControl('skip')">
                <Minus :size="14" />
                <span>跳过</span>
              </button>
              <button class="icon-button danger" :disabled="operations.loading" @click="runOperationControl('abort')">
                <X :size="14" />
                <span>终止</span>
              </button>
            </div>
          </section>

          <section class="dashboard-card wide">
            <div class="section-title">
              <ListChecks :size="16" />
              <span>冲突文件</span>
            </div>
            <div class="chip-list">
              <button
                v-for="file in conflictedFiles"
                :key="`operation-conflict-${file.path}`"
                class="mini-button"
                @click="selectConflict(file.path)"
              >
                {{ file.path }}
              </button>
              <span v-if="conflictedFiles.length === 0">无冲突文件</span>
            </div>
          </section>
        </div>
        </template>
      </main>
      </template>
    </section>
  </div>
</template>

<style>
:root {
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #1d2428;
  background: #eef1ed;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-width: 960px;
  min-height: 100vh;
  overflow: hidden;
}

button,
input,
textarea,
select {
  font: inherit;
}

button {
  cursor: pointer;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.52;
}

.app-shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100vh;
  min-width: 0;
  background: #eef1ed;
}

.topbar {
  display: grid;
  grid-template-columns: minmax(360px, 520px) minmax(0, 1fr);
  align-items: center;
  gap: 12px;
  min-height: 58px;
  padding: 0 12px;
  overflow: hidden;
  border-bottom: 1px solid #cdd5d0;
  background: #fbfcfa;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.brand-mark {
  width: 32px;
  height: 32px;
  display: block;
  flex: 0 0 auto;
}

.brand-copy {
  display: grid;
  line-height: 1.2;
}

.brand-copy strong {
  font-size: 15px;
}

.brand-copy span {
  color: #65706a;
  font-size: 12px;
  max-width: 360px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-state {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  max-width: 250px;
  padding: 4px 8px;
  border: 1px solid #d2dad4;
  border-radius: 7px;
  color: #405047;
  background: #f5f8f5;
  font-size: 12px;
}

.topbar-state span,
.topbar-state small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-state span {
  font-weight: 700;
}

.topbar-state small {
  color: #6e7a73;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  min-width: 0;
  overflow-x: auto;
  padding: 6px 0;
  scrollbar-width: thin;
}

.toolbar::-webkit-scrollbar {
  height: 8px;
}

.toolbar::-webkit-scrollbar-thumb {
  border-radius: 8px;
  background: #c7d0ca;
}

.theme-switch {
  display: inline-grid;
  grid-template-columns: repeat(3, 76px);
  gap: 2px;
  padding: 3px;
  border: 1px solid #bdc8c1;
  border-radius: 8px;
  background: #edf1ec;
}

.theme-option {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 28px;
  border: 1px solid transparent;
  border-radius: 6px;
  color: #4c5a52;
  background: transparent;
  font-size: 12px;
}

.theme-option:hover {
  background: #ffffff;
}

.theme-option.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

.layout-menu {
  position: relative;
}

.layout-summary {
  list-style: none;
}

.layout-summary::-webkit-details-marker {
  display: none;
}

.layout-popover {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  z-index: 20;
  display: grid;
  gap: 6px;
  width: 178px;
  padding: 8px;
  border: 1px solid #bdc8c1;
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 0 16px 36px rgba(34, 48, 42, 0.18);
}

.layout-option,
.layout-reset {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 30px;
  padding: 0 8px;
  border-radius: 6px;
  color: #22302a;
  font-size: 13px;
}

.layout-option {
  cursor: pointer;
}

.layout-option input {
  width: 14px;
  height: 14px;
}

.layout-reset {
  width: 100%;
  border: 1px solid #bdc8c1;
  background: #f6f7f3;
}

.tool-button,
.icon-button,
.commit-button,
.hunk-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  min-height: 32px;
  min-width: 0;
  border: 1px solid #bdc8c1;
  border-radius: 7px;
  color: #22302a;
  background: #ffffff;
  white-space: nowrap;
}

.tool-button,
.icon-button {
  padding: 0 11px;
  font-size: 13px;
}

.tool-button.primary,
.commit-button {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

.tool-button.large {
  min-height: 40px;
  padding: 0 16px;
}

.icon-button.danger {
  border-color: #d9b4aa;
  color: #8a2e20;
}

.tool-button.danger {
  border-color: #d9b4aa;
  color: #8a2e20;
  background: #fff7f4;
}

.empty-workbench {
  display: grid;
  place-items: center;
}

.empty-panel {
  display: grid;
  justify-items: center;
  gap: 16px;
  color: #34413a;
}

.empty-panel h1 {
  margin: 0;
  font-size: 22px;
}

.workspace {
  display: grid;
  grid-template-columns: 220px 280px 390px minmax(0, 1fr);
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.workspace.is-resizing,
.workspace.is-resizing * {
  cursor: col-resize;
  user-select: none;
}

.workspace-empty {
  grid-template-columns: 220px minmax(0, 1fr);
}

.project-pane,
.repo-pane,
.history-pane,
.changes-pane,
.diff-pane {
  min-height: 0;
  min-width: 0;
  border-right: 1px solid #cdd5d0;
}

.pane-resizer {
  position: relative;
  min-height: 0;
  border-right: 1px solid #cdd5d0;
  background: #eef1ed;
  cursor: col-resize;
}

.pane-resizer::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 2px;
  width: 2px;
  background: transparent;
}

.pane-resizer:hover::before,
.pane-resizer.active::before,
.pane-resizer:focus-visible::before {
  background: #4c82d9;
}

.project-pane,
.repo-pane {
  overflow: auto;
  scrollbar-gutter: stable;
  background: #f6f7f3;
}

.workbench-rail {
  display: grid;
  align-content: start;
  gap: 6px;
  min-height: 0;
  padding: 8px 7px;
  border-right: 1px solid #cdd5d0;
  background: #eef1ed;
}

.rail-button {
  position: relative;
  display: grid;
  justify-items: center;
  gap: 4px;
  width: 54px;
  min-height: 54px;
  padding: 6px 4px;
  border: 1px solid transparent;
  border-radius: 8px;
  color: #526158;
  background: transparent;
  font-size: 11px;
}

.rail-button:hover {
  background: #ffffff;
}

.rail-button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

.rail-button small {
  position: absolute;
  top: 4px;
  right: 5px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  color: #ffffff;
  background: #b83e31;
  font-size: 10px;
  line-height: 16px;
}

.pane-section {
  padding: 12px;
  border-bottom: 1px solid #dce2dd;
}

.projects-section {
  padding: 12px;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.section-title,
.commit-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #4f5e56;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
}

.icon-only-button,
.project-remove {
  display: inline-grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 1px solid #bdc8c1;
  border-radius: 7px;
  color: #4f5e56;
  background: #ffffff;
}

.project-list {
  display: grid;
  gap: 6px;
  margin-top: 12px;
}

.project-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px;
  align-items: center;
  gap: 4px;
  border: 1px solid transparent;
  border-radius: 7px;
}

.project-row.active {
  border-color: #5b8fd7;
  background: #e8f0fb;
}

.project-switch {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  width: 100%;
  min-width: 0;
  min-height: 38px;
  padding: 7px 8px;
  border: 0;
  border-radius: 6px;
  color: #25312b;
  text-align: left;
  background: transparent;
}

.project-switch:hover {
  background: #edf1ec;
}

.project-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #88948d;
}

.project-row.active .project-dot {
  background: #4c82d9;
}

.project-copy {
  display: grid;
  min-width: 0;
  line-height: 1.25;
}

.project-copy strong,
.project-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-copy strong {
  font-size: 13px;
}

.project-copy small {
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
}

.project-remove {
  opacity: 0;
}

.project-row:hover .project-remove,
.project-row.active .project-remove {
  opacity: 1;
}

.add-project-empty {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  min-height: 38px;
  margin-top: 12px;
  border: 1px dashed #bdc8c1;
  border-radius: 7px;
  color: #4f5e56;
  background: #ffffff;
}

.quick-create {
  display: grid;
  gap: 7px;
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid #dce2dd;
}

.quick-create input {
  min-width: 0;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.repo-name {
  margin-top: 12px;
  font-size: 18px;
  font-weight: 800;
}

.repo-path,
.remote-row span {
  margin-top: 5px;
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  overflow-wrap: anywhere;
}

.branch-line,
.sync-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 12px;
  color: #304038;
}

.branch-line span {
  font-weight: 700;
}

.branch-line small,
.sync-line span {
  color: #6e7a73;
  font-size: 12px;
}

.remote-select {
  width: 100%;
  height: 32px;
  margin-top: 12px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  background: #ffffff;
}

.remote-row {
  display: grid;
  gap: 2px;
  margin-top: 12px;
}

.hosted-panel {
  display: grid;
  gap: 6px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid #d9e0dc;
}

.hosted-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  padding: 5px 7px;
  border: 1px solid #d2dad4;
  border-radius: 7px;
  background: #ffffff;
}

.hosted-row span {
  display: grid;
  min-width: 0;
  line-height: 1.2;
}

.hosted-row small {
  overflow: hidden;
  color: #68766f;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hosted-row a {
  color: #315f96;
  font-size: 12px;
  text-decoration: none;
}

.remote-editor,
.push-options {
  display: grid;
  gap: 7px;
  margin-top: 12px;
}

.remote-editor input,
.push-options input[type="text"] {
  min-width: 0;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.remote-editor-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
}

.push-options {
  padding-top: 10px;
  border-top: 1px solid #d9e0dc;
}

.push-options input[type="number"] {
  min-width: 0;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.push-options > label {
  display: grid;
  gap: 5px;
  color: #5e6b63;
  font-size: 12px;
}

.push-rejected-panel {
  display: grid;
  gap: 7px;
  margin-top: 12px;
  padding: 9px;
  border: 1px solid #e1c28a;
  border-radius: 7px;
  color: #5e4516;
  background: #fff6df;
}

.push-rejected-panel strong {
  font-size: 13px;
}

.push-rejected-panel span {
  color: #6f6553;
  font-size: 12px;
}

.branch-manager {
  display: grid;
  gap: 10px;
}

.branch-create {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 28px;
  gap: 7px;
}

.branch-create input {
  min-width: 0;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
}

.upstream-manager {
  display: grid;
  gap: 7px;
  padding-top: 10px;
  border-top: 1px solid #d9e0dc;
}

.branch-list {
  display: grid;
  gap: 5px;
}

.remote-branch-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px 30px;
  align-items: center;
  gap: 4px;
}

.remote-branch-item:hover .project-remove {
  opacity: 1;
}

.branch-group-label {
  margin-top: 4px;
  color: #738077;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.branch-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px 30px;
  align-items: center;
  gap: 4px;
  border: 1px solid transparent;
  border-radius: 7px;
}

.branch-row.active {
  border-color: #2f6f57;
  background: #e6eee7;
}

.branch-checkout,
.remote-branch-row {
  min-width: 0;
  border: 0;
  color: #25312b;
  text-align: left;
  background: transparent;
}

.branch-checkout {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  min-height: 38px;
  padding: 7px 8px;
  border-radius: 6px;
}

.branch-checkout:hover,
.remote-branch-row:hover {
  background: #edf1ec;
}

.branch-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #88948d;
}

.branch-row.active .branch-dot {
  background: #2f6f57;
}

.branch-copy {
  display: grid;
  min-width: 0;
  line-height: 1.25;
}

.branch-copy strong,
.branch-copy small,
.remote-branch-row {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-copy strong {
  font-size: 13px;
}

.branch-copy small {
  color: #68766f;
  font-size: 11px;
}

.remote-branch-row {
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  border-radius: 6px;
  font-size: 12px;
}

.tag-create {
  display: grid;
  gap: 7px;
  padding-top: 8px;
}

.tag-create input[type="text"],
.tag-create input:not([type]) {
  min-width: 0;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.tag-option {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: #5e6b63;
  font-size: 12px;
}

.tag-list {
  display: grid;
  gap: 5px;
}

.tag-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 28px 28px 28px;
  align-items: center;
  gap: 5px;
  min-height: 32px;
  padding: 4px 0;
}

.tag-row:hover .project-remove {
  opacity: 1;
}

.tag-copy {
  display: grid;
  min-width: 0;
  line-height: 1.25;
}

.tag-copy strong,
.tag-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-copy strong {
  font-size: 12px;
}

.tag-copy small {
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
}

.git-operation-panel,
.operation-form,
.operation-state {
  display: grid;
  gap: 10px;
}

.operation-state {
  padding: 9px;
  border: 1px solid #e1c28a;
  border-radius: 7px;
  color: #5e4516;
  background: #fff6df;
}

.operation-state strong {
  font-size: 13px;
  text-transform: uppercase;
}

.operation-state span,
.operation-options {
  color: #6f6553;
  font-size: 12px;
}

.operation-actions,
.operation-options,
.log-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.operation-options label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.operation-options input {
  width: 13px;
  height: 13px;
}

.advanced-rebase {
  display: grid;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid #d9e0dc;
}

.advanced-rebase summary {
  cursor: pointer;
  color: #526158;
  font-size: 12px;
  font-weight: 700;
}

.advanced-rebase input {
  min-width: 0;
  width: 100%;
  height: 30px;
  margin-top: 7px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.log-option {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 30px;
  color: #526158;
  font-size: 12px;
}

.log-option input {
  width: 13px;
  height: 13px;
}

.reset-select {
  height: 30px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.shelves {
  display: grid;
  gap: 9px;
}

.shelf-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 30px;
  align-items: center;
  gap: 4px;
  width: 100%;
  padding: 4px;
  border: 1px solid #d2dad4;
  border-radius: 7px;
  color: #28342e;
  background: #ffffff;
}

.shelf-row:hover .project-remove {
  opacity: 1;
}

.shelf-restore {
  display: grid;
  gap: 2px;
  min-width: 0;
  min-height: 34px;
  padding: 4px;
  border: 0;
  color: inherit;
  text-align: left;
  background: transparent;
}

.shelf-row small {
  color: #6b766f;
}

.changes-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #fbfcfa;
}

.changelist-panel {
  display: grid;
  gap: 7px;
  padding: 8px;
  border-bottom: 1px solid #dce2dd;
  background: #f6f8f6;
}

.changelist-tabs {
  display: flex;
  gap: 6px;
  overflow-x: auto;
}

.changelist-tabs button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 28px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #4c5a52;
  background: #ffffff;
  font-size: 12px;
  white-space: nowrap;
}

.changelist-tabs button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

.changelist-tabs small {
  font-size: 11px;
  opacity: 0.78;
}

.changelist-create {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 28px;
  gap: 6px;
}

.changelist-create input {
  min-width: 0;
  height: 28px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.changelist-actions {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 6px;
}

.changelist-actions .remote-select {
  margin-top: 0;
}

.conflict-panel {
  display: grid;
  gap: 8px;
  max-height: 300px;
  overflow: auto;
  padding: 8px;
  border-bottom: 1px solid #dce2dd;
  background: #fff8e8;
}

.conflict-header {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  color: #5e4516;
  font-size: 12px;
}

.conflict-file-tabs,
.conflict-actions,
.conflict-block-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.conflict-file-tabs button,
.mini-button {
  min-height: 26px;
  padding: 0 8px;
  border: 1px solid #e1c28a;
  border-radius: 6px;
  color: #5e4516;
  background: #ffffff;
  font-size: 12px;
}

.mini-button.danger {
  border-color: #d9b4aa;
  color: #8a2e20;
  background: #fff7f4;
}

.conflict-file-tabs button.active {
  color: #ffffff;
  background: #9a6a19;
}

.conflict-block {
  display: grid;
  gap: 6px;
  padding: 8px;
  border: 1px solid #ead4a7;
  border-radius: 7px;
  background: #ffffff;
}

.conflict-block-title {
  color: #75531a;
  font-size: 12px;
  font-weight: 700;
}

.conflict-block-preview {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.conflict-block-preview pre {
  max-height: 92px;
  margin: 0;
  overflow: auto;
  padding: 6px;
  border-radius: 6px;
  color: #26312c;
  background: #f7f3e8;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  white-space: pre-wrap;
}

.merge-header {
  flex-wrap: wrap;
}

.merge-header-actions,
.merge-save-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.merge-editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid #eef1ed;
  color: #526158;
  font-size: 12px;
}

.merge-editor-toolbar .warning {
  color: #9a4b16;
  font-weight: 700;
}

.merge-editor {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.25fr) minmax(0, 1fr);
  gap: 1px;
  flex: 1 1 auto;
  min-height: 0;
  background: #dce2dd;
}

.merge-column {
  display: grid;
  grid-template-rows: 38px minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
  background: #fbfcfa;
}

.merge-column.result {
  background: #ffffff;
}

.merge-column-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 0 12px;
  border-bottom: 1px solid #e5ebe7;
  color: #25312b;
  font-size: 12px;
}

.merge-column-title span {
  color: #738077;
  font-size: 11px;
  text-transform: uppercase;
}

.merge-column pre,
.merge-column textarea,
.merge-base-panel pre {
  margin: 0;
  min-width: 0;
  overflow: auto;
  padding: 12px;
  border: 0;
  color: #26312c;
  background: transparent;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  resize: none;
  tab-size: 2;
  white-space: pre;
}

.merge-column textarea {
  width: 100%;
  height: 100%;
  outline: none;
}

.merge-base-panel {
  flex: 0 0 auto;
  max-height: 220px;
  overflow: auto;
  border-top: 1px solid #dce2dd;
  background: #f8faf8;
}

.merge-base-panel summary {
  cursor: pointer;
  padding: 8px 16px;
  color: #526158;
  font-size: 12px;
  font-weight: 700;
}

.history-pane {
  display: grid;
  grid-template-rows: 46px auto minmax(0, 1fr);
  background: #fbfcfa;
}

.log-filter-panel {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(112px, 0.7fr);
  gap: 6px;
  padding: 8px;
  border-bottom: 1px solid #dce2dd;
  background: #f6f8f6;
}

.log-filter-panel label {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  min-width: 0;
  color: #68766f;
}

.log-filter-panel input,
.log-filter-panel .remote-select {
  min-width: 0;
  width: 100%;
  height: 30px;
  margin-top: 0;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.advanced-sidebar {
  display: grid;
  grid-template-rows: 46px minmax(0, 1fr);
  min-height: 0;
  border-right: 1px solid #cdd5d0;
  background: #fbfcfa;
}

.advanced-nav {
  display: grid;
  align-content: start;
  gap: 6px;
  padding: 10px;
  overflow: auto;
}

.advanced-nav button {
  min-height: 34px;
  border: 1px solid #d2dad4;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  text-align: left;
  padding: 0 10px;
}

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 0 12px;
  border-bottom: 1px solid #dce2dd;
  color: #6b766f;
  font-size: 12px;
}

.commit-list {
  min-height: 0;
  overflow: auto;
}

.commit-row {
  display: grid;
  grid-template-columns: 14px minmax(0, 1fr) auto;
  align-items: start;
  gap: 8px;
  width: 100%;
  min-height: 58px;
  padding: 8px 10px;
  border: 0;
  border-bottom: 1px solid #eef1ed;
  text-align: left;
  color: #25312b;
  background: transparent;
}

.commit-row:hover,
.commit-row.active {
  background: #e8f0fb;
}

.commit-node {
  width: 9px;
  height: 9px;
  margin-top: 5px;
  border: 2px solid #3f6ea5;
  border-radius: 50%;
  background: #ffffff;
}

.commit-copy {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.commit-copy strong,
.commit-copy small,
.commit-row code {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-copy strong {
  font-size: 13px;
}

.commit-copy small {
  color: #6b766f;
  font-size: 11px;
}

.commit-row code {
  max-width: 82px;
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
}

.commit-refs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.commit-refs em {
  max-width: 110px;
  padding: 1px 5px;
  border-radius: 5px;
  color: #365f91;
  background: #e9f0f7;
  font-size: 10px;
  font-style: normal;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.segmented {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  padding: 8px;
  border-bottom: 1px solid #dce2dd;
}

.segmented button {
  border: 1px solid transparent;
  border-radius: 7px;
  color: #4c5a52;
  background: #edf1ec;
}

.segmented button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

.file-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(74px, 1fr));
  align-items: stretch;
  gap: 6px;
  padding: 6px 8px;
  border-bottom: 1px solid #dce2dd;
}

.file-actions .icon-button {
  width: 100%;
  padding: 0 8px;
}

.file-list {
  flex: 1 1 180px;
  min-height: 0;
  overflow: auto;
}

.file-row {
  display: grid;
  grid-template-columns: 20px 12px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 44px;
  padding: 6px 10px;
  border: 0;
  border-bottom: 1px solid #eef1ed;
  text-align: left;
  color: #25312b;
  background: transparent;
}

.file-row:hover,
.file-row.active {
  background: #e8f0fb;
}

.file-row input {
  width: 14px;
  height: 14px;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #88948d;
}

.status-dot.added {
  background: #2f8a57;
}

.status-dot.modified {
  background: #b57c22;
}

.status-dot.deleted {
  background: #b83e31;
}

.status-dot.renamed {
  background: #426db6;
}

.status-dot.conflicted {
  background: #8b3fa8;
}

.file-main {
  display: grid;
  min-width: 0;
}

.file-main strong,
.file-main small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-main strong {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.file-main small {
  color: #738077;
  font-size: 11px;
}

.kind-badge {
  max-width: 96px;
  padding: 2px 6px;
  border-radius: 6px;
  color: #5a665f;
  background: #edf1ec;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shelve-box {
  flex: 0 0 auto;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  padding: 6px 8px;
  border-top: 1px solid #dce2dd;
}

.shelve-box input,
.commit-author,
.commit-box textarea {
  width: 100%;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
}

.shelve-box input {
  height: 32px;
  padding: 0 9px;
}

.commit-box {
  flex: 0 0 auto;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto auto auto;
  gap: 8px;
  min-height: 206px;
  padding: 10px;
  border-top: 1px solid #dce2dd;
}

.commit-box textarea {
  min-height: 0;
  resize: none;
  padding: 9px;
}

.commit-button {
  width: 100%;
}

.commit-options,
.commit-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.commit-options label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: #526158;
  font-size: 12px;
}

.commit-options input {
  width: 13px;
  height: 13px;
}

.commit-author {
  height: 30px;
  padding: 0 9px;
  font-size: 12px;
}

.commit-actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
}

.commit-button.secondary {
  border-color: #6d8bb7;
  color: #31557f;
  background: #eef5ff;
}

.advanced-workbench {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
  align-content: start;
  gap: 12px;
  min-height: 0;
  overflow: auto;
  padding: 14px 16px 24px;
}

.context-dashboard {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  align-content: start;
  gap: 12px;
  min-height: 0;
  overflow: auto;
  padding: 14px 16px 24px;
}

.dashboard-card {
  display: grid;
  align-content: start;
  gap: 10px;
  min-width: 0;
  padding: 14px;
  border: 1px solid #d8e0db;
  border-radius: 8px;
  background: #fbfcfa;
}

.dashboard-card.wide {
  grid-column: 1 / -1;
}

.dashboard-card > strong {
  min-width: 0;
  overflow: hidden;
  color: #24312b;
  font-size: 18px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-card > small {
  color: #6b766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  overflow-wrap: anywhere;
}

.metric-row,
.metric-grid {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.metric-row span,
.metric-grid span,
.chip-list span,
.remote-dashboard-list div {
  min-height: 28px;
  padding: 5px 8px;
  border: 1px solid #d2dad4;
  border-radius: 7px;
  color: #526158;
  background: #f6f8f6;
  font-size: 12px;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
}

.metric-grid strong {
  color: #24312b;
}

.chip-list {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.remote-dashboard-list {
  display: grid;
  gap: 6px;
}

.remote-dashboard-list div {
  display: grid;
  gap: 2px;
}

.remote-dashboard-list strong,
.remote-dashboard-list span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.remote-dashboard-list span {
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.advanced-card {
  display: grid;
  align-content: start;
  gap: 10px;
  min-width: 0;
  padding: 12px;
  border: 1px solid #d8e0db;
  border-radius: 8px;
  background: #fbfcfa;
}

.advanced-card.wide {
  grid-column: 1 / -1;
}

.advanced-form {
  display: grid;
  gap: 8px;
}

.advanced-form.two {
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
}

.advanced-form.three {
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}

.advanced-form input,
.advanced-form select {
  min-width: 0;
  width: 100%;
  height: 32px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-size: 12px;
}

.advanced-actions {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
}

.advanced-textarea {
  width: 100%;
  min-height: 140px;
  padding: 10px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  resize: vertical;
  white-space: pre;
}

.comparison-summary {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  color: #526158;
  font-size: 12px;
}

.comparison-summary span {
  padding: 3px 7px;
  border-radius: 6px;
  background: #edf1ec;
}

.advanced-list {
  display: grid;
  gap: 6px;
  min-height: 0;
  overflow: auto;
}

.advanced-list.compact {
  max-height: 240px;
}

.advanced-row,
.stash-row,
.message-history-row {
  min-width: 0;
  padding: 8px;
  border: 1px solid #e1e7e2;
  border-radius: 7px;
  color: #26312c;
  background: #ffffff;
  text-align: left;
}

.advanced-row {
  display: grid;
  gap: 2px;
}

.advanced-row.with-action {
  grid-template-columns: minmax(0, 1fr) 30px;
  align-items: center;
}

.advanced-row:hover .project-remove {
  opacity: 1;
}

.advanced-row strong,
.advanced-row small,
.stash-row strong,
.stash-row small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.advanced-row small,
.stash-row small {
  color: #68766f;
  font-size: 11px;
}

.stash-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto auto;
  align-items: center;
  gap: 6px;
}

.blame-list {
  max-height: 320px;
}

.blame-row {
  display: grid;
  grid-template-columns: 44px 76px 120px minmax(0, 1fr);
  align-items: start;
  gap: 8px;
  padding: 5px 0;
  border-bottom: 1px solid #eef1ed;
  font-size: 12px;
}

.blame-row code,
.blame-row span {
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.blame-row strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.blame-row pre,
.advanced-output {
  margin: 0;
  overflow: auto;
  color: #26312c;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  white-space: pre;
}

.advanced-output {
  max-height: 180px;
  padding: 8px;
  border-radius: 7px;
  background: #ffffff;
}

.message-history-row {
  cursor: pointer;
}

.diff-pane {
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

.log-actions {
  justify-content: flex-end;
  min-width: 0;
}

.diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
  padding: 14px 16px;
  border-bottom: 1px solid #dce2dd;
}

.eyebrow {
  color: #627168;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.diff-header h2 {
  margin: 2px 0 0;
  max-width: min(760px, 72vw);
  color: #202b26;
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #536159;
  font-size: 12px;
  white-space: nowrap;
}

.message {
  margin: 10px 16px 0;
  padding: 8px 10px;
  border-radius: 7px;
  font-size: 13px;
}

.message.error {
  color: #842c22;
  background: #fde9e3;
}

.message.notice {
  color: #255942;
  background: #e4f3e8;
}

.hunk-strip {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding: 10px 16px;
  border-bottom: 1px solid #eef1ed;
}

.commit-detail-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 1px solid #eef1ed;
}

.commit-detail-strip div {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.commit-detail-strip span {
  color: #718078;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.commit-detail-strip strong {
  overflow: hidden;
  color: #25312b;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-files {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding: 10px 16px;
  border-bottom: 1px solid #eef1ed;
}

.commit-file-row {
  display: inline-grid;
  grid-template-columns: 16px auto minmax(80px, max-content);
  grid-template-rows: auto auto;
  align-items: center;
  column-gap: 7px;
  max-width: 280px;
  padding: 6px 8px;
  border: 1px solid #dce2dd;
  border-radius: 7px;
  background: #ffffff;
}

.commit-file-row input {
  grid-row: 1 / span 2;
  width: 13px;
  height: 13px;
}

.commit-file-row .kind-badge {
  grid-row: 1 / span 2;
}

.commit-file-row strong,
.commit-file-row small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-file-row strong {
  color: #25312b;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}

.commit-file-row small {
  color: #738077;
  font-size: 11px;
}

.hunk-button {
  flex: 0 0 auto;
  max-width: 300px;
  padding: 6px 9px;
}

.hunk-button small {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-scroller {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  background: #fbfcfa;
}

.diff-empty {
  display: grid;
  place-items: center;
  height: 100%;
  color: #6c7971;
}

.diff-lines {
  margin: 0;
  padding: 12px 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  tab-size: 2;
}

.diff-line {
  display: grid;
  grid-template-columns: 58px max-content;
  min-height: 18px;
  white-space: pre;
}

.line-number {
  padding-right: 12px;
  color: #96a19b;
  text-align: right;
  user-select: none;
}

.line-content {
  padding: 0 18px 0 10px;
}

.diff-line.add {
  background: #e6f4e9;
}

.diff-line.delete {
  background: #fdece7;
}

.diff-line.hunk {
  color: #365f91;
  background: #e9f0f7;
}

.diff-line.file {
  color: #6a5a22;
  background: #f7f0d7;
}

@media (max-width: 1180px) {
  body {
    min-width: 760px;
  }

  .workspace {
    grid-template-columns: 190px 240px 340px minmax(0, 1fr);
  }

  .workspace-empty {
    grid-template-columns: 190px minmax(0, 1fr);
  }

  .topbar .toolbar > .tool-button > span {
    display: none;
  }

  .theme-switch {
    grid-template-columns: repeat(3, 60px);
  }
}

/* PhpStorm Dark Theme inspired workbench */
html[data-theme="dark"] ::selection {
  color: #ffffff;
  background: #214283;
}

html[data-theme="dark"] body {
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] button,
html[data-theme="dark"] input,
html[data-theme="dark"] textarea,
html[data-theme="dark"] select {
  color: inherit;
}

html[data-theme="dark"] button:focus-visible,
html[data-theme="dark"] input:focus-visible,
html[data-theme="dark"] textarea:focus-visible,
html[data-theme="dark"] select:focus-visible {
  outline: 1px solid #4c82d9;
  outline-offset: -1px;
}

html[data-theme="dark"] button:disabled {
  opacity: 0.44;
}

html[data-theme="dark"] .app-shell {
  background: #1e1f22;
}

html[data-theme="dark"] .topbar {
  border-bottom-color: #3c3f41;
  background: #2b2d30;
  box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.32);
}

html[data-theme="dark"] .brand-mark {
  border: 1px solid #56595f;
  color: #ffffff;
  background:
    linear-gradient(135deg, #21d789 0 25%, transparent 25%),
    linear-gradient(225deg, #00cfff 0 28%, transparent 28%),
    linear-gradient(45deg, #7b61ff 0 30%, transparent 30%),
    #111318;
  box-shadow: inset 0 0 0 4px #111318;
  letter-spacing: 0;
}

html[data-theme="dark"] .brand-copy strong,
html[data-theme="dark"] .repo-name,
html[data-theme="dark"] .diff-header h2,
html[data-theme="dark"] .file-main strong,
html[data-theme="dark"] .dashboard-card > strong,
html[data-theme="dark"] .metric-grid strong {
  color: #f0f2f5;
}

html[data-theme="dark"] .brand-copy span,
html[data-theme="dark"] .topbar-state small,
html[data-theme="dark"] .repo-path,
html[data-theme="dark"] .remote-row span,
html[data-theme="dark"] .branch-line small,
html[data-theme="dark"] .sync-line span,
html[data-theme="dark"] .branch-copy small,
html[data-theme="dark"] .branch-group-label,
html[data-theme="dark"] .file-main small,
html[data-theme="dark"] .history-header,
html[data-theme="dark"] .commit-copy small,
html[data-theme="dark"] .commit-row code,
html[data-theme="dark"] .commit-detail-strip span,
html[data-theme="dark"] .commit-file-row small,
html[data-theme="dark"] .operation-state span,
html[data-theme="dark"] .operation-options,
html[data-theme="dark"] .shelf-row small,
html[data-theme="dark"] .toggle-row,
html[data-theme="dark"] .push-options > label,
html[data-theme="dark"] .eyebrow,
html[data-theme="dark"] .diff-empty {
  color: #8f949b;
}

html[data-theme="dark"] .topbar-state {
  border-color: #4e5258;
  color: #c9d1d9;
  background: #252629;
}

html[data-theme="dark"] .toolbar {
  gap: 6px;
}

html[data-theme="dark"] .theme-switch {
  border-color: #4e5258;
  background: #252629;
}

html[data-theme="dark"] .theme-option {
  color: #b8bec7;
}

html[data-theme="dark"] .theme-option:hover {
  background: #313335;
}

html[data-theme="dark"] .theme-option.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .layout-popover {
  border-color: #4e5258;
  background: #2b2d30;
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.36);
}

html[data-theme="dark"] .layout-option,
html[data-theme="dark"] .layout-reset {
  color: #c9d1d9;
}

html[data-theme="dark"] .layout-option:hover {
  background: #313335;
}

html[data-theme="dark"] .layout-option input {
  accent-color: #4c82d9;
}

html[data-theme="dark"] .layout-reset {
  border-color: #4e5258;
  background: #252629;
}

html[data-theme="dark"] .layout-reset:hover {
  border-color: #4c82d9;
  background: #313335;
}

html[data-theme="dark"] .tool-button,
html[data-theme="dark"] .icon-button,
html[data-theme="dark"] .commit-button,
html[data-theme="dark"] .hunk-button,
html[data-theme="dark"] .icon-only-button,
html[data-theme="dark"] .project-remove,
html[data-theme="dark"] .add-project-empty {
  border-color: #4e5258;
  color: #c9d1d9;
  background: #3c3f41;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

html[data-theme="dark"] .tool-button:hover:not(:disabled),
html[data-theme="dark"] .icon-button:hover:not(:disabled),
html[data-theme="dark"] .hunk-button:hover:not(:disabled),
html[data-theme="dark"] .icon-only-button:hover:not(:disabled),
html[data-theme="dark"] .project-remove:hover:not(:disabled),
html[data-theme="dark"] .add-project-empty:hover:not(:disabled) {
  border-color: #5d626a;
  background: #45494d;
}

html[data-theme="dark"] .tool-button.primary,
html[data-theme="dark"] .commit-button,
html[data-theme="dark"] .segmented button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .tool-button.primary:hover:not(:disabled),
html[data-theme="dark"] .commit-button:hover:not(:disabled),
html[data-theme="dark"] .segmented button.active:hover:not(:disabled) {
  border-color: #75a7f0;
  background: #4a7fbf;
}

html[data-theme="dark"] .commit-button.secondary {
  border-color: #4e6687;
  color: #c8dcff;
  background: #253246;
}

html[data-theme="dark"] .tool-button.danger,
html[data-theme="dark"] .icon-button.danger,
html[data-theme="dark"] .icon-only-button.danger {
  border-color: #6b3c3a;
  color: #ffb4a8;
  background: #3f2b2a;
}

html[data-theme="dark"] .tool-button.danger:hover:not(:disabled),
html[data-theme="dark"] .icon-button.danger:hover:not(:disabled),
html[data-theme="dark"] .icon-only-button.danger:hover:not(:disabled) {
  border-color: #8b4a47;
  background: #4a302f;
}

html[data-theme="dark"] .empty-workbench {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(0, 0, 0, 0)),
    #1e1f22;
}

html[data-theme="dark"] .empty-panel {
  color: #dfe1e5;
}

html[data-theme="dark"] .project-pane,
html[data-theme="dark"] .repo-pane,
html[data-theme="dark"] .workbench-rail,
html[data-theme="dark"] .history-pane,
html[data-theme="dark"] .changes-pane,
html[data-theme="dark"] .diff-pane {
  border-right-color: #3c3f41;
}

html[data-theme="dark"] .pane-resizer {
  border-right-color: #3c3f41;
  background: #252629;
}

html[data-theme="dark"] .pane-resizer:hover::before,
html[data-theme="dark"] .pane-resizer.active::before,
html[data-theme="dark"] .pane-resizer:focus-visible::before {
  background: #4c82d9;
}

html[data-theme="dark"] .project-pane,
html[data-theme="dark"] .repo-pane,
html[data-theme="dark"] .workbench-rail {
  background: #252629;
}

html[data-theme="dark"] .rail-button {
  color: #a9b7c6;
}

html[data-theme="dark"] .rail-button:hover {
  background: #313335;
}

html[data-theme="dark"] .rail-button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .changes-pane,
html[data-theme="dark"] .history-pane,
html[data-theme="dark"] .advanced-sidebar,
html[data-theme="dark"] .diff-header,
html[data-theme="dark"] .changelist-panel,
html[data-theme="dark"] .log-filter-panel,
html[data-theme="dark"] .hunk-strip,
html[data-theme="dark"] .commit-detail-strip,
html[data-theme="dark"] .commit-files {
  background: #2b2d30;
}

html[data-theme="dark"] .diff-pane,
html[data-theme="dark"] .diff-scroller {
  background: #1e1f22;
}

html[data-theme="dark"] .pane-section,
html[data-theme="dark"] .segmented,
html[data-theme="dark"] .history-header,
html[data-theme="dark"] .conflict-panel,
html[data-theme="dark"] .file-actions,
html[data-theme="dark"] .commit-box,
html[data-theme="dark"] .diff-header,
html[data-theme="dark"] .hunk-strip,
html[data-theme="dark"] .commit-detail-strip,
html[data-theme="dark"] .commit-files {
  border-color: #3c3f41;
}

html[data-theme="dark"] .shelve-box {
  border-top-color: #3c3f41;
}

html[data-theme="dark"] .upstream-manager {
  border-top-color: #3c3f41;
}

html[data-theme="dark"] .section-title,
html[data-theme="dark"] .commit-title {
  color: #a9b7c6;
}

html[data-theme="dark"] .commit-options label {
  color: #8f949b;
}

html[data-theme="dark"] .icon-only-button,
html[data-theme="dark"] .project-remove {
  border-color: #4e5258;
  color: #a9b7c6;
  background: #2b2d30;
}

html[data-theme="dark"] .icon-only-button:hover:not(:disabled),
html[data-theme="dark"] .project-remove:hover:not(:disabled),
html[data-theme="dark"] .add-project-empty:hover:not(:disabled) {
  border-color: #4c82d9;
  background: #313335;
}

html[data-theme="dark"] .project-row.active {
  border-color: #4c82d9;
  background: #28354b;
}

html[data-theme="dark"] .project-switch {
  color: #dfe1e5;
}

html[data-theme="dark"] .project-switch:hover {
  background: #313335;
}

html[data-theme="dark"] .project-dot {
  background: #6b7078;
}

html[data-theme="dark"] .project-row.active .project-dot {
  background: #7aa2f7;
}

html[data-theme="dark"] .project-copy small {
  color: #8f949b;
}

html[data-theme="dark"] .add-project-empty {
  border-color: #4e5258;
  color: #a9b7c6;
  background: #2b2d30;
}

html[data-theme="dark"] .branch-line,
html[data-theme="dark"] .sync-line,
html[data-theme="dark"] .remote-row strong {
  color: #dfe1e5;
}

html[data-theme="dark"] .remote-select,
html[data-theme="dark"] .remote-editor input,
html[data-theme="dark"] .push-options input[type="text"],
html[data-theme="dark"] .push-options input[type="number"],
html[data-theme="dark"] .quick-create input,
html[data-theme="dark"] .branch-create input,
html[data-theme="dark"] .tag-create input,
html[data-theme="dark"] .changelist-create input,
html[data-theme="dark"] .log-filter-panel input,
html[data-theme="dark"] .advanced-form input,
html[data-theme="dark"] .advanced-form select,
html[data-theme="dark"] .advanced-textarea,
html[data-theme="dark"] .reset-select,
html[data-theme="dark"] .shelve-box input,
html[data-theme="dark"] .commit-author,
html[data-theme="dark"] .commit-box textarea {
  border-color: #4e5258;
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .log-option {
  color: #8f949b;
}

html[data-theme="dark"] .remote-select {
  padding: 0 8px;
}

html[data-theme="dark"] .remote-select option,
html[data-theme="dark"] .reset-select option {
  color: #dfe1e5;
  background: #2b2d30;
}

html[data-theme="dark"] .push-options {
  border-top-color: #3c3f41;
}

html[data-theme="dark"] .tag-option,
html[data-theme="dark"] .tag-copy small {
  color: #8f949b;
}

html[data-theme="dark"] .branch-row {
  border-color: transparent;
}

html[data-theme="dark"] .branch-row.active {
  border-color: #4c82d9;
  background: #28354b;
}

html[data-theme="dark"] .branch-checkout,
html[data-theme="dark"] .remote-branch-row {
  color: #dfe1e5;
}

html[data-theme="dark"] .branch-checkout:hover,
html[data-theme="dark"] .remote-branch-row:hover {
  background: #313335;
}

html[data-theme="dark"] .branch-dot {
  background: #787d85;
}

html[data-theme="dark"] .branch-row.active .branch-dot {
  background: #7aa2f7;
}

html[data-theme="dark"] .branch-copy strong {
  color: #f0f2f5;
}

html[data-theme="dark"] .operation-state {
  border-color: #6b5930;
  color: #ffd98a;
  background: #332b1a;
}

html[data-theme="dark"] .operation-state strong,
html[data-theme="dark"] .push-rejected-panel strong,
html[data-theme="dark"] .conflict-header,
html[data-theme="dark"] .conflict-block-title {
  color: #ffd98a;
}

html[data-theme="dark"] .push-rejected-panel {
  border-color: #6b5930;
  color: #ffd98a;
  background: #332b1a;
}

html[data-theme="dark"] .push-rejected-panel span {
  color: #c9aa6a;
}

html[data-theme="dark"] .conflict-panel {
  background: #2c2619;
}

html[data-theme="dark"] .conflict-file-tabs button,
html[data-theme="dark"] .mini-button {
  border-color: #6b5930;
  color: #ffd98a;
  background: #2b2d30;
}

html[data-theme="dark"] .mini-button.danger {
  border-color: #6b3c3a;
  color: #ffb4a8;
}

html[data-theme="dark"] .conflict-file-tabs button.active {
  color: #1e1f22;
  background: #d7a642;
}

html[data-theme="dark"] .conflict-block {
  border-color: #4a422f;
  background: #2b2d30;
}

html[data-theme="dark"] .conflict-block-preview pre {
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .merge-editor-toolbar,
html[data-theme="dark"] .merge-base-panel {
  border-color: #3c3f41;
  color: #8f949b;
  background: #2b2d30;
}

html[data-theme="dark"] .merge-editor-toolbar .warning {
  color: #ffb86c;
}

html[data-theme="dark"] .merge-editor {
  background: #3c3f41;
}

html[data-theme="dark"] .merge-column,
html[data-theme="dark"] .merge-column.result {
  background: #1e1f22;
}

html[data-theme="dark"] .merge-column-title {
  border-color: #3c3f41;
  color: #dfe1e5;
  background: #2b2d30;
}

html[data-theme="dark"] .merge-column-title span,
html[data-theme="dark"] .merge-base-panel summary {
  color: #8f949b;
}

html[data-theme="dark"] .merge-column pre,
html[data-theme="dark"] .merge-column textarea,
html[data-theme="dark"] .merge-base-panel pre {
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .shelf-row {
  border-color: #3c3f41;
  color: #dfe1e5;
  background: #2b2d30;
}

html[data-theme="dark"] .shelf-row:hover {
  border-color: #4c82d9;
  background: #313335;
}

html[data-theme="dark"] .shelf-restore {
  color: #dfe1e5;
}

html[data-theme="dark"] .segmented button {
  color: #b8bec7;
  background: #252629;
}

html[data-theme="dark"] .changelist-tabs button,
html[data-theme="dark"] .advanced-nav button,
html[data-theme="dark"] .advanced-card,
html[data-theme="dark"] .dashboard-card,
html[data-theme="dark"] .advanced-row,
html[data-theme="dark"] .stash-row,
html[data-theme="dark"] .message-history-row,
html[data-theme="dark"] .advanced-output,
html[data-theme="dark"] .metric-row span,
html[data-theme="dark"] .metric-grid span,
html[data-theme="dark"] .chip-list span,
html[data-theme="dark"] .remote-dashboard-list div {
  border-color: #3c3f41;
  color: #dfe1e5;
  background: #2b2d30;
}

html[data-theme="dark"] .changelist-tabs button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .segmented button:hover:not(.active) {
  border-color: #4e5258;
  background: #313335;
}

html[data-theme="dark"] .project-row {
  border-color: transparent;
}

html[data-theme="dark"] .project-row.active {
  border-color: #4c82d9;
  background: #28354b;
}

html[data-theme="dark"] .project-switch {
  color: #dfe1e5;
}

html[data-theme="dark"] .project-switch:hover {
  background: #313335;
}

html[data-theme="dark"] .project-dot {
  background: #787d85;
}

html[data-theme="dark"] .project-row.active .project-dot {
  background: #7aa2f7;
}

html[data-theme="dark"] .project-copy strong {
  color: #f0f2f5;
}

html[data-theme="dark"] .project-copy small {
  color: #8f949b;
}

html[data-theme="dark"] .add-project-empty {
  border-style: dashed;
}

html[data-theme="dark"] .commit-row {
  border-bottom-color: #313335;
  border-left: 3px solid transparent;
  color: #c9d1d9;
}

html[data-theme="dark"] .commit-row:hover {
  background: #313335;
}

html[data-theme="dark"] .commit-row.active {
  border-left-color: #4c82d9;
  background: #3a3f47;
}

html[data-theme="dark"] .commit-node {
  border-color: #7aa2f7;
  background: #2b2d30;
}

html[data-theme="dark"] .commit-copy strong {
  color: #f0f2f5;
}

html[data-theme="dark"] .commit-refs em {
  color: #a9c7ff;
  background: #28354b;
}

html[data-theme="dark"] .commit-detail-strip strong,
html[data-theme="dark"] .commit-file-row strong {
  color: #dfe1e5;
}

html[data-theme="dark"] .commit-file-row {
  border-color: #3c3f41;
  background: #2b2d30;
}

html[data-theme="dark"] .file-row {
  border-bottom-color: #313335;
  border-left: 3px solid transparent;
  color: #c9d1d9;
}

html[data-theme="dark"] .file-row:hover {
  background: #313335;
}

html[data-theme="dark"] .file-row.active {
  border-left-color: #4c82d9;
  background: #3a3f47;
}

html[data-theme="dark"] .file-row input {
  accent-color: #4c82d9;
}

html[data-theme="dark"] .status-dot {
  background: #787d85;
}

html[data-theme="dark"] .status-dot.added {
  background: #6aab73;
}

html[data-theme="dark"] .status-dot.modified {
  background: #ffc66d;
}

html[data-theme="dark"] .status-dot.deleted {
  background: #cf6679;
}

html[data-theme="dark"] .status-dot.renamed {
  background: #6f9fee;
}

html[data-theme="dark"] .status-dot.conflicted {
  background: #cc7832;
}

html[data-theme="dark"] .kind-badge {
  color: #aeb6c2;
  background: #393b40;
}

html[data-theme="dark"] .file-row.active .kind-badge {
  color: #dce7f7;
  background: #354966;
}

html[data-theme="dark"] .remote-editor input::placeholder,
html[data-theme="dark"] .push-options input::placeholder,
html[data-theme="dark"] .quick-create input::placeholder,
html[data-theme="dark"] .changelist-create input::placeholder,
html[data-theme="dark"] .log-filter-panel input::placeholder,
html[data-theme="dark"] .advanced-form input::placeholder,
html[data-theme="dark"] .advanced-textarea::placeholder,
html[data-theme="dark"] .shelve-box input::placeholder,
html[data-theme="dark"] .commit-author::placeholder,
html[data-theme="dark"] .commit-box textarea::placeholder {
  color: #70757d;
}

html[data-theme="dark"] .commit-box textarea,
html[data-theme="dark"] .diff-lines,
html[data-theme="dark"] .repo-path,
html[data-theme="dark"] .remote-row span,
html[data-theme="dark"] .file-main strong,
html[data-theme="dark"] .file-main small {
  font-family: "JetBrains Mono", "SF Mono", SFMono-Regular, Menlo, Consolas, monospace;
}

html[data-theme="dark"] .message {
  border: 1px solid transparent;
}

html[data-theme="dark"] .message.error {
  border-color: #6b3c3a;
  color: #ffb4a8;
  background: #3c2525;
}

html[data-theme="dark"] .message.notice {
  border-color: #49617e;
  color: #a8d8b9;
  background: #223427;
}

html[data-theme="dark"] .hunk-button {
  color: #d6dbe3;
  background: #313335;
}

html[data-theme="dark"] .hunk-button small {
  color: #8f949b;
}

html[data-theme="dark"] .diff-header {
  padding-top: 10px;
  padding-bottom: 10px;
}

html[data-theme="dark"] .eyebrow {
  color: #7f8590;
}

html[data-theme="dark"] .toggle-row input {
  accent-color: #4c82d9;
}

html[data-theme="dark"] .diff-lines {
  padding: 0;
  color: #c9d1d9;
  background: #1e1f22;
}

html[data-theme="dark"] .diff-line {
  grid-template-columns: 66px max-content;
}

html[data-theme="dark"] .diff-line.context {
  background: #1e1f22;
}

html[data-theme="dark"] .line-number {
  padding-right: 14px;
  border-right: 1px solid #323438;
  color: #60656d;
  background: #252629;
}

html[data-theme="dark"] .line-content {
  min-width: 100%;
  padding-left: 12px;
}

html[data-theme="dark"] .diff-line.add {
  color: #6a8759;
  background: #243729;
}

html[data-theme="dark"] .diff-line.delete {
  color: #cc6666;
  background: #3f2a2a;
}

html[data-theme="dark"] .diff-line.hunk {
  color: #6897bb;
  background: #263247;
}

html[data-theme="dark"] .diff-line.file {
  color: #ffc66d;
  background: #332d20;
}

html[data-theme="dark"] .diff-line.add .line-number {
  background: #203024;
}

html[data-theme="dark"] .diff-line.delete .line-number {
  background: #352525;
}

html[data-theme="dark"] .diff-line.hunk .line-number {
  background: #222b3d;
}

html[data-theme="dark"] .diff-line.file .line-number {
  background: #2d281d;
}

html[data-theme="dark"] ::-webkit-scrollbar {
  width: 11px;
  height: 11px;
}

html[data-theme="dark"] ::-webkit-scrollbar-track {
  background: #1e1f22;
}

html[data-theme="dark"] ::-webkit-scrollbar-thumb {
  border: 2px solid #1e1f22;
  border-radius: 8px;
  background: #4a4d52;
}

html[data-theme="dark"] ::-webkit-scrollbar-thumb:hover {
  background: #5b5f66;
}
</style>
