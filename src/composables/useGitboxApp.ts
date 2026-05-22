import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { LoaderCircle } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  commitFileDiff,
  copyProjectEntry,
  createProjectDirectory,
  createProjectFile,
  deleteProjectEntry,
  filterProjectDirectories,
  moveProjectEntry,
  renameProjectEntry,
} from "../lib/gitboxCommands";
import { useAdvancedStore } from "../stores/advanced";
import { useBranchesStore } from "../stores/branches";
import { useChangelistsStore } from "../stores/changelists";
import { useChangesStore } from "../stores/changes";
import { useCommitStore } from "../stores/commit";
import { useDiffStore } from "../stores/diff";
import { useHistoryStore } from "../stores/history";
import { useOperationsStore } from "../stores/operations";
import { PROJECT_ROOT_PATH, useProjectStore } from "../stores/project";
import { useRemoteStore } from "../stores/remote";
import { useRepositoriesStore } from "../stores/repositories";
import { useSettingsStore } from "../stores/settings";
import type { LayoutPanelKey, ThemeMode } from "../stores/settings";
import type {
  BranchInfo,
  ChangeSide,
  ChangedFile,
  CommitFileDiffMode,
  CommitFileChange,
  CommitSummary,
  ConflictBlock,
  ConflictDetails,
  DiffHunk,
  DiffResponse,
  ProjectFileEntry,
  PullPreflight,
  ShelfInfo,
  TagInfo,
} from "../types/gitbox";

