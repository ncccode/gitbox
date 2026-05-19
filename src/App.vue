<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Archive,
  ArchiveRestore,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  ChevronDown,
  ChevronRight,
  Check,
  Columns3,
  Download,
  File as FileIcon,
  FileSearch,
  Folder,
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
  UserRound,
  X,
} from "@lucide/vue";
import AppTopbar from "./components/AppTopbar.vue";
import VcsIcon from "./components/icons/VcsIcon.vue";
import ProjectPane from "./components/ProjectPane.vue";
import WorkbenchRail from "./components/WorkbenchRail.vue";
import {
  commitFileDiff,
  copyProjectEntry,
  createProjectDirectory,
  createProjectFile,
  deleteProjectEntry,
  moveProjectEntry,
  renameProjectEntry,
} from "./lib/gitboxCommands";
import { useAdvancedStore } from "./stores/advanced";
import { useBranchesStore } from "./stores/branches";
import { useChangelistsStore } from "./stores/changelists";
import { useChangesStore } from "./stores/changes";
import { useCommitStore } from "./stores/commit";
import { useDiffStore } from "./stores/diff";
import { useHistoryStore } from "./stores/history";
import { useOperationsStore } from "./stores/operations";
import { PROJECT_ROOT_PATH, useProjectStore } from "./stores/project";
import { useRemoteStore } from "./stores/remote";
import { useRepositoriesStore } from "./stores/repositories";
import { useSettingsStore } from "./stores/settings";
import type { LayoutPanelKey, ThemeMode } from "./stores/settings";
import type {
  BranchInfo,
  ChangeSide,
  ChangedFile,
  CommitFileDiffMode,
  CommitFileChange,
  CommitSummary,
  DiffHunk,
  DiffResponse,
  ProjectFileEntry,
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
const project = useProjectStore();
const remote = useRemoteStore();
const settings = useSettingsStore();
const shelveMessage = ref("");
const newBranchName = ref("");
const newTagName = ref("");
const newTagTarget = ref("");
const annotatedTag = ref(false);
const tagMessage = ref("");
const selectedCommitFilePaths = ref<string[]>([]);
const expandedCommitFileDirectories = ref<Record<string, boolean>>({});
const logAuthorPickerOpen = ref(false);
const logFilePickerOpen = ref(false);
const logFilePickerSearch = ref("");
const logFilePickerDraft = ref<string[]>([]);
const logRefSearch = ref("");
const logRefPanelCollapsed = ref(false);
const logFavoriteRefsOnly = ref(false);
const logRefSearchInput = ref<HTMLInputElement | null>(null);
const expandedLogFilePickerDirectories = ref<Record<string, boolean>>({
  [PROJECT_ROOT_PATH]: true,
});
const expandedLogRefGroups = ref<Record<string, boolean>>({
  local: true,
  remote: true,
  tags: true,
});
const changeFileContextMenu = ref<ChangeFileContextMenu | null>(null);
const changeListContextMenu = ref<ChangeListContextMenu | null>(null);
const projectFileContextMenu = ref<ProjectFileContextMenu | null>(null);
const projectFileClipboard = ref<ProjectFileClipboard>(null);
const projectNameDialog = ref<ProjectNameDialog | null>(null);
const projectCloseDialog = ref<ProjectCloseDialog | null>(null);
const projectEditorTextarea = ref<HTMLTextAreaElement | null>(null);
const changeDiffScroller = ref<HTMLElement | null>(null);
const logDiffScroller = ref<HTMLElement | null>(null);
const activeChangeDiffHunkIndex = ref<number | null>(null);
const activeLogDiffHunkIndex = ref(0);
const syncingSideBySideScroll = new WeakSet<HTMLElement>();
const projectEditorScrollTop = ref(0);
const projectEditorScrollLeft = ref(0);
const expandedProjectHunkIndex = ref<number | null>(null);
const expandedChangeFileGroups = ref<Record<string, boolean>>({
  staged: true,
  tracked: true,
  untracked: true,
});
type WorkbenchMode = "changes" | "log" | "project" | "branches" | "remote" | "operations" | "advanced";
type ChangeFileGroup = {
  key: string;
  label: string;
  side: ChangeSide;
  files: ChangedFile[];
  changelistId?: string;
};
type ProjectCodeToken = {
  text: string;
  kind?: "comment" | "string" | "keyword" | "number" | "function" | "property" | "operator";
};
type SideBySideDiffCell = {
  lineNumber: number | null;
  content: string;
  type: "context" | "add" | "delete" | "empty" | "meta";
  tokens: ProjectCodeToken[];
};
type SideBySideDiffRow = {
  id: string;
  type: "context" | "add" | "delete" | "modify" | "meta";
  hunkIndex: number | null;
  anchorHunkIndex: number | null;
  old: SideBySideDiffCell;
  new: SideBySideDiffCell;
};
type ProjectEditorLine = {
  index: number;
  number: number;
  tokens: ProjectCodeToken[];
};
type ProjectOriginalLine = {
  index: number;
  lineNumber: number;
  content: string;
  tokens: ProjectCodeToken[];
};
type ProjectEditorHunkView = {
  index: number;
  header: string;
  tone: "added" | "deleted" | "modified";
  lineStart: number;
  lineCount: number;
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  addedLines: number;
  deletedLines: number;
  originalLines: ProjectOriginalLine[];
};
type LogDiffTab = {
  id: string;
  oid: string;
  shortOid: string;
  path: string;
  mode: CommitFileDiffMode;
  title: string;
  subtitle: string;
  diff: DiffResponse | null;
  loading: boolean;
  error: string;
};
type ChangeFileContextMenu = {
  file: ChangedFile;
  side: ChangeSide;
  x: number;
  y: number;
};
type ChangeListContextMenu = {
  listId: string;
  x: number;
  y: number;
};
type ProjectFileContextMenu = {
  file: ProjectFileEntry | null;
  x: number;
  y: number;
};
type ProjectFileClipboard = {
  mode: "cut" | "copy";
  path: string;
  name: string;
  directory: boolean;
} | null;
type ProjectNameDialog = {
  title: string;
  value: string;
  error: string;
  validate: (value: string) => string;
  resolve: (value: string | null) => void;
};
type ProjectCloseDialog = {
  path: string;
  name: string;
  saving: boolean;
  error: string;
};
type NoticeToast = {
  id: number;
  message: string;
};

const workbenchMode = ref<WorkbenchMode>("changes");
const LOG_TAB_ID = "log-root";
const activeLogTabId = ref(LOG_TAB_ID);
const logDiffTabs = ref<LogDiffTab[]>([]);
const noticeToast = ref<NoticeToast | null>(null);
const activeResizePanel = ref<LayoutPanelKey | null>(null);
const systemPrefersDark = ref(
  typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches,
);
let stopSystemThemeWatch: (() => void) | null = null;
let autoFetchTimer: number | null = null;
let noticeToastTimer: number | null = null;
let noticeToastId = 0;
const repositoryContextModes = new Set<WorkbenchMode>(["branches", "remote", "operations"]);
const workbenchContextModes = new Set<WorkbenchMode>(["changes", "log", "project", "advanced"]);
const PROJECT_EDITOR_LINE_HEIGHT = 18;
const PROJECT_EDITOR_PADDING_TOP = 12;

const activeFiles = computed(() => {
  return changes.filesForSide(settings.selectedSide);
});
const changeFileGroups = computed<ChangeFileGroup[]>(() => {
  if (settings.selectedSide === "staged") {
    const stagedGroups: ChangeFileGroup[] = [
      {
        key: "staged",
        label: "暂存的变更",
        side: "staged",
        files: activeFiles.value,
        changelistId: "default",
      },
    ];

    return stagedGroups.filter((group) => group.files.length > 0);
  }

  const listGroups = new Map<string, ChangeFileGroup>(
    changelists.lists.map((list) => [
      `changelist-${list.id}`,
      {
        key: `changelist-${list.id}`,
        label: list.id === "default" && list.name === "默认变更" ? "变更" : list.name,
        side: "unstaged",
        files: [],
        changelistId: list.id,
      },
    ]),
  );
  const untrackedFiles: ChangedFile[] = [];

  for (const file of activeFiles.value) {
    const list = changelists.listForPath(file.path);
    if (file.untracked && list.id === "default") {
      untrackedFiles.push(file);
      continue;
    }

    listGroups.get(`changelist-${list.id}`)?.files.push(file);
  }

  const unstagedGroups: ChangeFileGroup[] = [
    ...listGroups.values(),
    {
      key: "untracked",
      label: "未纳入版本控制的文件",
      side: "unstaged",
      files: untrackedFiles,
      changelistId: "default",
    },
  ];

  return unstagedGroups.filter((group) => group.key !== "untracked" || group.files.length > 0);
});
const usesRepositoryContext = computed(() => repositoryContextModes.has(workbenchMode.value));
const usesWorkbenchContext = computed(() => workbenchContextModes.has(workbenchMode.value));
const counts = computed(() => changes.status?.counts);
const branch = computed(() => changes.branch);
const brandSubtitle = computed(() =>
  repos.current
    ? repos.name
    : repos.selectedPath
      ? `${repos.projectName(repos.selectedPath)} · 未初始化`
      : `${repos.items.length} 个项目`,
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
    (workbenchMode.value === "project" ? project.error : "") ||
    remote.error ||
    advanced.error,
);
const activeNotice = computed(
  () =>
    operations.notice ||
    advanced.notice ||
    branches.notice ||
    changes.notice ||
    remote.notice,
);
const selectedDiffFileTitle = computed(() => changes.selectedFile ?? "未选择文件");
const activeChangeDiffLanguage = computed(() => projectLanguageForPath(changes.selectedFile));
const activeChangeSideBySideDiffRows = computed(() =>
  buildSideBySideDiffRows(diff.current, activeChangeDiffLanguage.value),
);
const activeChangeDiffHasContent = computed(() => Boolean(diff.current?.text?.trim()));
const activeChangeDiffHunkCount = computed(() => diff.current?.hunks.length ?? 0);
const currentChangeDiffHunkPosition = computed(() => {
  if (activeChangeDiffHunkCount.value === 0) return 0;
  const index = diff.current?.hunks.findIndex((hunk) => hunk.index === activeChangeDiffHunkIndex.value) ?? -1;
  return index >= 0 ? index + 1 : 1;
});
const activeChangeDiffFileIndex = computed(() =>
  changes.selectedFile ? activeFiles.value.findIndex((file) => file.path === changes.selectedFile) : -1,
);
const activeChangeDiffFilePosition = computed(() =>
  activeChangeDiffFileIndex.value >= 0
    ? `${activeChangeDiffFileIndex.value + 1}/${activeFiles.value.length}`
    : `0/${activeFiles.value.length}`,
);
const canSelectPreviousChangeDiffFile = computed(() => activeChangeDiffFileIndex.value > 0);
const canSelectNextChangeDiffFile = computed(
  () => activeChangeDiffFileIndex.value >= 0 && activeChangeDiffFileIndex.value < activeFiles.value.length - 1,
);
const changeDiffLeftLabel = computed(() => (settings.selectedSide === "staged" ? "提交" : "索引"));
const changeDiffLeftDetail = computed(() =>
  settings.selectedSide === "staged" ? shortHash(branch.value?.head) : "暂存快照",
);
const changeDiffRightLabel = computed(() => (settings.selectedSide === "staged" ? "暂存区" : "工作区"));
const changeDiffRightDetail = computed(() => changes.selectedFile ?? "");
const canCommit = computed(() =>
  Boolean(commit.message.trim() && ((counts.value?.staged ?? 0) > 0 || (commit.amend && branch.value?.head))),
);
const selectedCommitTitle = computed(() => {
  if (!history.details) return "未选择提交";
  return `${history.details.commit.shortOid} · ${history.details.commit.summary}`;
});
const activeLogDiffTab = computed(() => logDiffTabs.value.find((tab) => tab.id === activeLogTabId.value) ?? null);
const activeLogDiffLanguage = computed(() => projectLanguageForPath(activeLogDiffTab.value?.path));
const activeLogSideBySideDiffRows = computed(() =>
  buildSideBySideDiffRows(activeLogDiffTab.value?.diff ?? null, activeLogDiffLanguage.value),
);
const activeLogDiffHasContent = computed(() => Boolean(activeLogDiffTab.value?.diff?.text?.trim()));
const activeLogDiffHunkCount = computed(() => activeLogDiffTab.value?.diff?.hunks.length ?? 0);
const currentLogDiffHunkPosition = computed(() =>
  activeLogDiffHunkCount.value > 0 ? Math.min(activeLogDiffHunkIndex.value + 1, activeLogDiffHunkCount.value) : 0,
);
const activeLogDiffFileIndex = computed(() => {
  const path = activeLogDiffTab.value?.path;
  return path ? (history.details?.files ?? []).findIndex((file) => file.path === path) : -1;
});
const activeLogDiffFilePosition = computed(() =>
  activeLogDiffFileIndex.value >= 0
    ? `${activeLogDiffFileIndex.value + 1}/${history.details?.files.length ?? 0}`
    : `0/${history.details?.files.length ?? 0}`,
);
const canSelectPreviousLogDiffFile = computed(() => activeLogDiffFileIndex.value > 0);
const canSelectNextLogDiffFile = computed(
  () => activeLogDiffFileIndex.value >= 0 && activeLogDiffFileIndex.value < (history.details?.files.length ?? 0) - 1,
);
const projectEditorText = computed({
  get: () => project.editorText,
  set: (value: string) => {
    expandedProjectHunkIndex.value = null;
    project.setEditorText(value);
  },
});
const projectLanguage = computed(() => projectLanguageForPath(project.selectedPath));
const projectEditorLines = computed<ProjectEditorLine[]>(() =>
  projectEditorText.value.split("\n").map((content, index) => ({
    index,
    number: index + 1,
    tokens: tokenizeProjectLine(content || " ", projectLanguage.value),
  })),
);
const projectEditorHunks = computed<ProjectEditorHunkView[]>(() => {
  if (!project.content || project.content.binary || !project.diff?.hunks.length) return [];
  return project.diff.hunks.map((hunk) => buildProjectEditorHunkView(hunk, projectLanguage.value));
});
const expandedProjectHunk = computed(
  () => projectEditorHunks.value.find((hunk) => hunk.index === expandedProjectHunkIndex.value) ?? null,
);
const projectEditorRenderStyle = computed(() => ({
  "--project-editor-scroll-top-offset": `${-projectEditorScrollTop.value}px`,
  "--project-editor-scroll-left-offset": `${-projectEditorScrollLeft.value}px`,
}));
const projectRootEntry = computed<ProjectFileEntry | null>(() => {
  if (!repos.current) return null;
  return {
    path: PROJECT_ROOT_PATH,
    name: repos.name,
    parent: null,
    depth: 0,
    directory: true,
    size: null,
  };
});
const projectChildrenByParent = computed(() => {
  const groups = new Map<string, ProjectFileEntry[]>();
  for (const file of project.files) {
    const parent = file.parent ?? PROJECT_ROOT_PATH;
    const children = groups.get(parent) ?? [];
    children.push(file);
    groups.set(parent, children);
  }

  for (const children of groups.values()) {
    children.sort(compareProjectTreeEntries);
  }
  return groups;
});
const expandedProjectDirectories = computed(() => new Set(project.expandedPaths));
const visibleProjectFiles = computed(() => {
  if (!projectRootEntry.value) return project.files;

  const rows: ProjectFileEntry[] = [projectRootEntry.value];
  const appendChildren = (parentPath: string) => {
    if (!expandedProjectDirectories.value.has(parentPath)) return;
    for (const child of projectChildrenByParent.value.get(parentPath) ?? []) {
      rows.push(child);
      if (child.directory) {
        appendChildren(child.path);
      }
    }
  };
  appendChildren(PROJECT_ROOT_PATH);
  return rows;
});
type ProjectGitStatus =
  | "conflicted"
  | "deleted"
  | "added"
  | "modified"
  | "renamed"
  | "typechange"
  | "ignored"
  | "unknown";
const projectStatusPriority: Record<ProjectGitStatus, number> = {
  unknown: 0,
  ignored: 1,
  renamed: 2,
  typechange: 3,
  modified: 4,
  added: 5,
  deleted: 6,
  conflicted: 7,
};
const projectStatusByPath = computed(() => {
  const statuses = new Map<string, ProjectGitStatus>();
  for (const file of changes.files) {
    const status = normalizeProjectGitStatus(file);
    setProjectGitStatus(statuses, PROJECT_ROOT_PATH, status);
    setProjectGitStatus(statuses, file.path, status);

    const segments = file.path.split("/").filter(Boolean);
    for (let index = 1; index < segments.length; index += 1) {
      setProjectGitStatus(statuses, segments.slice(0, index).join("/"), status);
    }
  }
  return statuses;
});
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

  if (repos.current || repos.selectedPath) {
    columns.push("68px");
  }

  if (repos.current && usesRepositoryContext.value && settings.panelVisibility.repo) {
    columns.push(`${settings.panelWidths.repo}px`, "6px");
  }

  if (repos.current && usesWorkbenchContext.value && settings.panelVisibility.changes) {
    const workbenchContextWidth =
      workbenchMode.value === "log" && logRefPanelCollapsed.value
        ? "42px"
        : `${settings.panelWidths.changes}px`;
    columns.push(workbenchContextWidth, "6px");
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
const changeFileIconLabels: Record<string, string> = {
  css: "CSS",
  html: "<>",
  js: "JS",
  json: "{}",
  jsx: "JS",
  md: "MD",
  rs: "RS",
  ts: "TS",
  tsx: "TS",
  vue: "VUE",
  wxml: "<>",
  wxss: "CSS",
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
const commitFileDiffModeLabels: Record<CommitFileDiffMode, string> = {
  commit: "储存库差异",
  worktree: "与本地比较",
  "parent-worktree": "之前版本与本地比较",
};
const graphLaneWidth = 14;
const graphLaneInset = 10;
const graphRowHeight = 30;
const graphRowMid = graphRowHeight / 2;
const graphMaxVisibleLanes = 6;
const graphPalette = ["#b89445", "#8e63c8", "#4f9d76", "#4f82c9", "#c86d56", "#70a6a1"];
type LogGraphPath = {
  key: string;
  d: string;
  color: string;
};
type LogGraphActiveLane = {
  oid: string;
  color: string;
};
type LogGraphRow = {
  item: CommitSummary;
  paths: LogGraphPath[];
  laneIndex: number;
  color: string;
  nodeLeft: number;
  graphWidth: number;
  hasMerge: boolean;
};
type LogRemoteGroup = {
  name: string;
  branches: BranchInfo[];
};
type LogRefGroupKey = "local" | "remote" | "tags" | `remote:${string}`;
type LogFileTreeRow = {
  id: string;
  name: string;
  path: string;
  parent: string | null;
  depth: number;
  directory: boolean;
  fileCount?: number;
  status?: string;
  oldPath?: string | null;
};
type LogFileContextMenu = {
  x: number;
  y: number;
  row: LogFileTreeRow;
};
type LogRefContextMenu =
  | {
      kind: "local" | "remote";
      x: number;
      y: number;
      branch: BranchInfo;
    }
  | {
      kind: "tag";
      x: number;
      y: number;
      tag: TagInfo;
    };
type LogAuthorOption = {
  value: string;
  label: string;
  meta: string;
  count: number;
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
const logHeadLabel = computed(() => (branch.value?.detached ? "HEAD (游离)" : "HEAD (目前分支)"));
const logRemoteGroups = computed<LogRemoteGroup[]>(() => {
  const groups = new Map<string, BranchInfo[]>();
  for (const item of branches.sortedRemoteBranches) {
    const parts = item.name.split("/");
    const remoteName = parts[0] || "remote";
    const group = groups.get(remoteName) ?? [];
    group.push(item);
    groups.set(remoteName, group);
  }
  return [...groups.entries()].map(([name, groupBranches]) => ({ name, branches: groupBranches }));
});
const logRefSearchQuery = computed(() => logRefSearch.value.trim().toLocaleLowerCase());
const logRefFiltering = computed(() => Boolean(logRefSearchQuery.value));
const showLogHeadRef = computed(() => {
  const query = logRefSearchQuery.value;
  return (
    !logFavoriteRefsOnly.value &&
    logRefMatches(query, logHeadLabel.value, branch.value?.currentBranch, branch.value?.head, "head")
  );
});
const visibleLogLocalBranches = computed<BranchInfo[]>(() => {
  const query = logRefSearchQuery.value;
  const localBranches = logFavoriteRefsOnly.value
    ? branches.sortedLocalBranches.filter((item) => isLogBranchFavorite(item))
    : branches.sortedLocalBranches;
  if (!query || logRefMatches(query, "本地", "local")) return localBranches;
  return localBranches.filter((item) => logRefMatches(query, item.name, item.fullName, item.upstream, item.target));
});
const visibleLogRemoteGroups = computed<LogRemoteGroup[]>(() => {
  const query = logRefSearchQuery.value;
  const remoteGroups = logRemoteGroups.value
    .map((group) => ({
      ...group,
      branches: logFavoriteRefsOnly.value
        ? group.branches.filter((item) => isLogBranchFavorite(item))
        : group.branches,
    }))
    .filter((group) => group.branches.length > 0);
  if (!query) return remoteGroups;
  return remoteGroups
    .map((group) => {
      if (logRefMatches(query, group.name, "远端", "remote")) return group;
      return {
        ...group,
        branches: group.branches.filter((item) =>
          logRefMatches(query, item.name, item.fullName, item.upstream, item.target),
        ),
      };
    })
    .filter((group) => group.branches.length > 0);
});
const visibleLogTags = computed<TagInfo[]>(() => {
  const query = logRefSearchQuery.value;
  const tags = (branches.list?.tags ?? []).filter(
    (item) =>
      !logFavoriteRefsOnly.value ||
      branches.isFavorite(item.name) ||
      branches.isFavorite(`refs/tags/${item.name}`),
  );
  if (!query || logRefMatches(query, "标签", "tag", "tags")) return tags;
  return tags.filter((item) => logRefMatches(query, item.name, item.target));
});
const activeLogBranchRef = computed<BranchInfo | null>(() => {
  const refName = history.branchFilter || branch.value?.currentBranch || "";
  if (!refName) return null;
  return (
    (branches.list?.branches ?? []).find(
      (item) => item.name === refName || item.fullName === refName || formatRefName(item.fullName) === refName,
    ) ?? null
  );
});
const activeLogBranchFavorite = computed(() => {
  const refName = activeLogBranchRef.value?.fullName;
  return Boolean(refName && branches.isFavorite(refName));
});
const logRefGroupsFullyExpanded = computed(() => {
  const keys: LogRefGroupKey[] = ["local", "remote", "tags", ...logRemoteGroups.value.map((group) => logRemoteGroupKey(group.name))];
  return keys.every((key) => isLogRefGroupExpanded(key));
});
const hasVisibleLogRefs = computed(
  () =>
    showLogHeadRef.value ||
    visibleLogLocalBranches.value.length > 0 ||
    visibleLogRemoteGroups.value.length > 0 ||
    visibleLogTags.value.length > 0,
);
const logGraphRows = computed<LogGraphRow[]>(() => buildLogGraphRows(history.commits));
const commitFileTreeRows = computed<LogFileTreeRow[]>(() => buildCommitFileTreeRows(history.details?.files ?? []));
const visibleCommitFileTreeRows = computed<LogFileTreeRow[]>(() => {
  const hiddenDirectories = new Set<string>();
  const rows: LogFileTreeRow[] = [];

  for (const row of commitFileTreeRows.value) {
    if (row.parent && hiddenDirectories.has(row.parent)) {
      if (row.directory) hiddenDirectories.add(row.path);
      continue;
    }

    rows.push(row);
    if (row.directory && !isCommitFileDirectoryExpanded(row.path)) {
      hiddenDirectories.add(row.path);
    }
  }

  return rows;
});
const logAuthorOptions = computed<LogAuthorOption[]>(() => {
  const options = new Map<string, LogAuthorOption>();

  for (const commitItem of history.authorCandidates) {
    const value = formatAuthorFilterValue(commitItem);
    const existing = options.get(value);
    if (existing) {
      existing.count += 1;
      continue;
    }
    options.set(value, {
      value,
      label: commitItem.authorName || commitItem.authorEmail || "未知作者",
      meta: commitItem.authorEmail,
      count: 1,
    });
  }

  for (const value of history.authorFilters) {
    if (options.has(value)) continue;
    options.set(value, {
      value,
      label: displayAuthorFilterValue(value),
      meta: "",
      count: 0,
    });
  }

  return [...options.values()].sort((left, right) =>
    left.label.localeCompare(right.label, undefined, { sensitivity: "base" }),
  );
});
const logAuthorFilterLabel = computed(() => {
  const count = history.authorFilters.length;
  if (count === 0) return "作者";
  if (count === 1) return displayAuthorFilterValue(history.authorFilters[0]);
  return `${count} 位作者`;
});
const logFileFilterLabel = computed(() => {
  const count = history.pathFilters.length;
  if (count === 0) return "文件";
  if (count === 1) return shortProjectPathLabel(history.pathFilters[0]);
  return `${count} 个文件`;
});
const visibleLogFilePickerRows = computed<ProjectFileEntry[]>(() => {
  const root = projectRootEntry.value;
  if (!root) return [];

  const query = logFilePickerSearch.value.trim().toLowerCase();
  if (query) {
    return [
      root,
      ...project.files
        .filter((file) => !file.directory && file.path.toLowerCase().includes(query))
        .sort(compareProjectTreeEntries),
    ];
  }

  const rows: ProjectFileEntry[] = [root];
  const appendChildren = (parentPath: string) => {
    if (!isLogFilePickerDirectoryExpanded(parentPath)) return;
    for (const child of projectChildrenByParent.value.get(parentPath) ?? []) {
      rows.push(child);
      if (child.directory) {
        appendChildren(child.path);
      }
    }
  };
  appendChildren(PROJECT_ROOT_PATH);
  return rows;
});
const selectedCommitRefs = computed(() => history.details?.commit.refs.map(formatRefName) ?? []);
const activeLogRefLabel = computed(() => history.branchFilter || "全部引用");
const logFilterActive = computed(() =>
  Boolean(
    history.branchFilter ||
      history.query.trim() ||
      history.authorFilters.length ||
      history.pathFilters.length,
  ),
);
const logFileContextMenu = ref<LogFileContextMenu | null>(null);
const logRefContextMenu = ref<LogRefContextMenu | null>(null);
const commitFileSignature = computed(() =>
  history.details
    ? `${history.details.commit.oid}:${history.details.files
        .map((file) => `${file.status}:${file.oldPath ?? ""}:${file.path}`)
        .join("\u0000")}`
    : "",
);

watch(
  () => [changes.selectedFile, changes.selectedSide],
  () => {
    diff.loadSelected().catch(() => undefined);
  },
);

watch(
  () => diff.current?.text,
  () => {
    activeChangeDiffHunkIndex.value = diff.current?.hunks[0]?.index ?? null;
  },
);

watch(
  () => activeLogDiffTab.value?.id,
  () => {
    activeLogDiffHunkIndex.value = 0;
    logDiffScroller.value?.scrollTo({ top: 0, left: 0 });
  },
);

watch(
  () => activeLogDiffTab.value?.diff?.text,
  () => {
    activeLogDiffHunkIndex.value = 0;
  },
);

watch(
  () => project.selectedPath,
  () => {
    expandedProjectHunkIndex.value = null;
    projectEditorScrollTop.value = 0;
    projectEditorScrollLeft.value = 0;
    if (projectEditorTextarea.value) {
      projectEditorTextarea.value.scrollTop = 0;
      projectEditorTextarea.value.scrollLeft = 0;
    }
  },
);

watch(
  () => project.diff?.text,
  () => {
    projectEditorScrollTop.value = projectEditorTextarea.value?.scrollTop ?? 0;
    if (
      expandedProjectHunkIndex.value !== null &&
      !projectEditorHunks.value.some((hunk) => hunk.index === expandedProjectHunkIndex.value)
    ) {
      expandedProjectHunkIndex.value = null;
    }
  },
);

watch(
  commitFileSignature,
  () => {
    selectedCommitFilePaths.value = history.details?.files.map((file) => file.path) ?? [];
    expandedCommitFileDirectories.value = {};
    logFileContextMenu.value = null;
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

watch(
  activeNotice,
  (message) => {
    if (!message) return;
    showNoticeToast(message);
    clearNoticeSources();
  },
  { flush: "post" },
);

watch(workbenchMode, (mode) => {
  if (mode === "advanced") {
    loadAdvancedSnapshots().catch(() => undefined);
  } else if (mode === "project") {
    project.refresh().catch(() => undefined);
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
  } else if (repos.selectedPath) {
    prepareUninitializedProject();
  }
  scheduleAutoFetch();
});

onUnmounted(() => {
  stopSystemThemeWatch?.();
  clearAutoFetchTimer();
  clearNoticeToastTimer();
});

function clearNoticeToastTimer() {
  if (noticeToastTimer !== null && typeof window !== "undefined") {
    window.clearTimeout(noticeToastTimer);
  }
  noticeToastTimer = null;
}

function showNoticeToast(message: string) {
  const id = noticeToastId + 1;
  noticeToastId = id;
  noticeToast.value = { id, message };
  clearNoticeToastTimer();
  if (typeof window !== "undefined") {
    noticeToastTimer = window.setTimeout(() => {
      dismissNoticeToast(id);
    }, 3200);
  }
}

function dismissNoticeToast(id?: number) {
  if (id !== undefined && noticeToast.value?.id !== id) return;
  clearNoticeToastTimer();
  noticeToast.value = null;
}

function clearNoticeSources() {
  operations.notice = "";
  advanced.notice = "";
  branches.notice = "";
  changes.notice = "";
  remote.notice = "";
}

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

  await repos.openMany(paths);
  await loadSelectedProject();
}

async function initSelectedProject() {
  if (!repos.selectedPath) return;
  advanced.initDirectory = repos.selectedPath;
  const repo = await advanced.initAt();
  if (!repo) return;
  repos.setCurrent(repo);
  await loadCurrentRepository();
}

async function switchRepository(path: string) {
  if (repos.selectedPath === path) return;
  await repos.select(path);
  await loadSelectedProject();
}

async function removeRepository(path: string) {
  const wasCurrent = repos.selectedPath === path;
  repos.remove(path);
  if (!wasCurrent) return;

  await loadSelectedProject();
}

async function loadSelectedProject() {
  if (repos.current) {
    await loadCurrentRepository();
  } else {
    prepareUninitializedProject();
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
  if (workbenchMode.value === "project") {
    await project.refresh();
  }
}

function clearProjectView() {
  clearLogDiffTabs();
  advanced.resetForRepositorySwitch();
  branches.resetForRepositorySwitch();
  changelists.resetForRepositorySwitch();
  changes.resetForRepositorySwitch();
  history.resetForRepositorySwitch();
  operations.resetForRepositorySwitch();
  project.resetForRepositorySwitch();
  diff.current = null;
  diff.error = "";
  remote.error = "";
  remote.notice = "";
  commit.error = "";
  commit.lastCommit = "";
}

function prepareUninitializedProject() {
  clearProjectView();
  advanced.initDirectory = repos.selectedPath;
  workbenchMode.value = "project";
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
  if (workbenchMode.value === "project") {
    await project.refresh();
  }
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
  } else if (workbenchMode.value === "project") {
    await project.refresh();
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

function selectChangeFileForContext(file: ChangedFile, side: ChangeSide) {
  if (!changes.selectedPaths.includes(file.path)) {
    selectFile(file, side);
    return;
  }

  changes.selectedFile = file.path;
  changes.selectedSide = side;
  settings.setSide(side);
}

function openChangeFileContextMenu(file: ChangedFile, side: ChangeSide, event: MouseEvent) {
  closeChangeListContextMenu();
  selectChangeFileForContext(file, side);
  const menuWidth = 260;
  const menuHeight = Math.min(420, 258 + changelists.lists.length * 28);
  changeFileContextMenu.value = {
    file,
    side,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function closeChangeFileContextMenu() {
  changeFileContextMenu.value = null;
}

function openChangeListContextMenu(listId: string | null | undefined, event: MouseEvent) {
  closeChangeFileContextMenu();
  const targetId = listId && changelists.lists.some((item) => item.id === listId)
    ? listId
    : changelists.activeId || "default";
  const menuWidth = 260;
  const menuHeight = 92;
  changeListContextMenu.value = {
    listId: targetId,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function closeChangeListContextMenu() {
  changeListContextMenu.value = null;
}

function changelistById(id: string) {
  return changelists.lists.find((item) => item.id === id) ?? changelists.activeList;
}

function canDeleteChangelist(id: string) {
  return id !== "default";
}

function changeContextPaths(file: ChangedFile) {
  return changes.selectedPaths.includes(file.path) ? changes.selectedPaths : [file.path];
}

function changeContextFiles(file: ChangedFile) {
  const paths = new Set(changeContextPaths(file));
  return changes.files.filter((item) => paths.has(item.path));
}

function changeContextLabel(file: ChangedFile) {
  const count = changeContextPaths(file).length;
  return count > 1 ? `${count} 个文件` : fileBaseName(file.path);
}

function canDeleteChangeFile(file: ChangedFile) {
  return !file.kind.split("|").includes("deleted");
}

function deletableChangeContextPaths(file: ChangedFile) {
  return changeContextFiles(file).filter(canDeleteChangeFile).map((item) => item.path);
}

function changelistForChangeContext(file: ChangedFile) {
  return changelists.listForPath(file.path);
}

function changelistMoveTargets(file: ChangedFile) {
  const currentIds = new Set(changeContextPaths(file).map((path) => changelists.listForPath(path).id));
  return changelists.lists.filter((list) => !(currentIds.size === 1 && currentIds.has(list.id)));
}

function validateChangelistName(value: string, editingId?: string) {
  const name = value.trim();
  if (!name) return "请输入变更清单名称";
  const exists = changelists.lists.some(
    (item) => item.id !== editingId && item.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
  );
  return exists ? "变更清单名称已存在" : "";
}

function expandChangelistGroup(id: string) {
  expandedChangeFileGroups.value = {
    ...expandedChangeFileGroups.value,
    [`changelist-${id}`]: true,
  };
}

async function showChangeFileDiffFromContext(file: ChangedFile, side: ChangeSide) {
  selectFile(file, side);
  workbenchMode.value = "changes";
  closeChangeFileContextMenu();
  await diff.loadSelected().catch(() => undefined);
}

async function discardChangeFilesFromContext(file: ChangedFile) {
  const paths = changeContextPaths(file);
  if (paths.length === 0) return;
  if (!window.confirm(`回滚 ${changeContextLabel(file)} 的本地变更？`)) return;
  closeChangeFileContextMenu();
  await runAndReload(() => changes.discardSelected());
}

function moveChangeFilesToChangelistFromContext(file: ChangedFile, listId: string) {
  const paths = changeContextPaths(file);
  changelists.movePaths(paths, listId);
  expandChangelistGroup(listId);
  changes.notice = `已移动 ${paths.length} 个文件到 ${changelists.lists.find((item) => item.id === listId)?.name ?? "变更清单"}`;
  closeChangeFileContextMenu();
}

async function createChangelistFromChangeContext(file: ChangedFile) {
  const paths = changeContextPaths(file);
  closeChangeFileContextMenu();
  const name = await promptProjectName("新建变更清单", "", (value) => validateChangelistName(value));
  if (!name) return;

  const id = changelists.createListFrom(name, "", true);
  if (!id) return;
  changelists.movePaths(paths, id);
  expandChangelistGroup(id);
  changes.notice = `已新建变更清单 ${name}`;
}

async function createChangelistFromListContext() {
  closeChangeListContextMenu();
  const name = await promptProjectName("新建变更清单", "", (value) => validateChangelistName(value));
  if (!name) return;

  const id = changelists.createListFrom(name, "", true);
  if (!id) return;
  expandChangelistGroup(id);
  changes.notice = `已新建变更清单 ${name}`;
}

async function editChangelistFromChangeContext(file: ChangedFile) {
  const list = changelistForChangeContext(file);
  closeChangeFileContextMenu();
  const name = await promptProjectName("编辑变更清单", list.name, (value) => validateChangelistName(value, list.id));
  if (!name || name === list.name) return;

  changelists.updateList(list.id, { name });
  expandChangelistGroup(list.id);
  changes.notice = `已更新变更清单 ${name}`;
}

async function editChangelistFromListContext(listId: string) {
  const list = changelistById(listId);
  closeChangeListContextMenu();
  const name = await promptProjectName("编辑变更清单", list.name, (value) => validateChangelistName(value, list.id));
  if (!name || name === list.name) return;

  changelists.updateList(list.id, { name });
  expandChangelistGroup(list.id);
  changes.notice = `已更新变更清单 ${name}`;
}

function deleteChangelistFromListContext(listId: string) {
  const list = changelistById(listId);
  if (!canDeleteChangelist(list.id)) return;
  if (!window.confirm(`删除变更清单 ${list.name}？其中的文件会移回默认变更。`)) return;

  closeChangeListContextMenu();
  changelists.deleteList(list.id);
  expandChangelistGroup("default");
  changes.notice = `已删除变更清单 ${list.name}`;
}

async function deleteChangeFilesFromContext(file: ChangedFile) {
  const paths = deletableChangeContextPaths(file);
  if (paths.length === 0 || !repos.path) return;
  if (!window.confirm(`删除 ${paths.length > 1 ? `${paths.length} 个文件` : paths[0]}？`)) return;

  closeChangeFileContextMenu();
  changes.loading = true;
  changes.error = "";
  try {
    for (const path of paths) {
      await deleteProjectEntry(repos.path, path);
    }
    changes.notice = `已删除 ${paths.length} 个文件`;
    await reloadAfterProjectFileOperation();
  } catch (error) {
    changes.error = String(error);
  } finally {
    changes.loading = false;
  }
}

async function showChangeFileHistoryFromContext(file: ChangedFile) {
  const paths = changeContextPaths(file);
  history.pathFilters = paths;
  activeLogTabId.value = LOG_TAB_ID;
  workbenchMode.value = "log";
  closeChangeFileContextMenu();
  await history.refresh().catch(() => undefined);
}

async function selectAdjacentChangeDiffFile(direction: -1 | 1) {
  const files = activeFiles.value;
  if (!files.length) return;

  const currentIndex = activeChangeDiffFileIndex.value >= 0 ? activeChangeDiffFileIndex.value : 0;
  const nextIndex = Math.min(Math.max(currentIndex + direction, 0), files.length - 1);
  if (nextIndex === activeChangeDiffFileIndex.value) return;

  selectFile(files[nextIndex], settings.selectedSide);
}

async function jumpChangeDiffHunk(direction: -1 | 1) {
  const hunks = diff.current?.hunks ?? [];
  if (!hunks.length) return;

  const currentIndex = hunks.findIndex((hunk) => hunk.index === activeChangeDiffHunkIndex.value);
  const nextIndex = ((currentIndex >= 0 ? currentIndex : 0) + direction + hunks.length) % hunks.length;
  activeChangeDiffHunkIndex.value = hunks[nextIndex].index;
  await nextTick();
  scrollSideBySideHunkIntoView(changeDiffScroller.value, hunks[nextIndex].index);
}

function commitFileRowFromChange(file: CommitFileChange): LogFileTreeRow {
  return {
    id: `file:${file.path}`,
    name: fileBaseName(file.path),
    path: file.path,
    parent: null,
    depth: 0,
    directory: false,
    status: file.status,
    oldPath: file.oldPath,
  };
}

async function selectAdjacentLogDiffFile(direction: -1 | 1) {
  const tab = activeLogDiffTab.value;
  const files = history.details?.files ?? [];
  if (!tab || !files.length) return;

  const currentIndex = activeLogDiffFileIndex.value >= 0 ? activeLogDiffFileIndex.value : 0;
  const nextIndex = Math.min(Math.max(currentIndex + direction, 0), files.length - 1);
  if (nextIndex === activeLogDiffFileIndex.value) return;

  await showCommitFileDiff(commitFileRowFromChange(files[nextIndex]), tab.mode);
}

async function jumpLogDiffHunk(direction: -1 | 1) {
  const count = activeLogDiffHunkCount.value;
  if (!count) return;

  const currentIndex =
    activeLogDiffHunkIndex.value >= 0 && activeLogDiffHunkIndex.value < count ? activeLogDiffHunkIndex.value : 0;
  const nextIndex = (currentIndex + direction + count) % count;
  activeLogDiffHunkIndex.value = nextIndex;
  await nextTick();
  scrollSideBySideHunkIntoView(logDiffScroller.value, nextIndex);
}

function syncSideBySideEditorScroll(event: Event) {
  const source = event.currentTarget as HTMLElement | null;
  if (!source) return;
  if (syncingSideBySideScroll.has(source)) {
    syncingSideBySideScroll.delete(source);
    return;
  }

  const group = source.closest<HTMLElement>(".side-by-side-editors");
  if (!group) return;

  for (const target of Array.from(group.querySelectorAll<HTMLElement>(".side-by-side-column"))) {
    if (target === source) continue;
    if (target.scrollTop === source.scrollTop && target.scrollLeft === source.scrollLeft) continue;
    syncingSideBySideScroll.add(target);
    target.scrollTop = source.scrollTop;
    target.scrollLeft = source.scrollLeft;
  }
}

function scrollSideBySideHunkIntoView(container: HTMLElement | null, hunkIndex: number) {
  const anchors = container?.querySelectorAll<HTMLElement>(`[data-hunk-anchor="${hunkIndex}"]`) ?? [];
  for (const anchor of Array.from(anchors)) {
    const column = anchor.closest<HTMLElement>(".side-by-side-column");
    if (!column) {
      anchor.scrollIntoView({ block: "center", inline: "nearest", behavior: "smooth" });
      continue;
    }
    const nextTop = anchor.offsetTop - column.clientHeight / 2 + anchor.clientHeight / 2;
    column.scrollTo({ top: Math.max(0, nextTop), behavior: "smooth" });
  }
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

function formatBytes(bytes?: number | null) {
  if (!bytes) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function fileIndent(depth: number) {
  return { paddingLeft: `${6 + depth * 14}px` };
}

function projectFileIndent(file: ProjectFileEntry) {
  return fileIndent(file.path === PROJECT_ROOT_PATH ? 0 : file.depth + 1);
}

function logFileIndent(depth: number) {
  return { paddingLeft: `${10 + depth * 15}px` };
}

function compareProjectTreeEntries(left: ProjectFileEntry, right: ProjectFileEntry) {
  if (left.directory !== right.directory) {
    return left.directory ? -1 : 1;
  }
  return left.name.toLocaleLowerCase().localeCompare(right.name.toLocaleLowerCase());
}

function projectLanguageForPath(path?: string | null) {
  const extension = path?.split(".").pop()?.toLowerCase() ?? "";
  if (["js", "jsx", "mjs", "cjs"].includes(extension)) return "javascript";
  if (["ts", "tsx"].includes(extension)) return "typescript";
  if (extension === "vue") return "vue";
  if (["json", "jsonc"].includes(extension)) return "json";
  if (["css", "scss", "sass", "less"].includes(extension)) return "css";
  if (["html", "xml", "svg"].includes(extension)) return "markup";
  if (extension === "rs") return "rust";
  if (extension === "toml") return "toml";
  if (["md", "markdown"].includes(extension)) return "markdown";
  if (["sh", "zsh", "bash"].includes(extension)) return "shell";
  return "plain";
}

function tokenizeProjectLine(content: string, language: string): ProjectCodeToken[] {
  const tokens: ProjectCodeToken[] = [];
  const keywords = projectKeywords(language);
  let index = 0;

  const push = (text: string, kind?: ProjectCodeToken["kind"]) => {
    if (text) tokens.push(kind ? { text, kind } : { text });
  };

  while (index < content.length) {
    const rest = content.slice(index);
    const char = content[index];

    if (rest.startsWith("//") || (rest.startsWith("#") && ["shell", "toml"].includes(language))) {
      push(rest, "comment");
      break;
    }

    if (rest.startsWith("/*")) {
      push(rest, "comment");
      break;
    }

    if (char === "\"" || char === "'" || char === "`") {
      let cursor = index + 1;
      while (cursor < content.length) {
        if (content[cursor] === "\\") {
          cursor += 2;
          continue;
        }
        if (content[cursor] === char) {
          cursor += 1;
          break;
        }
        cursor += 1;
      }
      push(content.slice(index, cursor), "string");
      index = cursor;
      continue;
    }

    const numberMatch = rest.match(/^\b\d+(?:\.\d+)?\b/);
    if (numberMatch) {
      push(numberMatch[0], "number");
      index += numberMatch[0].length;
      continue;
    }

    const wordMatch = rest.match(/^[A-Za-z_$][\w$-]*/);
    if (wordMatch) {
      const word = wordMatch[0];
      const before = content.slice(0, index).trimEnd();
      const after = content.slice(index + word.length).trimStart();
      if (keywords.has(word)) {
        push(word, "keyword");
      } else if (before.endsWith(".")) {
        push(word, "property");
      } else if (after.startsWith("(")) {
        push(word, "function");
      } else {
        push(word);
      }
      index += word.length;
      continue;
    }

    if (/^[{}()[\].,;:+\-*/%=!&|<>?@]/.test(char)) {
      push(char, "operator");
      index += 1;
      continue;
    }

    push(char);
    index += 1;
  }

  return tokens.length ? tokens : [{ text: " " }];
}

function projectKeywords(language: string) {
  const shared = [
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "let",
    "new",
    "null",
    "return",
    "static",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "while",
  ];
  const byLanguage: Record<string, string[]> = {
    rust: ["fn", "impl", "let", "match", "mod", "mut", "pub", "self", "struct", "trait", "use", "where"],
    css: ["important", "media", "supports"],
    json: ["false", "null", "true"],
    markup: ["DOCTYPE"],
    markdown: [],
    shell: ["do", "done", "elif", "fi", "for", "function", "if", "in", "then"],
    toml: ["false", "true"],
  };
  return new Set([...(byLanguage[language] ?? shared), ...(language === "plain" ? [] : shared)]);
}

function buildProjectEditorHunkView(hunk: DiffHunk, language: string): ProjectEditorHunkView {
  const parsed = parseProjectHunkPatch(hunk, language);
  const lineStart = Math.max(1, hunk.newStart || 1);
  const lineCount = Math.max(1, hunk.newLines || 0);
  const tone =
    parsed.deletedLines > 0 && parsed.addedLines > 0
      ? "modified"
      : parsed.deletedLines > 0
        ? "deleted"
        : "added";

  return {
    index: hunk.index,
    header: hunk.header,
    tone,
    lineStart,
    lineCount,
    oldStart: hunk.oldStart,
    oldLines: hunk.oldLines,
    newStart: hunk.newStart,
    newLines: hunk.newLines,
    addedLines: parsed.addedLines,
    deletedLines: parsed.deletedLines,
    originalLines: parsed.originalLines,
  };
}

function parseProjectHunkPatch(hunk: DiffHunk, language: string) {
  const originalLines: ProjectOriginalLine[] = [];
  let oldLineNumber = hunk.oldStart;
  let inHunk = false;
  let addedLines = 0;
  let deletedLines = 0;

  for (const line of hunk.patch.split("\n")) {
    if (line.startsWith("@@ ")) {
      inHunk = true;
      oldLineNumber = hunk.oldStart;
      continue;
    }
    if (!inHunk || line.startsWith("\\ No newline")) continue;

    const prefix = line.charAt(0);
    const content = line.slice(1);
    if (prefix === "-") {
      originalLines.push({
        index: originalLines.length,
        lineNumber: oldLineNumber,
        content,
        tokens: tokenizeProjectLine(content || " ", language),
      });
      oldLineNumber += 1;
      deletedLines += 1;
    } else if (prefix === "+") {
      addedLines += 1;
    } else if (prefix === " ") {
      oldLineNumber += 1;
    }
  }

  return { originalLines, addedLines, deletedLines };
}

function projectEditorHunkMarkerStyle(hunk: ProjectEditorHunkView) {
  const top =
    PROJECT_EDITOR_PADDING_TOP +
    (hunk.lineStart - 1) * PROJECT_EDITOR_LINE_HEIGHT -
    projectEditorScrollTop.value;
  const height =
    hunk.newLines === 0
      ? 10
      : Math.max(10, hunk.lineCount * PROJECT_EDITOR_LINE_HEIGHT - 2);
  return {
    top: `${top}px`,
    height: `${height}px`,
  };
}

function projectEditorOriginalPanelStyle(hunk: ProjectEditorHunkView) {
  const markerTop =
    PROJECT_EDITOR_PADDING_TOP +
    (hunk.lineStart - 1) * PROJECT_EDITOR_LINE_HEIGHT -
    projectEditorScrollTop.value;
  const markerHeight =
    hunk.newLines === 0
      ? 10
      : Math.max(10, hunk.lineCount * PROJECT_EDITOR_LINE_HEIGHT - 2);
  return {
    top: `${Math.max(6, markerTop + markerHeight + 4)}px`,
  };
}

function projectEditorHunkTitle(hunk: ProjectEditorHunkView) {
  const currentEnd = hunk.newLines > 0 ? hunk.newStart + hunk.newLines - 1 : hunk.newStart;
  const originalEnd = hunk.oldLines > 0 ? hunk.oldStart + hunk.oldLines - 1 : hunk.oldStart;
  return `当前 ${hunk.newStart}-${currentEnd}，原本 ${hunk.oldStart}-${originalEnd}`;
}

function toggleProjectEditorHunk(index: number) {
  expandedProjectHunkIndex.value = expandedProjectHunkIndex.value === index ? null : index;
}

function syncProjectEditorScroll(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  projectEditorScrollTop.value = target.scrollTop;
  projectEditorScrollLeft.value = target.scrollLeft;
}

async function discardProjectEditorHunk(index: number) {
  if (project.editorDirty && !window.confirm("当前文件有未保存编辑，撤回此块会放弃未保存内容并还原 Git 原本内容。继续？")) {
    return;
  }
  await project.discardHunk(index);
  expandedProjectHunkIndex.value = null;
}

async function saveProjectEditor() {
  await project.saveSelectedContent();
}

function closeProjectEditorTab(path: string) {
  const tab = project.openTabs.find((item) => item.path === path);
  if (!project.isPathDirty(path)) {
    project.closeTab(path).catch(() => undefined);
    return;
  }

  projectCloseDialog.value = {
    path,
    name: tab?.name ?? path,
    saving: false,
    error: "",
  };
}

function cancelProjectCloseDialog() {
  if (projectCloseDialog.value?.saving) return;
  projectCloseDialog.value = null;
}

async function discardAndCloseProjectFile() {
  const dialog = projectCloseDialog.value;
  if (!dialog || dialog.saving) return;

  await project.closeTab(dialog.path);
  projectCloseDialog.value = null;
}

async function saveAndCloseProjectFile() {
  const dialog = projectCloseDialog.value;
  if (!dialog || dialog.saving) return;

  dialog.saving = true;
  dialog.error = "";
  try {
    await project.saveContent(dialog.path);
    await project.closeTab(dialog.path);
    projectCloseDialog.value = null;
  } catch (error) {
    dialog.error = String(error);
    dialog.saving = false;
  }
}

function openProjectEntry(file: ProjectFileEntry) {
  if (file.directory) {
    project.toggleDirectory(file.path);
    return;
  }
  project.openFile(file.path).catch(() => undefined);
}

function projectDirectoryParam(path: string | null | undefined) {
  return !path || path === PROJECT_ROOT_PATH ? null : path;
}

function projectMenuTargetDirectory(file: ProjectFileEntry | null) {
  if (!file) return null;
  return file.directory ? projectDirectoryParam(file.path) : projectDirectoryParam(file.parent);
}

function ensureProjectDirectoryExpanded(path: string | null | undefined) {
  const directoryPath = path ?? PROJECT_ROOT_PATH;
  if (project.expandedPaths.includes(directoryPath)) return;
  project.expandedPaths = [...project.expandedPaths, directoryPath];
}

function projectClipboardParentPath(path: string) {
  const parts = path.split("/").filter(Boolean);
  parts.pop();
  return parts.join("/");
}

function canCreateInProjectContext(file: ProjectFileEntry | null) {
  return !file || file.directory;
}

function canModifyProjectEntry(file: ProjectFileEntry | null) {
  return Boolean(file && file.path !== PROJECT_ROOT_PATH);
}

function canPasteProjectEntry(file: ProjectFileEntry | null) {
  const item = projectFileClipboard.value;
  if (!item) return false;
  if (file && !file.directory) return false;

  const targetDirectory = projectMenuTargetDirectory(file) ?? "";
  if (item.mode === "cut" && targetDirectory === projectClipboardParentPath(item.path)) {
    return false;
  }
  if (item.directory && (targetDirectory === item.path || targetDirectory.startsWith(`${item.path}/`))) {
    return false;
  }
  return true;
}

function projectAbsolutePath(file: ProjectFileEntry) {
  const root = repos.current?.workdir ?? repos.path;
  if (!root || file.path === PROJECT_ROOT_PATH) return root;
  const separator = root.endsWith("/") || root.endsWith("\\") ? "" : "/";
  return `${root}${separator}${file.path}`;
}

function projectExistingChildNames(directoryPath: string | null | undefined) {
  const parent = directoryPath ?? PROJECT_ROOT_PATH;
  return new Set((projectChildrenByParent.value.get(parent) ?? []).map((file) => file.name.toLocaleLowerCase()));
}

function nextAvailableProjectName(directoryPath: string | null | undefined, baseName: string, extension = "") {
  const existing = projectExistingChildNames(directoryPath);
  for (let index = 0; index < 1000; index += 1) {
    const suffix = index === 0 ? "" : ` ${index + 1}`;
    const candidate = `${baseName}${suffix}${extension}`;
    if (!existing.has(candidate.toLocaleLowerCase())) return candidate;
  }
  return `${baseName}${extension}`;
}

function validateProjectCreatePath(value: string) {
  if (value.startsWith("/") || value.includes("\\") || value.split("/").some((part) => !part || part === "." || part === "..")) {
    return "名称不能是绝对路径，不能包含空路径、.、.. 或反斜杠";
  }
  return "";
}

function validateProjectRenameName(value: string) {
  if (value === "." || value === ".." || value.includes("/") || value.includes("\\")) {
    return "重命名只能修改当前名称，不能包含路径分隔符";
  }
  return "";
}

function promptProjectName(title: string, defaultValue: string, validate: (value: string) => string) {
  return new Promise<string | null>((resolve) => {
    projectNameDialog.value = {
      title,
      value: defaultValue,
      error: "",
      validate,
      resolve,
    };
  });
}

function submitProjectNameDialog() {
  const dialog = projectNameDialog.value;
  if (!dialog) return;

  const value = dialog.value.trim();
  if (!value) {
    dialog.error = "请输入名称";
    return;
  }

  const error = dialog.validate(value);
  if (error) {
    dialog.error = error;
    return;
  }

  dialog.resolve(value);
  projectNameDialog.value = null;
}

function cancelProjectNameDialog() {
  projectNameDialog.value?.resolve(null);
  projectNameDialog.value = null;
}

async function writeClipboardText(text: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    window.prompt("复制内容", text);
  }
}

async function reloadAfterProjectFileOperation() {
  await Promise.all([project.refresh(), changes.refresh()]);
  changelists.pruneMissingPaths(changes.files.map((file) => file.path));
  await diff.loadSelected().catch(() => undefined);
}

async function runProjectFileOperation<T>(operation: () => Promise<T>) {
  if (!repos.path) return null;
  project.error = "";
  try {
    return await operation();
  } catch (error) {
    project.error = String(error);
    return null;
  }
}

function openProjectFileContextMenu(file: ProjectFileEntry | null, event: MouseEvent) {
  const menuWidth = 260;
  const menuHeight = 326;
  projectFileContextMenu.value = {
    file,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function closeProjectFileContextMenu() {
  projectFileContextMenu.value = null;
}

function closeContextMenus() {
  closeChangeFileContextMenu();
  closeChangeListContextMenu();
  closeProjectFileContextMenu();
  closeLogFileContextMenu();
  closeLogRefContextMenu();
}

async function createProjectFileFromContext(file: ProjectFileEntry | null) {
  const directoryPath = projectMenuTargetDirectory(file);
  const defaultName = nextAvailableProjectName(directoryPath, "未命名文件", ".txt");
  closeProjectFileContextMenu();
  const name = await promptProjectName("新建文件", defaultName, validateProjectCreatePath);
  if (!name) return;

  ensureProjectDirectoryExpanded(directoryPath);
  const result = await runProjectFileOperation(() => createProjectFile(repos.path, directoryPath, name));
  if (!result) return;

  await reloadAfterProjectFileOperation();
  ensureProjectDirectoryExpanded(directoryPath);
  await project.openFile(result.path);
}

async function createProjectDirectoryFromContext(file: ProjectFileEntry | null) {
  const directoryPath = projectMenuTargetDirectory(file);
  const defaultName = nextAvailableProjectName(directoryPath, "新建文件夹");
  closeProjectFileContextMenu();
  const name = await promptProjectName("新建文件夹", defaultName, validateProjectCreatePath);
  if (!name) return;

  ensureProjectDirectoryExpanded(directoryPath);
  const result = await runProjectFileOperation(() => createProjectDirectory(repos.path, directoryPath, name));
  if (!result) return;

  await reloadAfterProjectFileOperation();
  ensureProjectDirectoryExpanded(directoryPath);
  ensureProjectDirectoryExpanded(result.path);
}

function cutProjectEntry(file: ProjectFileEntry) {
  if (!canModifyProjectEntry(file)) return;
  projectFileClipboard.value = {
    mode: "cut",
    path: file.path,
    name: file.name,
    directory: file.directory,
  };
  closeProjectFileContextMenu();
}

function copyProjectEntryToInternalClipboard(file: ProjectFileEntry) {
  if (!canModifyProjectEntry(file)) return;
  projectFileClipboard.value = {
    mode: "copy",
    path: file.path,
    name: file.name,
    directory: file.directory,
  };
  closeProjectFileContextMenu();
}

async function pasteProjectEntryToContext(file: ProjectFileEntry | null) {
  const item = projectFileClipboard.value;
  if (!item || !canPasteProjectEntry(file)) return;

  const targetDirectory = projectMenuTargetDirectory(file);
  const shouldOpenMovedFile = item.mode === "cut" && !item.directory && project.selectedPath === item.path;
  closeProjectFileContextMenu();
  ensureProjectDirectoryExpanded(targetDirectory);

  const result =
    item.mode === "cut"
      ? await runProjectFileOperation(() => moveProjectEntry(repos.path, item.path, targetDirectory))
      : await runProjectFileOperation(() => copyProjectEntry(repos.path, item.path, targetDirectory));
  if (!result) return;

  if (item.mode === "cut") {
    projectFileClipboard.value = null;
  }
  await reloadAfterProjectFileOperation();
  ensureProjectDirectoryExpanded(targetDirectory);
  if (result.directory) {
    ensureProjectDirectoryExpanded(result.path);
  } else if (shouldOpenMovedFile) {
    await project.openFile(result.path);
  }
}

async function copyProjectAbsolutePath(file: ProjectFileEntry) {
  await writeClipboardText(projectAbsolutePath(file));
  closeProjectFileContextMenu();
}

async function copyProjectRelativePath(file: ProjectFileEntry) {
  await writeClipboardText(file.path === PROJECT_ROOT_PATH ? "." : file.path);
  closeProjectFileContextMenu();
}

async function renameProjectEntryFromContext(file: ProjectFileEntry) {
  if (!canModifyProjectEntry(file)) return;
  closeProjectFileContextMenu();
  const newName = await promptProjectName("重命名", file.name, validateProjectRenameName);
  if (!newName || newName === file.name) {
    return;
  }

  const wasOpen = project.openPaths.includes(file.path);
  const wasSelected = project.selectedPath === file.path;
  const result = await runProjectFileOperation(() => renameProjectEntry(repos.path, file.path, newName));
  if (!result) return;

  await reloadAfterProjectFileOperation();
  if (!result.directory && (wasOpen || wasSelected)) {
    await project.openFile(result.path);
  }
}

async function deleteProjectEntryFromContext(file: ProjectFileEntry) {
  if (!canModifyProjectEntry(file)) return;
  const message = file.directory ? `删除文件夹 ${file.name} 及其所有内容？` : `删除文件 ${file.name}？`;
  if (!window.confirm(message)) return;

  closeProjectFileContextMenu();
  const result = await runProjectFileOperation(() => deleteProjectEntry(repos.path, file.path));
  if (!result) return;

  if (projectFileClipboard.value?.path === file.path || projectFileClipboard.value?.path.startsWith(`${file.path}/`)) {
    projectFileClipboard.value = null;
  }
  await reloadAfterProjectFileOperation();
}

async function openProjectEntryLog(file: ProjectFileEntry) {
  history.pathFilters = file.path === PROJECT_ROOT_PATH ? [] : [file.path];
  activeLogTabId.value = LOG_TAB_ID;
  workbenchMode.value = "log";
  closeProjectFileContextMenu();
  await history.refresh().catch(() => undefined);
}

function normalizeProjectGitStatus(file: ChangedFile): ProjectGitStatus {
  let status: ProjectGitStatus = "unknown";
  for (const part of file.kind.split("|")) {
    if (isProjectGitStatus(part) && projectStatusPriority[part] > projectStatusPriority[status]) {
      status = part;
    }
  }
  if (file.conflicted) return "conflicted";
  if (file.ignored) return "ignored";
  return status;
}

function isProjectGitStatus(value: string): value is ProjectGitStatus {
  return value in projectStatusPriority;
}

function setProjectGitStatus(
  statuses: Map<string, ProjectGitStatus>,
  path: string,
  status: ProjectGitStatus,
) {
  const current = statuses.get(path);
  if (!current || projectStatusPriority[status] > projectStatusPriority[current]) {
    statuses.set(path, status);
  }
}

function projectStatusForPath(path: string) {
  return projectStatusByPath.value.get(path) ?? "";
}

function projectStatusLabel(status: ProjectGitStatus | "") {
  if (!status) return "";
  return statusKindLabels[status] ?? status;
}

function projectFileTitle(file: ProjectFileEntry) {
  const label = projectStatusLabel(projectStatusForPath(file.path));
  const dirty = !file.directory && project.isPathDirty(file.path);
  if (file.path === PROJECT_ROOT_PATH) {
    return label ? `${repos.path} · ${label}` : repos.path;
  }
  return [file.path, dirty ? "未保存" : "", label].filter(Boolean).join(" · ");
}

function projectFileClass(file: ProjectFileEntry) {
  const status = projectStatusForPath(file.path);
  return {
    active: project.selectedPath === file.path,
    directory: file.directory,
    expanded: file.directory && project.isExpanded(file.path),
    root: file.path === PROJECT_ROOT_PATH,
    [`status-${status}`]: Boolean(status),
  };
}

function projectTabClass(file: ProjectFileEntry) {
  const status = projectStatusForPath(file.path);
  return {
    active: project.selectedPath === file.path,
    dirty: project.isPathDirty(file.path),
    [`status-${status}`]: Boolean(status),
  };
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

function isChangeFileGroupExpanded(key: string) {
  return expandedChangeFileGroups.value[key] ?? true;
}

function toggleChangeFileGroup(key: string) {
  expandedChangeFileGroups.value = {
    ...expandedChangeFileGroups.value,
    [key]: !isChangeFileGroupExpanded(key),
  };
}

function isChangeFileGroupSelected(files: ChangedFile[]) {
  return files.length > 0 && files.every((file) => changes.selectedPaths.includes(file.path));
}

function isChangeFileGroupPartiallySelected(files: ChangedFile[]) {
  return files.some((file) => changes.selectedPaths.includes(file.path)) && !isChangeFileGroupSelected(files);
}

function toggleChangeFileGroupSelection(files: ChangedFile[]) {
  const groupPaths = files.map((file) => file.path);
  const groupPathSet = new Set(groupPaths);

  if (isChangeFileGroupSelected(files)) {
    changes.selectedPaths = changes.selectedPaths.filter((path) => !groupPathSet.has(path));
    return;
  }

  changes.selectedPaths = [...new Set([...changes.selectedPaths, ...groupPaths])];
  if (!changes.selectedFile && groupPaths[0]) {
    changes.selectedFile = groupPaths[0];
  }
}

function fileDirectoryName(path: string) {
  const parts = path.split("/").filter(Boolean);
  if (parts.length <= 1) return "";
  return parts.slice(0, -1).join("/");
}

function fileContextPath(path: string) {
  return fileDirectoryName(path) || repos.name;
}

function fileExtension(path: string) {
  const name = fileBaseName(path);
  const dotIndex = name.lastIndexOf(".");
  if (dotIndex <= 0 || dotIndex === name.length - 1) return "";
  return name.slice(dotIndex + 1).toLowerCase();
}

function fileTypeLabel(path: string) {
  return changeFileIconLabels[fileExtension(path)] ?? "";
}

function changeFileIconClass(path: string) {
  const extension = fileExtension(path) || "file";
  return {
    [`ext-${extension}`]: true,
    labeled: Boolean(fileTypeLabel(path)),
  };
}

function formatCommitTime(seconds: number) {
  return new Date(seconds * 1000).toLocaleString();
}

function formatCompactCommitTime(seconds: number) {
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "numeric",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(seconds * 1000));
}

function formatRefName(ref: string) {
  return ref.replace(/^refs\/heads\//, "").replace(/^refs\/remotes\//, "").replace(/^refs\/tags\//, "");
}

function formatAuthorFilterValue(commitItem: CommitSummary) {
  if (commitItem.authorEmail) {
    return `${commitItem.authorName || "未知作者"} <${commitItem.authorEmail}>`;
  }
  return commitItem.authorName || "未知作者";
}

function displayAuthorFilterValue(value: string) {
  return value.replace(/\s*<[^>]+>\s*$/, "").trim() || value;
}

function shortProjectPathLabel(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function isLogAuthorSelected(value: string) {
  return history.authorFilters.includes(value);
}

function toggleLogAuthorPicker() {
  logAuthorPickerOpen.value = !logAuthorPickerOpen.value;
}

function toggleLogAuthorFilter(value: string) {
  if (isLogAuthorSelected(value)) {
    history.authorFilters = history.authorFilters.filter((item) => item !== value);
  } else {
    history.authorFilters = [...history.authorFilters, value];
  }
  history.refresh().catch(() => undefined);
}

function clearLogAuthorFilters() {
  if (history.authorFilters.length === 0) return;
  history.authorFilters = [];
  history.refresh().catch(() => undefined);
}

async function openLogFilePicker() {
  logFilePickerOpen.value = true;
  logFilePickerSearch.value = "";
  logFilePickerDraft.value = [...history.pathFilters];
  expandedLogFilePickerDirectories.value = {
    ...expandedLogFilePickerDirectories.value,
    [PROJECT_ROOT_PATH]: true,
  };
  if (repos.current && project.files.length === 0) {
    await project.refresh();
  }
}

function closeLogFilePicker() {
  logFilePickerOpen.value = false;
}

function isLogFilePickerDirectoryExpanded(path: string) {
  return expandedLogFilePickerDirectories.value[path] ?? path === PROJECT_ROOT_PATH;
}

function toggleLogFilePickerDirectory(path: string) {
  expandedLogFilePickerDirectories.value = {
    ...expandedLogFilePickerDirectories.value,
    [path]: !isLogFilePickerDirectoryExpanded(path),
  };
}

function isLogFileFilterSelected(path: string) {
  return logFilePickerDraft.value.includes(path);
}

function toggleLogFileFilter(path: string) {
  if (isLogFileFilterSelected(path)) {
    logFilePickerDraft.value = logFilePickerDraft.value.filter((item) => item !== path);
  } else {
    logFilePickerDraft.value = [...logFilePickerDraft.value, path];
  }
}

function applyLogFileFilters() {
  history.pathFilters = [...logFilePickerDraft.value];
  closeLogFilePicker();
  history.refresh().catch(() => undefined);
}

function clearLogFilePickerDraft() {
  logFilePickerDraft.value = [];
}

function logFilePickerRowClass(file: ProjectFileEntry) {
  return {
    directory: file.directory,
    root: file.path === PROJECT_ROOT_PATH,
    selected: !file.directory && isLogFileFilterSelected(file.path),
  };
}

function logFilePickerIndent(file: ProjectFileEntry) {
  if (file.path === PROJECT_ROOT_PATH) return fileIndent(0);
  return fileIndent(file.depth + 1);
}

function shortRemoteBranchName(name: string, remoteName: string) {
  return name.startsWith(`${remoteName}/`) ? name.slice(remoteName.length + 1) : name;
}

function isLogBranchFavorite(item: BranchInfo) {
  return branches.isFavorite(item.fullName) || branches.isFavorite(item.name);
}

function logRefMatches(query: string, ...values: Array<string | null | undefined>) {
  if (!query) return true;
  return values.some((value) => value?.toLocaleLowerCase().includes(query));
}

function isLogRefActive(refName: string) {
  return history.branchFilter === refName;
}

function logRemoteGroupKey(name: string): LogRefGroupKey {
  return `remote:${name}`;
}

function isLogRefGroupExpanded(key: LogRefGroupKey) {
  return expandedLogRefGroups.value[key] ?? true;
}

function toggleLogRefGroup(key: LogRefGroupKey) {
  expandedLogRefGroups.value = {
    ...expandedLogRefGroups.value,
    [key]: !isLogRefGroupExpanded(key),
  };
}

function toggleLogRefPanelCollapsed() {
  logRefPanelCollapsed.value = !logRefPanelCollapsed.value;
}

function focusLogRefSearch() {
  logRefPanelCollapsed.value = false;
  nextTick(() => logRefSearchInput.value?.focus());
}

function toggleLogFavoriteRefsOnly() {
  logFavoriteRefsOnly.value = !logFavoriteRefsOnly.value;
  logRefPanelCollapsed.value = false;
}

function toggleAllLogRefGroups() {
  const expanded = !logRefGroupsFullyExpanded.value;
  const nextGroups: Record<string, boolean> = {
    local: expanded,
    remote: expanded,
    tags: expanded,
  };
  for (const group of logRemoteGroups.value) {
    nextGroups[logRemoteGroupKey(group.name)] = expanded;
  }
  expandedLogRefGroups.value = {
    ...expandedLogRefGroups.value,
    ...nextGroups,
  };
}

function clearLogRef() {
  history.branchFilter = "";
  history.refresh().catch(() => undefined);
}

function clearLogRefContext() {
  logRefSearch.value = "";
  clearLogRef();
}

function selectLogRef(refName: string) {
  history.branchFilter = refName;
  history.refresh().catch(() => undefined);
}

function contextMenuPoint(event: MouseEvent, menuWidth = 260, menuHeight = 320) {
  return {
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function openLogBranchContextMenu(branchItem: BranchInfo, event: MouseEvent) {
  closeContextMenus();
  logRefContextMenu.value = {
    kind: branchItem.branchType === "remote" ? "remote" : "local",
    branch: branchItem,
    ...contextMenuPoint(event, 270, branchItem.branchType === "remote" ? 282 : 306),
  };
}

function openLogTagContextMenu(tag: TagInfo, event: MouseEvent) {
  closeContextMenus();
  logRefContextMenu.value = {
    kind: "tag",
    tag,
    ...contextMenuPoint(event, 270, 248),
  };
}

function closeLogRefContextMenu() {
  logRefContextMenu.value = null;
}

function logRefContextBranchItem(menu: LogRefContextMenu | null) {
  return menu?.kind === "local" || menu?.kind === "remote" ? menu.branch : null;
}

function logRefContextTagItem(menu: LogRefContextMenu | null) {
  return menu?.kind === "tag" ? menu.tag : null;
}

function logRefContextRefName(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  return branchItem?.name ?? logRefContextTagItem(menu)?.name ?? "";
}

function logRefContextFullName(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  return branchItem?.fullName ?? (menu?.kind === "tag" ? `refs/tags/${menu.tag.name}` : "");
}

function logRefContextFavoriteKey(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  if (branchItem) {
    if (branches.isFavorite(branchItem.fullName)) return branchItem.fullName;
    if (branches.isFavorite(branchItem.name)) return branchItem.name;
    return branchItem.fullName;
  }
  if (menu?.kind !== "tag") return "";
  const tagFullName = `refs/tags/${menu.tag.name}`;
  if (branches.isFavorite(tagFullName)) return tagFullName;
  if (branches.isFavorite(menu.tag.name)) return menu.tag.name;
  return tagFullName;
}

function isLogRefContextFavorite(menu: LogRefContextMenu | null) {
  const key = logRefContextFavoriteKey(menu);
  return Boolean(key && branches.isFavorite(key));
}

function isLogBranchContextTarget(branchItem: BranchInfo) {
  return logRefContextMenu.value?.kind !== "tag" && logRefContextMenu.value?.branch.fullName === branchItem.fullName;
}

function isLogTagContextTarget(tag: TagInfo) {
  return logRefContextMenu.value?.kind === "tag" && logRefContextMenu.value.tag.name === tag.name;
}

function canCheckoutLogRefContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  return menu?.kind === "tag" || Boolean(branchItem && !branchItem.current);
}

function canMergeOrRebaseLogRefContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  return Boolean(branchItem && !branchItem.current && branch.value?.currentBranch);
}

function canRenameLogRefContext(menu: LogRefContextMenu | null) {
  return menu?.kind === "local";
}

function canDeleteLogRefContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  return menu?.kind === "tag" || Boolean(branchItem && !branchItem.current);
}

function canSetLogRefContextUpstream(menu: LogRefContextMenu | null) {
  return menu?.kind === "remote" && Boolean(branch.value?.currentBranch);
}

function branchStartPointName(menu: LogRefContextMenu | null) {
  return logRefContextRefName(menu);
}

function branchNameBaseFromRef(menu: LogRefContextMenu | null) {
  if (menu?.kind === "remote") {
    const remoteName = menu.branch.name.split("/")[0] || "";
    return `${shortRemoteBranchName(menu.branch.name, remoteName)}-local`;
  }
  const refName = logRefContextRefName(menu).replace(/^refs\/tags\//, "");
  return menu?.kind === "tag" ? `${refName}-branch` : `${refName}-copy`;
}

function nextAvailableBranchName(baseName: string) {
  const base = baseName
    .trim()
    .replace(/^\-+/, "")
    .replace(/[\s~^:?*\[\\\]]+/g, "-")
    .replace(/\/+/g, "/")
    .replace(/[/.]+$/g, "");
  const fallback = base || "new-branch";
  if (!validateBranchName(fallback)) return fallback;
  for (let index = 2; index < 1000; index += 1) {
    const candidate = `${fallback}-${index}`;
    if (!validateBranchName(candidate)) return candidate;
  }
  return fallback;
}

function showLogRefFromContext(menu: LogRefContextMenu | null) {
  const refName = logRefContextRefName(menu);
  if (!refName) return;
  closeLogRefContextMenu();
  activeLogTabId.value = LOG_TAB_ID;
  workbenchMode.value = "log";
  selectLogRef(refName);
}

async function checkoutLogRefFromContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  const tag = logRefContextTagItem(menu);
  if (branchItem) {
    closeLogRefContextMenu();
    await checkoutSelectedBranch(branchItem);
    return;
  }
  if (!tag) return;
  if (!window.confirm(`检出标签 ${tag.name} 为游离 HEAD？`)) return;
  closeLogRefContextMenu();
  await advanced.checkoutDetached(tag.name);
  await loadCurrentRepository();
}

async function createBranchFromLogRefContext(menu: LogRefContextMenu | null) {
  const startPoint = branchStartPointName(menu);
  if (!startPoint) return;
  const defaultName = nextAvailableBranchName(branchNameBaseFromRef(menu));
  closeLogRefContextMenu();
  const name = await promptProjectName(`从 ${startPoint} 新建分支`, defaultName, validateBranchName);
  if (!name) return;
  const previousFilter = history.branchFilter;
  history.branchFilter = name;
  try {
    await branches.create(name, true, startPoint);
    await loadCurrentRepository();
  } catch (error) {
    history.branchFilter = previousFilter;
    throw error;
  }
}

async function renameLogBranchFromContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  if (!branchItem || menu?.kind !== "local") return;
  closeLogRefContextMenu();
  const newName = await promptProjectName("重命名分支", branchItem.name, (value) =>
    validateBranchName(value, branchItem.name),
  );
  if (!newName || newName === branchItem.name) return;
  if (!window.confirm(`将分支 ${branchItem.name} 重命名为 ${newName}？`)) return;
  await branches.rename(branchItem.name, newName);
  if (history.branchFilter === branchItem.name) {
    history.branchFilter = newName;
  }
  await loadCurrentRepository();
}

async function deleteLogRefFromContext(menu: LogRefContextMenu | null) {
  const branchItem = logRefContextBranchItem(menu);
  const tag = logRefContextTagItem(menu);
  const refName = logRefContextRefName(menu);
  if (branchItem) {
    if (branchItem.current) return;
    const confirmed =
      branchItem.branchType === "remote"
        ? window.confirm(`删除远程分支 ${branchItem.name}？这会推送删除到远程仓库。`)
        : window.confirm(`删除本地分支 ${branchItem.name}？`);
    if (!confirmed) return;
    const previousFilter = history.branchFilter;
    if (history.branchFilter === refName) history.branchFilter = "";
    closeLogRefContextMenu();
    try {
      if (branchItem.branchType === "remote") {
        await branches.deleteRemote(branchItem.name);
      } else {
        await branches.delete(branchItem.name, false);
      }
      await loadCurrentRepository();
    } catch (error) {
      history.branchFilter = previousFilter;
      throw error;
    }
    return;
  }
  if (!tag || !window.confirm(`删除本地标签 ${tag.name}？`)) return;
  const previousFilter = history.branchFilter;
  if (history.branchFilter === refName) history.branchFilter = "";
  closeLogRefContextMenu();
  try {
    await branches.deleteTag(tag.name);
    await loadCurrentRepository();
  } catch (error) {
    history.branchFilter = previousFilter;
    throw error;
  }
}

async function mergeLogRefIntoCurrent(menu: LogRefContextMenu | null) {
  const target = logRefContextBranchItem(menu)?.name ?? "";
  if (!target || !canMergeOrRebaseLogRefContext(menu)) return;
  if (!window.confirm(`将 ${target} 合并到当前分支？`)) return;
  closeLogRefContextMenu();
  operations.mergeTarget = target;
  await operations.merge();
  await reloadAfterGitOperation();
}

async function rebaseCurrentOntoLogRef(menu: LogRefContextMenu | null) {
  const target = logRefContextBranchItem(menu)?.name ?? "";
  if (!target || !canMergeOrRebaseLogRefContext(menu)) return;
  if (!window.confirm(`将当前分支变基到 ${target}？`)) return;
  closeLogRefContextMenu();
  operations.rebaseTarget = target;
  await operations.rebase();
  await reloadAfterGitOperation();
}

async function setCurrentBranchUpstreamFromContext(menu: LogRefContextMenu | null) {
  const target = logRefContextBranchItem(menu)?.name ?? "";
  const current = branch.value?.currentBranch;
  if (!target || !current || menu?.kind !== "remote") return;
  if (!window.confirm(`将 ${current} 的上游设置为 ${target}？`)) return;
  closeLogRefContextMenu();
  await branches.setUpstream(current, target);
  await loadCurrentRepository();
}

function toggleLogRefFavoriteFromContext(menu: LogRefContextMenu | null) {
  const key = logRefContextFavoriteKey(menu);
  if (!key) return;
  branches.toggleFavorite(key);
  closeLogRefContextMenu();
}

async function copyLogRefNameFromContext(menu: LogRefContextMenu | null) {
  const refName = logRefContextRefName(menu);
  if (!refName) return;
  await writeClipboardText(refName);
  branches.notice = `已复制 ${refName}`;
  closeLogRefContextMenu();
}

async function pushLogTagFromContext(menu: LogRefContextMenu | null) {
  const tag = logRefContextTagItem(menu);
  if (!tag) return;
  closeLogRefContextMenu();
  await pushSelectedTag(tag);
}

async function deleteRemoteLogTagFromContext(menu: LogRefContextMenu | null) {
  const tag = logRefContextTagItem(menu);
  if (!tag) return;
  closeLogRefContextMenu();
  await deleteSelectedRemoteTag(tag);
}

function validateBranchName(value: string, existingLocalName = "") {
  const name = value.trim();
  if (!name) return "请输入分支名称";
  if (
    name.startsWith("-") ||
    name.endsWith("/") ||
    name.endsWith(".") ||
    name.includes("..") ||
    name.includes("@{") ||
    /[\s~^:?*\[\\\]]/.test(name)
  ) {
    return "分支名包含 Git 不支持的字符";
  }
  if (branches.localBranches.some((item) => item.name !== existingLocalName && item.name === name)) {
    return "本地分支已存在";
  }
  if (branches.remoteBranches.some((item) => item.name === name || item.name.endsWith(`/${name}`))) {
    return "远程分支已存在";
  }
  return "";
}

async function createLogBranchFromHead() {
  logRefPanelCollapsed.value = false;
  const name = await promptProjectName("新建分支", "", validateBranchName);
  if (!name) return;
  await branches.create(name, true);
  await loadCurrentRepository();
  history.branchFilter = name;
  await history.refresh().catch(() => undefined);
}

async function deleteActiveLogBranch() {
  const selected = activeLogBranchRef.value;
  if (!selected) return;
  if (selected.branchType === "remote") {
    await deleteRemoteBranchItem(selected);
  } else {
    await deleteLocalBranch(selected);
  }
  clearLogRef();
}

function toggleActiveLogBranchFavorite() {
  const selected = activeLogBranchRef.value;
  if (!selected) return;
  branches.toggleFavorite(selected.fullName);
}

function graphLaneX(index: number) {
  return graphLaneInset + index * graphLaneWidth;
}

function graphPathBetween(x1: number, y1: number, x2: number, y2: number) {
  if (x1 === x2) {
    return `M ${x1} ${y1} L ${x2} ${y2}`;
  }

  const controlOffset = Math.max(5, Math.min(9, Math.abs(y2 - y1) * 0.45));
  return `M ${x1} ${y1} C ${x1} ${y1 + controlOffset}, ${x2} ${y2 - controlOffset}, ${x2} ${y2}`;
}

function buildLogGraphRows(commits: CommitSummary[]): LogGraphRow[] {
  const rows: LogGraphRow[] = [];
  let lanes: LogGraphActiveLane[] = [];
  let colorCursor = 0;

  const nextColor = () => {
    const color = graphPalette[colorCursor % graphPalette.length];
    colorCursor += 1;
    return color;
  };

  for (const item of commits) {
    let laneIndex = lanes.findIndex((lane) => lane.oid === item.oid);
    if (laneIndex === -1) {
      laneIndex = Math.min(lanes.length, graphMaxVisibleLanes - 1);
      lanes.splice(laneIndex, 0, { oid: item.oid, color: nextColor() });
    }

    const topLanes = lanes.map((lane) => ({ ...lane }));
    const currentLane = topLanes[laneIndex];
    const nextLanes = topLanes.filter((_lane, index) => index !== laneIndex);
    const [firstParent, ...mergeParents] = item.parents;

    if (firstParent) {
      const existingParentIndex = nextLanes.findIndex((lane) => lane.oid === firstParent);
      if (existingParentIndex === -1) {
        nextLanes.splice(Math.min(laneIndex, nextLanes.length), 0, {
          oid: firstParent,
          color: currentLane.color,
        });
      }
    }

    mergeParents.forEach((parent, parentIndex) => {
      if (nextLanes.some((lane) => lane.oid === parent)) return;
      nextLanes.splice(Math.min(laneIndex + parentIndex + 1, nextLanes.length), 0, {
        oid: parent,
        color: nextColor(),
      });
    });

    const visibleTopLanes = topLanes.slice(0, graphMaxVisibleLanes);
    const visibleNextLanes = nextLanes.slice(0, graphMaxVisibleLanes);
    const bottomIndexByOid = new Map(visibleNextLanes.map((lane, index) => [lane.oid, index]));
    const paths: LogGraphPath[] = [];

    visibleTopLanes.forEach((lane, index) => {
      if (index === laneIndex && lane.oid === item.oid) return;
      const x = graphLaneX(index);
      const bottomIndex = bottomIndexByOid.get(lane.oid);
      const d =
        bottomIndex === undefined
          ? graphPathBetween(x, 0, x, graphRowMid)
          : graphPathBetween(x, 0, graphLaneX(bottomIndex), graphRowHeight);
      paths.push({
        key: `${item.oid}-lane-${lane.oid}-${index}`,
        d,
        color: lane.color,
      });
    });

    if (laneIndex < graphMaxVisibleLanes) {
      const nodeX = graphLaneX(laneIndex);
      paths.push({
        key: `${item.oid}-node-in`,
        d: graphPathBetween(nodeX, 0, nodeX, graphRowMid),
        color: currentLane.color,
      });

      item.parents.forEach((parent, parentIndex) => {
        const bottomIndex = bottomIndexByOid.get(parent);
        if (bottomIndex === undefined) return;
        const parentLane = visibleNextLanes[bottomIndex];
        paths.push({
          key: `${item.oid}-parent-${parent}-${parentIndex}`,
          d: graphPathBetween(nodeX, graphRowMid, graphLaneX(bottomIndex), graphRowHeight),
          color: parentIndex === 0 ? currentLane.color : parentLane.color,
        });
      });
    }

    const laneCount = Math.max(
      1,
      Math.min(graphMaxVisibleLanes, Math.max(visibleTopLanes.length, visibleNextLanes.length, laneIndex + 1)),
    );

    rows.push({
      item,
      paths,
      laneIndex,
      color: currentLane.color,
      nodeLeft: graphLaneX(laneIndex),
      graphWidth: Math.max(42, graphLaneX(laneCount - 1) + 12),
      hasMerge: item.parents.length > 1,
    });

    lanes = nextLanes.slice(0, graphMaxVisibleLanes);
  }

  return rows;
}

function buildCommitFileTreeRows(files: CommitFileChange[]): LogFileTreeRow[] {
  type LogFileTreeNode = LogFileTreeRow & {
    children: Map<string, LogFileTreeNode>;
  };

  const root: LogFileTreeNode = {
    id: "root",
    name: "",
    path: "",
    parent: null,
    depth: -1,
    directory: true,
    fileCount: files.length,
    children: new Map(),
  };
  const rows: LogFileTreeRow[] = [];
  const sortedFiles = [...files].sort((left, right) => left.path.localeCompare(right.path));

  for (const file of sortedFiles) {
    const parts = file.path.split("/").filter(Boolean);
    let parent = root;
    let currentPath = "";
    for (let index = 0; index < parts.length - 1; index += 1) {
      currentPath = currentPath ? `${currentPath}/${parts[index]}` : parts[index];
      const key = `dir:${parts[index]}`;
      let directory = parent.children.get(key);
      if (!directory) {
        directory = {
          id: `dir:${currentPath}`,
          name: parts[index],
          path: currentPath,
          parent: parent.path || null,
          depth: index,
          directory: true,
          fileCount: 0,
          children: new Map(),
        };
        parent.children.set(key, directory);
      }
      directory.fileCount = (directory.fileCount ?? 0) + 1;
      parent = directory;
    }

    const fileName = parts.length > 0 ? parts[parts.length - 1] : file.path;
    parent.children.set(`file:${file.path}`, {
      id: `file:${file.status}:${file.oldPath ?? ""}:${file.path}`,
      name: fileName,
      path: file.path,
      parent: parent.path || null,
      depth: Math.max(0, parts.length - 1),
      directory: false,
      status: file.status,
      oldPath: file.oldPath,
      children: new Map(),
    });
  }

  const appendRows = (parent: LogFileTreeNode) => {
    const children = [...parent.children.values()].sort(compareLogFileTreeNodes);
    for (const child of children) {
      rows.push({
        id: child.id,
        name: child.name,
        path: child.path,
        parent: child.parent,
        depth: child.depth,
        directory: child.directory,
        fileCount: child.fileCount,
        status: child.status,
        oldPath: child.oldPath,
      });
      if (child.directory) appendRows(child);
    }
  };

  appendRows(root);
  return rows;
}

function compareLogFileTreeNodes(left: LogFileTreeRow, right: LogFileTreeRow) {
  if (left.directory !== right.directory) {
    return left.directory ? -1 : 1;
  }
  return left.name.toLocaleLowerCase().localeCompare(right.name.toLocaleLowerCase());
}

function isCommitFileDirectoryExpanded(path: string) {
  return expandedCommitFileDirectories.value[path] ?? true;
}

function toggleCommitFileDirectory(path: string) {
  expandedCommitFileDirectories.value = {
    ...expandedCommitFileDirectories.value,
    [path]: !isCommitFileDirectoryExpanded(path),
  };
}

function logGraphStyle(row: LogGraphRow) {
  return { width: `${row.graphWidth}px` };
}

function logGraphViewBox(row: LogGraphRow) {
  return `0 0 ${row.graphWidth} ${graphRowHeight}`;
}

function logNodeStyle(row: LogGraphRow) {
  return {
    left: `${row.nodeLeft}px`,
    backgroundColor: row.color,
  };
}

function commitFileStatusTone(status = "") {
  const code = status.charAt(0);
  const tones: Record<string, string> = {
    A: "added",
    C: "copied",
    D: "deleted",
    M: "modified",
    R: "renamed",
    T: "typechange",
    U: "conflicted",
  };
  return tones[code] ?? "unknown";
}

function logFileTreeRowClass(row: LogFileTreeRow) {
  const statusTone = row.directory ? "" : commitFileStatusTone(row.status);
  return {
    directory: row.directory,
    expanded: row.directory && isCommitFileDirectoryExpanded(row.path),
    selected: !row.directory && selectedCommitFilePaths.value.includes(row.path),
    [`status-${statusTone}`]: Boolean(statusTone),
  };
}

function logFileTreeRowTitle(row: LogFileTreeRow) {
  if (row.directory) return `${row.path} · ${row.fileCount ?? 0} 个文件`;
  const status = formatCommitFileStatusCode(row.status);
  return row.oldPath ? `${row.oldPath} -> ${row.path} · ${status}` : `${row.path} · ${status}`;
}

function formatCommitFileStatusCode(status = "") {
  const code = status.charAt(0);
  const labels: Record<string, string> = {
    A: "新增",
    C: "复制",
    D: "删除",
    M: "修改",
    R: "重命名",
    T: "类型变更",
    U: "冲突",
  };
  return labels[code] ?? status;
}

function buildSideBySideDiffRows(response: DiffResponse | null, language: string): SideBySideDiffRow[] {
  if (!response?.text) return [];

  const rows: SideBySideDiffRow[] = [];
  let rowIndex = 0;
  const anchoredHunks = new Set<number>();

  const emptyCell = (): SideBySideDiffCell => ({
    lineNumber: null,
    content: "",
    type: "empty",
    tokens: [{ text: " " }],
  });

  const diffCell = (
    lineNumber: number | null,
    content: string,
    type: SideBySideDiffCell["type"],
  ): SideBySideDiffCell => ({
    lineNumber,
    content,
    type,
    tokens: tokenizeProjectLine(content || " ", language),
  });

  const pushRow = (
    oldCell: SideBySideDiffCell,
    newCell: SideBySideDiffCell,
    type: SideBySideDiffRow["type"],
    hunkIndex: number | null,
  ) => {
    const anchorHunkIndex =
      hunkIndex !== null && type !== "context" && type !== "meta" && !anchoredHunks.has(hunkIndex)
        ? hunkIndex
        : null;
    if (anchorHunkIndex !== null) anchoredHunks.add(anchorHunkIndex);

    rows.push({
      id: `side-diff-${rowIndex}`,
      type,
      hunkIndex,
      anchorHunkIndex,
      old: oldCell,
      new: newCell,
    });
    rowIndex += 1;
  };

  const pushChangeGroup = (
    pendingDeletes: Array<{ lineNumber: number; content: string }>,
    pendingAdds: Array<{ lineNumber: number; content: string }>,
    hunkIndex: number,
  ) => {
    const total = Math.max(pendingDeletes.length, pendingAdds.length);
    for (let index = 0; index < total; index += 1) {
      const deleted = pendingDeletes[index];
      const added = pendingAdds[index];
      const rowType = deleted && added ? "modify" : deleted ? "delete" : "add";
      pushRow(
        deleted ? diffCell(deleted.lineNumber, deleted.content, "delete") : emptyCell(),
        added ? diffCell(added.lineNumber, added.content, "add") : emptyCell(),
        rowType,
        hunkIndex,
      );
    }
    pendingDeletes.length = 0;
    pendingAdds.length = 0;
  };

  const appendPatchHunk = (patch: string, hunkIndex: number, fallbackOldStart: number, fallbackNewStart: number) => {
    const pendingDeletes: Array<{ lineNumber: number; content: string }> = [];
    const pendingAdds: Array<{ lineNumber: number; content: string }> = [];
    let oldLine = fallbackOldStart;
    let newLine = fallbackNewStart;
    let insideHunk = false;

    const flushChanges = () => pushChangeGroup(pendingDeletes, pendingAdds, hunkIndex);

    for (const line of patch.split("\n")) {
      if (!line && !insideHunk) continue;

      if (line.startsWith("@@ ")) {
        flushChanges();
        insideHunk = true;
        const ranges = parseUnifiedHunkRange(line);
        oldLine = ranges.oldStart;
        newLine = ranges.newStart;
        continue;
      }

      if (!insideHunk) continue;

      if (line.startsWith(" ")) {
        flushChanges();
        const content = line.slice(1);
        pushRow(diffCell(oldLine, content, "context"), diffCell(newLine, content, "context"), "context", hunkIndex);
        oldLine += 1;
        newLine += 1;
        continue;
      }

      if (line.startsWith("-")) {
        pendingDeletes.push({ lineNumber: oldLine, content: line.slice(1) });
        oldLine += 1;
        continue;
      }

      if (line.startsWith("+")) {
        pendingAdds.push({ lineNumber: newLine, content: line.slice(1) });
        newLine += 1;
        continue;
      }

      if (line.startsWith("\\")) {
        flushChanges();
        pushRow(emptyCell(), diffCell(null, line, "meta"), "meta", hunkIndex);
      }
    }

    flushChanges();
    return { oldLine: Math.max(oldLine, 1), newLine: Math.max(newLine, 1) };
  };

  const hasCompleteText =
    response.oldText !== undefined &&
    response.newText !== undefined &&
    (response.oldText !== null || response.newText !== null);
  if (hasCompleteText) {
    const oldLines = splitFileContentLines(response.oldText ?? "");
    const newLines = splitFileContentLines(response.newText ?? "");
    let oldCursor = 1;
    let newCursor = 1;

    const pushUnchangedGap = (oldEndExclusive: number, newEndExclusive: number) => {
      while (oldCursor < oldEndExclusive || newCursor < newEndExclusive) {
        if (oldCursor < oldEndExclusive && newCursor < newEndExclusive) {
          pushRow(
            diffCell(oldCursor, oldLines[oldCursor - 1] ?? "", "context"),
            diffCell(newCursor, newLines[newCursor - 1] ?? "", "context"),
            "context",
            null,
          );
          oldCursor += 1;
          newCursor += 1;
        } else if (oldCursor < oldEndExclusive) {
          pushRow(diffCell(oldCursor, oldLines[oldCursor - 1] ?? "", "context"), emptyCell(), "context", null);
          oldCursor += 1;
        } else {
          pushRow(emptyCell(), diffCell(newCursor, newLines[newCursor - 1] ?? "", "context"), "context", null);
          newCursor += 1;
        }
      }
    };

    for (const hunk of response.hunks) {
      const oldStart = hunk.oldStart > 0 ? hunk.oldStart : oldCursor;
      const newStart = hunk.newStart > 0 ? hunk.newStart : newCursor;
      pushUnchangedGap(oldStart, newStart);
      const next = appendPatchHunk(hunk.patch, hunk.index, oldStart, newStart);
      oldCursor = next.oldLine;
      newCursor = next.newLine;
    }

    pushUnchangedGap(oldLines.length + 1, newLines.length + 1);
    return rows;
  }

  for (const hunk of response.hunks) {
    appendPatchHunk(hunk.patch, hunk.index, hunk.oldStart, hunk.newStart);
  }

  return rows;
}

function splitFileContentLines(content: string) {
  if (!content) return [];
  const lines = content.split("\n");
  if (lines[lines.length - 1] === "") lines.pop();
  return lines;
}

function parseUnifiedHunkRange(header: string) {
  const match = header.match(/^@@\s+-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@/);
  return {
    oldStart: Number(match?.[1] ?? 0),
    newStart: Number(match?.[2] ?? 0),
  };
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

function toggleCommitFile(path: string) {
  if (selectedCommitFilePaths.value.includes(path)) {
    selectedCommitFilePaths.value = selectedCommitFilePaths.value.filter((item) => item !== path);
  } else {
    selectedCommitFilePaths.value = [...selectedCommitFilePaths.value, path];
  }
}

function selectedLogFilePaths(row: LogFileTreeRow) {
  return selectedCommitFilePaths.value.includes(row.path) ? selectedCommitFilePaths.value : [row.path];
}

function logDiffTabId(oid: string, path: string, mode: CommitFileDiffMode) {
  return `log-diff:${oid}:${mode}:${path}`;
}

function fileBaseName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function logDiffTabTitle(path: string, mode: CommitFileDiffMode) {
  return `${commitFileDiffModeLabels[mode]}: ${fileBaseName(path)}`;
}

function updateLogDiffTab(id: string, patch: Partial<LogDiffTab>) {
  let updated: LogDiffTab | null = null;
  logDiffTabs.value = logDiffTabs.value.map((tab) => {
    if (tab.id !== id) return tab;
    updated = { ...tab, ...patch };
    return updated;
  });
  return updated;
}

function selectLogRootTab() {
  activeLogTabId.value = LOG_TAB_ID;
}

function logDiffTabClass(tab: LogDiffTab) {
  return {
    active: activeLogTabId.value === tab.id,
    loading: tab.loading,
    error: Boolean(tab.error),
  };
}

function closeLogDiffTab(id: string) {
  const index = logDiffTabs.value.findIndex((tab) => tab.id === id);
  if (index < 0) return;

  const nextTabs = logDiffTabs.value.filter((tab) => tab.id !== id);
  logDiffTabs.value = nextTabs;
  if (activeLogTabId.value !== id) return;

  activeLogTabId.value = nextTabs[index]?.id ?? nextTabs[index - 1]?.id ?? LOG_TAB_ID;
}

function clearLogDiffTabs() {
  logDiffTabs.value = [];
  activeLogTabId.value = LOG_TAB_ID;
}

function openLogFileContextMenu(row: LogFileTreeRow, event: MouseEvent) {
  if (row.directory) return;
  if (!selectedCommitFilePaths.value.includes(row.path)) {
    selectedCommitFilePaths.value = [row.path];
  }
  const menuWidth = 260;
  const menuHeight = 384;
  logFileContextMenu.value = {
    row,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function closeLogFileContextMenu() {
  logFileContextMenu.value = null;
}

async function showCommitFileDiff(row: LogFileTreeRow, mode: CommitFileDiffMode = "commit") {
  if (row.directory || !history.selectedOid || !repos.path) return null;

  const oid = history.selectedOid;
  const id = logDiffTabId(oid, row.path, mode);
  const existing = logDiffTabs.value.find((tab) => tab.id === id);
  activeLogTabId.value = id;
  closeLogFileContextMenu();

  if (existing?.diff || existing?.loading) return existing;

  const tab: LogDiffTab =
    existing ??
    {
      id,
      oid,
      shortOid: oid.slice(0, 10),
      path: row.path,
      mode,
      title: logDiffTabTitle(row.path, mode),
      subtitle: selectedCommitTitle.value,
      diff: null,
      loading: true,
      error: "",
    };

  if (!existing) {
    logDiffTabs.value = [...logDiffTabs.value, tab];
  } else {
    updateLogDiffTab(id, { loading: true, error: "" });
  }

  try {
    const response = await commitFileDiff(repos.path, oid, row.path, mode);
    history.error = "";
    return updateLogDiffTab(id, { diff: response, loading: false, error: "" });
  } catch (error) {
    const message = String(error);
    history.error = message;
    updateLogDiffTab(id, { loading: false, error: message });
    return null;
  }
}

async function cherryPickLogFile(row: LogFileTreeRow) {
  const oid = history.selectedOid;
  if (!oid || row.directory) return;
  const paths = selectedLogFilePaths(row);
  await operations.cherryPickFiles(oid, paths);
  workbenchMode.value = "changes";
  closeLogFileContextMenu();
  await reloadAfterGitOperation();
}

async function revertLogFileChange(row: LogFileTreeRow) {
  const oid = history.selectedOid;
  if (!oid || row.directory) return;
  const paths = selectedLogFilePaths(row);
  if (!window.confirm(`还原 ${oid.slice(0, 10)} 的 ${paths.length} 个文件变更到工作区？`)) return;
  await operations.revertFiles(oid, paths);
  workbenchMode.value = "changes";
  closeLogFileContextMenu();
  await reloadAfterGitOperation();
}

async function createPatchFromLogFile(row: LogFileTreeRow) {
  if (row.directory) return;
  const tab = await showCommitFileDiff(row, "commit");
  if (!tab?.diff) return;
  advanced.generatedPatch = tab?.diff?.text ?? "";
  advanced.notice = `已生成 ${row.path} 的补丁`;
}

function showLogFileHistory(row: LogFileTreeRow) {
  if (row.directory) return;
  history.pathFilters = [row.path];
  closeLogFileContextMenu();
  history.refresh().catch(() => undefined);
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

</script>

<template>
  <div class="app-shell" :data-theme="effectiveTheme" @click="closeContextMenus">
    <AppTopbar
      :brand-subtitle="brandSubtitle"
      :has-repository="Boolean(repos.current)"
      :current-branch="branchNameLabel(branch?.currentBranch)"
      :ahead="branch?.ahead ?? 0"
      :behind="branch?.behind ?? 0"
    />

    <Transition name="notice-toast">
      <div v-if="noticeToast" class="notice-toast" role="status" aria-live="polite" @click.stop>
        <Check :size="15" />
        <span>{{ noticeToast.message }}</span>
        <button type="button" title="关闭通知" @click="dismissNoticeToast(noticeToast.id)">
          <X :size="13" />
        </button>
      </div>
    </Transition>

    <section
      class="workspace"
      :class="{ 'workspace-empty': !repos.current, 'is-resizing': activeResizePanel }"
      :style="workspaceGridStyle"
    >
      <ProjectPane
        v-if="settings.panelVisibility.project"
        @choose-repository="chooseRepository"
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
        v-if="repos.current || repos.selectedPath"
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
        <div v-if="repos.selectedPath" class="empty-panel project-init-panel">
          <ListChecks :size="40" />
          <h1>项目尚未初始化</h1>
          <p>{{ repos.selectedPath }}</p>
          <button
            class="tool-button primary large"
            :disabled="advanced.loading || !repos.selectedPath"
            @click="initSelectedProject"
          >
            <Plus :size="18" />
            <span>{{ advanced.loading ? "初始化中" : "初始化仓库" }}</span>
          </button>
        </div>
        <div v-else class="empty-panel">
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
          <label class="toggle-row file-actions-toggle">
            <input
              :checked="settings.includeIgnored"
              type="checkbox"
              @change="setIncludeIgnored"
            />
            <span>显示忽略文件</span>
          </label>
        </div>

        <div class="file-list source-control-tree" @contextmenu.prevent="openChangeListContextMenu(null, $event)">
          <div v-if="changeFileGroups.length === 0" class="file-list-empty">没有文件变更</div>
          <template v-else>
            <section
              v-for="group in changeFileGroups"
              :key="group.key"
              class="change-file-group"
              @contextmenu.prevent.stop="openChangeListContextMenu(group.changelistId, $event)"
            >
              <div
                class="change-file-group-header"
                @contextmenu.prevent.stop="openChangeListContextMenu(group.changelistId, $event)"
              >
                <button class="change-group-toggle" type="button" @click="toggleChangeFileGroup(group.key)">
                  <ChevronDown v-if="isChangeFileGroupExpanded(group.key)" :size="14" />
                  <ChevronRight v-else :size="14" />
                </button>
                <input
                  class="change-group-checkbox"
                  type="checkbox"
                  :checked="isChangeFileGroupSelected(group.files)"
                  :disabled="group.files.length === 0"
                  :indeterminate.prop="isChangeFileGroupPartiallySelected(group.files)"
                  @change="toggleChangeFileGroupSelection(group.files)"
                />
                <button class="change-group-title" type="button" @click="toggleChangeFileGroup(group.key)">
                  <span>{{ group.label }}</span>
                  <small>{{ group.files.length }} 个文件</small>
                </button>
              </div>
              <div v-if="isChangeFileGroupExpanded(group.key)" class="change-file-group-list">
                <div v-if="group.files.length === 0" class="change-file-group-empty">没有文件</div>
                <button
                  v-for="file in group.files"
                  :key="`${group.side}-${file.path}`"
                  class="file-row"
                  :class="{
                    active: changes.selectedFile === file.path,
                    selected: changes.selectedPaths.includes(file.path),
                    [`status-${file.kind.split('|')[0]}`]: true,
                  }"
                  :title="`${file.path} · ${formatStatusKind(file.kind)}`"
                  @click="selectFile(file, group.side)"
                  @contextmenu.prevent.stop="openChangeFileContextMenu(file, group.side, $event)"
                >
                  <input
                    type="checkbox"
                    :checked="changes.selectedPaths.includes(file.path)"
                    @click.stop
                    @change="changes.togglePath(file.path)"
                  />
                  <span class="status-dot" :class="file.kind.split('|')[0]" />
                  <span class="change-file-icon" :class="changeFileIconClass(file.path)">
                    <span v-if="fileTypeLabel(file.path)">{{ fileTypeLabel(file.path) }}</span>
                    <FileIcon v-else :size="13" />
                  </span>
                  <span class="file-main">
                    <strong>{{ fileBaseName(file.path) }}</strong>
                    <small>{{ fileContextPath(file.path) }}</small>
                  </span>
                </button>
              </div>
            </section>
          </template>
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

      <section
        v-else-if="settings.panelVisibility.changes && workbenchMode === 'log'"
        class="history-pane log-ref-pane"
        :class="{ collapsed: logRefPanelCollapsed }"
      >
        <nav class="log-ref-toolbar" aria-label="日志引用工具栏">
          <button
            class="log-ref-tool-button"
            type="button"
            :title="logRefPanelCollapsed ? '展开分支栏' : '收起分支栏'"
            @click="toggleLogRefPanelCollapsed"
          >
            <ArrowRight v-if="logRefPanelCollapsed" :size="16" />
            <ArrowLeft v-else :size="16" />
          </button>
          <span class="log-ref-tool-separator" />
          <button
            class="log-ref-tool-button"
            type="button"
            title="搜索分支或标签"
            @click="focusLogRefSearch"
          >
            <Search :size="15" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            title="刷新引用和日志"
            :disabled="history.loading || branches.loading"
            @click="refreshAll"
          >
            <RefreshCw :size="15" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            title="获取远程引用"
            :disabled="remote.loading || !remote.selectedRemote"
            @click="runRemoteAction('fetch')"
          >
            <Download :size="15" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            title="拉取代码"
            :disabled="remote.loading || !remote.selectedRemote"
            @click="runRemoteAction('pull')"
          >
            <ArrowDown :size="15" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            title="推送代码"
            :disabled="remote.loading || !remote.selectedRemote"
            @click="runRemoteAction('push')"
          >
            <ArrowUp :size="15" />
          </button>
          <span class="log-ref-tool-separator" />
          <button
            class="log-ref-tool-button"
            type="button"
            title="从当前 HEAD 创建并切换分支"
            :disabled="branches.loading"
            @click="createLogBranchFromHead"
          >
            <Plus :size="16" />
          </button>
          <button
            class="log-ref-tool-button danger"
            type="button"
            title="删除当前选中的分支"
            :disabled="branches.loading || !activeLogBranchRef || activeLogBranchRef.current"
            @click="deleteActiveLogBranch"
          >
            <Trash2 :size="15" />
          </button>
          <button
            class="log-ref-tool-button"
            :class="{ active: activeLogBranchFavorite || logFavoriteRefsOnly }"
            type="button"
            :title="activeLogBranchRef ? (activeLogBranchFavorite ? '取消收藏当前分支' : '收藏当前分支') : '仅显示收藏引用'"
            @click="activeLogBranchRef ? toggleActiveLogBranchFavorite() : toggleLogFavoriteRefsOnly()"
          >
            <Star :size="15" :fill="activeLogBranchFavorite || logFavoriteRefsOnly ? 'currentColor' : 'none'" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            :title="logRefGroupsFullyExpanded ? '折叠引用分组' : '展开引用分组'"
            @click="toggleAllLogRefGroups"
          >
            <ChevronDown v-if="logRefGroupsFullyExpanded" :size="16" />
            <ChevronRight v-else :size="16" />
          </button>
          <button
            class="log-ref-tool-button"
            type="button"
            title="清除引用筛选"
            :disabled="!history.branchFilter && !logRefSearch"
            @click="clearLogRefContext"
          >
            <X :size="15" />
          </button>
        </nav>

        <div v-if="!logRefPanelCollapsed" class="log-ref-content">
          <div class="log-ref-search-bar">
            <label class="log-ref-search-field">
              <Search :size="14" />
              <input
                ref="logRefSearchInput"
                v-model="logRefSearch"
                type="search"
                placeholder="Branch or tag"
                aria-label="筛选分支或标签"
              />
            </label>
          </div>

          <div class="log-ref-list">
          <button
            v-if="showLogHeadRef"
            class="log-ref-head-row"
            :class="{ active: !history.branchFilter }"
            type="button"
            title="显示全部引用"
            @click="clearLogRef"
          >
            {{ logHeadLabel }}
          </button>

          <button
            v-if="visibleLogLocalBranches.length"
            class="log-ref-toggle"
            type="button"
            @click="toggleLogRefGroup('local')"
          >
            <ChevronDown v-if="logRefFiltering || isLogRefGroupExpanded('local')" :size="13" />
            <ChevronRight v-else :size="13" />
            <span>本地</span>
          </button>
          <div v-if="visibleLogLocalBranches.length && (logRefFiltering || isLogRefGroupExpanded('local'))" class="log-ref-children">
            <button
              v-for="branchItem in visibleLogLocalBranches"
              :key="`log-local-${branchItem.fullName}`"
              class="log-ref-row local"
              :class="{
                active: isLogRefActive(branchItem.name),
                current: branchItem.current,
                favorite: branches.isFavorite(branchItem.fullName),
                'context-target': isLogBranchContextTarget(branchItem),
              }"
              :title="branchItem.fullName"
              type="button"
              @click="selectLogRef(branchItem.name)"
              @contextmenu.prevent.stop="openLogBranchContextMenu(branchItem, $event)"
            >
              <Star v-if="branches.isFavorite(branchItem.fullName)" :size="13" fill="currentColor" />
              <GitBranch v-else :size="13" />
              <span>
                <strong>{{ branchItem.name }}</strong>
              </span>
            </button>
          </div>

          <button
            v-if="visibleLogRemoteGroups.length"
            class="log-ref-toggle"
            type="button"
            @click="toggleLogRefGroup('remote')"
          >
            <ChevronDown v-if="logRefFiltering || isLogRefGroupExpanded('remote')" :size="13" />
            <ChevronRight v-else :size="13" />
            <span>远端</span>
          </button>
          <div v-if="visibleLogRemoteGroups.length && (logRefFiltering || isLogRefGroupExpanded('remote'))" class="log-ref-children">
            <section v-for="group in visibleLogRemoteGroups" :key="`log-remote-${group.name}`" class="log-ref-group">
              <button
                class="log-ref-toggle remote-root"
                type="button"
                @click="toggleLogRefGroup(logRemoteGroupKey(group.name))"
              >
                <ChevronDown v-if="logRefFiltering || isLogRefGroupExpanded(logRemoteGroupKey(group.name))" :size="13" />
                <ChevronRight v-else :size="13" />
                <Folder :size="13" />
                <span>{{ group.name }}</span>
              </button>
              <div
                v-if="logRefFiltering || isLogRefGroupExpanded(logRemoteGroupKey(group.name))"
                class="log-ref-children remote-branch-list"
              >
                <button
                  v-for="branchItem in group.branches"
                  :key="`log-remote-${branchItem.fullName}`"
                  class="log-ref-row remote"
                  :class="{
                    active: isLogRefActive(branchItem.name),
                    favorite: branches.isFavorite(branchItem.fullName),
                    'context-target': isLogBranchContextTarget(branchItem),
                  }"
                  :title="branchItem.fullName"
                  type="button"
                  @click="selectLogRef(branchItem.name)"
                  @contextmenu.prevent.stop="openLogBranchContextMenu(branchItem, $event)"
                >
                  <Star v-if="branches.isFavorite(branchItem.fullName)" :size="13" fill="currentColor" />
                  <GitBranch v-else :size="13" />
                  <span>
                    <strong>{{ shortRemoteBranchName(branchItem.name, group.name) }}</strong>
                  </span>
                </button>
              </div>
            </section>
          </div>

          <button
            v-if="visibleLogTags.length"
            class="log-ref-toggle"
            type="button"
            @click="toggleLogRefGroup('tags')"
          >
            <ChevronDown v-if="logRefFiltering || isLogRefGroupExpanded('tags')" :size="13" />
            <ChevronRight v-else :size="13" />
            <span>标签</span>
          </button>
          <div v-if="visibleLogTags.length && (logRefFiltering || isLogRefGroupExpanded('tags'))" class="log-ref-children">
            <button
              v-for="tag in visibleLogTags"
              :key="`log-tag-${tag.name}`"
              class="log-ref-row tag-ref"
              :class="{ active: isLogRefActive(tag.name), 'context-target': isLogTagContextTarget(tag) }"
              :title="tag.name"
              type="button"
              @click="selectLogRef(tag.name)"
              @contextmenu.prevent.stop="openLogTagContextMenu(tag, $event)"
            >
              <span class="tag-dot" />
              <span>
                <strong>{{ tag.name }}</strong>
              </span>
            </button>
          </div>

            <div v-if="!hasVisibleLogRefs" class="log-ref-empty">没有匹配的引用</div>
          </div>
        </div>
      </section>

      <section v-else-if="settings.panelVisibility.changes && workbenchMode === 'project'" class="project-tree-pane">
        <div class="history-header">
          <div class="section-title">
            <FolderOpen :size="16" />
            <span>项目文件</span>
          </div>
          <button class="icon-only-button" title="刷新项目文件" :disabled="project.loading" @click="project.refresh()">
            <RefreshCw :size="14" />
          </button>
        </div>

        <div class="project-file-browser">
          <div class="project-file-list" @contextmenu.prevent="openProjectFileContextMenu(null, $event)">
            <button
              v-for="file in visibleProjectFiles"
              :key="file.path"
              class="project-file-row"
              :class="projectFileClass(file)"
              :style="projectFileIndent(file)"
              :title="projectFileTitle(file)"
              :aria-expanded="file.directory ? project.isExpanded(file.path) : undefined"
              @click="openProjectEntry(file)"
              @contextmenu.prevent.stop="openProjectFileContextMenu(file, $event)"
            >
              <span class="project-file-disclosure">
                <ChevronDown v-if="file.directory && project.isExpanded(file.path)" :size="13" />
                <ChevronRight v-else-if="file.directory" :size="13" />
              </span>
              <FolderOpen v-if="file.directory && project.isExpanded(file.path)" :size="14" />
              <Folder v-else-if="file.directory" :size="14" />
              <FileIcon v-else :size="14" />
              <span class="project-file-name" :class="{ root: file.path === PROJECT_ROOT_PATH }">
                <template v-if="file.path === PROJECT_ROOT_PATH">
                  <strong>{{ file.name }}</strong>
                  <small>{{ repos.path }}</small>
                </template>
                <template v-else>{{ file.name }}</template>
              </span>
              <span class="project-status-marker" :title="projectStatusLabel(projectStatusForPath(file.path))" />
            </button>
            <div v-if="project.loading" class="project-file-empty">加载中</div>
            <div v-else-if="project.files.length === 0" class="project-file-empty">暂无文件</div>
          </div>
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

        <template v-else-if="workbenchMode === 'project'">
        <div v-if="project.openTabs.length" class="project-tabs" role="tablist" aria-label="打开的项目文件">
          <div
            v-for="tab in project.openTabs"
            :key="tab.path"
            class="project-tab"
            :class="projectTabClass(tab)"
          >
            <button
              class="project-tab-select"
              :class="{ dirty: project.isPathDirty(tab.path) }"
              role="tab"
              :aria-selected="project.selectedPath === tab.path"
              :title="projectFileTitle(tab)"
              @click="project.selectTab(tab.path)"
            >
              <FileIcon :size="14" />
              <span v-if="project.isPathDirty(tab.path)" class="project-tab-dirty" aria-label="未保存" title="未保存" />
              <span class="project-tab-title">{{ tab.name }}</span>
            </button>
            <button class="project-tab-close" title="关闭文件" @click.stop="closeProjectEditorTab(tab.path)">
              <X :size="13" />
            </button>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>

        <div class="project-editor">
          <div v-if="project.contentLoading || project.loading" class="diff-empty">加载中</div>
          <div v-else-if="project.content?.binary" class="diff-empty">
            二进制文件，大小 {{ formatBytes(project.content.size) }}
          </div>
          <div v-else-if="!project.content" class="diff-empty">选择一个项目文件</div>
          <div v-else class="project-edit-pane">
            <div class="project-editor-render" :style="projectEditorRenderStyle" aria-hidden="true"><div class="project-editor-render-content">
              <span
                v-for="line in projectEditorLines"
                :key="line.index"
                class="project-render-line"
              ><span class="line-number">{{ line.number }}</span><span class="project-render-code"><template
                v-for="(token, tokenIndex) in line.tokens"
                :key="tokenIndex"
              ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span></span>
            </div></div>
            <div v-if="projectEditorHunks.length" class="project-editor-change-layer">
              <button
                v-for="hunk in projectEditorHunks"
                :key="hunk.index"
                class="project-change-marker"
                :class="[hunk.tone, { expanded: expandedProjectHunk?.index === hunk.index }]"
                :style="projectEditorHunkMarkerStyle(hunk)"
                type="button"
                :title="projectEditorHunkTitle(hunk)"
                @click="toggleProjectEditorHunk(hunk.index)"
              >
                <ChevronDown v-if="expandedProjectHunk?.index === hunk.index" :size="12" />
                <ChevronRight v-else :size="12" />
              </button>

              <div
                v-if="expandedProjectHunk"
                class="project-original-panel project-original-popover"
                :style="projectEditorOriginalPanelStyle(expandedProjectHunk)"
                @mousedown.stop
              >
                <div class="project-original-gutter">{{ expandedProjectHunk.oldStart }}</div>
                <div class="project-original-card">
                  <div class="project-original-toolbar">
                    <span>{{ projectEditorHunkTitle(expandedProjectHunk) }}</span>
                    <div class="project-original-actions">
                      <button
                        class="icon-button danger"
                        type="button"
                        :disabled="project.contentLoading || project.contentSaving"
                        @click="discardProjectEditorHunk(expandedProjectHunk.index)"
                      >
                        <RotateCcw :size="14" />
                        <span>撤回此块</span>
                      </button>
                      <button class="icon-button" type="button" title="关闭" @click="expandedProjectHunkIndex = null">
                        <X :size="14" />
                      </button>
                    </div>
                  </div>
                  <div v-if="expandedProjectHunk.originalLines.length" class="project-original-code">
                    <div
                      v-for="line in expandedProjectHunk.originalLines"
                      :key="line.index"
                      class="project-original-line"
                    >
                      <span class="line-number">{{ line.lineNumber }}</span>
                      <span class="line-content"><template
                        v-for="(token, tokenIndex) in line.tokens"
                        :key="tokenIndex"
                      ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                    </div>
                  </div>
                  <div v-else class="project-original-empty">原本没有内容，这一块是新增行</div>
                </div>
              </div>
            </div>
            <textarea
              ref="projectEditorTextarea"
              v-model="projectEditorText"
              class="project-editor-textarea"
              spellcheck="false"
              :aria-label="project.selectedPath || '项目文件'"
              :disabled="project.contentSaving"
              @scroll="syncProjectEditorScroll"
              @keydown.meta.s.prevent="saveProjectEditor"
              @keydown.ctrl.s.prevent="saveProjectEditor"
            />
          </div>
        </div>
      </template>

        <template v-else-if="workbenchMode === 'changes'">
        <div class="diff-header">
          <div class="diff-title-block">
            <span class="eyebrow">文件差异</span>
            <h2 :title="selectedDiffFileTitle">{{ selectedDiffFileTitle }}</h2>
            <small>
              文件 {{ activeChangeDiffFilePosition }} · 差异 {{ currentChangeDiffHunkPosition }}/{{ activeChangeDiffHunkCount }}
            </small>
          </div>
          <div class="diff-header-actions">
            <div class="diff-nav-group" aria-label="差异跳转">
              <button
                class="icon-only-button diff-nav-button"
                title="上一个差异"
                :disabled="activeChangeDiffHunkCount === 0"
                @click="jumpChangeDiffHunk(-1)"
              >
                <ArrowUp :size="15" />
              </button>
              <button
                class="icon-only-button diff-nav-button"
                title="下一个差异"
                :disabled="activeChangeDiffHunkCount === 0"
                @click="jumpChangeDiffHunk(1)"
              >
                <ArrowDown :size="15" />
              </button>
            </div>
            <div class="diff-nav-group" aria-label="差异文件切换">
              <button
                class="icon-only-button diff-nav-button"
                title="上一个文件"
                :disabled="!canSelectPreviousChangeDiffFile"
                @click="selectAdjacentChangeDiffFile(-1)"
              >
                <ArrowLeft :size="15" />
              </button>
              <button
                class="icon-only-button diff-nav-button"
                title="下一个文件"
                :disabled="!canSelectNextChangeDiffFile"
                @click="selectAdjacentChangeDiffFile(1)"
              >
                <ArrowRight :size="15" />
              </button>
            </div>
          </div>
        </div>

        <div v-if="activeError" class="message error">{{ activeError }}</div>

        <div ref="changeDiffScroller" class="diff-scroller side-by-side-scroller">
          <div v-if="diff.loading" class="diff-empty">加载中</div>
          <div v-else-if="!activeChangeDiffHasContent" class="diff-empty">没有差异</div>
          <div v-else-if="activeChangeSideBySideDiffRows.length === 0" class="diff-empty">无法以文本方式显示此差异</div>
          <div v-else class="side-by-side-diff">
            <div class="side-by-side-file-header">
              <div class="side-by-side-title">
                <strong>{{ changeDiffLeftLabel }}</strong>
                <span>{{ changeDiffLeftDetail }}</span>
              </div>
              <div class="side-by-side-title">
                <strong>{{ changeDiffRightLabel }}</strong>
                <span>{{ changeDiffRightDetail }}</span>
              </div>
            </div>
            <div class="side-by-side-editors">
              <div class="side-by-side-column old" @scroll="syncSideBySideEditorScroll">
                <div class="side-by-side-column-lines">
                  <div
                    v-for="row in activeChangeSideBySideDiffRows"
                    :key="`change-old-${row.id}`"
                    class="side-by-side-line"
                    :class="[
                      row.type,
                      { active: row.hunkIndex !== null && row.hunkIndex === activeChangeDiffHunkIndex },
                    ]"
                    :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
                  >
                    <div class="diff-cell old" :class="row.old.type">
                      <span class="line-number">{{ row.old.lineNumber ?? "" }}</span>
                      <span class="line-content"><template
                        v-for="(token, tokenIndex) in row.old.tokens"
                        :key="tokenIndex"
                      ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                    </div>
                  </div>
                </div>
              </div>
              <div class="side-by-side-column new" @scroll="syncSideBySideEditorScroll">
                <div class="side-by-side-column-lines">
                  <div
                    v-for="row in activeChangeSideBySideDiffRows"
                    :key="`change-new-${row.id}`"
                    class="side-by-side-line"
                    :class="[
                      row.type,
                      { active: row.hunkIndex !== null && row.hunkIndex === activeChangeDiffHunkIndex },
                    ]"
                    :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
                  >
                    <div class="diff-cell new" :class="row.new.type">
                      <span class="line-number">{{ row.new.lineNumber ?? "" }}</span>
                      <span class="line-content"><template
                        v-for="(token, tokenIndex) in row.new.tokens"
                        :key="tokenIndex"
                      ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
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
        <div class="log-tab-workspace">
          <div class="log-workspace-tabs" role="tablist" aria-label="日志标签页">
            <button
              class="log-root-tab"
              :class="{ active: activeLogTabId === LOG_TAB_ID }"
              role="tab"
              :aria-selected="activeLogTabId === LOG_TAB_ID"
              title="日志"
              @click="selectLogRootTab"
            >
              <VcsIcon :size="14" />
              <span>日志</span>
            </button>
            <div
              v-for="tab in logDiffTabs"
              :key="tab.id"
              class="log-workspace-tab"
              :class="logDiffTabClass(tab)"
            >
              <button
                class="log-workspace-tab-select"
                role="tab"
                :aria-selected="activeLogTabId === tab.id"
                :title="`${tab.title} · ${tab.subtitle}`"
                @click="activeLogTabId = tab.id"
              >
                <FileIcon :size="14" />
                <span>{{ tab.title }}</span>
                <small>{{ tab.shortOid }}</small>
              </button>
              <button class="log-workspace-tab-close" title="关闭标签页" @click.stop="closeLogDiffTab(tab.id)">
                <X :size="13" />
              </button>
            </div>
          </div>

        <div v-if="activeLogTabId === LOG_TAB_ID" class="log-workbench">
          <section class="log-commit-panel">
            <div class="log-topbar">
              <label class="log-search-field">
                <Search :size="14" />
                <input v-model="history.query" placeholder="文字或哈希" @keydown.enter="history.refresh" />
              </label>
              <div class="log-filter-picker">
                <button
                  class="log-filter-button"
                  :class="{ active: history.authorFilters.length > 0 }"
                  title="筛选作者"
                  @click="toggleLogAuthorPicker"
                >
                  <UserRound :size="14" />
                  <span>{{ logAuthorFilterLabel }}</span>
                  <ChevronDown :size="13" />
                </button>
                <div v-if="logAuthorPickerOpen" class="log-filter-popover author" @click.stop>
                  <div class="log-filter-popover-head">
                    <strong>作者</strong>
                    <button class="project-remove" title="清空作者" :disabled="history.authorFilters.length === 0" @click="clearLogAuthorFilters">
                      <X :size="13" />
                    </button>
                  </div>
                  <div class="log-filter-options">
                    <button
                      v-for="option in logAuthorOptions"
                      :key="option.value"
                      class="log-check-row"
                      :class="{ selected: isLogAuthorSelected(option.value) }"
                      @click="toggleLogAuthorFilter(option.value)"
                    >
                      <span class="log-checkmark">
                        <Check v-if="isLogAuthorSelected(option.value)" :size="12" />
                      </span>
                      <span class="log-check-label">
                        <strong>{{ option.label }}</strong>
                        <small>{{ option.meta || `${option.count} 个提交` }}</small>
                      </span>
                      <small>{{ option.count }}</small>
                    </button>
                    <div v-if="logAuthorOptions.length === 0" class="log-picker-empty">暂无作者</div>
                  </div>
                </div>
              </div>
              <button
                class="log-filter-button"
                :class="{ active: history.pathFilters.length > 0 }"
                title="选择文件"
                @click="openLogFilePicker"
              >
                <FileSearch :size="14" />
                <span>{{ logFileFilterLabel }}</span>
                <ChevronDown :size="13" />
              </button>
              <span class="log-filter-chip" :class="{ active: logFilterActive }">
                分支: {{ activeLogRefLabel }}
                <button v-if="history.branchFilter" title="清除分支过滤" @click="clearLogRef">
                  <X :size="12" />
                </button>
              </span>
              <button class="icon-only-button" title="刷新日志" :disabled="history.loading" @click="history.refresh()">
                <RefreshCw :size="14" />
              </button>
            </div>

            <div v-if="activeError" class="message error">{{ activeError }}</div>

            <div class="log-table-head">
              <span />
              <span>提交</span>
              <span>作者</span>
              <span>日期</span>
            </div>

            <div class="log-commit-list">
              <button
                v-for="row in logGraphRows"
                :key="row.item.oid"
                class="log-commit-row"
                :class="{ active: history.selectedOid === row.item.oid }"
                @click="selectCommit(row.item.oid)"
              >
                <span class="log-graph-cell" :class="{ merge: row.hasMerge }" :style="logGraphStyle(row)">
                  <svg class="log-graph-svg" :viewBox="logGraphViewBox(row)" preserveAspectRatio="none" aria-hidden="true">
                    <path
                      v-for="path in row.paths"
                      :key="path.key"
                      class="log-graph-path"
                      :d="path.d"
                      :stroke="path.color"
                    />
                  </svg>
                  <span class="log-graph-node" :style="logNodeStyle(row)" />
                </span>
                <span class="log-subject">
                  <strong>{{ row.item.summary }}</strong>
                  <span v-if="row.item.refs.length" class="commit-refs">
                    <em v-for="refName in row.item.refs" :key="refName">{{ formatRefName(refName) }}</em>
                  </span>
                </span>
                <span class="log-author">{{ row.item.authorName }}</span>
                <time class="log-date">{{ formatCompactCommitTime(row.item.authorTime) }}</time>
              </button>
              <div v-if="history.loading" class="diff-empty">加载中</div>
              <div v-else-if="history.commits.length === 0" class="diff-empty">没有提交历史</div>
            </div>
          </section>

          <aside class="log-detail-panel">
            <section class="log-files-panel">
              <div class="log-panel-header">
                <div class="section-title">
                  <FolderOpen :size="15" />
                  <span>文件</span>
                </div>
                <small>{{ history.details?.files.length ?? 0 }} 个文件</small>
              </div>
              <div class="log-file-tree">
                <div v-if="history.detailLoading" class="diff-empty">加载中</div>
                <div v-else-if="!history.details" class="diff-empty">选择一个提交</div>
                <div v-else-if="commitFileTreeRows.length === 0" class="diff-empty">没有文件变更</div>
                <template v-else>
                  <button
                    v-for="row in visibleCommitFileTreeRows"
                    :key="row.id"
                    class="log-file-tree-row"
                    :class="logFileTreeRowClass(row)"
                    :style="logFileIndent(row.depth)"
                    :title="logFileTreeRowTitle(row)"
                    :aria-expanded="row.directory ? isCommitFileDirectoryExpanded(row.path) : undefined"
                    @click="row.directory ? toggleCommitFileDirectory(row.path) : toggleCommitFile(row.path)"
                    @dblclick.stop="!row.directory && showCommitFileDiff(row)"
                    @contextmenu.prevent.stop="openLogFileContextMenu(row, $event)"
                  >
                    <span class="log-file-disclosure">
                      <ChevronDown v-if="row.directory && isCommitFileDirectoryExpanded(row.path)" :size="13" />
                      <ChevronRight v-else-if="row.directory" :size="13" />
                    </span>
                    <FolderOpen v-if="row.directory && isCommitFileDirectoryExpanded(row.path)" :size="14" />
                    <Folder v-else-if="row.directory" :size="14" />
                    <FileIcon v-else :size="14" />
                    <span class="log-file-name">
                      <strong>{{ row.name }}</strong>
                      <small v-if="!row.directory && row.oldPath">{{ row.oldPath }}</small>
                    </span>
                    <small>{{ row.directory ? `${row.fileCount ?? 0} 个文件` : formatCommitFileStatusCode(row.status) }}</small>
                  </button>
                </template>
              </div>
            </section>

            <section class="log-info-panel">
              <div v-if="history.details" class="log-info-body">
                <h2 :title="selectedCommitTitle">{{ history.details.commit.summary }}</h2>
                <p>
                  <strong>{{ history.details.commit.shortOid }}</strong>
                  {{ history.details.commit.authorName }} &lt;{{ history.details.commit.authorEmail }}&gt;
                </p>
                <p>{{ formatCommitTime(history.details.commit.authorTime) }}</p>
                <div v-if="selectedCommitRefs.length" class="commit-refs">
                  <em v-for="refName in selectedCommitRefs" :key="refName">{{ refName }}</em>
                </div>
                <p>父提交: {{ history.details.commit.parents.length || "无" }}</p>
              </div>
              <div v-else class="diff-empty">选择一个提交</div>

              <div class="log-detail-actions">
                <button class="icon-only-button" title="挑选提交" :disabled="!history.selectedOid || operations.loading" @click="cherryPickSelectedCommit">
                  <GitCommitVertical :size="14" />
                </button>
                <button class="icon-only-button" title="检出此提交" :disabled="!history.selectedOid || advanced.loading" @click="checkoutSelectedRevision">
                  <GitBranch :size="14" />
                </button>
                <button class="icon-only-button" title="创建修正提交" :disabled="!history.selectedOid || advanced.loading" @click="fixupSelectedCommit(false)">
                  <Plus :size="14" />
                </button>
                <button class="icon-only-button" title="创建压缩提交" :disabled="!history.selectedOid || advanced.loading" @click="fixupSelectedCommit(true)">
                  <Plus :size="14" />
                </button>
                <button
                  class="icon-only-button"
                  title="应用选中文件"
                  :disabled="!history.selectedOid || selectedCommitFilePaths.length === 0 || operations.loading"
                  @click="cherryPickSelectedFiles"
                >
                  <Download :size="14" />
                </button>
                <button class="icon-only-button" title="反向提交" :disabled="!history.selectedOid || operations.loading" @click="revertSelectedCommit">
                  <RotateCcw :size="14" />
                </button>
                <button class="icon-only-button danger" title="重置到此提交" :disabled="!history.selectedOid || operations.loading" @click="resetToSelectedCommit">
                  <Trash2 :size="14" />
                </button>
                <button class="icon-only-button danger" title="撤销最后一次提交" :disabled="!branch?.head || operations.loading" @click="undoLastCommit">
                  <RotateCcw :size="14" />
                </button>
                <button class="icon-only-button danger" title="丢弃提交" :disabled="!history.selectedOid || advanced.loading" @click="dropSelectedCommit">
                  <Trash2 :size="14" />
                </button>
                <button class="icon-only-button" title="推送提交" :disabled="!history.selectedOid || advanced.loading || !remote.selectedRemote" @click="pushSelectedCommit">
                  <Upload :size="14" />
                </button>
              </div>

              <div class="log-operation-options">
                <label class="log-option" title="只应用反向变更，不自动创建 revert 提交">
                  <input v-model="operations.revertNoCommit" type="checkbox" />
                  <span>revert 不提交</span>
                </label>
                <label class="log-option" title="撤销提交后保留为已暂存变更">
                  <input v-model="operations.undoKeepStaged" type="checkbox" />
                  <span>撤销后暂存</span>
                </label>
                <select v-model="operations.resetMode" class="reset-select" title="重置模式">
                  <option value="soft">软重置</option>
                  <option value="mixed">混合重置</option>
                  <option value="hard">硬重置</option>
                </select>
              </div>

            </section>
          </aside>
        </div>
          <section v-else-if="activeLogDiffTab" class="log-diff-tab-pane">
            <div class="diff-header">
              <div class="diff-title-block">
                <span class="eyebrow">{{ activeLogDiffTab ? commitFileDiffModeLabels[activeLogDiffTab.mode] : "差异" }}</span>
                <h2 :title="activeLogDiffTab?.path">{{ activeLogDiffTab?.path }}</h2>
                <small>
                  文件 {{ activeLogDiffFilePosition }} · 差异 {{ currentLogDiffHunkPosition }}/{{ activeLogDiffHunkCount }}
                </small>
              </div>
              <div class="diff-header-actions">
                <div class="diff-nav-group" aria-label="日志差异跳转">
                  <button
                    class="icon-only-button diff-nav-button"
                    title="上一个差异"
                    :disabled="activeLogDiffHunkCount === 0"
                    @click="jumpLogDiffHunk(-1)"
                  >
                    <ArrowUp :size="15" />
                  </button>
                  <button
                    class="icon-only-button diff-nav-button"
                    title="下一个差异"
                    :disabled="activeLogDiffHunkCount === 0"
                    @click="jumpLogDiffHunk(1)"
                  >
                    <ArrowDown :size="15" />
                  </button>
                </div>
                <div class="diff-nav-group" aria-label="日志文件切换">
                  <button
                    class="icon-only-button diff-nav-button"
                    title="上一个文件"
                    :disabled="!canSelectPreviousLogDiffFile"
                    @click="selectAdjacentLogDiffFile(-1)"
                  >
                    <ArrowLeft :size="15" />
                  </button>
                  <button
                    class="icon-only-button diff-nav-button"
                    title="下一个文件"
                    :disabled="!canSelectNextLogDiffFile"
                    @click="selectAdjacentLogDiffFile(1)"
                  >
                    <ArrowRight :size="15" />
                  </button>
                </div>
                <button class="tool-button" @click="selectLogRootTab">
                  <VcsIcon :size="14" />
                  <span>返回日志</span>
                </button>
                <button
                  class="icon-only-button"
                  title="关闭标签页"
                  @click="activeLogDiffTab && closeLogDiffTab(activeLogDiffTab.id)"
                >
                  <X :size="14" />
                </button>
              </div>
            </div>

            <div ref="logDiffScroller" class="diff-scroller side-by-side-scroller">
              <div v-if="activeLogDiffTab?.loading" class="diff-empty">加载中</div>
              <div v-else-if="activeLogDiffTab?.error" class="diff-empty">{{ activeLogDiffTab?.error }}</div>
              <div v-else-if="!activeLogDiffHasContent" class="diff-empty">没有差异</div>
              <div v-else-if="activeLogSideBySideDiffRows.length === 0" class="diff-empty">无法以文本方式显示此差异</div>
              <div v-else class="side-by-side-diff">
                <div class="side-by-side-file-header">
                  <div class="side-by-side-title">
                    <strong>提交</strong>
                    <span>{{ activeLogDiffTab?.shortOid }}</span>
                  </div>
                  <div class="side-by-side-title">
                    <strong>来源</strong>
                    <span>{{ activeLogDiffTab?.subtitle }}</span>
                  </div>
                </div>
                <div class="side-by-side-editors">
                  <div class="side-by-side-column old" @scroll="syncSideBySideEditorScroll">
                    <div class="side-by-side-column-lines">
                      <div
                        v-for="row in activeLogSideBySideDiffRows"
                        :key="`old-${row.id}`"
                        class="side-by-side-line"
                        :class="[row.type, { active: row.hunkIndex === activeLogDiffHunkIndex }]"
                        :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
                      >
                        <div class="diff-cell old" :class="row.old.type">
                          <span class="line-number">{{ row.old.lineNumber ?? "" }}</span>
                          <span class="line-content"><template
                            v-for="(token, tokenIndex) in row.old.tokens"
                            :key="tokenIndex"
                          ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="side-by-side-column new" @scroll="syncSideBySideEditorScroll">
                    <div class="side-by-side-column-lines">
                      <div
                        v-for="row in activeLogSideBySideDiffRows"
                        :key="`new-${row.id}`"
                        class="side-by-side-line"
                        :class="[row.type, { active: row.hunkIndex === activeLogDiffHunkIndex }]"
                        :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
                      >
                        <div class="diff-cell new" :class="row.new.type">
                          <span class="line-number">{{ row.new.lineNumber ?? "" }}</span>
                          <span class="line-content"><template
                            v-for="(token, tokenIndex) in row.new.tokens"
                            :key="tokenIndex"
                          ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </section>
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
            <button class="tool-button" :disabled="repos.initializedItems.length === 0 || remote.loading" @click="fetchAllRepositories">
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

    <div v-if="logFilePickerOpen" class="modal-backdrop" @click.self="closeLogFilePicker">
      <section class="log-file-picker-modal" role="dialog" aria-modal="true" aria-label="选择日志文件">
        <header class="log-file-picker-head">
          <div class="section-title">
            <FileSearch :size="16" />
            <span>文件</span>
          </div>
          <button class="icon-only-button" title="关闭" @click="closeLogFilePicker">
            <X :size="14" />
          </button>
        </header>

        <label class="log-search-field log-file-picker-search">
          <Search :size="14" />
          <input v-model="logFilePickerSearch" placeholder="搜索文件" />
        </label>

        <div class="log-file-picker-tree">
          <div v-if="project.loading" class="diff-empty">加载中</div>
          <div v-else-if="project.error" class="message error">{{ project.error }}</div>
          <div v-else-if="visibleLogFilePickerRows.length <= 1" class="diff-empty">没有文件</div>
          <template v-else>
            <button
              v-for="file in visibleLogFilePickerRows"
              :key="`log-file-picker-${file.path}`"
              class="log-file-picker-row"
              :class="logFilePickerRowClass(file)"
              :style="logFilePickerIndent(file)"
              :title="projectFileTitle(file)"
              @click="file.directory ? toggleLogFilePickerDirectory(file.path) : toggleLogFileFilter(file.path)"
            >
              <span class="project-file-disclosure">
                <ChevronDown v-if="file.directory && isLogFilePickerDirectoryExpanded(file.path)" :size="13" />
                <ChevronRight v-else-if="file.directory" :size="13" />
              </span>
              <FolderOpen v-if="file.directory && isLogFilePickerDirectoryExpanded(file.path)" :size="14" />
              <Folder v-else-if="file.directory" :size="14" />
              <FileIcon v-else :size="14" />
              <span class="project-file-name" :class="{ root: file.path === PROJECT_ROOT_PATH }">
                <template v-if="file.path === PROJECT_ROOT_PATH">
                  <strong>{{ repos.name }}</strong>
                  <small>{{ repos.path }}</small>
                </template>
                <template v-else>{{ file.name }}</template>
              </span>
              <span v-if="!file.directory" class="log-file-picker-check">
                <Check v-if="!file.directory && isLogFileFilterSelected(file.path)" :size="13" />
              </span>
              <span v-else />
            </button>
          </template>
        </div>

        <footer class="log-file-picker-footer">
          <span>{{ logFilePickerDraft.length }} 个文件</span>
          <button class="mini-button" :disabled="logFilePickerDraft.length === 0" @click="clearLogFilePickerDraft">清空</button>
          <button class="icon-button" @click="closeLogFilePicker">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button class="icon-button primary" @click="applyLogFileFilters">
            <Check :size="14" />
            <span>应用</span>
          </button>
        </footer>
      </section>
    </div>

    <div v-if="projectNameDialog" class="modal-backdrop" @click.self="cancelProjectNameDialog">
      <form class="project-name-modal" role="dialog" aria-modal="true" :aria-label="projectNameDialog.title" @submit.prevent="submitProjectNameDialog">
        <header>
          <h2>{{ projectNameDialog.title }}</h2>
          <button class="icon-only-button" type="button" title="关闭" @click="cancelProjectNameDialog">
            <X :size="14" />
          </button>
        </header>
        <input
          v-model="projectNameDialog.value"
          autofocus
          placeholder="输入名称"
          @input="projectNameDialog.error = ''"
          @keydown.esc.prevent="cancelProjectNameDialog"
        />
        <p v-if="projectNameDialog.error" class="project-name-error">{{ projectNameDialog.error }}</p>
        <footer>
          <button class="icon-button" type="button" @click="cancelProjectNameDialog">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button class="icon-button primary" type="submit">
            <Check :size="14" />
            <span>确认</span>
          </button>
        </footer>
      </form>
    </div>

    <div v-if="projectCloseDialog" class="modal-backdrop" @click.self="cancelProjectCloseDialog">
      <section class="project-unsaved-modal" role="dialog" aria-modal="true" aria-label="保存未保存的文件">
        <header>
          <h2>保存未保存的文件？</h2>
          <button
            class="icon-only-button"
            type="button"
            title="关闭"
            :disabled="projectCloseDialog.saving"
            @click="cancelProjectCloseDialog"
          >
            <X :size="14" />
          </button>
        </header>
        <p>
          <strong>{{ projectCloseDialog.name }}</strong> 有未保存的修改。关闭前要保存吗？
        </p>
        <p class="project-unsaved-path">{{ projectCloseDialog.path }}</p>
        <p v-if="projectCloseDialog.error" class="project-name-error">{{ projectCloseDialog.error }}</p>
        <footer>
          <button class="icon-button danger" type="button" :disabled="projectCloseDialog.saving" @click="discardAndCloseProjectFile">
            <Trash2 :size="14" />
            <span>不保存</span>
          </button>
          <button class="icon-button" type="button" :disabled="projectCloseDialog.saving" @click="cancelProjectCloseDialog">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button class="icon-button primary" type="button" :disabled="projectCloseDialog.saving" @click="saveAndCloseProjectFile">
            <Check :size="14" />
            <span>{{ projectCloseDialog.saving ? "保存中" : "保存文件" }}</span>
          </button>
        </footer>
      </section>
    </div>

    <div
      v-if="changeFileContextMenu"
      class="context-menu change-file-menu"
      :style="{ left: `${changeFileContextMenu.x}px`, top: `${changeFileContextMenu.y}px` }"
      @click.stop
    >
      <button @click="discardChangeFilesFromContext(changeFileContextMenu.file)">
        <span>回滚变更</span>
        <small>{{ changeContextLabel(changeFileContextMenu.file) }}</small>
      </button>
      <button
        v-for="list in changelistMoveTargets(changeFileContextMenu.file)"
        :key="`move-change-${list.id}`"
        @click="moveChangeFilesToChangelistFromContext(changeFileContextMenu.file, list.id)"
      >
        <span>移至另一个变更清单</span>
        <small>{{ list.name }}</small>
      </button>
      <button v-if="changelistMoveTargets(changeFileContextMenu.file).length === 0" disabled>
        <span>移至另一个变更清单</span>
        <small>无其他清单</small>
      </button>
      <button @click="showChangeFileDiffFromContext(changeFileContextMenu.file, changeFileContextMenu.side)">
        <span>显示差异</span>
        <small>{{ changeFileContextMenu.side === "staged" ? "暂存区" : "工作区" }}</small>
      </button>
      <button
        :disabled="deletableChangeContextPaths(changeFileContextMenu.file).length === 0"
        @click="deleteChangeFilesFromContext(changeFileContextMenu.file)"
      >
        <span>删除</span>
        <small>从工作区</small>
      </button>
      <div class="context-menu-separator" />
      <button @click="createChangelistFromChangeContext(changeFileContextMenu.file)">
        <span>新建变更清单</span>
      </button>
      <button @click="editChangelistFromChangeContext(changeFileContextMenu.file)">
        <span>编辑变更清单</span>
        <small>{{ changelistForChangeContext(changeFileContextMenu.file).name }}</small>
      </button>
      <div class="context-menu-separator" />
      <button @click="showChangeFileHistoryFromContext(changeFileContextMenu.file)">
        <span>查看提交记录</span>
      </button>
    </div>

    <div
      v-if="changeListContextMenu"
      class="context-menu change-list-menu"
      :style="{ left: `${changeListContextMenu.x}px`, top: `${changeListContextMenu.y}px` }"
      @click.stop
    >
      <button @click="createChangelistFromListContext">
        <span>新建变更清单</span>
      </button>
      <button @click="editChangelistFromListContext(changeListContextMenu.listId)">
        <span>编辑变更清单</span>
        <small>{{ changelistById(changeListContextMenu.listId).name }}</small>
      </button>
      <div class="context-menu-separator" />
      <button
        :disabled="!canDeleteChangelist(changeListContextMenu.listId)"
        @click="deleteChangelistFromListContext(changeListContextMenu.listId)"
      >
        <span>删除变更清单</span>
        <small>{{ canDeleteChangelist(changeListContextMenu.listId) ? changelistById(changeListContextMenu.listId).name : "默认清单不可删除" }}</small>
      </button>
    </div>

    <div
      v-if="projectFileContextMenu"
      class="context-menu project-file-menu"
      :style="{ left: `${projectFileContextMenu.x}px`, top: `${projectFileContextMenu.y}px` }"
      @click.stop
    >
      <button v-if="canCreateInProjectContext(projectFileContextMenu.file)" @click="createProjectFileFromContext(projectFileContextMenu.file)">
        <span>新建文件</span>
      </button>
      <button v-if="canCreateInProjectContext(projectFileContextMenu.file)" @click="createProjectDirectoryFromContext(projectFileContextMenu.file)">
        <span>新建文件夹</span>
      </button>
      <div v-if="canCreateInProjectContext(projectFileContextMenu.file) || canModifyProjectEntry(projectFileContextMenu.file)" class="context-menu-separator" />
      <button v-if="canModifyProjectEntry(projectFileContextMenu.file) && projectFileContextMenu.file" @click="cutProjectEntry(projectFileContextMenu.file)">
        <span>剪切</span>
      </button>
      <button v-if="canModifyProjectEntry(projectFileContextMenu.file) && projectFileContextMenu.file" @click="copyProjectEntryToInternalClipboard(projectFileContextMenu.file)">
        <span>复制</span>
      </button>
      <button v-if="canCreateInProjectContext(projectFileContextMenu.file)" :disabled="!canPasteProjectEntry(projectFileContextMenu.file)" @click="pasteProjectEntryToContext(projectFileContextMenu.file)">
        <span>粘贴</span>
        <small>{{ projectFileClipboard ? (projectFileClipboard.mode === "cut" ? "移动" : "复制") : "无内容" }}</small>
      </button>
      <div v-if="projectFileContextMenu.file && (canModifyProjectEntry(projectFileContextMenu.file) || canPasteProjectEntry(projectFileContextMenu.file))" class="context-menu-separator" />
      <button v-if="projectFileContextMenu.file" @click="copyProjectAbsolutePath(projectFileContextMenu.file)">
        <span>复制路径</span>
      </button>
      <button v-if="projectFileContextMenu.file" @click="copyProjectRelativePath(projectFileContextMenu.file)">
        <span>复制相对路径</span>
      </button>
      <div v-if="canModifyProjectEntry(projectFileContextMenu.file)" class="context-menu-separator" />
      <button v-if="canModifyProjectEntry(projectFileContextMenu.file) && projectFileContextMenu.file" @click="renameProjectEntryFromContext(projectFileContextMenu.file)">
        <span>重命名</span>
      </button>
      <button v-if="canModifyProjectEntry(projectFileContextMenu.file) && projectFileContextMenu.file" @click="deleteProjectEntryFromContext(projectFileContextMenu.file)">
        <span>删除</span>
      </button>
      <div v-if="projectFileContextMenu.file && !projectFileContextMenu.file.directory" class="context-menu-separator" />
      <button v-if="projectFileContextMenu.file && !projectFileContextMenu.file.directory" @click="openProjectEntryLog(projectFileContextMenu.file)">
        <span>查看变更日志</span>
      </button>
    </div>

    <div
      v-if="logRefContextMenu"
      class="context-menu log-ref-menu"
      :style="{ left: `${logRefContextMenu.x}px`, top: `${logRefContextMenu.y}px` }"
      @click.stop
    >
      <button @click="showLogRefFromContext(logRefContextMenu)">
        <span>查看此引用日志</span>
        <small>{{ logRefContextRefName(logRefContextMenu) }}</small>
      </button>
      <button :disabled="!canCheckoutLogRefContext(logRefContextMenu)" @click="checkoutLogRefFromContext(logRefContextMenu)">
        <span>{{ logRefContextMenu.kind === "tag" ? "检出标签" : logRefContextMenu.kind === "remote" ? "检出远程分支" : "切换到此分支" }}</span>
        <small>{{ canCheckoutLogRefContext(logRefContextMenu) ? logRefContextRefName(logRefContextMenu) : "当前分支" }}</small>
      </button>
      <button @click="createBranchFromLogRefContext(logRefContextMenu)">
        <span>从此处新建分支</span>
        <small>{{ logRefContextRefName(logRefContextMenu) }}</small>
      </button>

      <div class="context-menu-separator" />
      <button
        v-if="logRefContextMenu.kind !== 'tag'"
        :disabled="!canMergeOrRebaseLogRefContext(logRefContextMenu)"
        @click="mergeLogRefIntoCurrent(logRefContextMenu)"
      >
        <span>合并到当前分支</span>
        <small>{{ logRefContextMenu.kind === "local" && logRefContextMenu.branch.current ? "当前分支" : logRefContextRefName(logRefContextMenu) }}</small>
      </button>
      <button
        v-if="logRefContextMenu.kind !== 'tag'"
        :disabled="!canMergeOrRebaseLogRefContext(logRefContextMenu)"
        @click="rebaseCurrentOntoLogRef(logRefContextMenu)"
      >
        <span>变基当前分支到此处</span>
        <small>{{ logRefContextMenu.kind === "local" && logRefContextMenu.branch.current ? "当前分支" : logRefContextRefName(logRefContextMenu) }}</small>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'remote'"
        :disabled="!canSetLogRefContextUpstream(logRefContextMenu)"
        @click="setCurrentBranchUpstreamFromContext(logRefContextMenu)"
      >
        <span>设为当前分支上游</span>
        <small>{{ branchNameLabel(branch?.currentBranch) }}</small>
      </button>

      <div class="context-menu-separator" />
      <button @click="toggleLogRefFavoriteFromContext(logRefContextMenu)">
        <span>{{ isLogRefContextFavorite(logRefContextMenu) ? "取消收藏" : "收藏" }}</span>
        <small>{{ formatRefName(logRefContextFullName(logRefContextMenu)) }}</small>
      </button>
      <button @click="copyLogRefNameFromContext(logRefContextMenu)">
        <span>复制引用名称</span>
        <small>{{ logRefContextRefName(logRefContextMenu) }}</small>
      </button>

      <div class="context-menu-separator" />
      <button
        v-if="canRenameLogRefContext(logRefContextMenu)"
        @click="renameLogBranchFromContext(logRefContextMenu)"
      >
        <span>重命名分支</span>
        <small>{{ logRefContextRefName(logRefContextMenu) }}</small>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'tag'"
        :disabled="branches.loading || !remote.selectedRemote"
        @click="pushLogTagFromContext(logRefContextMenu)"
      >
        <span>推送标签</span>
        <small>{{ remote.selectedRemote || "origin" }}</small>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'tag'"
        :disabled="branches.loading || !remote.selectedRemote"
        @click="deleteRemoteLogTagFromContext(logRefContextMenu)"
      >
        <span>删除远程标签</span>
        <small>{{ remote.selectedRemote || "origin" }}</small>
      </button>
      <button
        class="danger-menu-item"
        :disabled="!canDeleteLogRefContext(logRefContextMenu)"
        @click="deleteLogRefFromContext(logRefContextMenu)"
      >
        <span>{{ logRefContextMenu.kind === "tag" ? "删除本地标签" : logRefContextMenu.kind === "remote" ? "删除远程分支" : "删除本地分支" }}</span>
        <small>{{ canDeleteLogRefContext(logRefContextMenu) ? logRefContextRefName(logRefContextMenu) : "当前分支不可删除" }}</small>
      </button>
    </div>

    <div
      v-if="logFileContextMenu"
      class="context-menu log-file-menu"
      :style="{ left: `${logFileContextMenu.x}px`, top: `${logFileContextMenu.y}px` }"
      @click.stop
    >
      <button @click="showCommitFileDiff(logFileContextMenu.row)">
        <span>显示差异</span>
        <small>⌘D</small>
      </button>
      <button @click="showCommitFileDiff(logFileContextMenu.row)">
        <span>在新标签页中显示差异</span>
      </button>
      <div class="context-menu-separator" />
      <button @click="showCommitFileDiff(logFileContextMenu.row, 'worktree')">
        <span>与本地比较</span>
      </button>
      <button @click="showCommitFileDiff(logFileContextMenu.row, 'parent-worktree')">
        <span>将之前版本与本地端比较</span>
      </button>
      <div class="context-menu-separator" />
      <button disabled>
        <span>编辑来源</span>
        <small>⌘↓</small>
      </button>
      <button disabled>
        <span>开启储存库版本</span>
      </button>
      <div class="context-menu-separator" />
      <button @click="revertLogFileChange(logFileContextMenu.row)">
        <span>还原选取的变更</span>
      </button>
      <button @click="cherryPickLogFile(logFileContextMenu.row)">
        <span>Cherry-pick 所选变更</span>
      </button>
      <button disabled>
        <span>将选取的变更提取到个别提交...</span>
      </button>
      <button disabled>
        <span>搁弃所选变更</span>
      </button>
      <div class="context-menu-separator" />
      <button @click="createPatchFromLogFile(logFileContextMenu.row)">
        <span>建立修补程式...</span>
      </button>
      <button @click="cherryPickLogFile(logFileContextMenu.row)">
        <span>从修订版本获取</span>
      </button>
      <button @click="showLogFileHistory(logFileContextMenu.row)">
        <span>截至此处的历程记录</span>
      </button>
      <button @click="showCommitFileDiff(logFileContextMenu.row)">
        <span>显示对父项的变更</span>
      </button>
    </div>
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

.notice-toast {
  position: fixed;
  top: 72px;
  right: 18px;
  z-index: 120;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) 24px;
  align-items: center;
  gap: 8px;
  min-width: 260px;
  max-width: min(520px, calc(100vw - 36px));
  min-height: 40px;
  padding: 8px 8px 8px 12px;
  border: 1px solid #8fc19a;
  border-radius: 8px;
  color: #1f6537;
  background: #ecf8ef;
  box-shadow: 0 18px 46px rgba(34, 48, 42, 0.2);
}

.notice-toast span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  font-weight: 700;
}

.notice-toast button {
  display: inline-grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 5px;
  color: inherit;
  background: transparent;
}

.notice-toast button:hover {
  background: rgba(31, 101, 55, 0.12);
}

.notice-toast-enter-active,
.notice-toast-leave-active {
  transition:
    opacity 0.16s ease,
    transform 0.16s ease;
}

.notice-toast-enter-from,
.notice-toast-leave-to {
  opacity: 0;
  transform: translateY(-8px);
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

.context-menu {
  position: fixed;
  z-index: 80;
  display: grid;
  gap: 2px;
  width: 260px;
  padding: 6px;
  border: 1px solid #bdc8c1;
  border-radius: 8px;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(34, 48, 42, 0.2);
}

.context-menu button {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  width: 100%;
  min-height: 26px;
  padding: 0 9px;
  border: 0;
  border-radius: 6px;
  color: #25312b;
  background: transparent;
  font-size: 13px;
  text-align: left;
}

.context-menu button:hover:not(:disabled) {
  color: #ffffff;
  background: #3f6ea5;
}

.context-menu button.danger-menu-item:not(:hover):not(:disabled) span {
  color: #b83e31;
}

.context-menu button:disabled {
  color: #a0aaa4;
  cursor: default;
}

.context-menu button span,
.context-menu button small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-menu button small {
  color: #7a867d;
  font-size: 11px;
}

.context-menu button:hover:not(:disabled) small {
  color: rgba(255, 255, 255, 0.78);
}

.context-menu-separator {
  height: 1px;
  margin: 4px 3px;
  background: #e3e8e4;
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
.icon-button.primary,
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

.empty-panel p {
  max-width: min(560px, 70vw);
  margin: -6px 0 2px;
  color: #68766f;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  overflow-wrap: anywhere;
  text-align: center;
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
.project-tree-pane,
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

.project-tree-pane {
  display: grid;
  grid-template-rows: 46px minmax(0, 1fr);
  background: #fbfcfa;
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

.project-row.uninitialized .project-dot {
  background: #c18a25;
}

.project-row.uninitialized.active .project-dot {
  background: #d79c2f;
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

.project-file-browser {
  display: grid;
  gap: 8px;
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid #dce2dd;
}

.project-tree-pane .project-file-browser {
  min-height: 0;
  margin-top: 0;
  padding-top: 0;
  border-top: 0;
}

.project-file-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.project-file-list {
  display: grid;
  align-content: start;
  max-height: calc(100vh - 250px);
  overflow: auto;
}

.project-tree-pane .project-file-list {
  min-height: 0;
  max-height: none;
}

.project-file-row {
  display: grid;
  grid-template-columns: 14px 16px minmax(0, 1fr) 8px;
  align-items: center;
  gap: 5px;
  min-height: 28px;
  padding: 0 8px 0 6px;
  border: 0;
  border-radius: 6px;
  color: #26312c;
  background: transparent;
  text-align: left;
}

.project-file-row.directory {
  color: #4f5e56;
  font-weight: 700;
}

.project-file-row.root {
  min-height: 30px;
  margin-bottom: 2px;
  color: #233228;
  background: #e7edf8;
  font-weight: 800;
}

.project-file-disclosure {
  display: grid;
  place-items: center;
  width: 14px;
  color: #7a877f;
}

.project-status-marker {
  width: 6px;
  height: 6px;
  border-radius: 999px;
}

.project-file-row.status-added,
.project-tab.status-added {
  color: #237044;
}

.project-file-row.status-modified,
.project-file-row.status-typechange,
.project-tab.status-modified,
.project-tab.status-typechange {
  color: #8b6500;
}

.project-file-row.status-deleted,
.project-tab.status-deleted {
  color: #b7332c;
}

.project-file-row.status-renamed,
.project-tab.status-renamed {
  color: #3463a6;
}

.project-file-row.status-conflicted,
.project-tab.status-conflicted {
  color: #b64200;
}

.project-file-row.status-ignored,
.project-tab.status-ignored {
  color: #8b9690;
}

.project-file-row.status-added .project-status-marker,
.project-tab.status-added::before {
  background: #2f9d58;
}

.project-file-row.status-modified .project-status-marker,
.project-file-row.status-typechange .project-status-marker,
.project-tab.status-modified::before,
.project-tab.status-typechange::before {
  background: #d39a00;
}

.project-file-row.status-deleted .project-status-marker,
.project-tab.status-deleted::before {
  background: #d14b42;
}

.project-file-row.status-renamed .project-status-marker,
.project-tab.status-renamed::before {
  background: #4b7fd0;
}

.project-file-row.status-conflicted .project-status-marker,
.project-tab.status-conflicted::before {
  background: #e06416;
}

.project-file-row.status-ignored .project-status-marker,
.project-tab.status-ignored::before {
  background: #9aa39e;
}

.project-file-row:hover:not(:disabled),
.project-file-row.active {
  background: #e8f0fb;
}

.project-file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.project-file-name.root {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}

.project-file-name.root strong,
.project-file-name.root small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-file-name.root strong {
  flex: 0 1 auto;
  min-width: max-content;
  font-size: 12px;
}

.project-file-name.root small {
  flex: 1 1 auto;
  min-width: 0;
  color: #76827b;
  font-size: 11px;
  font-weight: 600;
}

.project-file-empty {
  padding: 10px 8px;
  color: #738077;
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

.log-ref-pane {
  display: grid;
  grid-template-columns: 40px minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr);
}

.log-ref-pane.collapsed {
  grid-template-columns: 40px;
}

.log-ref-toolbar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  min-width: 0;
  min-height: 0;
  padding: 6px 4px;
  border-right: 1px solid #dce2dd;
  background: #f4f6f4;
}

.log-ref-tool-button {
  display: inline-grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 1px solid transparent;
  border-radius: 4px;
  color: #526158;
  background: transparent;
}

.log-ref-tool-button:hover:not(:disabled),
.log-ref-tool-button.active {
  border-color: #c5cec8;
  background: #ffffff;
}

.log-ref-tool-button.active {
  color: #d0a044;
}

.log-ref-tool-button.danger:hover:not(:disabled) {
  color: #b83e31;
}

.log-ref-tool-button:disabled {
  opacity: 0.42;
  cursor: not-allowed;
}

.log-ref-tool-separator {
  width: 18px;
  height: 1px;
  margin: 2px 0;
  background: #d4ddd7;
}

.log-ref-content {
  display: grid;
  grid-template-rows: 40px minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
}

.log-ref-search-bar {
  display: flex;
  align-items: center;
  padding: 5px 8px;
}

.log-ref-search-field {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: 6px;
  width: 100%;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 4px;
  color: #68766f;
  background: #ffffff;
}

.log-ref-search-field input {
  min-width: 0;
  width: 100%;
  border: 0;
  outline: 0;
  color: #26312c;
  background: transparent;
  font-size: 12px;
}

.log-ref-search-field input::placeholder {
  color: #87918b;
}

.log-ref-list {
  display: grid;
  align-content: start;
  gap: 1px;
  min-height: 0;
  overflow: auto;
  padding: 6px 8px 8px;
}

.log-ref-toggle {
  display: grid;
  grid-template-columns: 16px minmax(0, 1fr);
  align-items: center;
  gap: 5px;
  width: 100%;
  min-height: 24px;
  padding: 0 6px;
  border: 0;
  border-radius: 4px;
  color: #627168;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  text-align: left;
}

.log-ref-toggle.remote-root {
  grid-template-columns: 16px 16px minmax(0, 1fr);
  padding-left: 22px;
}

.log-ref-toggle:hover {
  background: #f0f4f1;
}

.log-ref-toggle span,
.log-ref-toggle small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-ref-children,
.log-ref-row,
.log-ref-head-row {
  display: grid;
}

.log-ref-children {
  gap: 0;
}

.log-ref-row {
  grid-template-columns: 16px minmax(0, 1fr);
  align-items: center;
  gap: 6px;
  width: 100%;
  min-height: 24px;
  padding: 0 6px;
  border: 0;
  border-radius: 4px;
  color: #25312b;
  background: transparent;
  text-align: left;
}

.log-ref-row.local,
.log-ref-row.tag-ref {
  padding-left: 28px;
}

.log-ref-row.remote {
  padding-left: 50px;
}

.log-ref-row:hover,
.log-ref-row.active {
  background: #e7eaed;
}

.log-ref-row.context-target:not(.active) {
  background: #e8eef5;
}

.log-ref-row.active {
  color: #17202a;
}

.log-ref-row.current:not(.active) {
  background: #f0f2f4;
}

.log-ref-row.current > svg,
.log-ref-row.favorite > svg {
  color: #d0a044;
}

.log-ref-row span {
  display: block;
  min-width: 0;
}

.log-ref-row strong {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-ref-row strong {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.2;
}

.log-ref-group {
  display: grid;
  gap: 0;
}

.log-ref-head-row {
  align-items: center;
  min-height: 24px;
  padding: 0 6px 0 24px;
  border: 0;
  border-radius: 4px;
  color: #627168;
  background: transparent;
  font-size: 12px;
  font-weight: 700;
  text-align: left;
}

.log-ref-head-row:hover,
.log-ref-head-row.active {
  background: #eef1f4;
}

.log-ref-empty {
  padding: 8px 6px;
  color: #7a867d;
  font-size: 12px;
}

.tag-dot {
  width: 9px;
  height: 9px;
  border-radius: 2px;
  background: #d0a044;
  transform: rotate(45deg);
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: 28px;
  background: rgba(24, 31, 27, 0.28);
}

.log-file-picker-modal {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  gap: 10px;
  width: min(680px, calc(100vw - 56px));
  height: min(720px, calc(100vh - 56px));
  min-height: 420px;
  padding: 14px;
  border: 1px solid #cbd5cf;
  border-radius: 8px;
  box-shadow: 0 24px 64px rgba(31, 47, 39, 0.24);
  background: #fbfcfa;
}

.project-name-modal,
.project-unsaved-modal {
  display: grid;
  gap: 12px;
  width: min(420px, calc(100vw - 56px));
  padding: 14px;
  border: 1px solid #cbd5cf;
  border-radius: 8px;
  box-shadow: 0 24px 64px rgba(31, 47, 39, 0.24);
  background: #fbfcfa;
}

.project-name-modal header,
.project-name-modal footer,
.project-unsaved-modal header,
.project-unsaved-modal footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.project-name-modal h2,
.project-unsaved-modal h2 {
  margin: 0;
  color: #26312b;
  font-size: 15px;
}

.project-name-modal input {
  width: 100%;
}

.project-name-error {
  margin: -4px 0 0;
  color: #b64242;
  font-size: 12px;
}

.project-name-modal footer {
  justify-content: flex-end;
}

.project-unsaved-modal p {
  margin: 0;
  color: #4b574f;
  font-size: 13px;
  line-height: 1.5;
}

.project-unsaved-modal strong {
  color: #25312b;
}

.project-unsaved-path {
  overflow: hidden;
  padding: 8px 10px;
  border: 1px solid #dce2dd;
  border-radius: 6px;
  color: #68756d;
  background: #f3f6f2;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-unsaved-modal footer {
  justify-content: flex-end;
}

.log-file-picker-head,
.log-file-picker-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.log-file-picker-search {
  width: 100%;
}

.log-file-picker-tree {
  min-height: 0;
  overflow: auto;
  padding: 6px;
  border: 1px solid #dce2dd;
  border-radius: 8px;
  background: #ffffff;
}

.log-file-picker-row {
  display: grid;
  grid-template-columns: 14px 16px minmax(0, 1fr) 18px;
  align-items: center;
  gap: 5px;
  width: 100%;
  min-height: 28px;
  padding: 0 8px 0 6px;
  border: 0;
  border-radius: 6px;
  color: #26312c;
  background: transparent;
  text-align: left;
}

.log-file-picker-row:hover,
.log-file-picker-row.selected {
  background: #e8f0fb;
}

.log-file-picker-row.directory {
  color: #4f5e56;
  font-weight: 700;
}

.log-file-picker-row.root {
  min-height: 30px;
  margin-bottom: 2px;
  color: #233228;
  background: #e7edf8;
  font-weight: 800;
}

.log-file-picker-footer {
  justify-content: flex-end;
}

.log-file-picker-footer > span {
  margin-right: auto;
  color: #627168;
  font-size: 12px;
}

.log-tab-workspace {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  background: #ffffff;
}

.log-workspace-tabs {
  display: flex;
  flex: 0 0 auto;
  min-height: 38px;
  overflow-x: auto;
  border-bottom: 1px solid #dce2dd;
  background: #f3f5f2;
}

.log-root-tab,
.log-workspace-tab {
  position: relative;
  flex: 0 0 auto;
  min-width: 118px;
  max-width: 260px;
  border-right: 1px solid #dce2dd;
  color: #445149;
  background: #edf1ec;
}

.log-root-tab {
  display: inline-grid;
  grid-template-columns: 16px minmax(0, 1fr);
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  border-top: 0;
  border-bottom: 0;
  border-left: 0;
  text-align: left;
}

.log-root-tab::before,
.log-workspace-tab::before {
  content: "";
  position: absolute;
  left: 10px;
  bottom: 0;
  width: 42px;
  height: 2px;
  background: transparent;
}

.log-root-tab.active,
.log-workspace-tab.active {
  color: #202b26;
  background: #fbfcfa;
}

.log-root-tab.active::before,
.log-workspace-tab.active::before {
  background: #4c82d9;
}

.log-workspace-tab {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 28px;
  align-items: stretch;
}

.log-workspace-tab.loading::before {
  background: #b89445;
}

.log-workspace-tab.error::before {
  background: #d94f43;
}

.log-workspace-tab-select,
.log-workspace-tab-close {
  min-width: 0;
  border: 0;
  color: inherit;
  background: transparent;
}

.log-workspace-tab-select {
  display: grid;
  grid-template-columns: 16px minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
  padding: 0 8px 0 10px;
  text-align: left;
}

.log-root-tab span,
.log-workspace-tab-select span,
.log-workspace-tab-select small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.log-workspace-tab-select small {
  color: #728078;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 10px;
}

.log-workspace-tab-close {
  display: grid;
  place-items: center;
  color: #7a877f;
}

.log-root-tab:hover,
.log-workspace-tab:hover,
.log-workspace-tab-close:hover {
  background: #e4e9e3;
}

.log-diff-tab-pane {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  background: #ffffff;
}

.log-workbench {
  display: grid;
  grid-template-columns: minmax(440px, 1fr) minmax(280px, 330px);
  flex: 1 1 auto;
  min-height: 0;
  min-width: 0;
  background: #ffffff;
}

.log-commit-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  border-right: 1px solid #dce2dd;
}

.log-topbar {
  display: grid;
  grid-template-columns: minmax(160px, 1.2fr) minmax(112px, 0.62fr) minmax(112px, 0.62fr) auto 34px;
  align-items: center;
  gap: 7px;
  flex: 0 0 48px;
  min-width: 0;
  padding: 7px 10px;
  border-bottom: 1px solid #dce2dd;
  background: #f6f8f6;
}

.log-search-field {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: 4px;
  min-width: 0;
  height: 32px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #68766f;
  background: #ffffff;
}

.log-search-field input,
.log-mini-filter {
  min-width: 0;
  width: 100%;
  border: 0;
  outline: 0;
  color: #26312c;
  background: transparent;
  font-size: 12px;
}

.log-mini-filter {
  height: 32px;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  background: #ffffff;
}

.log-filter-picker {
  position: relative;
  min-width: 0;
}

.log-filter-button {
  display: inline-grid;
  grid-template-columns: 16px minmax(0, 1fr) 14px;
  align-items: center;
  gap: 6px;
  width: 100%;
  height: 32px;
  min-width: 0;
  padding: 0 8px;
  border: 1px solid #c5cec8;
  border-radius: 7px;
  color: #627168;
  background: #ffffff;
  text-align: left;
}

.log-filter-button.active {
  border-color: #9db8dc;
  color: #365f91;
  background: #e9f0f7;
}

.log-filter-button span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.log-filter-popover {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 25;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  width: min(300px, 62vw);
  max-height: 320px;
  border: 1px solid #cbd5cf;
  border-radius: 8px;
  box-shadow: 0 18px 38px rgba(31, 47, 39, 0.16);
  background: #ffffff;
  overflow: hidden;
}

.log-filter-popover-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 38px;
  padding: 5px 8px 5px 12px;
  border-bottom: 1px solid #e6ebe7;
  color: #25312b;
}

.log-filter-popover-head strong {
  font-size: 12px;
}

.log-filter-options {
  display: grid;
  align-content: start;
  min-height: 0;
  overflow: auto;
  padding: 6px;
}

.log-check-row {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: 7px;
  min-height: 34px;
  padding: 3px 7px;
  border: 0;
  border-radius: 6px;
  color: #26312c;
  background: transparent;
  text-align: left;
}

.log-check-row:hover,
.log-check-row.selected {
  background: #eef4ef;
}

.log-checkmark,
.log-file-picker-check {
  display: inline-grid;
  place-items: center;
  width: 16px;
  height: 16px;
  border: 1px solid #c5cec8;
  border-radius: 4px;
  color: #ffffff;
}

.log-check-row.selected .log-checkmark,
.log-file-picker-row.selected .log-file-picker-check {
  border-color: #3f6ea5;
  background: #3f6ea5;
}

.log-check-label {
  display: grid;
  gap: 1px;
  min-width: 0;
}

.log-check-label strong,
.log-check-label small,
.log-check-row > small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-check-label strong {
  font-size: 12px;
}

.log-check-label small,
.log-check-row > small,
.log-picker-empty {
  color: #728078;
  font-size: 11px;
}

.log-picker-empty {
  padding: 10px 8px;
}

.log-filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  max-width: 180px;
  height: 30px;
  padding: 0 8px;
  border: 1px solid #d4dcd6;
  border-radius: 7px;
  color: #627168;
  background: #ffffff;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-filter-chip.active {
  border-color: #9db8dc;
  color: #365f91;
  background: #e9f0f7;
}

.log-filter-chip button {
  display: inline-grid;
  place-items: center;
  flex: 0 0 auto;
  width: 18px;
  height: 18px;
  padding: 0;
  border: 0;
  color: inherit;
  background: transparent;
}

.log-table-head,
.log-commit-row {
  display: grid;
  grid-template-columns: 94px minmax(0, 1fr) minmax(82px, 108px) minmax(124px, 152px);
  align-items: center;
  gap: 8px;
}

.log-table-head {
  flex: 0 0 28px;
  padding: 0 12px 0 0;
  border-bottom: 1px solid #eef1ed;
  color: #728078;
  background: #fbfcfa;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.log-commit-list {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  background: #ffffff;
}

.log-commit-row {
  position: relative;
  width: 100%;
  min-height: 30px;
  padding: 0 12px 0 0;
  border: 0;
  color: #25312b;
  background: transparent;
  text-align: left;
}

.log-commit-row:hover,
.log-commit-row.active {
  background: #e8f0fb;
}

.log-commit-row.active::before {
  content: "";
  position: absolute;
  inset: 0 auto 0 0;
  width: 3px;
  background: #4c82d9;
}

.log-graph-cell {
  position: relative;
  align-self: stretch;
  min-width: 42px;
  margin-left: 7px;
  overflow: visible;
}

.log-graph-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
}

.log-graph-path {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 2;
  vector-effect: non-scaling-stroke;
}

.log-graph-node {
  position: absolute;
  top: 50%;
  z-index: 1;
  width: 10px;
  height: 10px;
  border: 1.5px solid var(--graph-node-ring, #ffffff);
  border-radius: 50%;
  box-shadow: 0 0 0 1px rgba(57, 48, 28, 0.18);
  transform: translate(-50%, -50%);
}

.log-subject {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.log-subject strong,
.log-author,
.log-date {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-subject strong {
  min-width: 0;
  color: #25312b;
  font-size: 13px;
  font-weight: 700;
}

.log-author,
.log-date {
  color: #536159;
  font-size: 12px;
}

.log-date {
  text-align: right;
}

.log-detail-panel {
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto;
  min-height: 0;
  min-width: 0;
  background: #fbfcfa;
}

.log-files-panel {
  display: grid;
  grid-template-rows: 42px minmax(0, 1fr);
  min-height: 0;
  border-bottom: 1px solid #dce2dd;
}

.log-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 0 12px;
  border-bottom: 1px solid #eef1ed;
}

.log-panel-header small {
  color: #728078;
  font-size: 11px;
  white-space: nowrap;
}

.log-file-tree {
  min-height: 0;
  overflow: auto;
  padding: 6px 0;
}

.log-file-tree-row {
  display: grid;
  grid-template-columns: 14px 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: 5px;
  width: 100%;
  min-height: 26px;
  padding: 0 8px 0 6px;
  border: 0;
  border-radius: 6px;
  color: #25312b;
  background: transparent;
  text-align: left;
}

.log-file-tree-row:hover,
.log-file-tree-row.selected {
  background: #eef4ef;
}

.log-file-tree-row.directory {
  color: #4c5a52;
  font-weight: 700;
}

.log-file-disclosure {
  display: grid;
  place-items: center;
  width: 14px;
  color: #7a877f;
}

.log-file-name {
  display: grid;
  gap: 1px;
  min-width: 0;
}

.log-file-name strong,
.log-file-name small,
.log-file-tree-row > small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-file-name strong {
  color: inherit;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
}

.log-file-name small {
  color: #7a867d;
  font-size: 10px;
}

.log-file-tree-row.directory .log-file-name strong {
  font-family: inherit;
  font-size: 12px;
}

.log-file-tree-row small {
  color: #6b766f;
  font-size: 10px;
}

.log-file-tree-row.status-added .log-file-name strong {
  color: #237044;
}

.log-file-tree-row.status-modified .log-file-name strong,
.log-file-tree-row.status-typechange .log-file-name strong {
  color: #8b6500;
}

.log-file-tree-row.status-deleted .log-file-name strong {
  color: #b7332c;
}

.log-file-tree-row.status-renamed .log-file-name strong {
  color: #3463a6;
}

.log-file-tree-row.status-copied .log-file-name strong {
  color: #28736c;
}

.log-file-tree-row.status-conflicted .log-file-name strong {
  color: #b64200;
}

.log-info-panel {
  display: grid;
  gap: 8px;
  min-height: 190px;
  padding: 14px;
}

.log-info-body {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.log-info-body h2 {
  margin: 0;
  color: #25312b;
  font-size: 15px;
  line-height: 1.35;
}

.log-info-body p {
  margin: 0;
  color: #5d6a63;
  font-size: 12px;
  overflow-wrap: anywhere;
}

.log-info-body p strong {
  color: #25312b;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.log-detail-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.log-operation-options {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.log-operation-options .reset-select {
  height: 30px;
}

.log-diff-preview {
  min-width: 0;
}

.log-diff-preview summary {
  cursor: pointer;
  color: #536159;
  font-size: 12px;
  font-weight: 700;
}

.log-diff-preview .diff-scroller {
  max-height: 260px;
  margin-top: 8px;
  border: 1px solid #dce2dd;
  border-radius: 7px;
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

.file-actions-toggle {
  grid-column: 1 / -1;
  min-height: 24px;
  padding: 0 3px;
}

.file-list {
  flex: 1 1 180px;
  min-height: 0;
  overflow: auto;
  padding: 4px 0 8px;
  background: #fbfcfa;
}

.file-list-empty {
  padding: 12px 10px;
  color: #68766f;
  font-size: 12px;
}

.change-file-group {
  display: grid;
  align-content: start;
}

.change-file-group-header {
  display: grid;
  grid-template-columns: 15px 16px minmax(0, 1fr);
  align-items: center;
  gap: 5px;
  width: 100%;
  min-height: 26px;
  padding: 0 10px 0 6px;
  border: 0;
  color: #26312c;
  background: transparent;
  font-size: 12px;
  font-weight: 700;
  text-align: left;
}

.change-file-group-header:hover {
  background: #edf1ec;
}

.change-group-toggle,
.change-group-title {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  min-height: 26px;
  padding: 0;
  border: 0;
  color: inherit;
  background: transparent;
  font: inherit;
}

.change-group-toggle {
  justify-content: center;
}

.change-group-title {
  gap: 5px;
  text-align: left;
}

.change-group-title span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.change-file-group-header small {
  color: #6b766f;
  font-size: 12px;
  font-weight: 500;
}

.change-group-checkbox,
.file-row input {
  width: 14px;
  height: 14px;
  margin: 0;
  accent-color: #3677d8;
}

.change-file-group-list {
  display: grid;
}

.change-file-group-empty {
  min-height: 24px;
  padding: 5px 10px 5px 54px;
  color: #87928b;
  font-size: 12px;
}

.file-row {
  display: grid;
  grid-template-columns: 18px 10px 18px minmax(0, 1fr);
  align-items: center;
  gap: 7px;
  width: 100%;
  min-height: 24px;
  padding: 0 10px 0 12px;
  border: 0;
  text-align: left;
  color: #25312b;
  background: transparent;
}

.file-row:hover,
.file-row.active {
  background: #dfeafd;
}

.file-row.selected:not(.active) {
  background: #f0f5ff;
}

.status-dot {
  width: 8px;
  height: 8px;
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

.change-file-icon {
  display: inline-grid;
  place-items: center;
  width: 18px;
  height: 18px;
  color: #68766f;
}

.change-file-icon.labeled span {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 9px;
  font-weight: 800;
  line-height: 1;
}

.change-file-icon.ext-js,
.change-file-icon.ext-jsx {
  color: #a06f16;
}

.change-file-icon.ext-ts,
.change-file-icon.ext-tsx {
  color: #2f70b6;
}

.change-file-icon.ext-json,
.change-file-icon.ext-wxml,
.change-file-icon.ext-html,
.change-file-icon.ext-vue {
  color: #5f6d65;
}

.change-file-icon.ext-css,
.change-file-icon.ext-wxss {
  color: #426db6;
}

.file-main {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.file-main strong,
.file-main small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-main strong {
  flex: 0 1 auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
}

.file-main small {
  flex: 1 1 auto;
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

.diff-title-block {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.diff-title-block small {
  color: #6b7971;
  font-size: 11px;
}

.diff-header-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
  min-width: 0;
}

.diff-nav-group {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.diff-nav-button {
  width: 30px;
  height: 30px;
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

.side-by-side-scroller {
  overflow-x: hidden;
  background: #ffffff;
}

.side-by-side-diff {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  width: 100%;
  min-width: 0;
  color: #25312b;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  tab-size: 2;
}

.side-by-side-file-header {
  position: sticky;
  top: 0;
  z-index: 3;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  border-bottom: 1px solid #dce2dd;
  background: #f3f5f2;
}

.side-by-side-title {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  min-width: 0;
  min-height: 34px;
  padding: 0 12px;
  border-right: 1px solid #dce2dd;
}

.side-by-side-title strong {
  color: #405047;
  font-size: 11px;
  font-weight: 800;
}

.side-by-side-title span {
  overflow: hidden;
  color: #718078;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.side-by-side-editors {
  position: relative;
  isolation: isolate;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  min-height: 0;
}

.side-by-side-editors::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  z-index: 3;
  width: 5px;
  pointer-events: none;
  background: linear-gradient(
    90deg,
    #ffffff 0,
    #ffffff 2px,
    #dce2dd 2px,
    #dce2dd 3px,
    #ffffff 3px,
    #ffffff 100%
  );
  transform: translateX(-3px);
}

.side-by-side-column {
  position: relative;
  z-index: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  background: #ffffff;
}

.side-by-side-column.old {
  border-right: 1px solid #dce2dd;
}

.side-by-side-column-lines {
  display: grid;
  width: max-content;
  min-width: 100%;
  padding: 8px 0 14px;
}

.side-by-side-line {
  min-height: 18px;
}

.diff-cell {
  display: grid;
  grid-template-columns: 58px max-content;
  width: max-content;
  min-width: 100%;
  min-height: 18px;
  white-space: pre;
  background: #ffffff;
}

.diff-cell.empty {
  color: transparent;
  background: #f7f8f6;
}

.side-by-side-line.add .diff-cell.add,
.side-by-side-line.modify .diff-cell.add {
  background: #e6f4e9;
}

.side-by-side-line.delete .diff-cell.delete,
.side-by-side-line.modify .diff-cell.delete {
  background: #fdece7;
}

.side-by-side-line.meta .diff-cell.meta {
  color: #7a6758;
  background: #fff7e0;
}

.side-by-side-line.active .diff-cell.add,
.side-by-side-line.active .diff-cell.delete {
  box-shadow: inset 3px 0 0 #4c82d9;
}

.project-tabs {
  display: flex;
  flex: 0 0 auto;
  min-height: 38px;
  overflow-x: auto;
  border-bottom: 1px solid #dce2dd;
  background: #f3f5f2;
}

.project-tab {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 28px;
  align-items: stretch;
  min-width: 136px;
  max-width: 240px;
  border-right: 1px solid #dce2dd;
  color: #445149;
  background: #edf1ec;
}

.project-tab::before {
  content: "";
  position: absolute;
  left: 10px;
  bottom: 0;
  width: 42px;
  height: 2px;
  background: transparent;
}

.project-tab.active {
  color: #202b26;
  background: #fbfcfa;
}

.project-tab.active::before {
  background: #4c82d9;
}

.project-tab-select,
.project-tab-close {
  min-width: 0;
  border: 0;
  color: inherit;
  background: transparent;
}

.project-tab-select {
  display: grid;
  grid-template-columns: 16px minmax(0, 1fr);
  align-items: center;
  gap: 6px;
  padding: 0 8px 0 10px;
  text-align: left;
}

.project-tab-select.dirty {
  grid-template-columns: 16px 8px minmax(0, 1fr);
}

.project-tab-dirty {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: #3f7fdb;
  box-shadow: 0 0 0 2px rgba(63, 127, 219, 0.12);
}

.project-tab-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.project-tab-close {
  display: grid;
  place-items: center;
  color: #7a877f;
}

.project-tab:hover,
.project-tab-close:hover {
  background: #e4e9e3;
}

.project-editor {
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

.diff-lines,
.project-lines {
  margin: 0;
  padding: 12px 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  tab-size: 2;
}

.project-edit-pane {
  position: relative;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.project-editor-render {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  color: #25312b;
  background: #ffffff;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  tab-size: 2;
  pointer-events: none;
}

.project-editor-render-content {
  display: block;
  min-width: max-content;
  padding: 12px 0;
  transform: translateY(var(--project-editor-scroll-top-offset));
}

.project-render-line {
  display: grid;
  grid-template-columns: 46px max-content;
  min-height: 18px;
  white-space: pre;
}

.project-render-line .line-number {
  position: relative;
  z-index: 1;
  background: #ffffff;
}

.project-render-code {
  display: inline-block;
  min-width: 100%;
  padding: 0 18px 0 0;
  white-space: pre;
  transform: translateX(var(--project-editor-scroll-left-offset));
}

.project-editor-textarea {
  position: absolute;
  inset: 0;
  z-index: 1;
  width: 100%;
  height: 100%;
  min-height: 0;
  padding: 12px 14px 12px 46px;
  border: 0;
  border-radius: 0;
  outline: none;
  resize: none;
  color: transparent;
  caret-color: #25312b;
  background: transparent;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 18px;
  tab-size: 2;
  white-space: pre;
  overflow: auto;
  -webkit-text-fill-color: transparent;
}

.project-editor-change-layer {
  position: absolute;
  inset: 0;
  z-index: 3;
  pointer-events: none;
}

.project-change-marker {
  position: absolute;
  left: 13px;
  display: grid;
  place-items: center;
  width: 20px;
  min-height: 10px;
  padding: 0;
  border: 0;
  border-left: 3px solid #4c82d9;
  border-radius: 3px;
  color: #2f5f9f;
  background: rgba(76, 130, 217, 0.16);
  pointer-events: auto;
}

.project-change-marker:hover,
.project-change-marker.expanded {
  background: rgba(76, 130, 217, 0.28);
}

.project-change-marker.added {
  border-left-color: #3d8f55;
  color: #2f7b43;
  background: rgba(61, 143, 85, 0.16);
}

.project-change-marker.added:hover,
.project-change-marker.added.expanded {
  background: rgba(61, 143, 85, 0.28);
}

.project-change-marker.deleted {
  border-left-color: #df3f36;
  color: #b13029;
  background: rgba(223, 63, 54, 0.16);
}

.project-change-marker.deleted:hover,
.project-change-marker.deleted.expanded {
  background: rgba(223, 63, 54, 0.28);
}

.diff-line,
.project-line,
.project-original-line {
  display: grid;
  grid-template-columns: 58px max-content;
  min-height: 18px;
  white-space: pre;
}

.project-line {
  position: relative;
}

.project-line.changed {
  cursor: pointer;
  background: #fff6f3;
}

.project-line.changed:hover,
.project-line.changed.expanded {
  background: #ffece8;
}

.project-line.changed .line-number {
  position: relative;
  border-left: 2px solid #df3f36;
  color: #b13029;
  background: #fff0ee;
}

.project-line.change-start .line-number {
  box-shadow: inset 0 1px 0 #df3f36;
}

.project-line.change-end .line-number {
  box-shadow: inset 0 -1px 0 #df3f36;
}

.project-line.change-start.change-end .line-number {
  box-shadow: inset 0 1px 0 #df3f36, inset 0 -1px 0 #df3f36;
}

.project-line.changed .line-number::after {
  content: "";
  position: absolute;
  top: 3px;
  right: -2px;
  bottom: 3px;
  width: 3px;
  border-radius: 3px;
  background: #4c82d9;
}

.line-number {
  padding-right: 12px;
  color: #96a19b;
  text-align: right;
  user-select: none;
}

.side-by-side-column .line-number {
  position: sticky;
  left: 0;
  z-index: 1;
  background: inherit;
}

.line-content {
  padding: 0 18px 0 10px;
}

.side-by-side-diff .line-content {
  display: block;
  min-width: max-content;
  overflow: visible;
}

.syntax-comment {
  color: #7a8790;
  font-style: italic;
}

.syntax-string {
  color: #2f8a43;
}

.syntax-keyword {
  color: #8c4aa6;
  font-weight: 600;
}

.syntax-number {
  color: #986801;
}

.syntax-function {
  color: #2f6fc7;
}

.syntax-property {
  color: #6f42c1;
}

.syntax-operator {
  color: #6b7280;
}

.project-original-panel {
  display: grid;
  grid-template-columns: 58px minmax(460px, 1fr);
  border-top: 1px solid #f0b5ae;
  border-bottom: 1px solid #f0b5ae;
  background: #fffaf8;
}

.project-original-popover {
  position: absolute;
  right: 16px;
  left: 38px;
  z-index: 4;
  grid-template-columns: 58px minmax(0, 1fr);
  max-height: min(280px, calc(100% - 16px));
  overflow: auto;
  border: 1px solid #f0b5ae;
  box-shadow: 0 14px 34px rgba(35, 28, 24, 0.18);
  pointer-events: auto;
}

.project-original-gutter {
  padding: 7px 12px 0 0;
  border-left: 2px solid #df3f36;
  color: #a8a09a;
  text-align: right;
  user-select: none;
  background: #fff0ee;
}

.project-original-card {
  max-width: 980px;
  border-right: 1px solid #f0c7c1;
  background: #ffffff;
}

.project-original-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 34px;
  padding: 4px 10px;
  border-bottom: 1px solid #eadbd6;
  color: #7a4a43;
  background: #fff5f2;
}

.project-original-toolbar span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-original-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 0 0 auto;
}

.project-original-code {
  padding: 6px 0;
}

.project-original-empty {
  padding: 10px 12px;
  color: #8a7973;
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

html[data-theme="dark"] .notice-toast {
  border-color: #3d8b57;
  color: #b7f0c6;
  background: #163822;
  box-shadow: 0 18px 46px rgba(0, 0, 0, 0.42);
}

html[data-theme="dark"] .notice-toast button:hover {
  background: rgba(183, 240, 198, 0.12);
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

html[data-theme="dark"] .context-menu {
  border-color: #4e5258;
  background: #2b2d30;
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.42);
}

html[data-theme="dark"] .context-menu button {
  color: #dfe1e5;
}

html[data-theme="dark"] .context-menu button:hover:not(:disabled) {
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .context-menu button.danger-menu-item:not(:hover):not(:disabled) span {
  color: #ff7b72;
}

html[data-theme="dark"] .context-menu button:disabled {
  color: #696e77;
}

html[data-theme="dark"] .context-menu button small {
  color: #8f949b;
}

html[data-theme="dark"] .context-menu-separator {
  background: #3c3f41;
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
html[data-theme="dark"] .icon-button.primary,
html[data-theme="dark"] .commit-button,
html[data-theme="dark"] .segmented button.active {
  border-color: #5b8fd7;
  color: #ffffff;
  background: #3f6ea5;
}

html[data-theme="dark"] .tool-button.primary:hover:not(:disabled),
html[data-theme="dark"] .icon-button.primary:hover:not(:disabled),
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

html[data-theme="dark"] .empty-panel p {
  color: #8f949b;
}

html[data-theme="dark"] .project-pane,
html[data-theme="dark"] .repo-pane,
html[data-theme="dark"] .workbench-rail,
html[data-theme="dark"] .history-pane,
html[data-theme="dark"] .project-tree-pane,
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
html[data-theme="dark"] .project-tree-pane,
html[data-theme="dark"] .advanced-sidebar,
html[data-theme="dark"] .diff-header,
html[data-theme="dark"] .changelist-panel,
html[data-theme="dark"] .log-filter-panel,
html[data-theme="dark"] .hunk-strip,
html[data-theme="dark"] .commit-files {
  background: #2b2d30;
}

html[data-theme="dark"] .diff-pane,
html[data-theme="dark"] .diff-scroller,
html[data-theme="dark"] .project-editor {
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
html[data-theme="dark"] .commit-files {
  border-color: #3c3f41;
}

html[data-theme="dark"] .shelve-box {
  border-top-color: #3c3f41;
}

html[data-theme="dark"] .project-file-browser {
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

html[data-theme="dark"] .project-file-row {
  color: #c9d1d9;
}

html[data-theme="dark"] .project-file-row.directory {
  color: #dfe1e5;
}

html[data-theme="dark"] .project-file-row.root {
  color: #f0f2f5;
  background: #33445f;
}

html[data-theme="dark"] .project-file-name.root small {
  color: #a9b7c6;
}

html[data-theme="dark"] .project-file-row:hover:not(:disabled),
html[data-theme="dark"] .project-file-row.active {
  background: #313335;
}

html[data-theme="dark"] .project-file-disclosure,
html[data-theme="dark"] .project-tab-close {
  color: #8f949b;
}

html[data-theme="dark"] .project-tabs {
  border-bottom-color: #3c3f41;
  background: #252629;
}

html[data-theme="dark"] .project-tab {
  border-right-color: #3c3f41;
  color: #a9b7c6;
  background: #2b2d30;
}

html[data-theme="dark"] .project-tab.active {
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .project-tab-dirty {
  background: #78a8ff;
  box-shadow: 0 0 0 2px rgba(120, 168, 255, 0.16);
}

html[data-theme="dark"] .project-tab:hover,
html[data-theme="dark"] .project-tab-close:hover {
  background: #313335;
}

html[data-theme="dark"] .project-file-row.status-added,
html[data-theme="dark"] .project-tab.status-added {
  color: #6fce8b;
}

html[data-theme="dark"] .project-file-row.status-modified,
html[data-theme="dark"] .project-file-row.status-typechange,
html[data-theme="dark"] .project-tab.status-modified,
html[data-theme="dark"] .project-tab.status-typechange {
  color: #e0b95f;
}

html[data-theme="dark"] .project-file-row.status-deleted,
html[data-theme="dark"] .project-tab.status-deleted {
  color: #ff8177;
}

html[data-theme="dark"] .project-file-row.status-renamed,
html[data-theme="dark"] .project-tab.status-renamed {
  color: #89b4ff;
}

html[data-theme="dark"] .project-file-row.status-conflicted,
html[data-theme="dark"] .project-tab.status-conflicted {
  color: #ff9a5f;
}

html[data-theme="dark"] .project-file-row.status-ignored,
html[data-theme="dark"] .project-tab.status-ignored {
  color: #838991;
}

html[data-theme="dark"] .project-file-empty {
  color: #8f949b;
}

html[data-theme="dark"] .project-dot {
  background: #6b7078;
}

html[data-theme="dark"] .project-row.active .project-dot {
  background: #7aa2f7;
}

html[data-theme="dark"] .project-row.uninitialized .project-dot {
  background: #d0a044;
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

html[data-theme="dark"] .log-tab-workspace,
html[data-theme="dark"] .log-diff-tab-pane,
html[data-theme="dark"] .log-workbench,
html[data-theme="dark"] .log-commit-list {
  background: #1e1f22;
}

html[data-theme="dark"] .log-workspace-tabs {
  border-bottom-color: #3c3f41;
  background: #252629;
}

html[data-theme="dark"] .log-root-tab,
html[data-theme="dark"] .log-workspace-tab {
  border-right-color: #3c3f41;
  color: #a9b7c6;
  background: #2b2d30;
}

html[data-theme="dark"] .log-root-tab.active,
html[data-theme="dark"] .log-workspace-tab.active {
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .log-root-tab:hover,
html[data-theme="dark"] .log-workspace-tab:hover,
html[data-theme="dark"] .log-workspace-tab-close:hover {
  background: #313335;
}

html[data-theme="dark"] .log-workspace-tab-select small,
html[data-theme="dark"] .log-workspace-tab-close {
  color: #8f949b;
}

html[data-theme="dark"] .log-commit-panel,
html[data-theme="dark"] .log-files-panel,
html[data-theme="dark"] .log-panel-header,
html[data-theme="dark"] .log-topbar,
html[data-theme="dark"] .log-table-head {
  border-color: #3c3f41;
}

html[data-theme="dark"] .log-topbar,
html[data-theme="dark"] .log-table-head,
html[data-theme="dark"] .log-detail-panel {
  background: #2b2d30;
}

html[data-theme="dark"] .modal-backdrop {
  background: rgba(0, 0, 0, 0.46);
}

html[data-theme="dark"] .log-file-picker-modal,
html[data-theme="dark"] .project-name-modal,
html[data-theme="dark"] .project-unsaved-modal,
html[data-theme="dark"] .log-filter-popover {
  border-color: #4e5258;
  background: #2b2d30;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.46);
}

html[data-theme="dark"] .project-name-modal h2,
html[data-theme="dark"] .project-unsaved-modal h2,
html[data-theme="dark"] .project-unsaved-modal strong {
  color: #dfe1e5;
}

html[data-theme="dark"] .project-unsaved-modal p {
  color: #a9b7c6;
}

html[data-theme="dark"] .project-unsaved-path {
  border-color: #3c3f41;
  color: #8f949b;
  background: #1e1f22;
}

html[data-theme="dark"] .project-name-error {
  color: #ff8a8a;
}

html[data-theme="dark"] .log-file-picker-tree {
  border-color: #3c3f41;
  background: #1e1f22;
}

html[data-theme="dark"] .log-search-field,
html[data-theme="dark"] .log-mini-filter,
html[data-theme="dark"] .log-filter-button,
html[data-theme="dark"] .log-filter-chip,
html[data-theme="dark"] .log-diff-preview .diff-scroller {
  border-color: #4e5258;
  color: #dfe1e5;
  background: #1e1f22;
}

html[data-theme="dark"] .log-search-field input,
html[data-theme="dark"] .log-mini-filter {
  color: #dfe1e5;
}

html[data-theme="dark"] .log-filter-button.active {
  border-color: #4c82d9;
  color: #a9c7ff;
  background: #28354b;
}

html[data-theme="dark"] .log-filter-chip.active {
  border-color: #4c82d9;
  color: #a9c7ff;
  background: #28354b;
}

html[data-theme="dark"] .log-ref-pane {
  background: #1e1f22;
}

html[data-theme="dark"] .log-ref-toolbar {
  border-right-color: #3c3f41;
  background: #1e1f22;
}

html[data-theme="dark"] .log-ref-tool-button {
  color: #a9b7c6;
}

html[data-theme="dark"] .log-ref-tool-button:hover:not(:disabled),
html[data-theme="dark"] .log-ref-tool-button.active {
  border-color: #4e5258;
  background: #313335;
}

html[data-theme="dark"] .log-ref-tool-button.active {
  color: #ffc66d;
}

html[data-theme="dark"] .log-ref-tool-button.danger:hover:not(:disabled) {
  color: #ff7b72;
}

html[data-theme="dark"] .log-ref-tool-separator {
  background: #3c3f41;
}

html[data-theme="dark"] .log-ref-search-field {
  border-color: #3c3f41;
  color: #a9b7c6;
  background: #1e1f22;
}

html[data-theme="dark"] .log-ref-search-field input {
  color: #dfe1e5;
}

html[data-theme="dark"] .log-ref-search-field input::placeholder {
  color: #787f87;
}

html[data-theme="dark"] .log-ref-row,
html[data-theme="dark"] .log-ref-head-row,
html[data-theme="dark"] .log-ref-toggle,
html[data-theme="dark"] .log-file-tree-row,
html[data-theme="dark"] .log-file-picker-row,
html[data-theme="dark"] .log-check-row,
html[data-theme="dark"] .log-commit-row {
  color: #c9d1d9;
}

html[data-theme="dark"] .log-ref-toggle:hover {
  background: #313335;
}

html[data-theme="dark"] .log-ref-row:hover,
html[data-theme="dark"] .log-ref-row.active,
html[data-theme="dark"] .log-ref-head-row:hover,
html[data-theme="dark"] .log-ref-head-row.active,
html[data-theme="dark"] .log-commit-row:hover,
html[data-theme="dark"] .log-file-tree-row:hover,
html[data-theme="dark"] .log-file-tree-row.selected,
html[data-theme="dark"] .log-file-picker-row:hover,
html[data-theme="dark"] .log-file-picker-row.selected,
html[data-theme="dark"] .log-check-row:hover,
html[data-theme="dark"] .log-check-row.selected {
  background: #313335;
}

html[data-theme="dark"] .log-ref-row.active,
html[data-theme="dark"] .log-diff-preview .diff-scroller {
  border-color: #4c82d9;
}

html[data-theme="dark"] .log-ref-row.current:not(.active) {
  background: #33363a;
}

html[data-theme="dark"] .log-ref-row:hover,
html[data-theme="dark"] .log-ref-row.active,
html[data-theme="dark"] .log-ref-head-row:hover,
html[data-theme="dark"] .log-ref-head-row.active {
  color: #f0f2f5;
  background: #3c3f41;
}

html[data-theme="dark"] .log-ref-row.context-target:not(.active) {
  background: #343d4a;
}

html[data-theme="dark"] .log-commit-row.active {
  background: #3a3f47;
}

html[data-theme="dark"] .log-subject strong,
html[data-theme="dark"] .log-info-body h2,
html[data-theme="dark"] .log-info-body p strong,
html[data-theme="dark"] .log-check-label strong,
html[data-theme="dark"] .log-filter-popover-head {
  color: #f0f2f5;
}

html[data-theme="dark"] .log-author,
html[data-theme="dark"] .log-date,
html[data-theme="dark"] .log-ref-row small,
html[data-theme="dark"] .log-ref-empty,
html[data-theme="dark"] .log-ref-toggle,
html[data-theme="dark"] .log-ref-toggle small,
html[data-theme="dark"] .log-ref-group-title,
html[data-theme="dark"] .log-ref-group-title small,
html[data-theme="dark"] .log-panel-header small,
html[data-theme="dark"] .log-file-tree-row small,
html[data-theme="dark"] .log-file-picker-footer > span,
html[data-theme="dark"] .log-check-label small,
html[data-theme="dark"] .log-check-row > small,
html[data-theme="dark"] .log-picker-empty,
html[data-theme="dark"] .log-info-body p,
html[data-theme="dark"] .log-diff-preview summary {
  color: #8f949b;
}

html[data-theme="dark"] .log-file-tree-row.directory,
html[data-theme="dark"] .log-file-picker-row.directory {
  color: #dfe1e5;
}

html[data-theme="dark"] .log-file-disclosure,
html[data-theme="dark"] .log-file-name small {
  color: #8f949b;
}

html[data-theme="dark"] .log-filter-popover-head {
  border-bottom-color: #3c3f41;
}

html[data-theme="dark"] .log-checkmark,
html[data-theme="dark"] .log-file-picker-check {
  border-color: #4e5258;
}

html[data-theme="dark"] .log-file-picker-row.root {
  color: #dfe1e5;
  background: #28354b;
}

html[data-theme="dark"] .log-file-tree-row.status-added .log-file-name strong {
  color: #6dcc8e;
}

html[data-theme="dark"] .log-file-tree-row.status-modified .log-file-name strong,
html[data-theme="dark"] .log-file-tree-row.status-typechange .log-file-name strong {
  color: #d7b25f;
}

html[data-theme="dark"] .log-file-tree-row.status-deleted .log-file-name strong {
  color: #ef8379;
}

html[data-theme="dark"] .log-file-tree-row.status-renamed .log-file-name strong {
  color: #8fb7ff;
}

html[data-theme="dark"] .log-file-tree-row.status-copied .log-file-name strong {
  color: #65c7bd;
}

html[data-theme="dark"] .log-file-tree-row.status-conflicted .log-file-name strong {
  color: #ee9c5a;
}

html[data-theme="dark"] .log-graph-node {
  --graph-node-ring: #1e1f22;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.16);
}

html[data-theme="dark"] .commit-file-row strong {
  color: #dfe1e5;
}

html[data-theme="dark"] .commit-file-row {
  border-color: #3c3f41;
  background: #2b2d30;
}

html[data-theme="dark"] .file-list {
  background: #1e1f22;
}

html[data-theme="dark"] .file-list-empty {
  color: #8f949b;
}

html[data-theme="dark"] .change-file-group-header {
  color: #dfe1e5;
}

html[data-theme="dark"] .change-file-group-header:hover {
  background: #313335;
}

html[data-theme="dark"] .change-file-group-header small {
  color: #8f949b;
}

html[data-theme="dark"] .change-file-group-empty {
  color: #8f949b;
}

html[data-theme="dark"] .file-row {
  color: #c9d1d9;
}

html[data-theme="dark"] .file-row:hover {
  background: #2c313a;
}

html[data-theme="dark"] .file-row.selected:not(.active) {
  background: #263247;
}

html[data-theme="dark"] .file-row.active {
  color: #ffffff;
  background: #2f477a;
}

html[data-theme="dark"] .change-group-checkbox,
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

html[data-theme="dark"] .change-file-icon {
  color: #a9b7c6;
}

html[data-theme="dark"] .change-file-icon.ext-js,
html[data-theme="dark"] .change-file-icon.ext-jsx {
  color: #ffc66d;
}

html[data-theme="dark"] .change-file-icon.ext-ts,
html[data-theme="dark"] .change-file-icon.ext-tsx,
html[data-theme="dark"] .change-file-icon.ext-css,
html[data-theme="dark"] .change-file-icon.ext-wxss {
  color: #78a8ff;
}

html[data-theme="dark"] .change-file-icon.ext-json,
html[data-theme="dark"] .change-file-icon.ext-wxml,
html[data-theme="dark"] .change-file-icon.ext-html,
html[data-theme="dark"] .change-file-icon.ext-vue {
  color: #a9b7c6;
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
html[data-theme="dark"] .side-by-side-diff,
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

html[data-theme="dark"] .diff-title-block small {
  color: #8f949b;
}

html[data-theme="dark"] .toggle-row input {
  accent-color: #4c82d9;
}

html[data-theme="dark"] .side-by-side-scroller,
html[data-theme="dark"] .side-by-side-diff {
  color: #c9d1d9;
  background: #1e1f22;
}

html[data-theme="dark"] .side-by-side-file-header {
  border-bottom-color: #3c3f41;
  background: #2b2d30;
}

html[data-theme="dark"] .side-by-side-title {
  border-right-color: #3c3f41;
}

html[data-theme="dark"] .side-by-side-title strong {
  color: #dfe1e5;
}

html[data-theme="dark"] .side-by-side-title span {
  color: #8f949b;
}

html[data-theme="dark"] .side-by-side-column {
  background: #1e1f22;
}

html[data-theme="dark"] .side-by-side-column.old {
  border-right-color: #323438;
}

html[data-theme="dark"] .side-by-side-editors::after {
  background: linear-gradient(
    90deg,
    #1e1f22 0,
    #1e1f22 2px,
    #323438 2px,
    #323438 3px,
    #1e1f22 3px,
    #1e1f22 100%
  );
}

html[data-theme="dark"] .diff-cell {
  grid-template-columns: 66px max-content;
  background: #1e1f22;
}

html[data-theme="dark"] .diff-cell.empty {
  background: #232427;
}

html[data-theme="dark"] .side-by-side-line.add .diff-cell.add,
html[data-theme="dark"] .side-by-side-line.modify .diff-cell.add {
  color: #b6d8a8;
  background: #243729;
}

html[data-theme="dark"] .side-by-side-line.delete .diff-cell.delete,
html[data-theme="dark"] .side-by-side-line.modify .diff-cell.delete {
  color: #e6b0ad;
  background: #3f2a2a;
}

html[data-theme="dark"] .side-by-side-line.meta .diff-cell.meta {
  color: #d7ba7d;
  background: #332d20;
}

html[data-theme="dark"] .side-by-side-line.active .diff-cell.add,
html[data-theme="dark"] .side-by-side-line.active .diff-cell.delete {
  box-shadow: inset 3px 0 0 #6ea2f2;
}

html[data-theme="dark"] .diff-lines {
  padding: 0;
  color: #c9d1d9;
  background: #1e1f22;
}

html[data-theme="dark"] .project-lines {
  padding: 0;
  color: #c9d1d9;
  background: #1e1f22;
}

html[data-theme="dark"] .project-editor-render {
  color: #c9d1d9;
  background: #1e1f22;
}

html[data-theme="dark"] .project-render-line .line-number {
  background: #252629;
}

html[data-theme="dark"] .project-editor-textarea {
  color: transparent;
  caret-color: #dfe1e5;
  background: transparent;
  -webkit-text-fill-color: transparent;
}

html[data-theme="dark"] .project-change-marker {
  border-left-color: #6ea2f2;
  color: #a9c7ff;
  background: rgba(76, 130, 217, 0.22);
}

html[data-theme="dark"] .project-change-marker:hover,
html[data-theme="dark"] .project-change-marker.expanded {
  background: rgba(76, 130, 217, 0.36);
}

html[data-theme="dark"] .project-change-marker.added {
  border-left-color: #6a8759;
  color: #9bc27c;
  background: rgba(106, 135, 89, 0.22);
}

html[data-theme="dark"] .project-change-marker.added:hover,
html[data-theme="dark"] .project-change-marker.added.expanded {
  background: rgba(106, 135, 89, 0.36);
}

html[data-theme="dark"] .project-change-marker.deleted {
  border-left-color: #e05b55;
  color: #ee8a84;
  background: rgba(224, 91, 85, 0.22);
}

html[data-theme="dark"] .project-change-marker.deleted:hover,
html[data-theme="dark"] .project-change-marker.deleted.expanded {
  background: rgba(224, 91, 85, 0.36);
}

html[data-theme="dark"] .diff-line,
html[data-theme="dark"] .project-line,
html[data-theme="dark"] .project-original-line {
  grid-template-columns: 66px max-content;
}

html[data-theme="dark"] .diff-line.context,
html[data-theme="dark"] .project-line {
  background: #1e1f22;
}

html[data-theme="dark"] .project-line.changed {
  background: #2b2424;
}

html[data-theme="dark"] .project-line.changed:hover,
html[data-theme="dark"] .project-line.changed.expanded {
  background: #352929;
}

html[data-theme="dark"] .project-line.changed .line-number {
  border-left-color: #e05b55;
  color: #d46a64;
  background: #332525;
}

html[data-theme="dark"] .project-line.change-start .line-number {
  box-shadow: inset 0 1px 0 #e05b55;
}

html[data-theme="dark"] .project-line.change-end .line-number {
  box-shadow: inset 0 -1px 0 #e05b55;
}

html[data-theme="dark"] .project-line.change-start.change-end .line-number {
  box-shadow: inset 0 1px 0 #e05b55, inset 0 -1px 0 #e05b55;
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

html[data-theme="dark"] .side-by-side-diff .line-content {
  min-width: max-content;
}

html[data-theme="dark"] .syntax-comment {
  color: #7f8590;
}

html[data-theme="dark"] .syntax-string {
  color: #6a8759;
}

html[data-theme="dark"] .syntax-keyword {
  color: #cc7832;
}

html[data-theme="dark"] .syntax-number {
  color: #6897bb;
}

html[data-theme="dark"] .syntax-function {
  color: #ffc66d;
}

html[data-theme="dark"] .syntax-property {
  color: #9876aa;
}

html[data-theme="dark"] .syntax-operator {
  color: #a9b7c6;
}

html[data-theme="dark"] .project-original-panel {
  grid-template-columns: 66px minmax(460px, 1fr);
  border-color: #5b3634;
  background: #251f1f;
}

html[data-theme="dark"] .project-original-popover {
  grid-template-columns: 66px minmax(0, 1fr);
}

html[data-theme="dark"] .project-original-gutter {
  border-left-color: #e05b55;
  color: #696d75;
  background: #332525;
}

html[data-theme="dark"] .project-original-card {
  border-right-color: #3c3030;
  background: #1e1f22;
}

html[data-theme="dark"] .project-original-toolbar {
  border-bottom-color: #3c3030;
  color: #d3b6b2;
  background: #2b2424;
}

html[data-theme="dark"] .project-original-empty {
  color: #8d9299;
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