export function useGitboxApp() {
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
  const pendingUiActions = ref<Record<string, number>>({});
  const pendingCommitAction = ref<"commit" | "push" | null>(null);
  const activeRemoteAction = ref<"fetch" | "pull" | "push" | "fetchAll" | null>(null);
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
  const addRemoteDialog = ref<AddRemoteDialog | null>(null);
  const projectCloseDialog = ref<ProjectCloseDialog | null>(null);
  const projectDropActive = ref(false);
  const expandedSubmitConfirmDirectories = ref<Record<string, boolean>>({});
  const mergeCurrentScroller = ref<HTMLElement | null>(null);
  const mergeCurrentGutter = ref<HTMLElement | null>(null);
  const mergeResultGutter = ref<HTMLElement | null>(null);
  const mergeResultTextarea = ref<HTMLTextAreaElement | null>(null);
  const mergeIncomingScroller = ref<HTMLElement | null>(null);
  const mergeIncomingGutter = ref<HTMLElement | null>(null);
  const mergeResultScrollTop = ref(0);
  const mergeResultScrollLeft = ref(0);
  const commitMessageTextarea = ref<HTMLTextAreaElement | null>(null);
  const commitMessageHistoryIndex = ref(-1);
  const commitMessageHistoryDraft = ref("");
  const projectEditorTextarea = ref<HTMLTextAreaElement | null>(null);
  const changeFileListScroller = ref<HTMLElement | null>(null);
  const changeDiffScroller = ref<HTMLElement | null>(null);
  const logDiffScroller = ref<HTMLElement | null>(null);
  const activeChangeDiffHunkIndex = ref<number | null>(null);
  const activeLogDiffHunkIndex = ref(0);
  const activeMergeConflictOrdinal = ref(0);
  const changeFileListScrollTop = ref(0);
  const changeFileListViewportHeight = ref(720);
  const syncingSideBySideScroll = new WeakSet<HTMLElement>();
  const syncingMergeEditorScroll = new WeakSet<HTMLElement>();
  const projectEditorScrollTop = ref(0);
  const projectEditorScrollLeft = ref(0);
  const projectEditorViewportHeight = ref(0);
  const expandedProjectHunkIndex = ref<string | null>(null);
  const expandedChangeFileGroups = ref<Record<string, boolean>>({
    staged: true,
    tracked: true,
    untracked: true,
  });
  type WorkbenchMode = "changes" | "log" | "project";
  type ChangeFileGroup = {
    key: string;
    label: string;
    side: ChangeSide;
    files: ChangedFile[];
    conflictFiles: ChangedFile[];
    changelistId?: string;
  };
  type ChangeFileCollection = ChangeFileGroup | ChangedFile[];
  type VirtualChangeFileRow =
    | {
        kind: "group";
        key: string;
        group: ChangeFileGroup;
      }
    | {
        kind: "conflict-group";
        key: string;
        group: ChangeFileGroup;
      }
    | {
        kind: "empty";
        key: string;
        group: ChangeFileGroup;
      }
    | {
        kind: "file";
        key: string;
        group: ChangeFileGroup;
        file: ChangedFile;
        conflict: boolean;
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
    tokens: ProjectOriginalLineToken[];
    tone: "deleted" | "modified";
  };
  type ProjectOriginalLineToken = ProjectCodeToken & {
    diff?: boolean;
    insertMarker?: boolean;
  };
  type ProjectEditorHunkView = {
    id: string;
    hunkIndex: number;
    blockIndex: number;
    header: string;
    tone: "added" | "deleted" | "modified";
    lineStart: number;
    lineCount: number;
    changedNewStart: number;
    changedNewEnd: number;
    changedOldStart: number | null;
    changedOldEnd: number | null;
    oldStart: number;
    oldLines: number;
    newStart: number;
    newLines: number;
    patch: string;
    addedLines: number;
    deletedLines: number;
    originalLines: ProjectOriginalLine[];
  };
  type ProjectHunkEntry = {
    index: number;
    prefix: " " | "+" | "-";
    content: string;
    oldLineNumber: number | null;
    newLineNumber: number | null;
    oldAnchorLineNumber: number;
    newAnchorLineNumber: number;
  };
  type ProjectHunkChangeBlock = {
    blockIndex: number;
    startEntryIndex: number;
    endEntryIndex: number;
    entries: ProjectHunkEntry[];
    patch: string;
    changedOldStart: number | null;
    changedOldEnd: number | null;
    changedNewStart: number | null;
    changedNewEnd: number | null;
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
    placeholder: string;
    error: string;
    validate: (value: string) => string;
    resolve: (value: string | null) => void;
  };
  type AddRemoteDialog = {
    name: string;
    url: string;
    fetchAfterSave: boolean;
    loading: boolean;
    error: string;
  };
  type ProjectCloseDialog = {
    path: string;
    name: string;
    saving: boolean;
    error: string;
  };
  type MergeCodeLine = {
    id: string;
    number: number;
    text: string;
    tokens: ProjectCodeToken[];
    conflict: boolean;
    conflictIndex: number | null;
    conflictSide: MergeConflictSide | null;
    conflictStart: boolean;
    conflictEnd: boolean;
    autoMerge: boolean;
    autoMergeStart: boolean;
    autoMergeEnd: boolean;
  };
  type MergeConflictSide = "ours" | "base" | "theirs";
  type MergeDisplaySide = "ours" | "theirs";
  type MergeConflictSelection = MergeConflictSide | "combined";
  type MergeConflictSnippet = {
    index: number;
    content: string;
    side: MergeConflictSide;
  };
  type MergeAutoMergeSnippet = {
    lines: string[];
  };
  type MergeConflictLineRange = {
    startLine: number;
    endLine: number;
  };
  type MergeLineDiffRange = {
    leftStartLine: number;
    leftEndLine: number;
    rightStartLine: number;
    rightEndLine: number;
  };
  type MergeConflictConnection = MergeConflictLineRange & {
    key: string;
    source: "current" | "incoming";
    side: MergeConflictSide;
    resultStartLine: number;
    resultEndLine: number;
  };
  type NoticeToast = {
    id: number;
    message: string;
  };
  type ErrorDialog = {
    id: number;
    message: string;
  };
  type PullConfirmDialog = {
    preview: PullPreflight;
    loading: boolean;
  };
  type SubmitConfirmMode = "commit" | "commit-push" | "push";
  type SubmitConfirmDialog = {
    mode: SubmitConfirmMode;
    paths: string[];
    message: string;
    remoteName: string;
    currentBranch: string;
    targetBranch: string;
    options: string[];
    loading: boolean;
  };

  const workbenchMode = ref<WorkbenchMode>("changes");
  const LOG_TAB_ID = "log-root";
  const activeLogTabId = ref(LOG_TAB_ID);
  const logDiffTabs = ref<LogDiffTab[]>([]);
  const noticeToast = ref<NoticeToast | null>(null);
  const errorDialog = ref<ErrorDialog | null>(null);
  const pullConfirmDialog = ref<PullConfirmDialog | null>(null);
  const submitConfirmDialog = ref<SubmitConfirmDialog | null>(null);
  const activeResizePanel = ref<LayoutPanelKey | null>(null);
  const systemPrefersDark = ref(
    typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches,
  );
  let stopSystemThemeWatch: (() => void) | null = null;
  let projectEditorResizeObserver: ResizeObserver | null = null;
  let changeFileListResizeObserver: ResizeObserver | null = null;
  let autoFetchTimer: number | null = null;
  let noticeToastTimer: number | null = null;
  let noticeToastId = 0;
  let errorDialogId = 0;
  let stopProjectDragDrop: (() => void) | null = null;
  let projectDragDropDisposed = false;
  const PROJECT_EDITOR_LINE_HEIGHT = 18;
  const MIN_OPERATION_BUSY_MS = 520;
  const PROJECT_EDITOR_PADDING_TOP = 12;
  const PROJECT_EDITOR_OVERSCAN_LINES = 32;
  const PROJECT_EDITOR_DEFAULT_VIEWPORT_HEIGHT = 720;
  const PROJECT_HUNK_PATCH_CONTEXT_LINES = 3;
  const PROJECT_TOKEN_CACHE_LIMIT = 6000;
  const AUTO_LOAD_CHANGE_DIFF_LIMIT = 2000;
  const CHANGE_FILE_ROW_HEIGHT = 26;
  const CHANGE_FILE_OVERSCAN_ROWS = 18;
  const CHANGE_FILE_DEFAULT_VIEWPORT_HEIGHT = 720;
  const MERGE_EDITOR_LINE_HEIGHT = 18;
  const MERGE_EDITOR_PADDING_TOP = 12;
  const MERGE_CONNECTION_WIDTH = 46;
  const projectKeywordCache = new Map<string, Set<string>>();
  const projectLineTokenCache = new Map<string, ProjectCodeToken[]>();
  let skipNextChangeDiffLoad = false;

  const activeFiles = computed(() => {
    return filesForChangeSide(settings.selectedSide);
  });
  const selectedChangePathSet = computed(() => new Set(changes.selectedPaths));
  const changeFileGroups = computed<ChangeFileGroup[]>(() => {
    if (settings.selectedSide === "staged") {
      const stagedGroups: ChangeFileGroup[] = [
        {
          key: "staged",
          label: "暂存的变更",
          side: "staged",
          files: activeFiles.value,
          conflictFiles: [],
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
          conflictFiles: [],
          changelistId: list.id,
        },
      ]),
    );
    const untrackedFiles: ChangedFile[] = [];
    const workspaceTreeFiles = [...activeFiles.value];
    const workspaceTreePaths = new Set(workspaceTreeFiles.map((file) => file.path));
    for (const file of changes.files) {
      if (file.conflicted && !workspaceTreePaths.has(file.path)) {
        workspaceTreeFiles.push(file);
        workspaceTreePaths.add(file.path);
      }
    }

    for (const file of workspaceTreeFiles) {
      const list = changelists.listForPath(file.path);
      if (file.untracked && list.id === "default") {
        untrackedFiles.push(file);
        continue;
      }

      const group = listGroups.get(`changelist-${list.id}`);
      if (file.conflicted) {
        group?.conflictFiles.push(file);
      } else {
        group?.files.push(file);
      }
    }

    const unstagedGroups: ChangeFileGroup[] = [
      ...listGroups.values(),
      {
        key: "untracked",
        label: "未纳入版本控制的文件",
        side: "unstaged",
        files: untrackedFiles,
        conflictFiles: [],
        changelistId: "default",
      },
    ];

    return unstagedGroups.filter(
      (group) => group.key !== "untracked" || group.files.length + group.conflictFiles.length > 0,
    );
  });
  const flatChangeFileRows = computed<VirtualChangeFileRow[]>(() => {
    const rows: VirtualChangeFileRow[] = [];

    for (const group of changeFileGroups.value) {
      rows.push({
        kind: "group",
        key: `group:${group.key}`,
        group,
      });

      if (!isChangeFileGroupExpanded(group.key)) continue;

      if (changeFileGroupCount(group) === 0) {
        rows.push({
          kind: "empty",
          key: `empty:${group.key}`,
          group,
        });
      }

      if (group.conflictFiles.length > 0) {
        const conflictKey = changeConflictGroupKey(group);
        rows.push({
          kind: "conflict-group",
          key: `conflict-group:${conflictKey}`,
          group,
        });

        if (isChangeFileGroupExpanded(conflictKey)) {
          for (const file of group.conflictFiles) {
            rows.push({
              kind: "file",
              key: `${group.side}:conflict:${file.path}`,
              group,
              file,
              conflict: true,
            });
          }
        }
      }

      for (const file of group.files) {
        rows.push({
          kind: "file",
          key: `${group.side}:file:${file.path}`,
          group,
          file,
          conflict: false,
        });
      }
    }

    return rows;
  });
  const firstVisibleChangeFileRowIndex = computed(() => {
    const viewportHeight = changeFileListViewportHeight.value || CHANGE_FILE_DEFAULT_VIEWPORT_HEIGHT;
    const visibleCount = Math.ceil(viewportHeight / CHANGE_FILE_ROW_HEIGHT) + CHANGE_FILE_OVERSCAN_ROWS * 2;
    const maxStart = Math.max(0, flatChangeFileRows.value.length - visibleCount);
    const requestedStart = Math.floor(changeFileListScrollTop.value / CHANGE_FILE_ROW_HEIGHT) - CHANGE_FILE_OVERSCAN_ROWS;
    return Math.min(Math.max(0, requestedStart), maxStart);
  });
  const visibleChangeFileRows = computed(() => {
    const viewportHeight = changeFileListViewportHeight.value || CHANGE_FILE_DEFAULT_VIEWPORT_HEIGHT;
    const visibleCount = Math.ceil(viewportHeight / CHANGE_FILE_ROW_HEIGHT) + CHANGE_FILE_OVERSCAN_ROWS * 2;
    const start = firstVisibleChangeFileRowIndex.value;
    return flatChangeFileRows.value.slice(start, start + visibleCount);
  });
  const changeFileVirtualSpacerStyle = computed(() => ({
    height: `${flatChangeFileRows.value.length * CHANGE_FILE_ROW_HEIGHT}px`,
  }));
  const visibleChangeFileListStyle = computed(() => ({
    transform: `translateY(${firstVisibleChangeFileRowIndex.value * CHANGE_FILE_ROW_HEIGHT}px)`,
  }));
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
      (workbenchMode.value === "project" || logFilePickerOpen.value ? project.error : "") ||
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
  const workspaceRefreshBusy = computed(
    () => changes.loading || branches.loading || history.loading || operations.loading || diff.loading,
  );
  const selectedDiffFileTitle = computed(() => changes.selectedFile ?? "未选择文件");
  const activeChangeDiffLanguage = computed(() => projectLanguageForPath(changes.selectedFile));
  const activeChangeSideBySideDiffRows = computed(() =>
    buildSideBySideDiffRows(diff.current, activeChangeDiffLanguage.value),
  );
  const activeChangeDiffHasContent = computed(() => hasDisplayableDiffContent(diff.current));
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
  const selectedCommitPaths = computed(() => changes.selectedCommitPaths);
  const canCommit = computed(() => Boolean(commit.message.trim() && selectedCommitPaths.value.length > 0));
  const commitBusy = computed(
    () => commit.loading || pendingCommitAction.value !== null,
  );
  const commitButtonLabel = computed(() => (pendingCommitAction.value === "commit" ? "提交中" : "提交"));
  const commitPushButtonLabel = computed(() =>
    pendingCommitAction.value === "push" ? "提交并推送中" : "提交并推送",
  );
  const currentPushTargetBranch = computed(() => currentPushTargetBranchName());
  const unpushedCommitCount = computed(() => branch.value?.ahead ?? 0);
  const hasUnpushedCommits = computed(() => unpushedCommitCount.value > 0);
  const unpushedCommitTargetLabel = computed(() => {
    const localBranch = currentCommitBranchLabel();
    const remoteName = remote.selectedRemote || "origin";
    const targetBranch = currentPushTargetBranch.value;
    return targetBranch ? `${localBranch} -> ${remoteName}/${targetBranch}` : remoteName;
  });
  const canPushCurrentBranch = computed(() =>
    Boolean(
      hasUnpushedCommits.value &&
        repos.current &&
        remote.selectedRemote &&
        currentPushTargetBranch.value &&
        !branch.value?.detached &&
        !remote.loading &&
        !commit.loading,
    ),
  );
  const pushCurrentBranchButtonLabel = computed(() =>
    isUiActionActive(remoteActionKey("push")) ? "推送中" : "推送",
  );
  const commitMessageHistoryItems = computed(() => {
    const seen = new Set<string>();
    return advanced.commitMessages
      .map((message) => message.trim())
      .filter((message) => {
        if (!message || seen.has(message)) return false;
        seen.add(message);
        return true;
      });
  });
  const submitConfirmTitle = computed(() => {
    const mode = submitConfirmDialog.value?.mode;
    if (mode === "commit-push") return "确认提交并推送";
    if (mode === "push") return "确认推送";
    return "确认提交";
  });
  const submitConfirmActionLabel = computed(() => {
    const dialog = submitConfirmDialog.value;
    if (!dialog) return "确认";
    if (dialog.loading) {
      if (dialog.mode === "commit-push") return "提交并推送中";
      if (dialog.mode === "push") return "推送中";
      return "提交中";
    }
    if (dialog.mode === "commit-push") return "提交并推送";
    if (dialog.mode === "push") return "推送";
    return "提交";
  });
  const submitConfirmTargetLabel = computed(() => {
    const dialog = submitConfirmDialog.value;
    if (!dialog) return "";
    if (dialog.mode === "commit") return dialog.currentBranch || "当前分支";
    const remoteTarget = dialog.targetBranch || dialog.currentBranch || "当前分支";
    return `${dialog.remoteName || "origin"}/${remoteTarget}`;
  });
  const submitConfirmFileTreeRows = computed<LogFileTreeRow[]>(() => {
    const dialog = submitConfirmDialog.value;
    if (!dialog?.paths.length) return [];

    const files = dialog.paths.map((path) => {
      const changed = changes.files.find((file) => file.path === path);
      return {
        path,
        oldPath: changed?.oldPath,
        status: changed?.kind ?? "modified",
      };
    });
    return buildCommitFileTreeRows(files);
  });
  const visibleSubmitConfirmFileTreeRows = computed<LogFileTreeRow[]>(() => {
    const hiddenDirectories = new Set<string>();
    const rows: LogFileTreeRow[] = [];

    for (const row of submitConfirmFileTreeRows.value) {
      if (row.parent && hiddenDirectories.has(row.parent)) {
        if (row.directory) hiddenDirectories.add(row.path);
        continue;
      }

      rows.push(row);
      if (row.directory && !isSubmitConfirmDirectoryExpanded(row.path)) {
        hiddenDirectories.add(row.path);
      }
    }

    return rows;
  });
  const selectedCommitTitle = computed(() => {
    if (!history.details) return "未选择提交";
    return `${history.details.commit.shortOid} · ${history.details.commit.summary}`;
  });
  const activeLogDiffTab = computed(() => logDiffTabs.value.find((tab) => tab.id === activeLogTabId.value) ?? null);
  const activeLogDiffLanguage = computed(() => projectLanguageForPath(activeLogDiffTab.value?.path));
  const activeLogSideBySideDiffRows = computed(() =>
    buildSideBySideDiffRows(activeLogDiffTab.value?.diff ?? null, activeLogDiffLanguage.value),
  );
  const activeLogDiffHasContent = computed(() => hasDisplayableDiffContent(activeLogDiffTab.value?.diff));
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
  const projectEditorLineTexts = computed(() => projectEditorText.value.split("\n"));
  const projectEditorLineCount = computed(() => projectEditorLineTexts.value.length);
  const projectEditorVisibleRange = computed(() => {
    const viewportHeight =
      projectEditorViewportHeight.value ||
      projectEditorTextarea.value?.clientHeight ||
      PROJECT_EDITOR_DEFAULT_VIEWPORT_HEIGHT;
    const contentTop = Math.max(0, projectEditorScrollTop.value - PROJECT_EDITOR_PADDING_TOP);
    const firstVisibleLine = Math.floor(contentTop / PROJECT_EDITOR_LINE_HEIGHT);
    const visibleLineCount = Math.ceil(viewportHeight / PROJECT_EDITOR_LINE_HEIGHT);
    const start = Math.max(0, firstVisibleLine - PROJECT_EDITOR_OVERSCAN_LINES);
    const end = Math.min(
      projectEditorLineCount.value,
      firstVisibleLine + visibleLineCount + PROJECT_EDITOR_OVERSCAN_LINES,
    );
    return { start, end };
  });
  const projectEditorLines = computed<ProjectEditorLine[]>(() => {
    const { start, end } = projectEditorVisibleRange.value;
    return projectEditorLineTexts.value.slice(start, end).map((content, offset) => {
      const index = start + offset;
      return {
        index,
        number: index + 1,
        tokens: tokenizeProjectLine(content || " ", projectLanguage.value),
      };
    });
  });
  const projectEditorHunks = computed<ProjectEditorHunkView[]>(() => {
    if (!project.content || project.content.binary || !project.diff?.hunks.length) return [];
    return project.diff.hunks.flatMap((hunk) => buildProjectEditorHunkViews(hunk, projectLanguage.value));
  });
  const expandedProjectHunk = computed(
    () => projectEditorHunks.value.find((hunk) => hunk.id === expandedProjectHunkIndex.value) ?? null,
  );
  const projectEditorRenderStyle = computed(() => ({
    "--project-editor-scroll-left-offset": `${-projectEditorScrollLeft.value}px`,
  }));
  const projectEditorRenderContentStyle = computed(() => ({
    transform: `translateY(${
      PROJECT_EDITOR_PADDING_TOP +
      projectEditorVisibleRange.value.start * PROJECT_EDITOR_LINE_HEIGHT -
      projectEditorScrollTop.value
    }px)`,
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
  const resultHasConflictMarkers = computed(() => hasGitConflictMarkers(operations.resultDraft));
  const mergeConflictCount = computed(() => operations.conflict?.blocks.length ?? 0);
  const mergeConflictSummary = computed(() => {
    const changeLabel = operations.resultDirty ? "有未保存变更" : "没有变更";
    return `${changeLabel}。${mergeConflictCount.value} 个冲突。`;
  });
  const mergeConflictPositionLabel = computed(() => {
    if (mergeConflictCount.value === 0) return "0/0";
    return `${activeMergeConflictOrdinal.value + 1}/${mergeConflictCount.value}`;
  });
  const mergeResultStateLabel = computed(() => {
    if (resultHasConflictMarkers.value) return "结果仍包含冲突标记";
    return operations.resultDirty ? "结果有未保存修改" : "结果未修改";
  });
  const pullConfirmPreview = computed(() => pullConfirmDialog.value?.preview ?? null);
  const pullConfirmFiles = computed(() => pullConfirmPreview.value?.overlappingPaths.slice(0, 8) ?? []);
  const pullConfirmExtraCount = computed(() => {
    const total = pullConfirmPreview.value?.overlappingPaths.length ?? 0;
    return Math.max(0, total - pullConfirmFiles.value.length);
  });
  const pullConfirmModeLabel = computed(() => {
    const preview = pullConfirmPreview.value;
    if (!preview) return "";
    if (preview.fastForward) return "快进更新";
    if (preview.diverged) return "分叉合并";
    return preview.upToDate ? "已经最新" : "合并更新";
  });
  const isMergeConflictOperation = computed(() => operations.activeOperation === "merge");
  const showMergeConflictWorkbench = computed(
    () =>
      workbenchMode.value === "changes" &&
      Boolean(operations.conflict) &&
      operations.conflict?.path === changes.selectedFile,
  );
  const mergeCurrentSide = computed<MergeDisplaySide>(() =>
    normalizeMergeDisplaySide(operations.conflict?.currentSide, "ours"),
  );
  const mergeIncomingSide = computed<MergeDisplaySide>(() => {
    const incoming = normalizeMergeDisplaySide(operations.conflict?.incomingSide, "theirs");
    return incoming === mergeCurrentSide.value ? oppositeMergeDisplaySide(mergeCurrentSide.value) : incoming;
  });
  const mergeCurrentSourceLabel = computed(() => {
    const currentBranch = branch.value?.currentBranch || "当前版本";
    if (operations.conflict?.conflictSource === "autostash" && mergeCurrentSide.value === "theirs") {
      return `${currentBranch} 的本地修改`;
    }
    return currentBranch;
  });
  const mergeIncomingSourceLabel = computed(() => {
    const target = operations.mergeTarget.trim() || "传入版本";
    if (operations.conflict?.conflictSource === "autostash" && mergeIncomingSide.value === "ours") {
      return `${target} 已拉取版本`;
    }
    return target;
  });
  const mergeLanguage = computed(() => projectLanguageForPath(operations.conflict?.path));
  const mergeResultConflictSnippets = computed<MergeConflictSnippet[]>(() =>
    buildResultMergeConflictSnippets(
      operations.conflict?.blocks ?? [],
      operations.resultBlockSelections,
      operations.resultBlockReplacements,
    ),
  );
  const mergeCurrentConflictSnippets = computed<MergeConflictSnippet[]>(() =>
    buildComparedMergeConflictSnippets(
      operations.conflict?.blocks ?? [],
      mergeCurrentSide.value,
      operations.resultBlockSelections,
    ),
  );
  const mergeIncomingConflictSnippets = computed<MergeConflictSnippet[]>(() =>
    buildComparedMergeConflictSnippets(
      operations.conflict?.blocks ?? [],
      mergeIncomingSide.value,
      operations.resultBlockSelections,
    ),
  );
  const mergeCurrentConflictRanges = computed(() =>
    buildMergeSourceConflictRanges(
      mergeConflictSideContent(operations.conflict, mergeCurrentSide.value),
      operations.conflict?.blocks ?? [],
      mergeCurrentSide.value,
    ),
  );
  const mergeIncomingConflictRanges = computed(() =>
    buildMergeSourceConflictRanges(
      mergeConflictSideContent(operations.conflict, mergeIncomingSide.value),
      operations.conflict?.blocks ?? [],
      mergeIncomingSide.value,
    ),
  );
  const mergeAutoMergeDiff = computed(() =>
    buildMergeAutoMergeDiff(
      mergeConflictSideContent(operations.conflict, mergeCurrentSide.value),
      mergeConflictSideContent(operations.conflict, mergeIncomingSide.value),
      [...mergeCurrentConflictRanges.value.values()],
      [...mergeIncomingConflictRanges.value.values()],
    ),
  );
  const mergeResultAutoMergeRanges = computed(() =>
    buildMergeResultAutoMergeRanges(
      operations.resultDraft,
      mergeAutoMergeDiff.value.resultSnippets,
      mergeResultConflictSnippets.value,
    ),
  );
  const mergeCurrentLines = computed(() =>
    buildMergeCodeLines(
      mergeConflictSideContent(operations.conflict, mergeCurrentSide.value),
      mergeCurrentConflictSnippets.value,
      mergeAutoMergeDiff.value.currentRanges,
      mergeLanguage.value,
    ),
  );
  const mergeIncomingLines = computed(() =>
    buildMergeCodeLines(
      mergeConflictSideContent(operations.conflict, mergeIncomingSide.value),
      mergeIncomingConflictSnippets.value,
      mergeAutoMergeDiff.value.incomingRanges,
      mergeLanguage.value,
    ),
  );
  const mergeResultLines = computed(() =>
    buildMergeCodeLines(
      operations.resultDraft,
      mergeResultConflictSnippets.value,
      mergeResultAutoMergeRanges.value,
      mergeLanguage.value,
    ),
  );
  const mergeCurrentResultConnections = computed(() =>
    buildMergeConflictConnections(
      operations.conflict?.blocks ?? [],
      mergeCurrentConflictRanges.value,
      mergeResultLines.value,
      "current",
      mergeCurrentSide.value,
    ),
  );
  const mergeIncomingResultConnections = computed(() =>
    buildMergeConflictConnections(
      operations.conflict?.blocks ?? [],
      mergeIncomingConflictRanges.value,
      mergeResultLines.value,
      "incoming",
      mergeIncomingSide.value,
    ),
  );
  const mergeResultRenderStyle = computed(() => ({
    transform: `translate(${-mergeResultScrollLeft.value}px, ${-mergeResultScrollTop.value}px)`,
  }));
  const effectiveTheme = computed<Exclude<ThemeMode, "system">>(() => {
    if (settings.themeMode === "system") {
      return systemPrefersDark.value ? "dark" : "light";
    }
    return settings.themeMode;
  });
  const panelLabels: Record<LayoutPanelKey, string> = {
    project: "项目栏",
    changes: "工作区上下文",
  };
  const workspaceGridStyle = computed(() => {
    const columns: string[] = [];

    if (settings.panelVisibility.project) {
      columns.push(settings.projectPaneCollapsed ? "64px" : `${settings.panelWidths.project}px`);
      if (!settings.projectPaneCollapsed) {
        columns.push("6px");
      }
    }

    if (repos.current || repos.selectedPath) {
      columns.push("68px");
    }

    if (repos.current && settings.panelVisibility.changes) {
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
        kind: "remoteDirectory";
        x: number;
        y: number;
        remoteName?: string;
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
  const logHeadLabel = computed(() => (branch.value?.detached ? "HEAD (游离)" : "HEAD (目前分支)"));
  const configuredRemoteNames = computed(() =>
    (repos.current?.remotes ?? []).map((item) => item.name).filter(Boolean),
  );
  const logRemoteGroups = computed<LogRemoteGroup[]>(() => {
    const groups = new Map<string, BranchInfo[]>();
    const orderedNames: string[] = [];
    const ensureGroup = (name: string) => {
      if (!groups.has(name)) {
        groups.set(name, []);
        orderedNames.push(name);
      }
      return groups.get(name)!;
    };

    for (const item of branches.sortedRemoteBranches) {
      const parts = item.name.split("/");
      const remoteName = parts[0] || "remote";
      ensureGroup(remoteName).push(item);
    }
    for (const name of configuredRemoteNames.value) {
      ensureGroup(name);
    }
    return orderedNames.map((name) => ({ name, branches: groups.get(name) ?? [] }));
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
    const canShowEmptyGroups = !logFavoriteRefsOnly.value;
    const remoteGroups = logRemoteGroups.value
      .map((group) => ({
        ...group,
        branches: logFavoriteRefsOnly.value
          ? group.branches.filter((item) => isLogBranchFavorite(item))
          : group.branches,
      }))
      .filter((group) => group.branches.length > 0 || canShowEmptyGroups);
    if (!query) return remoteGroups;
    return remoteGroups
      .map((group) => {
        if (logRefMatches(query, group.name, "远程", "remote")) return group;
        return {
          ...group,
          branches: group.branches.filter((item) =>
            logRefMatches(query, item.name, item.fullName, item.upstream, item.target),
          ),
        };
      })
      .filter(
        (group) =>
          group.branches.length > 0 ||
          (canShowEmptyGroups &&
            logRefMatches(
              query,
              group.name,
              "远程",
              "remote",
              group.branches.length === 0 ? "暂无远程分支" : undefined,
            )),
      );
  });
  const showLogRemoteAddEntry = computed(() => {
    const query = logRefSearchQuery.value;
    return (
      Boolean(repos.current) &&
      !logFavoriteRefsOnly.value &&
      configuredRemoteNames.value.length === 0 &&
      branches.sortedRemoteBranches.length === 0 &&
      logRefMatches(query, "添加远程仓库", "添加", "远程", "remote", "add")
    );
  });
  const showLogRemoteRoot = computed(() => {
    const query = logRefSearchQuery.value;
    if (!repos.current || logFavoriteRefsOnly.value) return visibleLogRemoteGroups.value.length > 0;
    return !query || visibleLogRemoteGroups.value.length > 0 || showLogRemoteAddEntry.value;
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
      showLogRemoteRoot.value ||
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
      if (skipNextChangeDiffLoad) {
        skipNextChangeDiffLoad = false;
        return;
      }
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
    mergeConflictCount,
    (count) => {
      if (count === 0) {
        activeMergeConflictOrdinal.value = 0;
        return;
      }
      activeMergeConflictOrdinal.value = Math.min(activeMergeConflictOrdinal.value, count - 1);
    },
  );

  watch(
    () => operations.selectedConflictPath,
    () => {
      activeMergeConflictOrdinal.value = 0;
      mergeResultScrollTop.value = 0;
      mergeResultScrollLeft.value = 0;
    },
  );

  watch(
    projectEditorTextarea,
    (textarea) => {
      projectEditorResizeObserver?.disconnect();
      projectEditorResizeObserver = null;

      if (!textarea) return;
      updateProjectEditorViewport();
      if (typeof ResizeObserver !== "undefined") {
        projectEditorResizeObserver = new ResizeObserver(updateProjectEditorViewport);
        projectEditorResizeObserver.observe(textarea);
      }
    },
    { flush: "post" },
  );

  watch(
    changeFileListScroller,
    (scroller) => {
      changeFileListResizeObserver?.disconnect();
      changeFileListResizeObserver = null;

      if (!scroller) return;
      updateChangeFileListViewport();
      if (typeof ResizeObserver !== "undefined") {
        changeFileListResizeObserver = new ResizeObserver(updateChangeFileListViewport);
        changeFileListResizeObserver.observe(scroller);
      }
    },
    { flush: "post" },
  );

  watch(
    () => flatChangeFileRows.value.length,
    () => {
      nextTick(() => {
        const scroller = changeFileListScroller.value;
        if (!scroller) return;
        const maxScrollTop = Math.max(0, flatChangeFileRows.value.length * CHANGE_FILE_ROW_HEIGHT - scroller.clientHeight);
        if (scroller.scrollTop > maxScrollTop) {
          scroller.scrollTop = maxScrollTop;
        }
        updateChangeFileListViewport();
      }).catch(() => undefined);
    },
    { flush: "post" },
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
      nextTick(updateProjectEditorViewport).catch(() => undefined);
    },
  );

  watch(
    () => project.diff?.text,
    () => {
      projectEditorScrollTop.value = projectEditorTextarea.value?.scrollTop ?? 0;
      if (
        expandedProjectHunkIndex.value !== null &&
        !projectEditorHunks.value.some((hunk) => hunk.id === expandedProjectHunkIndex.value)
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

  watch(
    activeError,
    (message) => {
      if (!message) return;
      showErrorDialog(message);
      clearErrorSources();
    },
    { flush: "post" },
  );

  watch(workbenchMode, (mode) => {
    if (mode === "project") {
      project.refresh().catch(() => undefined);
    }
    if (mode === "log" && repos.current && history.commits.length === 0 && !history.loading) {
      history.refresh().catch(() => undefined);
    }
  });

  onMounted(() => {
    window.addEventListener("mousedown", preventRightClickTextSelection, { capture: true });
    window.addEventListener("contextmenu", preventNativeContextMenu, { capture: true });
    setupProjectDragDrop();

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
    window.removeEventListener("mousedown", preventRightClickTextSelection, { capture: true });
    window.removeEventListener("contextmenu", preventNativeContextMenu, { capture: true });
    projectDragDropDisposed = true;
    stopProjectDragDrop?.();
    stopSystemThemeWatch?.();
    projectEditorResizeObserver?.disconnect();
    changeFileListResizeObserver?.disconnect();
    clearAutoFetchTimer();
    clearNoticeToastTimer();
  });

  function preventNativeContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function preventRightClickTextSelection(event: MouseEvent) {
    if (event.button !== 2 || isEditableTarget(event.target)) return;
    event.preventDefault();
  }

  function isEditableTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return Boolean(target.closest("input, textarea, select, [contenteditable], [role='textbox']"));
  }

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

  function showErrorDialog(message: string) {
    const id = errorDialogId + 1;
    errorDialogId = id;
    errorDialog.value = { id, message };
  }

  function dismissErrorDialog(id?: number) {
    if (id !== undefined && errorDialog.value?.id !== id) return;
    errorDialog.value = null;
  }

  function isUiActionPending(key: string) {
    return (pendingUiActions.value[key] ?? 0) > 0;
  }

  function isUiActionActive(key: string) {
    if (isUiActionPending(key)) return true;

    switch (key) {
      case "workspace.refresh":
        return workspaceRefreshBusy.value;
      case "project.refresh":
        return project.loading;
      case "advanced.refresh":
        return advanced.loading;
      case "history.refresh":
        return history.loading;
      case "branches.refresh":
        return branches.loading;
      case "operations.refresh":
        return operations.loading;
      case "remote.fetch":
        return activeRemoteAction.value === "fetch" || remote.activeAction === "fetch";
      case "remote.pull":
        return activeRemoteAction.value === "pull" || remote.activeAction === "pull";
      case "remote.push":
        return activeRemoteAction.value === "push" || remote.activeAction === "push";
      case "remote.fetchAll":
        return activeRemoteAction.value === "fetchAll" || remote.activeAction === "fetchAll";
      default:
        return false;
    }
  }

  function actionIcon(key: string, icon: unknown) {
    return isUiActionActive(key) ? LoaderCircle : icon;
  }

  function actionIconClass(key: string) {
    return { "button-spinner": isUiActionActive(key) };
  }

  function actionButtonClass(key: string) {
    return { loading: isUiActionActive(key) };
  }

  function remoteActionKey(action: "fetch" | "pull" | "push") {
    return `remote.${action}`;
  }

  function branchActionKey(action: string, target = "") {
    return `branch.${action}:${target}`;
  }

  function nowMs() {
    return typeof performance !== "undefined" ? performance.now() : Date.now();
  }

  function waitMs(ms: number) {
    return new Promise<void>((resolve) => {
      globalThis.setTimeout(resolve, ms);
    });
  }

  function waitAnimationFrame() {
    return new Promise<void>((resolve) => {
      if (typeof window === "undefined" || typeof window.requestAnimationFrame !== "function") {
        globalThis.setTimeout(resolve, 16);
        return;
      }
      window.requestAnimationFrame(() => resolve());
    });
  }

  async function waitForOperationPaint() {
    await nextTick();
    await waitAnimationFrame();
    await waitAnimationFrame();
  }

  async function runUiAction<T>(key: string, action: () => Promise<T>) {
    const startedAt = nowMs();
    pendingUiActions.value = {
      ...pendingUiActions.value,
      [key]: (pendingUiActions.value[key] ?? 0) + 1,
    };
    await waitForOperationPaint();
    try {
      return await action();
    } finally {
      const remainingMs = MIN_OPERATION_BUSY_MS - (nowMs() - startedAt);
      if (remainingMs > 0) {
        await waitMs(remainingMs);
      }
      const nextCount = (pendingUiActions.value[key] ?? 1) - 1;
      const nextActions = { ...pendingUiActions.value };
      if (nextCount > 0) {
        nextActions[key] = nextCount;
      } else {
        delete nextActions[key];
      }
      pendingUiActions.value = nextActions;
    }
  }

  function clearNoticeSources() {
    operations.notice = "";
    advanced.notice = "";
    branches.notice = "";
    changes.notice = "";
    remote.notice = "";
  }

  function clearErrorSources() {
    repos.error = "";
    branches.error = "";
    changes.error = "";
    diff.error = "";
    commit.error = "";
    history.error = "";
    operations.error = "";
    project.error = "";
    remote.error = "";
    advanced.error = "";
  }

  function normalizeSelectedPaths(selected: string | string[] | null) {
    if (!selected) return [];
    return Array.isArray(selected) ? selected : [selected];
  }

  async function addRepositoryPaths(paths: string[]) {
    const uniquePaths = [...new Set(paths.map((path) => path.trim()).filter(Boolean))];
    if (uniquePaths.length === 0) return [];

    const opened = await repos.openMany(uniquePaths);
    await loadSelectedProject();
    return opened;
  }

  async function chooseRepository() {
    await runUiAction("repo.choose", async () => {
      const selected = await open({
        directory: true,
        multiple: true,
        title: "添加 Git 仓库",
      });

      const paths = normalizeSelectedPaths(selected);
      if (paths.length === 0) return;

      await addRepositoryPaths(paths);
    });
  }

  async function addDroppedProjectPaths(paths: string[]) {
    await runUiAction("repo.drop", async () => {
      const directories = await filterProjectDirectories(paths);
      if (directories.length === 0) {
        repos.error = "请拖入文件夹来添加项目";
        return;
      }

      await addRepositoryPaths(directories);
      showNoticeToast(directories.length === 1 ? "已添加拖入项目" : `已添加 ${directories.length} 个拖入项目`);
    });
  }

  async function setupProjectDragDrop() {
    try {
      const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          projectDropActive.value = true;
          return;
        }

        projectDropActive.value = false;
        if (event.payload.type !== "drop") return;

        addDroppedProjectPaths(event.payload.paths).catch((error) => {
          repos.error = String(error);
        });
      });

      if (projectDragDropDisposed) {
        unlisten();
        return;
      }
      stopProjectDragDrop = unlisten;
    } catch {
      // Browser-only dev sessions do not expose Tauri drag/drop events.
    }
  }

  async function initSelectedProject() {
    if (!repos.selectedPath) return;
    await runUiAction("repo.init", async () => {
      advanced.initDirectory = repos.selectedPath;
      const repo = await advanced.initAt();
      if (!repo) return;
      repos.setCurrent(repo);
      await loadCurrentRepository();
    });
  }

  async function switchRepository(path: string) {
    if (repos.selectedPath === path) return;
    await runUiAction(`repo.switch:${path}`, async () => {
      await repos.select(path);
      await loadSelectedProject();
    });
  }

  async function removeRepository(path: string) {
    await runUiAction(`repo.remove:${path}`, async () => {
      const wasCurrent = repos.selectedPath === path;
      repos.remove(path);
      if (!wasCurrent) return;

      await loadSelectedProject();
    });
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
    await refreshRepositoryMetadata();
    syncOperationTargets();
    syncSelectedRemote(true);
    await pickAutomaticChangeFile(settings.selectedSide);
    if (workbenchMode.value === "project") {
      await project.refresh();
    }
  }

  function clearProjectView() {
    clearLogDiffTabs();
    resetCommitMessageHistoryCursor();
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

  function automaticChangeCandidateCount(side: ChangeSide) {
    const preferred = filesForChangeSide(side);
    if (preferred.length > 0) return preferred.length;
    const fallbackSide: ChangeSide = side === "staged" ? "unstaged" : "staged";
    return filesForChangeSide(fallbackSide).length;
  }

  function hasSelectedChangeFile() {
    if (!changes.selectedFile) return false;
    return filesForChangeSide(changes.selectedSide).some((file) => file.path === changes.selectedFile);
  }

  function clearAutomaticChangeSelection() {
    clearMergeConflictView();
    changes.selectedFile = null;
    changes.selectedPaths = [];
    diff.current = null;
  }

  function prepareAutomaticChangeSelection() {
    const previousSelection = `${changes.selectedSide}:${changes.selectedFile ?? ""}`;
    skipNextChangeDiffLoad = true;

    return () => {
      const nextSelection = `${changes.selectedSide}:${changes.selectedFile ?? ""}`;
      if (nextSelection === previousSelection) {
        skipNextChangeDiffLoad = false;
      }
      return true;
    };
  }

  async function pickAutomaticChangeFile(side: ChangeSide) {
    if (hasSelectedChangeFile()) {
      await diff.loadSelected();
      return;
    }

    if (automaticChangeCandidateCount(side) > AUTO_LOAD_CHANGE_DIFF_LIMIT) {
      clearAutomaticChangeSelection();
      return;
    }

    const shouldLoadDiff = prepareAutomaticChangeSelection();
    pickFirstAvailable(side);
    if (shouldLoadDiff()) {
      await diff.loadSelected();
    }
  }

  async function refreshRepositoryMetadata(includeHistory = workbenchMode.value === "log") {
    const tasks: Promise<unknown>[] = [branches.refresh(), operations.refresh()];
    if (includeHistory) {
      tasks.push(history.refresh());
    }
    await Promise.all(tasks);
  }

  async function refreshAll() {
    await runUiAction("workspace.refresh", async () => {
      await changes.refresh();
      changelists.pruneMissingPaths(changes.files.map((file) => file.path));
      await refreshRepositoryMetadata();
      branches.syncUpstreamDraft();
      syncOperationTargets();
      syncSelectedRemote();
      await pickAutomaticChangeFile(settings.selectedSide);
      if (workbenchMode.value === "project") {
        await project.refresh();
      }
    });
  }

  async function refreshChangesOnly() {
    await runUiAction("workspace.refresh", async () => {
      await changes.refresh({ includeShelves: false });
      changelists.pruneMissingPaths(changes.files.map((file) => file.path));
      if (!changes.selectedFile) {
        diff.current = null;
      }
    });
  }

  async function reloadAfterGitOperation() {
    await changes.refresh();
    changelists.pruneMissingPaths(changes.files.map((file) => file.path));
    await refreshRepositoryMetadata();
    branches.syncUpstreamDraft();
    syncOperationTargets();
    syncSelectedRemote();
    await pickAutomaticChangeFile(settings.selectedSide);
    if (workbenchMode.value === "project") {
      await project.refresh();
    }
  }

  async function loadAdvancedSnapshots() {
    if (!repos.current) return;
    await runUiAction("advanced.refresh", async () => {
      await Promise.allSettled([
        advanced.refreshWorktrees(),
        advanced.refreshStashes(),
        advanced.refreshSubmodules(),
        advanced.refreshCommitMessages(),
      ]);
    });
  }

  async function refreshAfterRemoteAction() {
    await refreshRepositoryMetadata();
    branches.syncUpstreamDraft();
    syncOperationTargets();
    syncSelectedRemote();
  }

  async function executeRemoteAction(
    action: "fetch" | "pull" | "push",
    options: { smartMerge?: boolean; targetBranch?: string } = {},
  ) {
    const result = await remote.run(action, options);
    await refreshAfterRemoteAction();
    if (action === "pull" && result && !result.ok) {
      await openFirstPullConflict();
    }
    return result;
  }

  async function runRemoteAction(action: "fetch" | "pull" | "push") {
    const actionKey = remoteActionKey(action);
    if (isUiActionPending(actionKey)) return;
    if (action === "push") {
      confirmRemotePush();
      return;
    }
    activeRemoteAction.value = action;
    try {
      await runUiAction(actionKey, async () => {
        if (action === "pull") {
          const preview = await remote.previewPull();
          if (!preview) return;
          operations.mergeTarget = preview.target;
          if (preview.needsConfirmation) {
            pullConfirmDialog.value = {
              preview,
              loading: false,
            };
            return;
          }
        }

        await executeRemoteAction(action);
      });
    } finally {
      if (activeRemoteAction.value === action) {
        activeRemoteAction.value = null;
      }
    }
  }

  async function ensureCommitMessageHistory() {
    if (commitMessageHistoryItems.value.length > 0 || advanced.loading) return;
    await advanced.refreshCommitMessages();
  }

  function isCommitMessageAtFirstLine(textarea: HTMLTextAreaElement) {
    const start = Math.min(textarea.selectionStart, textarea.selectionEnd);
    return !commit.message.slice(0, start).includes("\n");
  }

  function isCommitMessageAtLastLine(textarea: HTMLTextAreaElement) {
    const end = Math.max(textarea.selectionStart, textarea.selectionEnd);
    return !commit.message.slice(end).includes("\n");
  }

  function focusCommitMessageEnd() {
    nextTick(() => {
      const textarea = commitMessageTextarea.value;
      if (!textarea) return;
      const end = commit.message.length;
      textarea.focus();
      textarea.setSelectionRange(end, end);
    }).catch(() => undefined);
  }

  function rememberCommitMessage(message: string) {
    const value = message.trim();
    if (!value) return;
    advanced.commitMessages = [
      value,
      ...advanced.commitMessages.filter((item) => item.trim() !== value),
    ].slice(0, 40);
  }

  function resetCommitMessageHistoryCursor() {
    commitMessageHistoryIndex.value = -1;
    commitMessageHistoryDraft.value = "";
  }

  async function navigateCommitMessageHistory(event: KeyboardEvent, direction: "previous" | "next") {
    if (event.isComposing) return;
    const textarea = event.currentTarget instanceof HTMLTextAreaElement ? event.currentTarget : null;
    if (!textarea) return;

    const browsingHistory = commitMessageHistoryIndex.value >= 0;
    if (direction === "next" && !browsingHistory) return;
    const canUseHistory =
      direction === "previous"
        ? browsingHistory || isCommitMessageAtFirstLine(textarea)
        : browsingHistory || isCommitMessageAtLastLine(textarea);
    if (!canUseHistory) return;

    event.preventDefault();
    try {
      await ensureCommitMessageHistory();
    } catch {
      return;
    }
    const messages = commitMessageHistoryItems.value;
    if (messages.length === 0) return;

    if (direction === "previous") {
      if (commitMessageHistoryIndex.value === -1) {
        commitMessageHistoryDraft.value = commit.message;
        commitMessageHistoryIndex.value = 0;
      } else {
        commitMessageHistoryIndex.value = Math.min(commitMessageHistoryIndex.value + 1, messages.length - 1);
      }
      commit.message = messages[commitMessageHistoryIndex.value] ?? commit.message;
    } else if (commitMessageHistoryIndex.value > 0) {
      commitMessageHistoryIndex.value -= 1;
      commit.message = messages[commitMessageHistoryIndex.value] ?? commit.message;
    } else {
      commitMessageHistoryIndex.value = -1;
      commit.message = commitMessageHistoryDraft.value;
      commitMessageHistoryDraft.value = "";
    }

    focusCommitMessageEnd();
  }

  async function runRemoteActionFromPointer(event: PointerEvent, action: "fetch" | "pull" | "push") {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    event.preventDefault();
    await runRemoteAction(action);
  }

  async function fetchAllRepositories() {
    if (isUiActionPending("remote.fetchAll")) return;
    activeRemoteAction.value = "fetchAll";
    try {
      await runUiAction("remote.fetchAll", async () => {
        await remote.fetchAllRepositories();
        await refreshAfterRemoteAction();
      });
    } finally {
      if (activeRemoteAction.value === "fetchAll") {
        activeRemoteAction.value = null;
      }
    }
  }

  function cancelPullConfirmDialog() {
    if (pullConfirmDialog.value?.loading) return;
    pullConfirmDialog.value = null;
  }

  async function confirmPullSmartMerge() {
    const dialog = pullConfirmDialog.value;
    if (!dialog || dialog.loading) return;

    dialog.loading = true;
    operations.mergeTarget = dialog.preview.target;
    try {
      await executeRemoteAction("pull", { smartMerge: true });
      pullConfirmDialog.value = null;
    } catch (error) {
      const message = String(error);
      pullConfirmDialog.value = null;
      clearErrorSources();
      showErrorDialog(message);
    }
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

  async function openFirstPullConflict() {
    const firstConflict = operations.conflictedPaths[0];
    if (!firstConflict) return;
    workbenchMode.value = "changes";
    await selectConflict(firstConflict);
  }

  async function saveRemoteConfig() {
    await runUiAction("remote.save", async () => {
      await remote.saveRemote();
      syncSelectedRemote(true);
      await refreshRepositoryMetadata();
      branches.syncUpstreamDraft();
      syncOperationTargets();
    });
  }

  async function unshallowCurrentRepository() {
    await runUiAction("advanced.unshallow", async () => {
      await advanced.unshallow(remote.selectedRemote || undefined);
      await reloadAfterGitOperation();
    });
  }

  async function renameSelectedBranch() {
    if (!advanced.branchRenameFrom || !advanced.branchRenameTo.trim()) return;
    if (!window.confirm(`将分支 ${advanced.branchRenameFrom} 重命名为 ${advanced.branchRenameTo.trim()}？`)) return;
    await runUiAction("advanced.branch.rename", async () => {
      await advanced.renameSelectedBranch();
      await loadCurrentRepository();
    });
  }

  async function cleanupMergedBranches() {
    const target = branch.value?.currentBranch || "HEAD";
    if (!window.confirm(`清理已合并到 ${target} 的本地分支？`)) return;
    await runUiAction("advanced.branch.cleanup", async () => {
      await advanced.cleanupMerged(target);
      await loadCurrentRepository();
    });
  }

  async function runRefComparison() {
    await runUiAction("advanced.compare", () => advanced.loadComparison());
  }

  async function generatePatch(staged = false) {
    await runUiAction(staged ? "advanced.patch.staged" : "advanced.patch.worktree", () =>
      advanced.generatePatch(staged),
    );
  }

  async function applyPatchDraft() {
    await runUiAction("advanced.patch.apply", async () => {
      await advanced.applyPatchDraft();
      await reloadAfterGitOperation();
    });
  }

  async function createWorktreeFromDraft() {
    await runUiAction("advanced.worktree.create", async () => {
      await advanced.createWorktreeFromDraft();
      await loadAdvancedSnapshots();
    });
  }

  async function removeWorktree(path: string) {
    if (!window.confirm(`移除工作树 ${path}？`)) return;
    await runUiAction(`advanced.worktree.remove:${path}`, async () => {
      await advanced.removeWorktreePath(path, true);
      await loadAdvancedSnapshots();
    });
  }

  async function runStashAction(stashRef: string, action: "apply" | "pop" | "drop") {
    if (action === "drop" && !window.confirm(`删除 ${stashRef}？`)) return;
    await runUiAction(`advanced.stash.${action}:${stashRef}`, async () => {
      await advanced.runStashAction(stashRef, action);
      await reloadAfterGitOperation();
    });
  }

  async function clearAllStashes() {
    if (!window.confirm("清空所有贮藏记录？")) return;
    await runUiAction("advanced.stash.clear", async () => {
      await advanced.clearAllStashes();
      await loadAdvancedSnapshots();
    });
  }

  async function updateAllSubmodules() {
    await runUiAction("advanced.submodule.update", async () => {
      await advanced.updateAllSubmodules();
      await reloadAfterGitOperation();
    });
  }

  async function loadLfsStatus() {
    await runUiAction("advanced.lfs", () => advanced.refreshLfsStatus());
  }

  async function loadSelectedFileHistory() {
    const selectedFile = changes.selectedFile;
    if (!selectedFile) return;
    await runUiAction("advanced.fileHistory", () => advanced.loadFileHistory(selectedFile));
  }

  async function loadSelectedBlame() {
    const selectedFile = changes.selectedFile;
    if (!selectedFile) return;
    await runUiAction("advanced.blame", () => advanced.loadBlame(selectedFile));
  }

  function selectSide(side: ChangeSide) {
    settings.setSide(side);
    changes.selectedSide = side;
    if (automaticChangeCandidateCount(side) > AUTO_LOAD_CHANGE_DIFF_LIMIT) {
      clearAutomaticChangeSelection();
      return;
    }
    pickFirstAvailable(side);
  }

  function setIncludeIgnored(event: Event) {
    settings.setIncludeIgnored((event.target as HTMLInputElement).checked);
    refreshChangesOnly().catch(() => undefined);
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

  function clearMergeConflictView() {
    if (!operations.conflict && !operations.selectedConflictPath) return;
    operations.conflict = null;
    operations.resultDraft = "";
    operations.resultDirty = false;
    operations.selectedConflictPath = "";
  }

  function openMergeConflictView(path: string) {
    if (operations.conflict?.path === path && operations.selectedConflictPath === path) return;
    operations.loadConflict(path).catch(() => undefined);
  }

  function selectFile(file: ChangedFile, side: ChangeSide, options: { openConflict?: boolean } = {}) {
    if (!file.conflicted) {
      clearMergeConflictView();
      changes.selectFile(file, side);
      scrollChangeFilePathIntoView(file.path, side);
      return;
    }

    changes.selectFile(file, side);
    scrollChangeFilePathIntoView(file.path, side);
    if (options.openConflict ?? true) {
      workbenchMode.value = "changes";
      openMergeConflictView(file.path);
    }
  }

  function selectChangeFileForContext(file: ChangedFile, side: ChangeSide) {
    if (file.conflicted) {
      selectFile(file, side);
      return;
    }

    if (!changes.selectedPaths.includes(file.path)) {
      selectFile(file, side);
      return;
    }

    clearMergeConflictView();
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
    const preferred = filesForChangeSide(side);
    const fallbackSide: ChangeSide = side === "staged" ? "unstaged" : "staged";
    const fallback = filesForChangeSide(fallbackSide);
    const nextSide = preferred.length > 0 ? side : fallbackSide;
    const file = preferred[0] ?? fallback[0];
    if (file) {
      selectFile(file, nextSide);
    } else {
      clearMergeConflictView();
      changes.selectedFile = null;
      changes.selectedPaths = [];
      diff.current = null;
    }
  }

  function filesForChangeSide(side: ChangeSide) {
    const files = changes.filesForSide(side);
    if (side !== "unstaged") return files;

    const paths = new Set(files.map((file) => file.path));
    const conflicted = changes.files.filter((file) => file.conflicted && !paths.has(file.path));
    return conflicted.length > 0 ? [...files, ...conflicted] : files;
  }

  async function runAndReload(action: () => Promise<unknown>, key?: string) {
    const runner = async () => {
      await action();
      await diff.loadSelected();
    };
    if (key) {
      await runUiAction(key, runner);
      return;
    }
    await runner();
  }

  async function stageSelected() {
    await runAndReload(() => changes.stageSelected(), "changes.stage");
  }

  async function unstageSelected() {
    await runAndReload(() => changes.unstageSelected(), "changes.unstage");
  }

  async function discardSelected() {
    if (changes.activePaths.length === 0) return;
    if (!window.confirm("回滚选中的本地变更？")) return;
    await runAndReload(() => changes.discardSelected(), "changes.discard");
  }

  async function shelveSelected() {
    await runAndReload(() => changes.shelveSelected(shelveMessage.value), "changes.shelve");
    shelveMessage.value = "";
  }

  async function deleteShelfRecord(record: ShelfInfo) {
    if (!window.confirm(`删除搁置 ${record.message}？`)) return;
    await runAndReload(() => changes.deleteShelfRecord(record), `changes.shelf.delete:${record.id ?? record.stashRef}`);
  }

  async function unshelveRecord(record: ShelfInfo) {
    if (record.appliedAt) return;
    await runAndReload(() => changes.unshelveRecord(record), `changes.shelf.restore:${record.id ?? record.stashRef}`);
  }

  function currentCommitBranchLabel() {
    return branch.value?.currentBranch || repos.current?.branch || "游离 HEAD";
  }

  function currentPushTargetBranchName(remoteName = remote.selectedRemote || "origin") {
    const upstream = branch.value?.upstream;
    const prefix = `${remoteName}/`;
    if (upstream?.startsWith(prefix)) {
      return upstream.slice(prefix.length);
    }
    const currentBranch = currentCommitBranchLabel();
    return currentBranch === "游离 HEAD" ? "" : currentBranch;
  }

  function selectedCommitOptionLabels() {
    const options: string[] = [];
    if (commit.amend) options.push("修正上次提交");
    if (commit.signOff) options.push("追加签署");
    if (commit.gpgSign) options.push("GPG 签名");
    if (commit.author.trim()) options.push(`覆盖作者：${commit.author.trim()}`);
    return options;
  }

  function selectedPushOptionLabels(targetBranch = currentPushTargetBranchName()) {
    const options: string[] = [];
    if (remote.setUpstream) options.push("设置上游");
    if (remote.forceWithLease) options.push("安全强推");
    if (remote.pushTags) options.push("同步标签");
    if (remote.isProtectedTarget(targetBranch)) {
      options.push(remote.allowProtectedPush ? "允许保护分支推送" : "确认后推送受保护分支");
    }
    return options;
  }

  function cancelSubmitConfirmDialog() {
    if (submitConfirmDialog.value?.loading) return;
    submitConfirmDialog.value = null;
  }

  function isSubmitConfirmDirectoryExpanded(path: string) {
    return expandedSubmitConfirmDirectories.value[path] ?? true;
  }

  function toggleSubmitConfirmDirectory(path: string) {
    expandedSubmitConfirmDirectories.value = {
      ...expandedSubmitConfirmDirectories.value,
      [path]: !isSubmitConfirmDirectoryExpanded(path),
    };
  }

  function stringifyError(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function containsChinese(text: string) {
    return /[\u4e00-\u9fff]/.test(text);
  }

  function translateGitError(error: unknown) {
    const raw = stringifyError(error).trim();
    const lower = raw.toLowerCase();

    if (
      lower.includes("non-fast-forward") ||
      lower.includes("[rejected]") ||
      lower.includes("fetch first") ||
      lower.includes("failed to push some refs") ||
      lower.includes("current branch is behind") ||
      lower.includes("tip of your current branch is behind")
    ) {
      return "推送失败：远程分支已有新的提交，本地分支落后于远程。请先拉取远程更新，完成合并或变基后再推送。";
    }

    if (lower.includes("permission denied") || lower.includes("publickey")) {
      return "认证失败：当前 SSH 密钥或账号没有访问远程仓库的权限。请检查远程地址、SSH 密钥和仓库权限后重试。";
    }

    if (lower.includes("authentication failed") || lower.includes("could not read username")) {
      return "认证失败：远程仓库需要有效账号或访问令牌。请检查 Git 凭据后重试。";
    }

    if (lower.includes("repository not found")) {
      return "远程仓库不存在或当前账号没有访问权限。请检查远程地址和仓库权限。";
    }

    if (
      lower.includes("no merge base found") ||
      lower.includes("unrelated histories") ||
      lower.includes("refusing to merge unrelated histories")
    ) {
      return "拉取失败：当前分支与远程目标没有共同提交历史。请确认远程或上游分支是否选错；若确实要合并两个无关仓库，请在命令行使用 --allow-unrelated-histories。";
    }

    if (lower.includes("could not resolve host")) {
      return "网络连接失败：无法解析远程仓库域名。请检查网络或 DNS 后重试。";
    }

    if (lower.includes("failed to connect") || lower.includes("timed out") || lower.includes("timeout")) {
      return "网络连接超时：无法连接远程仓库。请检查网络状态后重试。";
    }

    if (containsChinese(raw)) {
      return raw.replace(/^error:\s*/i, "").trim();
    }

    return "操作失败：Git 返回了无法识别的错误。请检查远程仓库状态、网络和权限后重试。";
  }

  function commitCurrent(pushAfter = false) {
    if (commitBusy.value || !canCommit.value || (pushAfter && !remote.selectedRemote)) return;
    commit.error = "";
    const paths = [...selectedCommitPaths.value];
    if (paths.length === 0) {
      commit.error = "请先勾选要提交的文件。";
      return;
    }

    submitConfirmDialog.value = {
      mode: pushAfter ? "commit-push" : "commit",
      paths,
      message: commit.message.trim(),
      remoteName: pushAfter ? remote.selectedRemote : "",
      currentBranch: currentCommitBranchLabel(),
      targetBranch: pushAfter ? currentPushTargetBranch.value : "",
      options: pushAfter
        ? [...selectedCommitOptionLabels(), ...selectedPushOptionLabels(currentPushTargetBranch.value)]
        : selectedCommitOptionLabels(),
      loading: false,
    };
    expandedSubmitConfirmDirectories.value = {};
  }

  function confirmRemotePush() {
    if (!repos.current || remote.loading) return;
    submitConfirmDialog.value = {
      mode: "push",
      paths: [],
      message: "",
      remoteName: remote.selectedRemote || "origin",
      currentBranch: currentCommitBranchLabel(),
      targetBranch: currentPushTargetBranch.value,
      options: selectedPushOptionLabels(currentPushTargetBranch.value),
      loading: false,
    };
    expandedSubmitConfirmDirectories.value = {};
  }

  async function confirmSubmitAction() {
    const dialog = submitConfirmDialog.value;
    if (!dialog || dialog.loading) return;

    dialog.loading = true;
    try {
      if (dialog.mode === "push") {
        const restoreAllowProtectedPush = remote.allowProtectedPush;
        const allowConfirmedProtectedPush =
          remote.isProtectedTarget(dialog.targetBranch) && !remote.allowProtectedPush;
        if (allowConfirmedProtectedPush) {
          remote.allowProtectedPush = true;
        }
        try {
          await runUiAction(remoteActionKey("push"), async () => {
            await executeRemoteAction("push", { targetBranch: dialog.targetBranch });
          });
        } finally {
          if (allowConfirmedProtectedPush) {
            remote.allowProtectedPush = restoreAllowProtectedPush;
          }
        }
      } else {
        pendingCommitAction.value = dialog.mode === "commit-push" ? "push" : "commit";
        await commit.commit(
          dialog.mode === "commit-push" ? dialog.remoteName || undefined : undefined,
          false,
          dialog.paths,
          dialog.mode === "commit-push" ? dialog.targetBranch : undefined,
        );
        rememberCommitMessage(dialog.message);
        resetCommitMessageHistoryCursor();
        await refreshRepositoryMetadata();
        syncOperationTargets();
      }
      submitConfirmDialog.value = null;
    } catch (error) {
      const message = translateGitError(error);
      clearErrorSources();
      submitConfirmDialog.value = null;
      showErrorDialog(message);
    } finally {
      pendingCommitAction.value = null;
      if (submitConfirmDialog.value === dialog) {
        dialog.loading = false;
      }
    }
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
    const cacheKey = `${language}\u0000${content}`;
    const cached = projectLineTokenCache.get(cacheKey);
    if (cached) return cached;

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

    return rememberProjectLineTokens(cacheKey, tokens.length ? tokens : [{ text: " " }]);
  }

  function rememberProjectLineTokens(key: string, tokens: ProjectCodeToken[]) {
    if (projectLineTokenCache.size >= PROJECT_TOKEN_CACHE_LIMIT) {
      const oldestKey = projectLineTokenCache.keys().next().value;
      if (oldestKey) {
        projectLineTokenCache.delete(oldestKey);
      }
    }
    projectLineTokenCache.set(key, tokens);
    return tokens;
  }

  function projectKeywords(language: string) {
    const cached = projectKeywordCache.get(language);
    if (cached) return cached;

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
    const keywords = new Set([...(byLanguage[language] ?? shared), ...(language === "plain" ? [] : shared)]);
    projectKeywordCache.set(language, keywords);
    return keywords;
  }

  function buildProjectEditorHunkViews(hunk: DiffHunk, language: string): ProjectEditorHunkView[] {
    return parseProjectHunkPatch(hunk, language).map((block) => {
      const fallbackLineStart = Math.max(1, block.newStart || hunk.newStart || 1);
      const lineStart = block.changedNewStart ?? fallbackLineStart;
      const changedNewEnd = block.changedNewEnd ?? lineStart;
      const lineCount = block.addedLines === 0 ? 1 : Math.max(1, changedNewEnd - lineStart + 1);
      const tone =
        block.deletedLines > 0 && block.addedLines > 0
          ? "modified"
          : block.deletedLines > 0
            ? "deleted"
            : "added";

      return {
        id: `${hunk.index}:${block.blockIndex}`,
        hunkIndex: hunk.index,
        blockIndex: block.blockIndex,
        header: hunk.header,
        tone,
        lineStart,
        lineCount,
        changedNewStart: lineStart,
        changedNewEnd,
        changedOldStart: block.changedOldStart,
        changedOldEnd: block.changedOldEnd,
        oldStart: block.changedOldStart ?? block.oldStart,
        oldLines: block.deletedLines,
        newStart: lineStart,
        newLines: block.addedLines,
        patch: block.patch,
        addedLines: block.addedLines,
        deletedLines: block.deletedLines,
        originalLines: block.originalLines,
      };
    });
  }

  function parseProjectHunkPatch(hunk: DiffHunk, language: string): ProjectHunkChangeBlock[] {
    const { fileHeader, entries } = parseProjectHunkEntries(hunk);
    const ranges = projectHunkChangeRanges(entries);

    return ranges.map((range, blockIndex) =>
      buildProjectHunkChangeBlock(hunk, language, fileHeader, entries, range, blockIndex),
    );
  }

  function parseProjectHunkEntries(hunk: DiffHunk) {
    const fileHeader: string[] = [];
    const entries: ProjectHunkEntry[] = [];
    let oldLineNumber = hunk.oldStart;
    let newLineNumber = hunk.newStart;
    let inHunk = false;

    for (const line of hunk.patch.split("\n")) {
      if (line.startsWith("@@ ")) {
        inHunk = true;
        oldLineNumber = hunk.oldStart;
        newLineNumber = hunk.newStart;
        continue;
      }
      if (!inHunk) {
        if (line) fileHeader.push(line);
        continue;
      }
      if (!line || line.startsWith("\\ No newline")) continue;

      const prefix = line.charAt(0);
      if (prefix !== " " && prefix !== "+" && prefix !== "-") continue;

      const oldAnchorLineNumber = oldLineNumber;
      const newAnchorLineNumber = newLineNumber;
      const entry: ProjectHunkEntry = {
        index: entries.length,
        prefix,
        content: line.slice(1),
        oldLineNumber: prefix === "+" ? null : oldLineNumber,
        newLineNumber: prefix === "-" ? null : newLineNumber,
        oldAnchorLineNumber,
        newAnchorLineNumber,
      };
      entries.push(entry);

      if (prefix !== "+") oldLineNumber += 1;
      if (prefix !== "-") newLineNumber += 1;
    }

    return { fileHeader, entries };
  }

  function projectHunkChangeRanges(entries: ProjectHunkEntry[]) {
    const ranges: { startEntryIndex: number; endEntryIndex: number }[] = [];
    let activeStart: number | null = null;
    let activeEnd: number | null = null;

    for (const entry of entries) {
      if (entry.prefix === " ") {
        if (activeStart !== null && activeEnd !== null) {
          ranges.push({ startEntryIndex: activeStart, endEntryIndex: activeEnd });
          activeStart = null;
          activeEnd = null;
        }
        continue;
      }

      activeStart = activeStart ?? entry.index;
      activeEnd = entry.index;
    }

    if (activeStart !== null && activeEnd !== null) {
      ranges.push({ startEntryIndex: activeStart, endEntryIndex: activeEnd });
    }

    return ranges;
  }

  function buildProjectHunkChangeBlock(
    hunk: DiffHunk,
    language: string,
    fileHeader: string[],
    entries: ProjectHunkEntry[],
    range: { startEntryIndex: number; endEntryIndex: number },
    blockIndex: number,
  ): ProjectHunkChangeBlock {
    const changeEntries = entries.slice(range.startEntryIndex, range.endEntryIndex + 1);
    const deletedEntries = changeEntries.filter((entry) => entry.prefix === "-");
    const addedEntries = changeEntries.filter((entry) => entry.prefix === "+");
    const changedOldStart = minEntryLine(deletedEntries, "oldLineNumber");
    const changedOldEnd = maxEntryLine(deletedEntries, "oldLineNumber");
    const changedNewStart = minEntryLine(addedEntries, "newLineNumber") ?? changeEntries[0]?.newAnchorLineNumber ?? hunk.newStart;
    const changedNewEnd = maxEntryLine(addedEntries, "newLineNumber") ?? changedNewStart;
    const originalLines = deletedEntries.map((entry, index) => {
      const pairedEntry = addedEntries[index] ?? null;
      return {
        index,
        lineNumber: entry.oldLineNumber ?? entry.oldAnchorLineNumber,
        content: entry.content,
        tokens: buildProjectOriginalLineTokens(entry.content, pairedEntry?.content ?? null, language),
        tone: pairedEntry ? ("modified" as const) : ("deleted" as const),
      };
    });
    const patchEntries = projectPatchEntriesForRange(entries, range);
    const oldStart = patchStartLine(patchEntries, "oldLineNumber", range, entries, "old");
    const newStart = patchStartLine(patchEntries, "newLineNumber", range, entries, "new");
    const oldLines = patchEntries.filter((entry) => entry.prefix !== "+").length;
    const newLines = patchEntries.filter((entry) => entry.prefix !== "-").length;

    return {
      blockIndex,
      startEntryIndex: range.startEntryIndex,
      endEntryIndex: range.endEntryIndex,
      entries: changeEntries,
      patch: buildProjectHunkPatch(fileHeader, oldStart, oldLines, newStart, newLines, patchEntries),
      changedOldStart,
      changedOldEnd,
      changedNewStart,
      changedNewEnd,
      oldStart,
      oldLines,
      newStart,
      newLines,
      addedLines: addedEntries.length,
      deletedLines: deletedEntries.length,
      originalLines,
    };
  }

  function projectPatchEntriesForRange(
    entries: ProjectHunkEntry[],
    range: { startEntryIndex: number; endEntryIndex: number },
  ) {
    let start = range.startEntryIndex;
    let end = range.endEntryIndex;
    let beforeContext = 0;
    let afterContext = 0;

    while (
      start > 0 &&
      beforeContext < PROJECT_HUNK_PATCH_CONTEXT_LINES &&
      entries[start - 1]?.prefix === " "
    ) {
      start -= 1;
      beforeContext += 1;
    }

    while (
      end < entries.length - 1 &&
      afterContext < PROJECT_HUNK_PATCH_CONTEXT_LINES &&
      entries[end + 1]?.prefix === " "
    ) {
      end += 1;
      afterContext += 1;
    }

    return entries.slice(start, end + 1);
  }

  function patchStartLine(
    entries: ProjectHunkEntry[],
    lineKey: "oldLineNumber" | "newLineNumber",
    range: { startEntryIndex: number; endEntryIndex: number },
    allEntries: ProjectHunkEntry[],
    side: "old" | "new",
  ) {
    const directLine = minEntryLine(entries, lineKey);
    if (directLine !== null) return directLine;

    const firstChange = allEntries[range.startEntryIndex];
    if (!firstChange) return 0;
    if (side === "old") return Math.max(0, firstChange.oldAnchorLineNumber - 1);
    return Math.max(0, firstChange.newAnchorLineNumber - 1);
  }

  function buildProjectHunkPatch(
    fileHeader: string[],
    oldStart: number,
    oldLines: number,
    newStart: number,
    newLines: number,
    entries: ProjectHunkEntry[],
  ) {
    const header = `@@ -${formatProjectPatchRange(oldStart, oldLines)} +${formatProjectPatchRange(newStart, newLines)} @@`;
    const patchLines = [...fileHeader, header, ...entries.map((entry) => `${entry.prefix}${entry.content}`)];
    return `${patchLines.join("\n")}\n`;
  }

  function formatProjectPatchRange(start: number, lines: number) {
    return lines === 1 ? `${start}` : `${start},${lines}`;
  }

  function minEntryLine(entries: ProjectHunkEntry[], key: "oldLineNumber" | "newLineNumber") {
    const lines = entries.map((entry) => entry[key]).filter((line): line is number => line !== null);
    return lines.length ? Math.min(...lines) : null;
  }

  function maxEntryLine(entries: ProjectHunkEntry[], key: "oldLineNumber" | "newLineNumber") {
    const lines = entries.map((entry) => entry[key]).filter((line): line is number => line !== null);
    return lines.length ? Math.max(...lines) : null;
  }

  function buildProjectOriginalLineTokens(
    original: string,
    current: string | null,
    language: string,
  ): ProjectOriginalLineToken[] {
    const tokens = tokenizeProjectLine(original || " ", language);
    const range = projectInlineDiffRange(original, current);
    const highlighted = splitProjectOriginalTokens(tokens, range.start, range.end);

    if (range.insertAt !== null) {
      const insertAt = Math.min(range.insertAt, original.length);
      const result: ProjectOriginalLineToken[] = [];
      let offset = 0;
      let inserted = false;

      for (const token of highlighted) {
        const tokenEnd = offset + token.text.length;
        if (!inserted && insertAt <= tokenEnd) {
          const splitAt = Math.max(0, insertAt - offset);
          if (splitAt > 0) {
            result.push({ ...token, text: token.text.slice(0, splitAt) });
          }
          result.push({ text: " ", diff: true, insertMarker: true });
          if (splitAt < token.text.length) {
            result.push({ ...token, text: token.text.slice(splitAt) });
          }
          inserted = true;
        } else {
          result.push(token);
        }
        offset = tokenEnd;
      }

      if (!inserted) {
        result.push({ text: " ", diff: true, insertMarker: true });
      }
      return result;
    }

    return highlighted;
  }

  function projectInlineDiffRange(original: string, current: string | null) {
    if (current === null) {
      return { start: 0, end: Math.max(1, original.length), insertAt: null as number | null };
    }
    if (original === current) {
      return { start: 0, end: 0, insertAt: null as number | null };
    }

    let prefixLength = 0;
    const maxPrefixLength = Math.min(original.length, current.length);
    while (prefixLength < maxPrefixLength && original[prefixLength] === current[prefixLength]) {
      prefixLength += 1;
    }

    let suffixLength = 0;
    const maxSuffixLength = Math.min(original.length - prefixLength, current.length - prefixLength);
    while (
      suffixLength < maxSuffixLength &&
      original[original.length - 1 - suffixLength] === current[current.length - 1 - suffixLength]
    ) {
      suffixLength += 1;
    }

    const start = prefixLength;
    const end = original.length - suffixLength;
    return end > start
      ? { start, end, insertAt: null as number | null }
      : { start, end, insertAt: start };
  }

  function splitProjectOriginalTokens(tokens: ProjectCodeToken[], start: number, end: number) {
    if (end <= start) return tokens;

    const result: ProjectOriginalLineToken[] = [];
    let offset = 0;
    for (const token of tokens) {
      const tokenStart = offset;
      const tokenEnd = offset + token.text.length;
      const diffStart = Math.max(tokenStart, start);
      const diffEnd = Math.min(tokenEnd, end);

      if (diffStart <= tokenStart && diffEnd >= tokenEnd) {
        result.push({ ...token, diff: true });
      } else if (diffEnd > diffStart) {
        if (diffStart > tokenStart) {
          result.push({ ...token, text: token.text.slice(0, diffStart - tokenStart) });
        }
        result.push({
          ...token,
          text: token.text.slice(diffStart - tokenStart, diffEnd - tokenStart),
          diff: true,
        });
        if (diffEnd < tokenEnd) {
          result.push({ ...token, text: token.text.slice(diffEnd - tokenStart) });
        }
      } else {
        result.push(token);
      }

      offset = tokenEnd;
    }

    return result.length ? result : [{ text: " ", diff: true }];
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
    const currentRange = formatProjectLineRange(hunk.changedNewStart, hunk.changedNewEnd);
    const originalRange =
      hunk.changedOldStart === null || hunk.changedOldEnd === null
        ? "新增"
        : formatProjectLineRange(hunk.changedOldStart, hunk.changedOldEnd);
    return `当前 ${currentRange}，原本 ${originalRange}`;
  }

  function formatProjectLineRange(start: number, end: number) {
    return start === end ? `${start}` : `${start}-${end}`;
  }

  function toggleProjectEditorHunk(id: string) {
    expandedProjectHunkIndex.value = expandedProjectHunkIndex.value === id ? null : id;
  }

  function updateChangeFileListViewport() {
    const scroller = changeFileListScroller.value;
    changeFileListViewportHeight.value = scroller?.clientHeight || CHANGE_FILE_DEFAULT_VIEWPORT_HEIGHT;
    changeFileListScrollTop.value = scroller?.scrollTop ?? 0;
  }

  function syncChangeFileListViewport(event?: Event) {
    const scroller = (event?.currentTarget as HTMLElement | null) ?? changeFileListScroller.value;
    changeFileListViewportHeight.value = scroller?.clientHeight || CHANGE_FILE_DEFAULT_VIEWPORT_HEIGHT;
    changeFileListScrollTop.value = scroller?.scrollTop ?? 0;
  }

  function scrollChangeFilePathIntoView(path: string, side: ChangeSide) {
    nextTick(() => {
      const scroller = changeFileListScroller.value;
      if (!scroller) return;

      const rowIndex = flatChangeFileRows.value.findIndex(
        (row) => row.kind === "file" && row.group.side === side && row.file.path === path,
      );
      if (rowIndex < 0) return;

      const rowTop = rowIndex * CHANGE_FILE_ROW_HEIGHT;
      const rowBottom = rowTop + CHANGE_FILE_ROW_HEIGHT;
      const viewportTop = scroller.scrollTop;
      const viewportBottom = viewportTop + scroller.clientHeight;

      if (rowTop < viewportTop) {
        scroller.scrollTop = rowTop;
      } else if (rowBottom > viewportBottom) {
        scroller.scrollTop = Math.max(0, rowBottom - scroller.clientHeight);
      }
      updateChangeFileListViewport();
    }).catch(() => undefined);
  }

  function updateProjectEditorViewport() {
    projectEditorViewportHeight.value =
      projectEditorTextarea.value?.clientHeight ?? PROJECT_EDITOR_DEFAULT_VIEWPORT_HEIGHT;
  }

  function syncProjectEditorScroll(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    projectEditorScrollTop.value = target.scrollTop;
    projectEditorScrollLeft.value = target.scrollLeft;
    projectEditorViewportHeight.value = target.clientHeight;
  }

  async function discardProjectEditorHunk(hunk: ProjectEditorHunkView) {
    if (project.editorDirty && !window.confirm("当前文件有未保存编辑，撤回此块会放弃未保存内容并还原 Git 原本内容。继续？")) {
      return;
    }
    await project.discardPatch(hunk.patch);
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

  function promptProjectName(
    title: string,
    defaultValue: string,
    validate: (value: string) => string,
    placeholder = "输入名称",
  ) {
    return new Promise<string | null>((resolve) => {
      projectNameDialog.value = {
        title,
        value: defaultValue,
        placeholder,
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

  function changeConflictGroupKey(group: ChangeFileGroup) {
    return `${group.key}:conflicts`;
  }

  function changeFileGroupCount(group: ChangeFileGroup) {
    return group.files.length + group.conflictFiles.length;
  }

  function eachChangeFile(collection: ChangeFileCollection, callback: (file: ChangedFile) => void) {
    if (Array.isArray(collection)) {
      collection.forEach(callback);
      return;
    }
    collection.files.forEach(callback);
    collection.conflictFiles.forEach(callback);
  }

  function collectChangeFilePaths(collection: ChangeFileCollection) {
    const paths: string[] = [];
    eachChangeFile(collection, (file) => paths.push(file.path));
    return paths;
  }

  function toggleChangeFileGroup(key: string) {
    expandedChangeFileGroups.value = {
      ...expandedChangeFileGroups.value,
      [key]: !isChangeFileGroupExpanded(key),
    };
  }

  function isChangeFileGroupSelected(collection: ChangeFileCollection) {
    const selected = selectedChangePathSet.value;
    let total = 0;
    let selectedCount = 0;
    eachChangeFile(collection, (file) => {
      total += 1;
      if (selected.has(file.path)) selectedCount += 1;
    });
    return total > 0 && selectedCount === total;
  }

  function isChangeFileGroupPartiallySelected(collection: ChangeFileCollection) {
    const selected = selectedChangePathSet.value;
    let total = 0;
    let selectedCount = 0;
    eachChangeFile(collection, (file) => {
      total += 1;
      if (selected.has(file.path)) selectedCount += 1;
    });
    return selectedCount > 0 && selectedCount < total;
  }

  function toggleChangeFileGroupSelection(collection: ChangeFileCollection) {
    const groupPaths = collectChangeFilePaths(collection);
    const groupPathSet = new Set(groupPaths);

    if (isChangeFileGroupSelected(collection)) {
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

  function isConfiguredRemoteName(name: string | undefined) {
    return Boolean(name && repos.current?.remotes.some((item) => item.name === name));
  }

  function nextAvailableRemoteName(baseName = "origin") {
    const base = baseName.trim() || "origin";
    if (!isConfiguredRemoteName(base)) return base;
    for (let index = 2; index < 1000; index += 1) {
      const candidate = `${base}-${index}`;
      if (!isConfiguredRemoteName(candidate)) return candidate;
    }
    return base;
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

  function openLogRemoteDirectoryContextMenu(remoteName: string | undefined, event: MouseEvent) {
    closeContextMenus();
    logRefContextMenu.value = {
      kind: "remoteDirectory",
      remoteName,
      ...contextMenuPoint(event, 270, isConfiguredRemoteName(remoteName) ? 118 : 78),
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
    return (
      (logRefContextMenu.value?.kind === "local" || logRefContextMenu.value?.kind === "remote") &&
      logRefContextMenu.value.branch.fullName === branchItem.fullName
    );
  }

  function isLogRemoteDirectoryContextTarget(remoteName?: string) {
    return (
      logRefContextMenu.value?.kind === "remoteDirectory" &&
      (logRefContextMenu.value.remoteName ?? "") === (remoteName ?? "")
    );
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
    await runUiAction("advanced.checkoutDetached", async () => {
      await advanced.checkoutDetached(tag.name);
      await loadCurrentRepository();
    });
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
      await runUiAction(branchActionKey("create"), async () => {
        await branches.create(name, true, startPoint);
        await loadCurrentRepository();
      });
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
    await runUiAction(branchActionKey("rename", branchItem.fullName), async () => {
      await branches.rename(branchItem.name, newName);
      if (history.branchFilter === branchItem.name) {
        history.branchFilter = newName;
      }
      await loadCurrentRepository();
    });
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
        await runUiAction(branchActionKey("delete", branchItem.fullName), async () => {
          if (branchItem.branchType === "remote") {
            await branches.deleteRemote(branchItem.name);
          } else {
            await branches.delete(branchItem.name, false);
          }
          await loadCurrentRepository();
        });
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
      await runUiAction(branchActionKey("tag.delete", tag.name), async () => {
        await branches.deleteTag(tag.name);
        await loadCurrentRepository();
      });
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
    await runUiAction("operation.merge", async () => {
      await operations.merge();
      await reloadAfterGitOperation();
    });
  }

  async function rebaseCurrentOntoLogRef(menu: LogRefContextMenu | null) {
    const target = logRefContextBranchItem(menu)?.name ?? "";
    if (!target || !canMergeOrRebaseLogRefContext(menu)) return;
    if (!window.confirm(`将当前分支变基到 ${target}？`)) return;
    closeLogRefContextMenu();
    operations.rebaseTarget = target;
    await runUiAction("operation.rebase", async () => {
      await operations.rebase();
      await reloadAfterGitOperation();
    });
  }

  async function setCurrentBranchUpstreamFromContext(menu: LogRefContextMenu | null) {
    const target = logRefContextBranchItem(menu)?.name ?? "";
    const current = branch.value?.currentBranch;
    if (!target || !current || menu?.kind !== "remote") return;
    if (!window.confirm(`将 ${current} 的上游设置为 ${target}？`)) return;
    closeLogRefContextMenu();
    await runUiAction(branchActionKey("upstream.set", current), async () => {
      await branches.setUpstream(current, target);
      await loadCurrentRepository();
    });
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

  function openAddRemoteDialog(remoteName?: string) {
    closeLogRefContextMenu();
    addRemoteDialog.value = {
      name: nextAvailableRemoteName(remoteName || "origin"),
      url: "",
      fetchAfterSave: true,
      loading: false,
      error: "",
    };
  }

  function cancelAddRemoteDialog() {
    if (addRemoteDialog.value?.loading) return;
    addRemoteDialog.value = null;
  }

  async function submitAddRemoteDialog() {
    const dialog = addRemoteDialog.value;
    if (!dialog || dialog.loading) return;

    const name = dialog.name.trim();
    const url = dialog.url.trim();
    const validationError = validateRemoteName(name) || validateRemoteUrl(url);
    if (validationError) {
      dialog.error = validationError;
      return;
    }

    dialog.loading = true;
    dialog.error = "";
    try {
      remote.selectedRemote = name;
      remote.remoteNameDraft = name;
      remote.remoteUrlDraft = url;
      remote.remotePushUrlDraft = "";
      remote.syncTargetFromBranch();
      await saveRemoteConfig();
      const shouldFetch = dialog.fetchAfterSave;
      addRemoteDialog.value = null;
      if (shouldFetch) {
        await executeRemoteAction("fetch");
      }
    } catch (error) {
      if (addRemoteDialog.value === dialog) {
        dialog.error = translateGitError(error);
      }
    } finally {
      dialog.loading = false;
    }
  }

  async function addRemoteFromLogRefContext(menu: LogRefContextMenu | null) {
    if (menu?.kind !== "remoteDirectory") return;
    openAddRemoteDialog(menu.remoteName);
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

  function validateRemoteName(value: string) {
    const name = value.trim();
    if (!name) return "请输入远程名称";
    if (
      name.startsWith("-") ||
      name.endsWith("/") ||
      name.endsWith(".") ||
      name.includes("..") ||
      /[\/\s~^:?*\[\\\]]/.test(name)
    ) {
      return "远程名称包含 Git 不支持的字符";
    }
    if (isConfiguredRemoteName(name)) {
      return "远程仓库已存在";
    }
    return "";
  }

  function validateRemoteUrl(value: string) {
    return value.trim() ? "" : "请输入远程地址";
  }

  async function createLogBranchFromHead() {
    logRefPanelCollapsed.value = false;
    const name = await promptProjectName("新建分支", "", validateBranchName);
    if (!name) return;
    await runUiAction(branchActionKey("create"), async () => {
      await branches.create(name, true);
      await loadCurrentRepository();
      history.branchFilter = name;
      await history.refresh().catch(() => undefined);
    });
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

  function formatSubmitConfirmFileStatus(status = "") {
    return status.length === 1 ? formatCommitFileStatusCode(status) : formatStatusKind(status);
  }

  function buildSideBySideDiffRows(response: DiffResponse | null, language: string): SideBySideDiffRow[] {
    if (!response) return [];

    const rows: SideBySideDiffRow[] = [];
    let rowIndex = 0;
    const anchoredHunks = new Set<number>();
    const hasCompleteText = hasCompleteDiffText(response);
    if (!response.text.trim() && !hasCompleteText) return [];

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
          pushRow(emptyCell(), diffCell(null, formatUnifiedDiffMetaLine(line), "meta"), "meta", hunkIndex);
        }
      }

      flushChanges();
      return { oldLine: Math.max(oldLine, 1), newLine: Math.max(newLine, 1) };
    };

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

  function hasDisplayableDiffContent(response: DiffResponse | null | undefined) {
    return Boolean(response?.text?.trim()) || hasCompleteDiffText(response);
  }

  function hasCompleteDiffText(response: DiffResponse | null | undefined) {
    return (
      response?.oldText !== undefined &&
      response?.newText !== undefined &&
      (response.oldText !== null || response.newText !== null)
    );
  }

  function formatUnifiedDiffMetaLine(line: string) {
    if (line.startsWith("\\ No newline")) return "文件末尾缺少换行符";
    return line;
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
    await runUiAction(branchActionKey("checkout", branch.fullName), async () => {
      if (branch.branchType === "remote") {
        await branches.checkoutRemote(branch.name);
      } else {
        await branches.checkout(branch.name);
      }
      await loadCurrentRepository();
    });
  }

  async function createBranchFromHead() {
    const name = newBranchName.value.trim();
    if (!name) return;
    await runUiAction(branchActionKey("create"), async () => {
      await branches.create(name, true);
      newBranchName.value = "";
      await loadCurrentRepository();
    });
  }

  async function deleteLocalBranch(branch: BranchInfo) {
    if (branch.current) return;
    if (!window.confirm(`删除本地分支 ${branch.name}？`)) return;
    await runUiAction(branchActionKey("delete", branch.fullName), async () => {
      await branches.delete(branch.name, false);
      await loadCurrentRepository();
    });
  }

  async function deleteRemoteBranchItem(branch: BranchInfo) {
    if (branch.branchType !== "remote") return;
    if (!window.confirm(`删除远程分支 ${branch.name}？这会推送删除到远程仓库。`)) return;
    await runUiAction(branchActionKey("delete", branch.fullName), async () => {
      await branches.deleteRemote(branch.name);
      await loadCurrentRepository();
    });
  }

  async function createTagFromInput() {
    const name = newTagName.value.trim();
    if (!name) return;
    await runUiAction(branchActionKey("tag.create", name), async () => {
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
    });
  }

  async function deleteLocalTag(tag: TagInfo) {
    if (!window.confirm(`删除本地标签 ${tag.name}？`)) return;
    await runUiAction(branchActionKey("tag.delete", tag.name), async () => {
      await branches.deleteTag(tag.name);
      await loadCurrentRepository();
    });
  }

  async function pushSelectedTag(tag: TagInfo) {
    await runUiAction(branchActionKey("tag.push", tag.name), async () => {
      await branches.pushTag(tag.name, remote.selectedRemote || undefined);
      await loadCurrentRepository();
    });
  }

  async function deleteSelectedRemoteTag(tag: TagInfo) {
    const remoteName = remote.selectedRemote || "origin";
    if (!window.confirm(`删除 ${remoteName} 上的标签 ${tag.name}？`)) return;
    await runUiAction(branchActionKey("tag.deleteRemote", tag.name), async () => {
      await branches.deleteRemoteTag(tag.name, remote.selectedRemote || undefined);
      await loadCurrentRepository();
    });
  }

  async function setSelectedUpstream() {
    if (!branches.selectedLocalBranch || !branches.upstreamTarget) return;
    await runUiAction(branchActionKey("upstream.set", branches.selectedLocalBranch), async () => {
      await branches.setUpstream(branches.selectedLocalBranch, branches.upstreamTarget);
      await loadCurrentRepository();
    });
  }

  async function unsetSelectedUpstream() {
    if (!branches.selectedLocalBranch) return;
    await runUiAction(branchActionKey("upstream.unset", branches.selectedLocalBranch), async () => {
      await branches.setUpstream(branches.selectedLocalBranch);
      await loadCurrentRepository();
    });
  }

  async function mergeSelectedTarget() {
    if (!operations.mergeTarget) return;
    if (!window.confirm(`将 ${operations.mergeTarget} 合并到当前分支？`)) return;
    await runUiAction("operation.merge", async () => {
      await operations.merge();
      await reloadAfterGitOperation();
    });
  }

  async function rebaseOntoSelectedTarget() {
    if (!operations.rebaseTarget) return;
    if (!window.confirm(`将当前分支变基到 ${operations.rebaseTarget}？`)) return;
    await runUiAction("operation.rebase", async () => {
      await operations.rebase();
      await reloadAfterGitOperation();
    });
  }

  async function rebaseWithAdvancedOptions() {
    if (!operations.rebaseTarget && !operations.rebaseRoot) return;
    if (!window.confirm("按当前高级参数执行变基？")) return;
    await runUiAction("operation.rebaseAdvanced", async () => {
      await operations.rebaseWithAdvancedOptions();
      await reloadAfterGitOperation();
    });
  }

  async function runOperationControl(action: "continue" | "abort" | "skip") {
    if (action === "abort") {
      const operationLabel = formatOperationName(operations.activeOperation);
      const message =
        operations.activeOperation === "merge"
          ? "中止当前合并？未应用的冲突处理结果会被丢弃，并恢复到合并前状态。"
          : `终止当前${operationLabel}操作？`;
      if (!window.confirm(message)) return;
    }
    await runUiAction(`operation.${action}`, async () => {
      await operations.control(action);
      await reloadAfterGitOperation();
    });
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

  function mergeEditorScrollTargets() {
    return [mergeCurrentScroller.value, mergeResultTextarea.value, mergeIncomingScroller.value].filter(
      (target): target is HTMLElement => Boolean(target),
    );
  }

  function syncMergeResultGutter(scrollTop: number) {
    if (mergeResultGutter.value) {
      mergeResultGutter.value.scrollTop = scrollTop;
    }
  }

  function syncMergeSourceGutters(scrollTop: number) {
    for (const gutter of [mergeCurrentGutter.value, mergeIncomingGutter.value]) {
      if (gutter && gutter.scrollTop !== scrollTop) {
        gutter.scrollTop = scrollTop;
      }
    }
  }

  function updateMergeResultRenderScroll(fallbackTop = 0, fallbackLeft = 0) {
    mergeResultScrollTop.value = mergeResultTextarea.value?.scrollTop ?? fallbackTop;
    mergeResultScrollLeft.value = mergeResultTextarea.value?.scrollLeft ?? fallbackLeft;
    syncMergeResultGutter(mergeResultScrollTop.value);
    syncMergeSourceGutters(mergeResultScrollTop.value);
  }

  function setMergeEditorScroll(top: number, left: number, source: HTMLElement | null = null) {
    for (const target of mergeEditorScrollTargets()) {
      if (target === source) continue;
      if (target.scrollTop === top && target.scrollLeft === left) continue;
      syncingMergeEditorScroll.add(target);
      target.scrollTop = top;
      target.scrollLeft = left;
    }
    updateMergeResultRenderScroll(top, left);
    syncMergeSourceGutters(top);
  }

  function syncMergeEditorScroll(event: Event) {
    const source = event.currentTarget as HTMLElement | null;
    if (!source) return;
    if (syncingMergeEditorScroll.has(source)) {
      syncingMergeEditorScroll.delete(source);
      if (source === mergeResultTextarea.value) {
        updateMergeResultRenderScroll();
      } else {
        syncMergeSourceGutters(source.scrollTop);
      }
      return;
    }

    setMergeEditorScroll(source.scrollTop, source.scrollLeft, source);
  }

  async function selectConflict(path: string) {
    const file = conflictedFiles.value.find((item) => item.path === path);
    if (file) {
      selectFile(file, "unstaged", { openConflict: false });
    } else {
      changes.selectedFile = path;
      changes.selectedSide = "unstaged";
      settings.setSide("unstaged");
    }
    scrollChangeFilePathIntoView(path, "unstaged");
    workbenchMode.value = "changes";
    await operations.loadConflict(path);
  }

  async function saveConflictResult(markResolved = false) {
    await operations.saveResult(markResolved);
    if (markResolved) {
      await reloadAfterGitOperation();
    }
  }

  function normalizeMergeDisplaySide(side: string | null | undefined, fallback: MergeDisplaySide): MergeDisplaySide {
    return side === "ours" || side === "theirs" ? side : fallback;
  }

  function oppositeMergeDisplaySide(side: MergeDisplaySide): MergeDisplaySide {
    return side === "ours" ? "theirs" : "ours";
  }

  function hasGitConflictMarkers(content: string) {
    return content
      .replace(/\r\n/g, "\n")
      .replace(/\r/g, "\n")
      .split("\n")
      .some(
        (line) =>
          line.startsWith("<<<<<<< ") ||
          line.startsWith("||||||| ") ||
          line === "=======" ||
          line.startsWith(">>>>>>> "),
      );
  }

  function mergeConflictSideContent(conflict: ConflictDetails | null, side: MergeDisplaySide) {
    if (!conflict) return "";
    return side === "ours" ? (conflict.ours ?? "") : (conflict.theirs ?? "");
  }

  function acceptConflictSide(side: MergeDisplaySide) {
    operations.useAllConflictBlocks(side);
  }

  function acceptConflictBlock(index: number | null, side: MergeDisplaySide) {
    if (index === null) return;
    operations.replaceResultBlock(index, side);
  }

  function mergeConflictBlockSelection(index: number | null): MergeConflictSelection | null {
    if (index === null) return null;
    return (operations.resultBlockSelections[index] as MergeConflictSelection | undefined) ?? null;
  }

  function shouldAppendConflictBlock(index: number | null, side: MergeDisplaySide) {
    const selection = mergeConflictBlockSelection(index);
    return Boolean(selection && selection !== "combined" && selection !== side);
  }

  function mergeConflictActionTitle(index: number | null, side: MergeDisplaySide, sideLabel: string) {
    return shouldAppendConflictBlock(index, side) ? `追加${sideLabel}块到结果后` : `接受${sideLabel}块到结果`;
  }

  function applyConflictBlock(index: number | null, side: MergeDisplaySide) {
    if (index === null) return;
    if (shouldAppendConflictBlock(index, side)) {
      operations.appendResultBlock(index, side);
      return;
    }
    acceptConflictBlock(index, side);
  }

  function mergeConflictLineClasses(line: MergeCodeLine) {
    return {
      conflict: line.conflict,
      "conflict-start": line.conflictStart,
      "conflict-end": line.conflictEnd,
      "conflict-ours": line.conflictSide === "ours",
      "conflict-base": line.conflictSide === "base",
      "conflict-theirs": line.conflictSide === "theirs",
      "auto-merge": line.autoMerge,
      "auto-merge-start": line.autoMergeStart,
      "auto-merge-end": line.autoMergeEnd,
    };
  }

  async function resetConflictResultDraft() {
    await operations.restoreInitialResult();
  }

  function splitMergeLines(content: string) {
    const normalized = content.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
    const lines = normalized.split("\n");
    if (lines.length > 1 && lines[lines.length - 1] === "") {
      lines.pop();
    }
    return lines.length ? lines : [""];
  }

  function findLineRange(lines: string[], needle: string[], startAt: number) {
    if (needle.length === 0) return -1;
    for (let index = startAt; index <= lines.length - needle.length; index += 1) {
      const matches = needle.every((line, offset) => lines[index + offset] === line);
      if (matches) return index;
    }
    return -1;
  }

  function sameMergeLines(left: string[], right: string[]) {
    return left.length === right.length && left.every((line, index) => line === right[index]);
  }

  function firstLinePositionAtOrAfter(positions: number[] | undefined, startAt: number) {
    if (!positions) return null;
    let low = 0;
    let high = positions.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (positions[mid] < startAt) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }
    return positions[low] ?? null;
  }

  function buildSequentialLineAnchors(left: string[], right: string[]) {
    const rightPositions = new Map<string, number[]>();
    right.forEach((line, index) => {
      const positions = rightPositions.get(line) ?? [];
      positions.push(index);
      rightPositions.set(line, positions);
    });

    const anchors: [number, number][] = [];
    let rightFrom = 0;
    left.forEach((line, leftIndex) => {
      const rightIndex = firstLinePositionAtOrAfter(rightPositions.get(line), rightFrom);
      if (rightIndex === null) return;
      anchors.push([leftIndex, rightIndex]);
      rightFrom = rightIndex + 1;
    });
    return anchors;
  }

  function buildLineDiffRanges(left: string[], right: string[]) {
    const ranges: MergeLineDiffRange[] = [];
    let leftStart = 0;
    let rightStart = 0;

    for (const [leftAnchor, rightAnchor] of [
      ...buildSequentialLineAnchors(left, right),
      [left.length, right.length] as [number, number],
    ]) {
      if (leftStart < leftAnchor || rightStart < rightAnchor) {
        const leftPart = left.slice(leftStart, leftAnchor);
        const rightPart = right.slice(rightStart, rightAnchor);
        if (!sameMergeLines(leftPart, rightPart)) {
          ranges.push({
            leftStartLine: leftStart + 1,
            leftEndLine: leftAnchor,
            rightStartLine: rightStart + 1,
            rightEndLine: rightAnchor,
          });
        }
      }

      leftStart = leftAnchor + 1;
      rightStart = rightAnchor + 1;
    }

    return ranges;
  }

  function hasMergeLineRange(range: MergeConflictLineRange) {
    return range.startLine <= range.endLine;
  }

  function lineRangeIntersects(left: MergeConflictLineRange, right: MergeConflictLineRange) {
    return left.startLine <= right.endLine && right.startLine <= left.endLine;
  }

  function lineRangeIntersectsAny(range: MergeConflictLineRange, exclusions: MergeConflictLineRange[]) {
    return exclusions.some((exclusion) => hasMergeLineRange(exclusion) && lineRangeIntersects(range, exclusion));
  }

  function subtractMergeLineRanges(range: MergeConflictLineRange, exclusions: MergeConflictLineRange[]) {
    if (!hasMergeLineRange(range)) return [];

    let segments = [range];
    const sortedExclusions = exclusions.filter(hasMergeLineRange).sort((left, right) => left.startLine - right.startLine);

    for (const exclusion of sortedExclusions) {
      segments = segments.flatMap((segment) => {
        if (!lineRangeIntersects(segment, exclusion)) return [segment];

        const nextSegments: MergeConflictLineRange[] = [];
        if (segment.startLine < exclusion.startLine) {
          nextSegments.push({
            startLine: segment.startLine,
            endLine: exclusion.startLine - 1,
          });
        }
        if (exclusion.endLine < segment.endLine) {
          nextSegments.push({
            startLine: exclusion.endLine + 1,
            endLine: segment.endLine,
          });
        }
        return nextSegments;
      });
    }

    return segments;
  }

  function mergeLinesInRange(lines: string[], range: MergeConflictLineRange) {
    if (!hasMergeLineRange(range)) return [];
    return lines.slice(range.startLine - 1, range.endLine);
  }

  function addMergeAutoMergeSegments(
    targetRanges: MergeConflictLineRange[],
    resultSnippets: MergeAutoMergeSnippet[],
    lines: string[],
    range: MergeConflictLineRange,
    conflictRanges: MergeConflictLineRange[],
  ) {
    for (const segment of subtractMergeLineRanges(range, conflictRanges)) {
      const segmentLines = mergeLinesInRange(lines, segment);
      if (segmentLines.length === 0) continue;

      targetRanges.push(segment);
      if (segmentLines.some((line) => line.length > 0)) {
        resultSnippets.push({ lines: segmentLines });
      }
    }
  }

  function buildMergeAutoMergeDiff(
    currentContent: string,
    incomingContent: string,
    currentConflictRanges: MergeConflictLineRange[],
    incomingConflictRanges: MergeConflictLineRange[],
  ) {
    const currentLines = splitMergeLines(currentContent);
    const incomingLines = splitMergeLines(incomingContent);
    const currentRanges: MergeConflictLineRange[] = [];
    const incomingRanges: MergeConflictLineRange[] = [];
    const resultSnippets: MergeAutoMergeSnippet[] = [];

    for (const range of buildLineDiffRanges(currentLines, incomingLines)) {
      addMergeAutoMergeSegments(
        currentRanges,
        resultSnippets,
        currentLines,
        { startLine: range.leftStartLine, endLine: range.leftEndLine },
        currentConflictRanges,
      );
      addMergeAutoMergeSegments(
        incomingRanges,
        resultSnippets,
        incomingLines,
        { startLine: range.rightStartLine, endLine: range.rightEndLine },
        incomingConflictRanges,
      );
    }

    return { currentRanges, incomingRanges, resultSnippets };
  }

  function buildMergeConflictSnippetRanges(lines: string[], snippets: MergeConflictSnippet[]) {
    const ranges: MergeConflictLineRange[] = [];
    let searchFrom = 0;

    for (const snippet of snippets) {
      const snippetLines = splitMergeLines(snippet.content);
      if (!snippetLines.some((line) => line.length > 0)) continue;

      const foundAt = findLineRange(lines, snippetLines, searchFrom);
      if (foundAt < 0) continue;

      ranges.push({
        startLine: foundAt + 1,
        endLine: foundAt + snippetLines.length,
      });
      searchFrom = foundAt + snippetLines.length;
    }

    return ranges;
  }

  function findAvailableLineRange(
    lines: string[],
    needle: string[],
    occupiedRanges: MergeConflictLineRange[],
    startAt = 0,
  ) {
    if (needle.length === 0) return -1;

    for (let index = startAt; index <= lines.length - needle.length; index += 1) {
      const candidateRange = {
        startLine: index + 1,
        endLine: index + needle.length,
      };
      if (lineRangeIntersectsAny(candidateRange, occupiedRanges)) continue;

      const matches = needle.every((line, offset) => lines[index + offset] === line);
      if (matches) return index;
    }

    return -1;
  }

  function buildMergeResultAutoMergeRanges(
    resultContent: string,
    snippets: MergeAutoMergeSnippet[],
    conflictSnippets: MergeConflictSnippet[],
  ) {
    const lines = splitMergeLines(resultContent);
    const ranges: MergeConflictLineRange[] = [];
    const occupiedRanges = buildMergeConflictSnippetRanges(lines, conflictSnippets);
    let searchFrom = 0;

    for (const snippet of snippets) {
      if (!snippet.lines.some((line) => line.length > 0)) continue;

      const foundAt = findAvailableLineRange(lines, snippet.lines, occupiedRanges, searchFrom);
      if (foundAt < 0) continue;

      const range = {
        startLine: foundAt + 1,
        endLine: foundAt + snippet.lines.length,
      };
      ranges.push(range);
      occupiedRanges.push(range);
      searchFrom = foundAt + snippet.lines.length;
    }

    return ranges;
  }

  function mergeConflictBlockContent(block: ConflictBlock, side: MergeConflictSide) {
    if (side === "base") return block.base ?? "";
    return block[side];
  }

  function mergeConflictSnippet(block: ConflictBlock, side: MergeConflictSide): MergeConflictSnippet {
    return {
      index: block.index,
      content: mergeConflictBlockContent(block, side),
      side,
    };
  }

  function mergeConflictBlockSnippets(block: ConflictBlock) {
    const snippets = [mergeConflictSnippet(block, "ours")];
    if (block.base) snippets.push(mergeConflictSnippet(block, "base"));
    snippets.push(mergeConflictSnippet(block, "theirs"));
    return snippets;
  }

  function buildResultMergeConflictSnippets(
    blocks: ConflictBlock[],
    selections: Record<number, MergeConflictSelection>,
    replacements: Record<number, string>,
  ) {
    return blocks.flatMap((block) => {
      const selection = selections[block.index];
      if (selection === "combined") {
        return [];
      }
      if (!selection) {
        return mergeConflictBlockSnippets(block);
      }

      return [
        {
          index: block.index,
          content: replacements[block.index] ?? mergeConflictBlockContent(block, selection),
          side: selection,
        },
      ];
    });
  }

  function buildComparedMergeConflictSnippets(
    blocks: ConflictBlock[],
    side: MergeConflictSide,
    selections: Record<number, MergeConflictSelection>,
  ) {
    return blocks
      .filter((block) => {
        const selection = selections[block.index];
        return selection !== "combined" && selection !== side;
      })
      .map((block) => mergeConflictSnippet(block, side));
  }

  function buildMergeSourceConflictRanges(
    content: string,
    blocks: ConflictBlock[],
    side: MergeConflictSide,
  ) {
    const lines = splitMergeLines(content);
    const ranges = new Map<number, MergeConflictLineRange>();
    let searchFrom = 0;

    for (const block of blocks) {
      const snippetLines = splitMergeLines(mergeConflictBlockContent(block, side));
      if (!snippetLines.some((line) => line.length > 0)) continue;

      const foundAt = findLineRange(lines, snippetLines, searchFrom);
      if (foundAt < 0) continue;

      ranges.set(block.index, {
        startLine: foundAt + 1,
        endLine: foundAt + snippetLines.length,
      });
      searchFrom = foundAt + snippetLines.length;
    }

    return ranges;
  }

  function mergeResultConflictSideRanges(lines: MergeCodeLine[], conflictIndex: number) {
    const ranges: (MergeConflictLineRange & { side: MergeConflictSide })[] = [];
    let activeRange: (MergeConflictLineRange & { side: MergeConflictSide }) | null = null;

    for (const line of lines) {
      if (line.conflictIndex !== conflictIndex || !line.conflictSide) {
        if (activeRange) {
          ranges.push(activeRange);
          activeRange = null;
        }
        continue;
      }

      if (activeRange && activeRange.side === line.conflictSide && activeRange.endLine + 1 === line.number) {
        activeRange.endLine = line.number;
        continue;
      }

      if (activeRange) {
        ranges.push(activeRange);
      }
      activeRange = {
        side: line.conflictSide,
        startLine: line.number,
        endLine: line.number,
      };
    }

    if (activeRange) {
      ranges.push(activeRange);
    }

    return ranges;
  }

  function buildMergeConflictConnections(
    blocks: ConflictBlock[],
    sourceRanges: Map<number, MergeConflictLineRange>,
    resultLines: MergeCodeLine[],
    source: "current" | "incoming",
    sourceSide: MergeDisplaySide,
  ) {
    const connections: MergeConflictConnection[] = [];

    for (const block of blocks) {
      const sourceRange = sourceRanges.get(block.index);
      if (!sourceRange) continue;

      const resultRanges = mergeResultConflictSideRanges(resultLines, block.index).filter(
        (range) => range.side === sourceSide || range.side === "base",
      );
      for (const resultRange of resultRanges) {
        connections.push({
          key: `${source}-${block.index}-${resultRange.side}-${resultRange.startLine}`,
          source,
          side: resultRange.side,
          startLine: sourceRange.startLine,
          endLine: sourceRange.endLine,
          resultStartLine: resultRange.startLine,
          resultEndLine: resultRange.endLine,
        });
      }
    }

    return connections;
  }

  function mergeConnectionLineTop(lineNumber: number) {
    return MERGE_EDITOR_PADDING_TOP + (lineNumber - 1) * MERGE_EDITOR_LINE_HEIGHT - mergeResultScrollTop.value;
  }

  function mergeConnectionLineBottom(lineNumber: number) {
    return mergeConnectionLineTop(lineNumber) + MERGE_EDITOR_LINE_HEIGHT;
  }

  function mergeConflictConnectionBounds(connection: MergeConflictConnection) {
    const sourceTop = mergeConnectionLineTop(connection.startLine);
    const sourceBottom = mergeConnectionLineBottom(connection.endLine);
    const resultTop = mergeConnectionLineTop(connection.resultStartLine);
    const resultBottom = mergeConnectionLineBottom(connection.resultEndLine);
    const top = Math.min(sourceTop, resultTop);
    const bottom = Math.max(sourceBottom, resultBottom);

    return {
      sourceTop,
      sourceBottom,
      resultTop,
      resultBottom,
      top,
      height: Math.max(1, bottom - top),
    };
  }

  function mergeConflictConnectionStyle(connection: MergeConflictConnection) {
    const bounds = mergeConflictConnectionBounds(connection);
    const edgeStyle =
      connection.source === "current"
        ? { right: `${-MERGE_CONNECTION_WIDTH / 2}px` }
        : { left: `${-MERGE_CONNECTION_WIDTH / 2}px` };

    return {
      ...edgeStyle,
      top: `${bounds.top}px`,
      width: `${MERGE_CONNECTION_WIDTH}px`,
      height: `${bounds.height}px`,
    };
  }

  function mergeConflictConnectionViewBox(connection: MergeConflictConnection) {
    return `0 0 ${MERGE_CONNECTION_WIDTH} ${mergeConflictConnectionBounds(connection).height}`;
  }

  function mergeConflictConnectionPath(connection: MergeConflictConnection) {
    const bounds = mergeConflictConnectionBounds(connection);
    const width = MERGE_CONNECTION_WIDTH;
    const sourceTop = sourceConnectionTop(bounds.sourceTop, bounds.top);
    const sourceBottom = sourceConnectionTop(bounds.sourceBottom, bounds.top);
    const resultTop = sourceConnectionTop(bounds.resultTop, bounds.top);
    const resultBottom = sourceConnectionTop(bounds.resultBottom, bounds.top);
    const sourceRadius = mergeConnectionRadius(sourceBottom - sourceTop);
    const resultRadius = mergeConnectionRadius(resultBottom - resultTop);
    const leftControl = width * 0.34;
    const rightControl = width * 0.66;

    if (connection.source === "current") {
      return [
        `M 0 ${sourceTop + sourceRadius}`,
        `Q 0 ${sourceTop} ${sourceRadius} ${sourceTop}`,
        `C ${leftControl} ${sourceTop} ${rightControl} ${resultTop} ${width - resultRadius} ${resultTop}`,
        `Q ${width} ${resultTop} ${width} ${resultTop + resultRadius}`,
        `L ${width} ${resultBottom - resultRadius}`,
        `Q ${width} ${resultBottom} ${width - resultRadius} ${resultBottom}`,
        `C ${rightControl} ${resultBottom} ${leftControl} ${sourceBottom} ${sourceRadius} ${sourceBottom}`,
        `Q 0 ${sourceBottom} 0 ${sourceBottom - sourceRadius}`,
        "Z",
      ].join(" ");
    }

    return [
      `M 0 ${resultTop + resultRadius}`,
      `Q 0 ${resultTop} ${resultRadius} ${resultTop}`,
      `C ${leftControl} ${resultTop} ${rightControl} ${sourceTop} ${width - sourceRadius} ${sourceTop}`,
      `Q ${width} ${sourceTop} ${width} ${sourceTop + sourceRadius}`,
      `L ${width} ${sourceBottom - sourceRadius}`,
      `Q ${width} ${sourceBottom} ${width - sourceRadius} ${sourceBottom}`,
      `C ${rightControl} ${sourceBottom} ${leftControl} ${resultBottom} ${resultRadius} ${resultBottom}`,
      `Q 0 ${resultBottom} 0 ${resultBottom - resultRadius}`,
      "Z",
    ].join(" ");
  }

  function sourceConnectionTop(lineTop: number, boundsTop: number) {
    return lineTop - boundsTop;
  }

  function mergeConnectionRadius(height: number) {
    return Math.min(7, Math.max(2, height / 2));
  }

  function buildMergeCodeLines(
    content: string,
    conflictSnippets: MergeConflictSnippet[],
    autoMergeRanges: MergeConflictLineRange[],
    language: string,
  ): MergeCodeLine[] {
    const lines = splitMergeLines(content);
    const conflictLines = new Map<
      number,
      { index: number; side: MergeConflictSide; start: boolean; end: boolean }
    >();
    const autoMergeLines = new Map<number, { start: boolean; end: boolean }>();
    let searchFrom = 0;

    conflictSnippets
      .map((snippet) => ({
        index: snippet.index,
        side: snippet.side,
        lines: splitMergeLines(snippet.content),
      }))
      .filter((snippet) => snippet.lines.some((line) => line.length > 0))
      .forEach((snippet) => {
        const foundAt = findLineRange(lines, snippet.lines, searchFrom);
        if (foundAt < 0) return;
        snippet.lines.forEach((_, offset) =>
          conflictLines.set(foundAt + offset, {
            index: snippet.index,
            side: snippet.side,
            start: offset === 0,
            end: offset === snippet.lines.length - 1,
          }),
        );
        searchFrom = foundAt + snippet.lines.length;
      });

    autoMergeRanges.filter(hasMergeLineRange).forEach((range) => {
      const startLine = Math.max(1, range.startLine);
      const endLine = Math.min(lines.length, range.endLine);
      for (let lineNumber = startLine; lineNumber <= endLine; lineNumber += 1) {
        const index = lineNumber - 1;
        if (conflictLines.has(index)) continue;
        autoMergeLines.set(index, {
          start: lineNumber === startLine,
          end: lineNumber === endLine,
        });
      }
    });

    return lines.map((line, index) => ({
      id: `${index}-${line}`,
      number: index + 1,
      text: line || " ",
      tokens: tokenizeProjectLine(line || " ", language),
      conflict: conflictLines.has(index),
      conflictIndex: conflictLines.get(index)?.index ?? null,
      conflictSide: conflictLines.get(index)?.side ?? null,
      conflictStart: conflictLines.get(index)?.start ?? false,
      conflictEnd: conflictLines.get(index)?.end ?? false,
      autoMerge: autoMergeLines.has(index),
      autoMergeStart: autoMergeLines.get(index)?.start ?? false,
      autoMergeEnd: autoMergeLines.get(index)?.end ?? false,
    }));
  }

  function firstMergeConflictLine(lines: MergeCodeLine[], conflictIndex: number) {
    const line = lines.find((item) => item.conflictIndex === conflictIndex);
    return line ? line.number - 1 : null;
  }

  function mergeLineScrollTop(target: HTMLElement, lineIndex: number) {
    const style = window.getComputedStyle(target);
    const lineHeight = Number.parseFloat(style.lineHeight) || 18;
    const paddingTop = Number.parseFloat(style.paddingTop) || 0;
    return Math.max(0, paddingTop + lineIndex * lineHeight - target.clientHeight / 2 + lineHeight / 2);
  }

  async function jumpMergeConflict(direction: -1 | 1) {
    const count = mergeConflictCount.value;
    if (count === 0) return;

    activeMergeConflictOrdinal.value = (activeMergeConflictOrdinal.value + direction + count) % count;
    await nextTick();

    const conflictIndex = operations.conflict?.blocks[activeMergeConflictOrdinal.value]?.index;
    const source = mergeResultTextarea.value ?? mergeCurrentScroller.value ?? mergeIncomingScroller.value;
    if (conflictIndex === undefined || !source) return;

    const lineIndex =
      firstMergeConflictLine(mergeResultLines.value, conflictIndex) ??
      firstMergeConflictLine(mergeCurrentLines.value, conflictIndex) ??
      firstMergeConflictLine(mergeIncomingLines.value, conflictIndex);
    if (lineIndex === null) return;

    setMergeEditorScroll(mergeLineScrollTop(source, lineIndex), source.scrollLeft);
  }


  return {
    PROJECT_ROOT_PATH,
    repos,
    advanced,
    branches,
    changes,
    commit,
    diff,
    history,
    operations,
    project,
    remote,
    settings,
    shelveMessage,
    pendingCommitAction,
    newBranchName,
    newTagName,
    newTagTarget,
    annotatedTag,
    tagMessage,
    logAuthorPickerOpen,
    logFilePickerOpen,
    logFilePickerSearch,
    logFilePickerDraft,
    logRefSearch,
    logRefPanelCollapsed,
    logFavoriteRefsOnly,
    logRefSearchInput,
    changeFileContextMenu,
    changeListContextMenu,
    projectFileContextMenu,
    projectFileClipboard,
    projectNameDialog,
    addRemoteDialog,
    projectCloseDialog,
    projectDropActive,
    mergeCurrentScroller,
    mergeCurrentGutter,
    mergeResultGutter,
    mergeResultTextarea,
    mergeIncomingScroller,
    mergeIncomingGutter,
    commitMessageTextarea,
    projectEditorTextarea,
    changeFileListScroller,
    changeDiffScroller,
    logDiffScroller,
    activeChangeDiffHunkIndex,
    activeLogDiffHunkIndex,
    expandedProjectHunkIndex,
    workbenchMode,
    LOG_TAB_ID,
    activeLogTabId,
    logDiffTabs,
    noticeToast,
    errorDialog,
    pullConfirmDialog,
    submitConfirmDialog,
    activeResizePanel,
    changeFileGroups,
    visibleChangeFileRows,
    changeFileVirtualSpacerStyle,
    visibleChangeFileListStyle,
    counts,
    branch,
    brandSubtitle,
    workspaceRefreshBusy,
    selectedDiffFileTitle,
    activeChangeSideBySideDiffRows,
    activeChangeDiffHasContent,
    activeChangeDiffHunkCount,
    currentChangeDiffHunkPosition,
    activeChangeDiffFilePosition,
    canSelectPreviousChangeDiffFile,
    canSelectNextChangeDiffFile,
    changeDiffLeftLabel,
    changeDiffLeftDetail,
    changeDiffRightLabel,
    changeDiffRightDetail,
    canCommit,
    commitBusy,
    commitButtonLabel,
    commitPushButtonLabel,
    unpushedCommitCount,
    hasUnpushedCommits,
    unpushedCommitTargetLabel,
    canPushCurrentBranch,
    pushCurrentBranchButtonLabel,
    submitConfirmTitle,
    submitConfirmActionLabel,
    submitConfirmTargetLabel,
    visibleSubmitConfirmFileTreeRows,
    selectedCommitTitle,
    activeLogDiffTab,
    activeLogSideBySideDiffRows,
    activeLogDiffHasContent,
    activeLogDiffHunkCount,
    currentLogDiffHunkPosition,
    activeLogDiffFilePosition,
    canSelectPreviousLogDiffFile,
    canSelectNextLogDiffFile,
    projectEditorText,
    projectEditorLines,
    projectEditorHunks,
    expandedProjectHunk,
    projectEditorRenderStyle,
    projectEditorRenderContentStyle,
    visibleProjectFiles,
    selectableBranchTargets,
    allRefTargets,
    conflictedFiles,
    canSkipOperation,
    resultHasConflictMarkers,
    mergeConflictCount,
    mergeConflictSummary,
    mergeConflictPositionLabel,
    mergeResultStateLabel,
    pullConfirmFiles,
    pullConfirmExtraCount,
    pullConfirmModeLabel,
    isMergeConflictOperation,
    showMergeConflictWorkbench,
    mergeCurrentSide,
    mergeIncomingSide,
    mergeCurrentSourceLabel,
    mergeIncomingSourceLabel,
    mergeCurrentLines,
    mergeIncomingLines,
    mergeResultLines,
    mergeCurrentResultConnections,
    mergeIncomingResultConnections,
    mergeResultRenderStyle,
    effectiveTheme,
    workspaceGridStyle,
    commitFileDiffModeLabels,
    logHeadLabel,
    logRefFiltering,
    showLogHeadRef,
    visibleLogLocalBranches,
    visibleLogRemoteGroups,
    showLogRemoteAddEntry,
    showLogRemoteRoot,
    visibleLogTags,
    activeLogBranchRef,
    activeLogBranchFavorite,
    logRefGroupsFullyExpanded,
    hasVisibleLogRefs,
    logGraphRows,
    commitFileTreeRows,
    visibleCommitFileTreeRows,
    logAuthorOptions,
    logAuthorFilterLabel,
    logFileFilterLabel,
    visibleLogFilePickerRows,
    selectedCommitRefs,
    activeLogRefLabel,
    logFilterActive,
    logFileContextMenu,
    logRefContextMenu,
    dismissNoticeToast,
    dismissErrorDialog,
    isUiActionPending,
    isUiActionActive,
    actionIcon,
    actionIconClass,
    actionButtonClass,
    remoteActionKey,
    branchActionKey,
    runUiAction,
    chooseRepository,
    initSelectedProject,
    switchRepository,
    removeRepository,
    refreshAll,
    refreshChangesOnly,
    loadAdvancedSnapshots,
    runRemoteAction,
    resetCommitMessageHistoryCursor,
    navigateCommitMessageHistory,
    runRemoteActionFromPointer,
    cancelPullConfirmDialog,
    confirmPullSmartMerge,
    unshallowCurrentRepository,
    renameSelectedBranch,
    cleanupMergedBranches,
    runRefComparison,
    generatePatch,
    applyPatchDraft,
    createWorktreeFromDraft,
    removeWorktree,
    runStashAction,
    clearAllStashes,
    updateAllSubmodules,
    loadLfsStatus,
    loadSelectedFileHistory,
    loadSelectedBlame,
    selectSide,
    setIncludeIgnored,
    nudgePanelWidth,
    resizeLabel,
    startPanelResize,
    selectFile,
    openChangeFileContextMenu,
    openChangeListContextMenu,
    changelistById,
    canDeleteChangelist,
    changeContextLabel,
    deletableChangeContextPaths,
    changelistForChangeContext,
    changelistMoveTargets,
    showChangeFileDiffFromContext,
    discardChangeFilesFromContext,
    moveChangeFilesToChangelistFromContext,
    createChangelistFromChangeContext,
    createChangelistFromListContext,
    editChangelistFromChangeContext,
    editChangelistFromListContext,
    deleteChangelistFromListContext,
    deleteChangeFilesFromContext,
    showChangeFileHistoryFromContext,
    selectAdjacentChangeDiffFile,
    jumpChangeDiffHunk,
    selectAdjacentLogDiffFile,
    jumpLogDiffHunk,
    syncSideBySideEditorScroll,
    stageSelected,
    unstageSelected,
    discardSelected,
    shelveSelected,
    deleteShelfRecord,
    unshelveRecord,
    cancelSubmitConfirmDialog,
    isSubmitConfirmDirectoryExpanded,
    toggleSubmitConfirmDirectory,
    commitCurrent,
    confirmSubmitAction,
    shortHash,
    formatTime,
    formatBytes,
    projectFileIndent,
    logFileIndent,
    projectEditorHunkMarkerStyle,
    projectEditorOriginalPanelStyle,
    projectEditorHunkTitle,
    toggleProjectEditorHunk,
    syncProjectEditorScroll,
    discardProjectEditorHunk,
    saveProjectEditor,
    closeProjectEditorTab,
    cancelProjectCloseDialog,
    discardAndCloseProjectFile,
    saveAndCloseProjectFile,
    openProjectEntry,
    canCreateInProjectContext,
    canModifyProjectEntry,
    canPasteProjectEntry,
    submitProjectNameDialog,
    cancelProjectNameDialog,
    openProjectFileContextMenu,
    closeContextMenus,
    createProjectFileFromContext,
    createProjectDirectoryFromContext,
    cutProjectEntry,
    copyProjectEntryToInternalClipboard,
    pasteProjectEntryToContext,
    copyProjectAbsolutePath,
    copyProjectRelativePath,
    renameProjectEntryFromContext,
    deleteProjectEntryFromContext,
    openProjectEntryLog,
    projectStatusForPath,
    projectStatusLabel,
    projectFileTitle,
    projectFileClass,
    projectTabClass,
    branchNameLabel,
    formatStatusKind,
    syncChangeFileListViewport,
    isChangeFileGroupExpanded,
    changeConflictGroupKey,
    changeFileGroupCount,
    toggleChangeFileGroup,
    isChangeFileGroupSelected,
    isChangeFileGroupPartiallySelected,
    toggleChangeFileGroupSelection,
    fileContextPath,
    fileTypeLabel,
    changeFileIconClass,
    formatCommitTime,
    formatCompactCommitTime,
    formatRefName,
    isLogAuthorSelected,
    toggleLogAuthorPicker,
    toggleLogAuthorFilter,
    clearLogAuthorFilters,
    openLogFilePicker,
    closeLogFilePicker,
    isLogFilePickerDirectoryExpanded,
    toggleLogFilePickerDirectory,
    isLogFileFilterSelected,
    toggleLogFileFilter,
    applyLogFileFilters,
    clearLogFilePickerDraft,
    logFilePickerRowClass,
    logFilePickerIndent,
    shortRemoteBranchName,
    nextAvailableRemoteName,
    isLogRefActive,
    logRemoteGroupKey,
    isLogRefGroupExpanded,
    toggleLogRefGroup,
    toggleLogRefPanelCollapsed,
    focusLogRefSearch,
    toggleLogFavoriteRefsOnly,
    toggleAllLogRefGroups,
    clearLogRef,
    clearLogRefContext,
    selectLogRef,
    openLogBranchContextMenu,
    openLogRemoteDirectoryContextMenu,
    openLogTagContextMenu,
    isLogRefContextFavorite,
    isLogBranchContextTarget,
    isLogRemoteDirectoryContextTarget,
    isLogTagContextTarget,
    canCheckoutLogRefContext,
    canMergeOrRebaseLogRefContext,
    canRenameLogRefContext,
    canDeleteLogRefContext,
    canSetLogRefContextUpstream,
    showLogRefFromContext,
    checkoutLogRefFromContext,
    createBranchFromLogRefContext,
    renameLogBranchFromContext,
    deleteLogRefFromContext,
    mergeLogRefIntoCurrent,
    rebaseCurrentOntoLogRef,
    setCurrentBranchUpstreamFromContext,
    toggleLogRefFavoriteFromContext,
    copyLogRefNameFromContext,
    openAddRemoteDialog,
    cancelAddRemoteDialog,
    submitAddRemoteDialog,
    addRemoteFromLogRefContext,
    pushLogTagFromContext,
    deleteRemoteLogTagFromContext,
    createLogBranchFromHead,
    deleteActiveLogBranch,
    toggleActiveLogBranchFavorite,
    isCommitFileDirectoryExpanded,
    toggleCommitFileDirectory,
    logGraphStyle,
    logGraphViewBox,
    logNodeStyle,
    logFileTreeRowClass,
    logFileTreeRowTitle,
    formatCommitFileStatusCode,
    formatSubmitConfirmFileStatus,
    formatOperationName,
    formatWorktreeLabel,
    selectCommit,
    checkoutSelectedBranch,
    createBranchFromHead,
    deleteLocalBranch,
    deleteRemoteBranchItem,
    createTagFromInput,
    deleteLocalTag,
    pushSelectedTag,
    deleteSelectedRemoteTag,
    setSelectedUpstream,
    unsetSelectedUpstream,
    mergeSelectedTarget,
    rebaseOntoSelectedTarget,
    rebaseWithAdvancedOptions,
    runOperationControl,
    toggleCommitFile,
    fileBaseName,
    selectLogRootTab,
    logDiffTabClass,
    closeLogDiffTab,
    openLogFileContextMenu,
    showCommitFileDiff,
    cherryPickLogFile,
    revertLogFileChange,
    createPatchFromLogFile,
    showLogFileHistory,
    setConflictResultFromEvent,
    syncMergeEditorScroll,
    selectConflict,
    saveConflictResult,
    acceptConflictSide,
    acceptConflictBlock,
    shouldAppendConflictBlock,
    mergeConflictActionTitle,
    applyConflictBlock,
    mergeConflictLineClasses,
    resetConflictResultDraft,
    mergeConflictConnectionStyle,
    mergeConflictConnectionViewBox,
    mergeConflictConnectionPath,
    jumpMergeConflict,
  };
}
