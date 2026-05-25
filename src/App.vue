<script setup lang="ts">
import {
  Archive,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  ChevronDown,
  ChevronRight,
  Check,
  CornerDownLeft,
  CornerDownRight,
  ChevronsLeft,
  ChevronsRight,
  Download,
  File as FileIcon,
  FileSearch,
  Folder,
  FolderOpen,
  GitCommitVertical,
  ListChecks,
  LoaderCircle,
  Lock,
  Minus,
  Plus,
  RefreshCw,
  RotateCcw,
  Search,
  Star,
  Trash2,
  UserRound,
  X,
} from "@lucide/vue";
import AppTopbar from "./components/AppTopbar.vue";
import DiffViewer from "./components/DiffViewer.vue";
import VcsIcon from "./components/icons/VcsIcon.vue";
import ProjectPane from "./components/ProjectPane.vue";
import WorkbenchRail from "./components/WorkbenchRail.vue";
import { useGitboxApp } from "./composables/useGitboxApp";
import "./styles/app.css";

const {
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
  commandPaletteOpen,
  commandPaletteQuery,
  commandPaletteSelectionIndex,
  commandPaletteInput,
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
  logViewMode,
  LOG_TAB_ID,
  activeLogTabId,
  logDiffTabs,
  visibleCommandPaletteItems,
  activeCommandPaletteItem,
  noticeToast,
  errorDialog,
  pullConfirmDialog,
  mergePreviewDialog,
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
  resultHasConflictMarkers,
  mergeConflictCount,
  mergeConflictSummary,
  mergeConflictAnalysisBlocks,
  mergeConflictAnalysisSummary,
  mergeConflictPositionLabel,
  mergeResultStateLabel,
  pullConfirmFiles,
  pullConfirmExtraCount,
  pullConfirmModeLabel,
  mergePreviewFiles,
  mergePreviewExtraCount,
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
  toggleRepositoryPinned,
  removeRepository,
  openCommandPalette,
  closeCommandPalette,
  moveCommandPaletteSelection,
  runCommandPaletteItem,
  refreshAll,
  refreshChangesOnly,
  runRemoteAction,
  resetCommitMessageHistoryCursor,
  navigateCommitMessageHistory,
  runRemoteActionFromPointer,
  cancelPullConfirmDialog,
  confirmPullSmartMerge,
  cancelMergePreviewDialog,
  confirmMergePreviewDialog,
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
  cancelSubmitConfirmDialog,
  isSubmitConfirmDirectoryExpanded,
  toggleSubmitConfirmDirectory,
  commitCurrent,
  confirmSubmitAction,
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
  previewLogRefMerge,
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
  conflictAnalysisKindLabel,
  conflictAnalysisSideLabel,
  mergePreviewCategoryLabel,
  selectCommit,
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
} = useGitboxApp();
</script>

<template>
  <div
    class="app-shell"
    :class="{ 'project-drop-active': projectDropActive }"
    :data-theme="effectiveTheme"
    @click="closeContextMenus"
  >
    <AppTopbar
      :brand-subtitle="brandSubtitle"
      :has-repository="Boolean(repos.current)"
      :current-branch="branchNameLabel(branch?.currentBranch)"
      :remote-branch="branch?.upstream"
      :ahead="branch?.ahead ?? 0"
      :behind="branch?.behind ?? 0"
      @open-command-palette="openCommandPalette"
    />

    <Transition name="project-drop-overlay">
      <div v-if="projectDropActive" class="project-drop-overlay" aria-hidden="true">
        <div class="project-drop-target">
          <FolderOpen :size="22" />
          <strong>松开以添加项目</strong>
          <span>支持多个文件夹</span>
        </div>
      </div>
    </Transition>

    <Transition name="notice-toast">
      <div v-if="noticeToast" class="notice-toast" role="status" aria-live="polite" @click.stop>
        <Check :size="15" />
        <span>{{ noticeToast.message }}</span>
        <button type="button" title="关闭通知" @click="dismissNoticeToast(noticeToast.id)">
          <X :size="13" />
        </button>
      </div>
    </Transition>

    <div v-if="commandPaletteOpen" class="modal-backdrop command-palette-backdrop" @click.self="closeCommandPalette">
      <section class="command-palette" role="dialog" aria-modal="true" aria-label="命令面板" @click.stop>
        <label class="command-palette-search">
          <Search :size="16" />
          <input
            ref="commandPaletteInput"
            v-model="commandPaletteQuery"
            placeholder="搜索命令或仓库"
            @keydown.esc.prevent="closeCommandPalette"
            @keydown.up.prevent="moveCommandPaletteSelection(-1)"
            @keydown.down.prevent="moveCommandPaletteSelection(1)"
            @keydown.enter.prevent="runCommandPaletteItem(activeCommandPaletteItem)"
          />
        </label>
        <div class="command-palette-list">
          <button
            v-for="(item, index) in visibleCommandPaletteItems"
            :key="item.id"
            class="command-palette-item"
            :class="{ active: index === commandPaletteSelectionIndex, disabled: item.disabled }"
            :disabled="item.disabled"
            @mouseenter="commandPaletteSelectionIndex = index"
            @click="runCommandPaletteItem(item)"
          >
            <span>{{ item.section }}</span>
            <strong>{{ item.title }}</strong>
            <small>{{ item.subtitle }}</small>
          </button>
          <div v-if="visibleCommandPaletteItems.length === 0" class="command-palette-empty">没有匹配项</div>
        </div>
      </section>
    </div>

    <div v-if="errorDialog" class="modal-backdrop" @click.self="dismissErrorDialog(errorDialog.id)">
      <section
        class="error-modal"
        role="alertdialog"
        aria-modal="true"
        aria-label="错误提示"
        tabindex="-1"
        @keydown.esc.prevent="dismissErrorDialog(errorDialog.id)"
      >
        <header>
          <h2>错误提示</h2>
          <button class="icon-only-button" type="button" title="关闭" @click="dismissErrorDialog(errorDialog.id)">
            <X :size="14" />
          </button>
        </header>
        <p class="error-modal-message">{{ errorDialog.message }}</p>
        <footer>
          <button class="icon-button primary" type="button" autofocus @click="dismissErrorDialog(errorDialog.id)">
            <Check :size="14" />
            <span>知道了</span>
          </button>
        </footer>
      </section>
    </div>

    <div v-if="pullConfirmDialog" class="modal-backdrop" @click.self="cancelPullConfirmDialog">
      <section
        class="pull-confirm-modal"
        role="dialog"
        aria-modal="true"
        aria-label="确认拉取更新"
        @keydown.esc.prevent="cancelPullConfirmDialog"
      >
        <header>
          <div>
            <h2>本地修改与远程更新重叠</h2>
          </div>
          <button
            class="icon-only-button"
            type="button"
            title="关闭"
            :disabled="pullConfirmDialog.loading"
            @click="cancelPullConfirmDialog"
          >
            <X :size="14" />
          </button>
        </header>
        <p>
          {{ pullConfirmDialog.preview.target }} 将执行{{ pullConfirmModeLabel }}，并更新本地已修改的文件。
          GitBox 可以先保护未提交修改，再执行智能合并；如果自动合并失败，会进入三栏合并编辑器。
        </p>
        <div class="pull-confirm-summary">
          <span>{{ pullConfirmDialog.preview.remote }}</span>
          <strong>{{ pullConfirmDialog.preview.target }}</strong>
          <small>{{ pullConfirmDialog.preview.overlappingPaths.length }} 个重叠文件</small>
        </div>
        <div class="pull-confirm-file-list" aria-label="重叠文件">
          <span v-for="path in pullConfirmFiles" :key="path">{{ path }}</span>
          <span v-if="pullConfirmExtraCount > 0">还有 {{ pullConfirmExtraCount }} 个文件</span>
        </div>
        <footer>
          <button class="icon-button" type="button" :disabled="pullConfirmDialog.loading" @click="cancelPullConfirmDialog">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button class="icon-button primary" type="button" :disabled="pullConfirmDialog.loading" @click="confirmPullSmartMerge">
            <LoaderCircle v-if="pullConfirmDialog.loading" :size="14" class="button-spinner" />
            <Check v-else :size="14" />
            <span>{{ pullConfirmDialog.loading ? "合并中" : "智能合并" }}</span>
          </button>
        </footer>
      </section>
    </div>

    <div v-if="mergePreviewDialog" class="modal-backdrop" @click.self="cancelMergePreviewDialog">
      <section
        class="pull-confirm-modal merge-preview-modal"
        role="dialog"
        aria-modal="true"
        aria-label="合并预览"
        @keydown.esc.prevent="cancelMergePreviewDialog"
      >
        <header>
          <div>
            <h2>合并预览</h2>
          </div>
          <button
            class="icon-only-button"
            type="button"
            title="关闭"
            :disabled="mergePreviewDialog.loading"
            @click="cancelMergePreviewDialog"
          >
            <X :size="14" />
          </button>
        </header>
        <p>
          {{ mergePreviewDialog.preview.target }} 将合并到当前分支。预览只读取提交内容，不会改动工作区。
        </p>
        <div class="pull-confirm-summary merge-preview-summary">
          <span>{{ mergePreviewDialog.preview.clean ? "可合并" : "需要检查" }}</span>
          <strong>{{ mergePreviewDialog.preview.target }}</strong>
          <small>
            {{ mergePreviewDialog.preview.summary.clean }} 直接 ·
            {{ mergePreviewDialog.preview.summary.autoResolvable }} 可合成 ·
            {{ mergePreviewDialog.preview.summary.manual }} 人工 ·
            {{ mergePreviewDialog.preview.summary.addDelete }} 增删 ·
            {{ mergePreviewDialog.preview.summary.binary }} 二进制
          </small>
        </div>
        <div class="merge-preview-file-list" aria-label="合并预览文件">
          <div v-for="file in mergePreviewFiles" :key="file.path" class="merge-preview-file" :class="file.category">
            <span>{{ mergePreviewCategoryLabel(file.category) }}</span>
            <strong>{{ file.path }}</strong>
            <small>{{ file.explanation }}</small>
          </div>
          <div v-if="mergePreviewExtraCount > 0" class="merge-preview-file">
            <span>更多</span>
            <strong>还有 {{ mergePreviewExtraCount }} 个文件</strong>
          </div>
        </div>
        <footer>
          <button class="icon-button" type="button" :disabled="mergePreviewDialog.loading" @click="cancelMergePreviewDialog">
            <X :size="14" />
            <span>关闭</span>
          </button>
          <button class="icon-button primary" type="button" :disabled="mergePreviewDialog.loading" @click="confirmMergePreviewDialog">
            <LoaderCircle v-if="mergePreviewDialog.loading" :size="14" class="button-spinner" />
            <Check v-else :size="14" />
            <span>{{ mergePreviewDialog.loading ? "合并中" : "继续合并" }}</span>
          </button>
        </footer>
      </section>
    </div>

    <div v-if="submitConfirmDialog" class="modal-backdrop" @click.self="cancelSubmitConfirmDialog">
      <section
        class="submit-confirm-modal"
        role="dialog"
        aria-modal="true"
        :aria-label="submitConfirmTitle"
        tabindex="-1"
        @keydown.esc.prevent="cancelSubmitConfirmDialog"
      >
        <header>
          <div>
            <h2>{{ submitConfirmTitle }}</h2>
          </div>
          <button
            class="icon-only-button"
            type="button"
            title="关闭"
            :disabled="submitConfirmDialog.loading"
            @click="cancelSubmitConfirmDialog"
          >
            <X :size="14" />
          </button>
        </header>
        <div class="submit-confirm-layout">
          <div class="submit-confirm-left">
            <div class="submit-confirm-summary">
              <span>{{ submitConfirmDialog.mode === "commit" ? "目标分支" : "远程目标" }}</span>
              <strong>{{ submitConfirmTargetLabel }}</strong>
              <small v-if="submitConfirmDialog.paths.length">{{ submitConfirmDialog.paths.length }} 个文件</small>
              <small v-else>当前分支</small>
            </div>
            <div class="submit-confirm-message">
              <span>提交信息</span>
              <strong v-if="submitConfirmDialog.message">{{ submitConfirmDialog.message }}</strong>
              <strong v-else>推送当前分支</strong>
            </div>
            <div class="submit-confirm-meta">
              <span>仓库</span>
              <strong>{{ repos.name }}</strong>
            </div>
            <div class="submit-confirm-meta">
              <span>当前分支</span>
              <strong>{{ submitConfirmDialog.currentBranch }}</strong>
            </div>
            <div v-if="submitConfirmDialog.options.length" class="submit-confirm-options" aria-label="提交选项">
              <span v-for="option in submitConfirmDialog.options" :key="option">{{ option }}</span>
            </div>
            <p v-else class="submit-confirm-empty">
              {{
                submitConfirmDialog.mode === "push"
                  ? "将推送当前分支中尚未同步的提交。"
                  : "没有额外提交选项。"
              }}
            </p>
          </div>

          <div class="submit-confirm-file-tree-panel">
            <div class="submit-confirm-file-tree-head">
              <span>文件树</span>
              <strong v-if="submitConfirmDialog.paths.length">{{ submitConfirmDialog.paths.length }} 个文件</strong>
              <strong v-else>当前分支</strong>
            </div>
            <div v-if="submitConfirmDialog.paths.length" class="submit-confirm-file-tree" aria-label="提交文件树">
              <button
                v-for="row in visibleSubmitConfirmFileTreeRows"
                :key="row.id"
                class="submit-confirm-file-row"
                :class="{ directory: row.directory }"
                type="button"
                :title="row.directory ? row.path : `${row.path} · ${formatSubmitConfirmFileStatus(row.status)}`"
                :style="{ paddingLeft: `${Math.max(0, row.depth) * 14 + 8}px` }"
                @click="row.directory && toggleSubmitConfirmDirectory(row.path)"
              >
                <ChevronDown v-if="row.directory && isSubmitConfirmDirectoryExpanded(row.path)" :size="13" />
                <ChevronRight v-else-if="row.directory" :size="13" />
                <span v-else class="submit-confirm-file-toggle-placeholder" />
                <FolderOpen v-if="row.directory && isSubmitConfirmDirectoryExpanded(row.path)" :size="14" />
                <Folder v-else-if="row.directory" :size="14" />
                <span v-else class="change-file-icon" :class="changeFileIconClass(row.path)">
                  <span v-if="fileTypeLabel(row.path)">{{ fileTypeLabel(row.path) }}</span>
                  <FileIcon v-else :size="13" />
                </span>
                <span class="submit-confirm-file-main">
                  <strong>{{ row.name }}</strong>
                  <small>{{ row.directory ? `${row.fileCount ?? 0} 个文件` : formatSubmitConfirmFileStatus(row.status) }}</small>
                </span>
              </button>
            </div>
            <p v-else class="submit-confirm-empty">没有单独的文件列表，操作对象是当前分支。</p>
          </div>
        </div>
        <footer>
          <button class="icon-button" type="button" :disabled="submitConfirmDialog.loading" @click="cancelSubmitConfirmDialog">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button class="icon-button primary" type="button" :disabled="submitConfirmDialog.loading" @click="confirmSubmitAction">
            <LoaderCircle v-if="submitConfirmDialog.loading" :size="14" class="button-spinner" />
            <Check v-else :size="14" />
            <span>{{ submitConfirmActionLabel }}</span>
          </button>
        </footer>
      </section>
    </div>

    <section
      class="workspace"
      :class="{ 'workspace-empty': !repos.current, 'is-resizing': activeResizePanel }"
      :style="workspaceGridStyle"
    >
      <ProjectPane
        v-if="settings.panelVisibility.project"
        :collapsed="settings.projectPaneCollapsed"
        @choose-repository="chooseRepository"
        @remove-repository="removeRepository"
        @switch-repository="switchRepository"
        @toggle-pinned="toggleRepositoryPinned"
        @toggle-collapsed="settings.setProjectPaneCollapsed(!settings.projectPaneCollapsed)"
      />

      <div
        v-if="settings.panelVisibility.project && !settings.projectPaneCollapsed"
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
      />

      <section v-if="!repos.current" class="empty-workbench">
        <div v-if="repos.selectedPath" class="empty-panel project-init-panel">
          <ListChecks :size="40" />
          <h1>项目尚未初始化</h1>
          <p>{{ repos.selectedPath }}</p>
          <button
            class="tool-button primary large"
            :class="actionButtonClass('repo.init')"
            :disabled="advanced.loading || !repos.selectedPath"
            :aria-busy="isUiActionPending('repo.init')"
            @click="initSelectedProject"
          >
            <component
              :is="actionIcon('repo.init', Plus)"
              :class="actionIconClass('repo.init')"
              :size="18"
            />
            <span>{{ advanced.loading ? "初始化中" : "初始化仓库" }}</span>
          </button>
        </div>
        <div v-else class="empty-panel">
          <ListChecks :size="40" />
          <h1>选择本地 Git 仓库</h1>
          <button
            class="tool-button primary large"
            :class="actionButtonClass('repo.choose')"
            :aria-busy="isUiActionPending('repo.choose')"
            @click="chooseRepository"
          >
            <component
              :is="actionIcon('repo.choose', FolderOpen)"
              :class="actionIconClass('repo.choose')"
              :size="18"
            />
            <span>{{ isUiActionPending('repo.choose') ? "添加中" : "添加项目" }}</span>
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

        <div class="file-actions">
          <button
            class="icon-only-button file-actions-refresh"
            :class="actionButtonClass('workspace.refresh')"
            type="button"
            title="刷新变更"
            :disabled="workspaceRefreshBusy"
            :aria-busy="isUiActionActive('workspace.refresh')"
            @click="refreshChangesOnly"
          >
            <component
              :is="actionIcon('workspace.refresh', RefreshCw)"
              :class="actionIconClass('workspace.refresh')"
              :size="14"
            />
          </button>
          <button
            class="icon-only-button file-actions-pull"
            :class="actionButtonClass(remoteActionKey('pull'))"
            type="button"
            title="拉取代码"
            :disabled="remote.loading || !remote.selectedRemote"
            :aria-busy="isUiActionActive(remoteActionKey('pull'))"
            @pointerdown="runRemoteActionFromPointer($event, 'pull')"
            @click="runRemoteAction('pull')"
          >
            <component
              :is="actionIcon(remoteActionKey('pull'), ArrowDown)"
              :class="actionIconClass(remoteActionKey('pull'))"
              :size="14"
            />
          </button>
          <button
            class="icon-button"
            :class="actionButtonClass('changes.stage')"
            title="暂存选中文件"
            :disabled="changes.activePaths.length === 0 || settings.selectedSide === 'staged' || changes.loading"
            :aria-busy="isUiActionPending('changes.stage')"
            @click="stageSelected"
          >
            <component
              :is="actionIcon('changes.stage', Check)"
              :class="actionIconClass('changes.stage')"
              :size="15"
            />
            <span>暂存</span>
          </button>
          <button
            class="icon-button"
            :class="actionButtonClass('changes.unstage')"
            title="取消暂存"
            :disabled="changes.activePaths.length === 0 || settings.selectedSide === 'unstaged' || changes.loading"
            :aria-busy="isUiActionPending('changes.unstage')"
            @click="unstageSelected"
          >
            <component
              :is="actionIcon('changes.unstage', Minus)"
              :class="actionIconClass('changes.unstage')"
              :size="15"
            />
            <span>移出</span>
          </button>
          <button
            class="icon-button danger"
            :class="actionButtonClass('changes.discard')"
            title="回滚变更"
            :disabled="changes.activePaths.length === 0 || changes.loading"
            :aria-busy="isUiActionPending('changes.discard')"
            @click="discardSelected"
          >
            <component
              :is="actionIcon('changes.discard', Trash2)"
              :class="actionIconClass('changes.discard')"
              :size="15"
            />
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

        <div
          ref="changeFileListScroller"
          class="file-list source-control-tree"
          @scroll="syncChangeFileListViewport"
          @contextmenu.prevent="openChangeListContextMenu(null, $event)"
        >
          <div v-if="changeFileGroups.length === 0" class="file-list-empty">没有文件变更</div>
          <div v-else class="change-file-virtual-spacer" :style="changeFileVirtualSpacerStyle">
            <div class="change-file-virtual-rows" :style="visibleChangeFileListStyle">
              <template v-for="row in visibleChangeFileRows" :key="row.key">
                <div
                  v-if="row.kind === 'group'"
                  class="change-file-group-header"
                  @contextmenu.prevent.stop="openChangeListContextMenu(row.group.changelistId, $event)"
                >
                  <button class="change-group-toggle" type="button" @click="toggleChangeFileGroup(row.group.key)">
                    <ChevronDown v-if="isChangeFileGroupExpanded(row.group.key)" :size="14" />
                    <ChevronRight v-else :size="14" />
                  </button>
                  <input
                    class="change-group-checkbox"
                    type="checkbox"
                    :checked="isChangeFileGroupSelected(row.group)"
                    :disabled="changeFileGroupCount(row.group) === 0"
                    :indeterminate.prop="isChangeFileGroupPartiallySelected(row.group)"
                    @change="toggleChangeFileGroupSelection(row.group)"
                  />
                  <button class="change-group-title" type="button" @click="toggleChangeFileGroup(row.group.key)">
                    <span>{{ row.group.label }}</span>
                    <small>{{ changeFileGroupCount(row.group) }} 个文件</small>
                  </button>
                </div>

                <div v-else-if="row.kind === 'empty'" class="change-file-group-empty">没有文件</div>

                <div
                  v-else-if="row.kind === 'conflict-group'"
                  class="change-conflict-header"
                  @contextmenu.prevent.stop="openChangeListContextMenu(row.group.changelistId, $event)"
                >
                  <button
                    class="change-group-toggle"
                    type="button"
                    @click="toggleChangeFileGroup(changeConflictGroupKey(row.group))"
                  >
                    <ChevronDown v-if="isChangeFileGroupExpanded(changeConflictGroupKey(row.group))" :size="14" />
                    <ChevronRight v-else :size="14" />
                  </button>
                  <input
                    class="change-group-checkbox"
                    type="checkbox"
                    :checked="isChangeFileGroupSelected(row.group.conflictFiles)"
                    :indeterminate.prop="isChangeFileGroupPartiallySelected(row.group.conflictFiles)"
                    @change="toggleChangeFileGroupSelection(row.group.conflictFiles)"
                  />
                  <button
                    class="change-group-title conflict"
                    type="button"
                    @click="toggleChangeFileGroup(changeConflictGroupKey(row.group))"
                  >
                    <span>合并冲突</span>
                  </button>
                </div>

                <button
                  v-else-if="row.kind === 'file'"
                  class="file-row"
                  :class="{
                    'conflict-file-row': row.conflict,
                    active: changes.selectedFile === row.file.path,
                    selected: changes.selectedPaths.includes(row.file.path),
                    [`status-${row.file.kind.split('|')[0]}`]: true,
                  }"
                  :title="`${row.file.path} · ${formatStatusKind(row.file.kind)}`"
                  @click="row.conflict ? selectConflict(row.file.path) : selectFile(row.file, row.group.side)"
                  @contextmenu.prevent.stop="openChangeFileContextMenu(row.file, row.group.side, $event)"
                >
                  <input
                    type="checkbox"
                    :checked="changes.selectedPaths.includes(row.file.path)"
                    @click.stop
                    @change="changes.togglePath(row.file.path)"
                  />
                  <span class="status-dot" :class="row.file.kind.split('|')[0]" />
                  <span class="change-file-icon" :class="changeFileIconClass(row.file.path)">
                    <span v-if="fileTypeLabel(row.file.path)">{{ fileTypeLabel(row.file.path) }}</span>
                    <FileIcon v-else :size="13" />
                  </span>
                  <span class="file-main">
                    <strong>{{ fileBaseName(row.file.path) }}</strong>
                    <small>{{ fileContextPath(row.file.path) }}</small>
                  </span>
                </button>
              </template>
            </div>
          </div>
        </div>

        <div class="shelve-box">
          <input v-model="shelveMessage" placeholder="搁置说明" />
          <button
            class="icon-button"
            :class="actionButtonClass('changes.shelve')"
            :disabled="changes.activePaths.length === 0 || changes.loading"
            :aria-busy="isUiActionPending('changes.shelve')"
            title="搁置选中变更"
            @click="shelveSelected"
          >
            <component
              :is="actionIcon('changes.shelve', Archive)"
              :class="actionIconClass('changes.shelve')"
              :size="15"
            />
            <span>搁置</span>
          </button>
        </div>

        <form class="commit-box" @submit.prevent="commitCurrent(false)">
          <div class="commit-title">
            <GitCommitVertical :size="16" />
            <span>提交</span>
          </div>
          <textarea
            ref="commitMessageTextarea"
            v-model="commit.message"
            rows="5"
            placeholder="提交信息"
            @input="resetCommitMessageHistoryCursor"
            @keydown.up="navigateCommitMessageHistory($event, 'previous')"
            @keydown.down="navigateCommitMessageHistory($event, 'next')"
          />
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
          <div v-if="hasUnpushedCommits" class="commit-push-banner">
            <div class="commit-push-banner-copy">
              <ArrowUp :size="14" />
              <div>
                <strong>待推送 {{ unpushedCommitCount }} 个提交</strong>
                <small>{{ unpushedCommitTargetLabel }}</small>
              </div>
            </div>
            <button
              class="icon-button commit-push-only-button"
              :class="actionButtonClass(remoteActionKey('push'))"
              type="button"
              title="推送当前分支"
              :disabled="!canPushCurrentBranch"
              :aria-busy="isUiActionActive(remoteActionKey('push'))"
              @pointerdown="runRemoteActionFromPointer($event, 'push')"
              @click="runRemoteAction('push')"
            >
              <component
                :is="actionIcon(remoteActionKey('push'), ArrowUp)"
                :class="actionIconClass(remoteActionKey('push'))"
                :size="14"
              />
              <span>{{ pushCurrentBranchButtonLabel }}</span>
            </button>
          </div>
          <div class="commit-actions">
            <button
              class="commit-button"
              :class="{ loading: pendingCommitAction === 'commit' }"
              :disabled="!canCommit || commitBusy"
              :aria-busy="pendingCommitAction === 'commit'"
            >
              <LoaderCircle v-if="pendingCommitAction === 'commit'" class="button-spinner" :size="14" />
              <span>{{ commitButtonLabel }}</span>
            </button>
            <button
              class="commit-button secondary"
              type="button"
              :class="{ loading: pendingCommitAction === 'push' }"
              :disabled="!canCommit || commitBusy || !remote.selectedRemote"
              :aria-busy="pendingCommitAction === 'push'"
              @click="commitCurrent(true)"
            >
              <LoaderCircle v-if="pendingCommitAction === 'push'" class="button-spinner" :size="14" />
              <span>{{ commitPushButtonLabel }}</span>
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
            :class="actionButtonClass('workspace.refresh')"
            type="button"
            title="刷新引用和日志"
            :disabled="history.loading || branches.loading"
            :aria-busy="isUiActionActive('workspace.refresh')"
            @click="refreshAll"
          >
            <component
              :is="actionIcon('workspace.refresh', RefreshCw)"
              :class="actionIconClass('workspace.refresh')"
              :size="15"
            />
          </button>
          <button
            class="log-ref-tool-button"
            :class="actionButtonClass(remoteActionKey('fetch'))"
            type="button"
            title="获取远程引用"
            :disabled="remote.loading || !remote.selectedRemote"
            :aria-busy="isUiActionActive(remoteActionKey('fetch'))"
            @pointerdown="runRemoteActionFromPointer($event, 'fetch')"
            @click="runRemoteAction('fetch')"
          >
            <component
              :is="actionIcon(remoteActionKey('fetch'), Download)"
              :class="actionIconClass(remoteActionKey('fetch'))"
              :size="15"
            />
          </button>
          <button
            class="log-ref-tool-button"
            :class="actionButtonClass(remoteActionKey('pull'))"
            type="button"
            title="拉取代码"
            :disabled="remote.loading || !remote.selectedRemote"
            :aria-busy="isUiActionActive(remoteActionKey('pull'))"
            @pointerdown="runRemoteActionFromPointer($event, 'pull')"
            @click="runRemoteAction('pull')"
          >
            <component
              :is="actionIcon(remoteActionKey('pull'), ArrowDown)"
              :class="actionIconClass(remoteActionKey('pull'))"
              :size="15"
            />
          </button>
          <span class="log-ref-tool-separator" />
          <button
            class="log-ref-tool-button"
            :class="actionButtonClass(branchActionKey('create'))"
            type="button"
            title="从当前 HEAD 创建并切换分支"
            :disabled="branches.loading"
            :aria-busy="isUiActionPending(branchActionKey('create'))"
            @click="createLogBranchFromHead"
          >
            <component
              :is="actionIcon(branchActionKey('create'), Plus)"
              :class="actionIconClass(branchActionKey('create'))"
              :size="16"
            />
          </button>
          <button
            class="log-ref-tool-button danger"
            :class="actionButtonClass(branchActionKey('delete', activeLogBranchRef?.fullName))"
            type="button"
            title="删除当前选中的分支"
            :disabled="branches.loading || !activeLogBranchRef || activeLogBranchRef.current"
            :aria-busy="isUiActionPending(branchActionKey('delete', activeLogBranchRef?.fullName))"
            @click="deleteActiveLogBranch"
          >
            <component
              :is="actionIcon(branchActionKey('delete', activeLogBranchRef?.fullName), Trash2)"
              :class="actionIconClass(branchActionKey('delete', activeLogBranchRef?.fullName))"
              :size="15"
            />
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
              <VcsIcon v-else :size="13" />
              <span>
                <strong>{{ branchItem.name }}</strong>
              </span>
            </button>
          </div>

          <button
            v-if="showLogRemoteRoot"
            class="log-ref-toggle"
            :class="{ 'context-target': isLogRemoteDirectoryContextTarget() }"
            type="button"
            @click="toggleLogRefGroup('remote')"
            @contextmenu.prevent.stop="openLogRemoteDirectoryContextMenu(undefined, $event)"
          >
            <ChevronDown v-if="logRefFiltering || isLogRefGroupExpanded('remote')" :size="13" />
            <ChevronRight v-else :size="13" />
            <span>远程</span>
          </button>
          <div
            v-if="(visibleLogRemoteGroups.length || showLogRemoteAddEntry) && (logRefFiltering || isLogRefGroupExpanded('remote'))"
            class="log-ref-children"
          >
            <section v-for="group in visibleLogRemoteGroups" :key="`log-remote-${group.name}`" class="log-ref-group">
              <button
                class="log-ref-toggle remote-root"
                :class="{ 'context-target': isLogRemoteDirectoryContextTarget(group.name) }"
                type="button"
                @click="toggleLogRefGroup(logRemoteGroupKey(group.name))"
                @contextmenu.prevent.stop="openLogRemoteDirectoryContextMenu(group.name, $event)"
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
                <div
                  v-if="group.branches.length === 0"
                  class="log-ref-row remote empty remote-empty"
                  :class="{ 'context-target': isLogRemoteDirectoryContextTarget(group.name) }"
                  title="远程仓库暂无分支"
                  @contextmenu.prevent.stop="openLogRemoteDirectoryContextMenu(group.name, $event)"
                >
                  <Minus :size="13" />
                  <span>
                    <strong>暂无远程分支</strong>
                  </span>
                </div>
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
                  <VcsIcon v-else :size="13" />
                  <span>
                    <strong>{{ shortRemoteBranchName(branchItem.name, group.name) }}</strong>
                  </span>
                </button>
              </div>
            </section>
            <button
              v-if="showLogRemoteAddEntry && visibleLogRemoteGroups.length === 0"
              class="log-ref-row remote-root-add empty remote-add"
              title="添加远程仓库"
              type="button"
              @click="openAddRemoteDialog()"
              @contextmenu.prevent.stop="openLogRemoteDirectoryContextMenu(undefined, $event)"
            >
              <Plus :size="13" />
              <span>
                <strong>添加远程仓库</strong>
              </span>
            </button>
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
          <button
            class="icon-only-button"
            :class="actionButtonClass('project.refresh')"
            title="刷新项目文件"
            :disabled="project.loading"
            :aria-busy="isUiActionPending('project.refresh')"
            @click="runUiAction('project.refresh', () => project.refresh())"
          >
            <component
              :is="actionIcon('project.refresh', RefreshCw)"
              :class="actionIconClass('project.refresh')"
              :size="14"
            />
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


      <div
        v-if="settings.panelVisibility.changes"
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
        <template v-if="showMergeConflictWorkbench">
        <div class="merge-workbench">
          <div class="diff-header merge-header">
            <div class="merge-title-block">
              <span class="eyebrow">合并</span>
              <h2>{{ operations.conflict?.path }}</h2>
            </div>
            <div class="merge-conflict-summary">{{ mergeConflictSummary }}</div>
          </div>

          <div class="merge-editor-toolbar">
            <div class="merge-toolbar-status">
              <div class="merge-conflict-jump-actions" aria-label="冲突导航">
                <button
                  class="icon-only-button"
                  :disabled="mergeConflictCount === 0"
                  aria-label="上一个冲突"
                  title="上一个冲突"
                  @click="jumpMergeConflict(-1)"
                >
                  <ArrowUp :size="14" />
                </button>
                <button
                  class="icon-only-button"
                  :disabled="mergeConflictCount === 0"
                  aria-label="下一个冲突"
                  title="下一个冲突"
                  @click="jumpMergeConflict(1)"
                >
                  <ArrowDown :size="14" />
                </button>
                <small class="merge-conflict-position">{{ mergeConflictPositionLabel }}</small>
              </div>
            </div>
            <div class="merge-save-actions">
              <button
                v-if="isMergeConflictOperation"
                type="button"
                class="tool-button danger"
                :disabled="operations.loading"
                title="中止合并"
                @click="runOperationControl('abort')"
              >
                <X :size="14" />
                <span>中止合并</span>
              </button>
              <span class="merge-result-state" :class="{ warning: resultHasConflictMarkers }">
                {{ mergeResultStateLabel }}
              </span>
              <button class="tool-button" :disabled="operations.loading || !operations.resultDirty" @click="saveConflictResult(false)">
                <RefreshCw :size="14" />
                <span>保存结果</span>
              </button>
            </div>
          </div>

          <div v-if="mergeConflictAnalysisBlocks.length > 0" class="merge-analysis-panel">
            <div class="merge-analysis-heading">
              <strong>冲突分析</strong>
              <span>{{ mergeConflictAnalysisSummary }}</span>
            </div>
            <div class="merge-analysis-list">
              <div
                v-for="block in mergeConflictAnalysisBlocks"
                :key="block.index"
                class="merge-analysis-item"
                :class="block.confidence"
              >
                <span class="merge-analysis-kind">{{ conflictAnalysisKindLabel(block.kind) }}</span>
                <strong>块 {{ block.index + 1 }}</strong>
                <small>{{ block.score }} · {{ conflictAnalysisSideLabel(block.suggestedSide) }}</small>
                <p>{{ block.explanation }}</p>
              </div>
            </div>
          </div>

          <div class="merge-editor">
            <div class="merge-connection-layer" aria-hidden="true">
              <div class="merge-connection-column current">
                <svg
                  v-for="connection in mergeCurrentResultConnections"
                  :key="connection.key"
                  class="merge-connection"
                  :class="[`merge-connection-${connection.source}`, `conflict-${connection.side}`]"
                  :style="mergeConflictConnectionStyle(connection)"
                  :viewBox="mergeConflictConnectionViewBox(connection)"
                  preserveAspectRatio="none"
                >
                  <path :d="mergeConflictConnectionPath(connection)" />
                </svg>
              </div>
              <div class="merge-connection-column result"></div>
              <div class="merge-connection-column incoming">
                <svg
                  v-for="connection in mergeIncomingResultConnections"
                  :key="connection.key"
                  class="merge-connection"
                  :class="[`merge-connection-${connection.source}`, `conflict-${connection.side}`]"
                  :style="mergeConflictConnectionStyle(connection)"
                  :viewBox="mergeConflictConnectionViewBox(connection)"
                  preserveAspectRatio="none"
                >
                  <path :d="mergeConflictConnectionPath(connection)" />
                </svg>
              </div>
            </div>
            <section class="merge-column current">
              <div class="merge-column-title">
                <div>
                  <strong>当前</strong>
                  <span><Lock :size="12" /> 来自 {{ mergeCurrentSourceLabel }}</span>
                </div>
              </div>
              <div
                class="merge-source-body current"
              >
                <div
                  ref="mergeCurrentScroller"
                  class="merge-code-view"
                  aria-label="当前版本内容"
                  @scroll="syncMergeEditorScroll"
                >
                  <div
                    v-for="line in mergeCurrentLines"
                    :key="line.id"
                    class="merge-code-line"
                    :class="mergeConflictLineClasses(line)"
                    :data-merge-conflict-index="line.conflictIndex ?? undefined"
                  >
                    <span class="merge-line-content"><template
                      v-for="(token, tokenIndex) in line.tokens"
                      :key="tokenIndex"
                    ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                  </div>
                </div>
                <div ref="mergeCurrentGutter" class="merge-source-gutter current" aria-label="当前版本行号">
                  <div
                    v-for="line in mergeCurrentLines"
                    :key="`current-gutter-${line.id}`"
                    class="merge-source-gutter-line"
                    :class="mergeConflictLineClasses(line)"
                  >
                    <span v-if="line.conflictStart" class="merge-line-actions current">
                      <button
                        type="button"
                        class="merge-inline-action clear"
                        :disabled="operations.loading"
                        title="拒绝当前块，使用传入块"
                        aria-label="拒绝当前块，使用传入块"
                        @click.stop="acceptConflictBlock(line.conflictIndex, mergeIncomingSide)"
                      >
                        <X :size="12" />
                      </button>
                      <button
                        type="button"
                        class="merge-inline-action accept"
                        :disabled="operations.loading"
                        :title="mergeConflictActionTitle(line.conflictIndex, mergeCurrentSide, '当前')"
                        :aria-label="mergeConflictActionTitle(line.conflictIndex, mergeCurrentSide, '当前')"
                        @click.stop="applyConflictBlock(line.conflictIndex, mergeCurrentSide)"
                      >
                        <CornerDownRight
                          v-if="shouldAppendConflictBlock(line.conflictIndex, mergeCurrentSide)"
                          :size="14"
                        />
                        <ChevronsRight v-else :size="14" />
                      </button>
                    </span>
                    <span class="merge-line-number-text">{{ line.number }}</span>
                  </div>
                </div>
              </div>
            </section>

            <section class="merge-column result">
              <div class="merge-column-title">
                <div>
                  <strong>结果</strong>
                  <span>{{ operations.resultDirty ? "已修改" : "未修改" }}</span>
                </div>
              </div>
              <div class="merge-result-body">
                <div ref="mergeResultGutter" class="merge-result-gutter" aria-hidden="true">
                  <span
                    v-for="line in mergeResultLines"
                    :key="`result-gutter-${line.id}`"
                    class="merge-result-gutter-line"
                    :class="mergeConflictLineClasses(line)"
                  >
                    {{ line.number }}
                  </span>
                </div>
                <div class="merge-result-editor">
                  <div class="merge-result-render" aria-hidden="true">
                    <div class="merge-result-render-content" :style="mergeResultRenderStyle">
                      <div
                        v-for="line in mergeResultLines"
                        :key="`result-render-${line.id}`"
                        class="merge-result-render-line"
                        :class="mergeConflictLineClasses(line)"
                      ><template
                        v-for="(token, tokenIndex) in line.tokens"
                        :key="tokenIndex"
                      ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></div>
                    </div>
                  </div>
                <textarea
                  ref="mergeResultTextarea"
                  :value="operations.resultDraft"
                  spellcheck="false"
                  @input="setConflictResultFromEvent"
                  @scroll="syncMergeEditorScroll"
                />
                </div>
              </div>
            </section>

            <section class="merge-column incoming">
              <div class="merge-column-title">
                <div>
                  <strong>传入</strong>
                  <span><Lock :size="12" /> 来自 {{ mergeIncomingSourceLabel }}</span>
                </div>
              </div>
              <div
                class="merge-source-body incoming"
              >
                <div ref="mergeIncomingGutter" class="merge-source-gutter incoming" aria-label="传入版本行号">
                  <div
                    v-for="line in mergeIncomingLines"
                    :key="`incoming-gutter-${line.id}`"
                    class="merge-source-gutter-line"
                    :class="mergeConflictLineClasses(line)"
                  >
                    <span class="merge-line-number-text">{{ line.number }}</span>
                    <span v-if="line.conflictStart" class="merge-line-actions incoming">
                      <button
                        type="button"
                        class="merge-inline-action accept"
                        :disabled="operations.loading"
                        :title="mergeConflictActionTitle(line.conflictIndex, mergeIncomingSide, '传入')"
                        :aria-label="mergeConflictActionTitle(line.conflictIndex, mergeIncomingSide, '传入')"
                        @click.stop="applyConflictBlock(line.conflictIndex, mergeIncomingSide)"
                      >
                        <CornerDownLeft
                          v-if="shouldAppendConflictBlock(line.conflictIndex, mergeIncomingSide)"
                          :size="14"
                        />
                        <ChevronsLeft v-else :size="14" />
                      </button>
                      <button
                        type="button"
                        class="merge-inline-action clear"
                        :disabled="operations.loading"
                        title="拒绝传入块，使用当前块"
                        aria-label="拒绝传入块，使用当前块"
                        @click.stop="acceptConflictBlock(line.conflictIndex, mergeCurrentSide)"
                      >
                        <X :size="12" />
                      </button>
                    </span>
                  </div>
                </div>
                <div
                  ref="mergeIncomingScroller"
                  class="merge-code-view"
                  aria-label="传入版本内容"
                  @scroll="syncMergeEditorScroll"
                >
                  <div
                    v-for="line in mergeIncomingLines"
                    :key="line.id"
                    class="merge-code-line"
                    :class="mergeConflictLineClasses(line)"
                    :data-merge-conflict-index="line.conflictIndex ?? undefined"
                  >
                    <span class="merge-line-content"><template
                      v-for="(token, tokenIndex) in line.tokens"
                      :key="tokenIndex"
                    ><span v-if="token.kind" :class="`syntax-${token.kind}`">{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                  </div>
                </div>
              </div>
            </section>
          </div>

          <div class="merge-editor-footer">
            <div class="merge-accept-actions">
              <button class="tool-button" :disabled="operations.loading" @click="acceptConflictSide(mergeCurrentSide)">
                接受左侧
              </button>
              <button class="tool-button" :disabled="operations.loading" @click="acceptConflictSide(mergeIncomingSide)">
                接受右侧
              </button>
            </div>
            <div class="merge-save-actions">
              <button class="tool-button" :disabled="operations.loading" @click="resetConflictResultDraft">取消</button>
              <button
                class="tool-button primary"
                :disabled="operations.loading || resultHasConflictMarkers"
                @click="saveConflictResult(true)"
              >
                <Check :size="14" />
                <span>应用</span>
              </button>
            </div>
          </div>
        </div>
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

        <div class="project-editor">
          <div v-if="project.contentLoading" class="diff-empty">加载中</div>
          <div v-else-if="project.content?.binary" class="diff-empty">
            二进制文件，大小 {{ formatBytes(project.content.size) }}
          </div>
          <div v-else-if="!project.content" class="diff-empty">选择一个项目文件</div>
          <div v-else class="project-edit-pane">
            <div class="project-editor-render" :style="projectEditorRenderStyle" aria-hidden="true"><div class="project-editor-render-content" :style="projectEditorRenderContentStyle">
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
                :key="hunk.id"
                class="project-change-marker"
                :class="[hunk.tone, { expanded: expandedProjectHunk?.id === hunk.id }]"
                :style="projectEditorHunkMarkerStyle(hunk)"
                type="button"
                :title="projectEditorHunkTitle(hunk)"
                @click="toggleProjectEditorHunk(hunk.id)"
              >
                <ChevronDown v-if="expandedProjectHunk?.id === hunk.id" :size="12" />
                <ChevronRight v-else :size="12" />
              </button>

              <div
                v-if="expandedProjectHunk"
                class="project-original-panel project-original-popover"
                :class="expandedProjectHunk.tone"
                :style="projectEditorOriginalPanelStyle(expandedProjectHunk)"
                @mousedown.stop
              >
                <div class="project-original-gutter">
                  {{ expandedProjectHunk.changedOldStart ?? expandedProjectHunk.oldStart }}
                </div>
                <div class="project-original-card">
                  <div class="project-original-toolbar">
                    <span>{{ projectEditorHunkTitle(expandedProjectHunk) }}</span>
                    <div class="project-original-actions">
                      <button
                        class="icon-button danger"
                        type="button"
                        :disabled="project.contentLoading || project.contentSaving"
                        @click="discardProjectEditorHunk(expandedProjectHunk)"
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
                      :class="line.tone"
                    >
                      <span class="line-number">{{ line.lineNumber }}</span>
                      <span class="line-content"><template
                        v-for="(token, tokenIndex) in line.tokens"
                        :key="tokenIndex"
                      ><span
                        v-if="token.kind || token.diff"
                        :class="[
                          token.kind ? `syntax-${token.kind}` : '',
                          token.diff ? 'project-original-diff-fragment' : '',
                          token.insertMarker ? 'insert-marker' : '',
                        ]"
                      >{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
                    </div>
                  </div>
                  <div v-else class="project-original-empty" :class="expandedProjectHunk.tone">原本没有内容，这一块是新增行</div>
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

        <div ref="changeDiffScroller" class="diff-scroller side-by-side-scroller">
          <div v-if="diff.loading" class="diff-empty">加载中</div>
          <div v-else-if="!activeChangeDiffHasContent" class="diff-empty">没有差异</div>
          <div v-else-if="activeChangeSideBySideDiffRows.length === 0" class="diff-empty">无法以文本方式显示此差异</div>
          <DiffViewer
            v-else
            v-memo="[diff.current?.text, activeChangeDiffHunkIndex, settings.selectedSide, changes.selectedFile]"
            row-key-prefix="change"
            :rows="activeChangeSideBySideDiffRows"
            :active-hunk-index="activeChangeDiffHunkIndex"
            :left-label="changeDiffLeftLabel"
            :left-detail="changeDiffLeftDetail"
            :right-label="changeDiffRightLabel"
            :right-detail="changeDiffRightDetail"
            @scroll="syncSideBySideEditorScroll"
          />
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
              <div class="log-view-toggle" role="group" aria-label="历史视图">
                <button
                  class="icon-only-button"
                  :class="{ active: logViewMode === 'list' }"
                  title="列表视图"
                  @click="logViewMode = 'list'"
                >
                  <ListChecks :size="14" />
                </button>
                <button
                  class="icon-only-button"
                  :class="{ active: logViewMode === 'dag' }"
                  title="DAG 视图"
                  @click="logViewMode = 'dag'"
                >
                  <GitCommitVertical :size="14" />
                </button>
              </div>
              <button
                class="icon-only-button"
                :class="actionButtonClass('history.refresh')"
                title="刷新日志"
                :disabled="history.loading"
                :aria-busy="isUiActionPending('history.refresh')"
                @click="runUiAction('history.refresh', () => history.refresh())"
              >
                <component
                  :is="actionIcon('history.refresh', RefreshCw)"
                  :class="actionIconClass('history.refresh')"
                  :size="14"
                />
              </button>
            </div>

            <template v-if="logViewMode === 'list'">
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
            </template>

            <div v-else class="log-dag-view">
              <button
                v-for="row in logGraphRows"
                :key="`dag-${row.item.oid}`"
                class="log-dag-row"
                :class="{ active: history.selectedOid === row.item.oid }"
                @click="selectCommit(row.item.oid)"
              >
                <span class="log-graph-cell log-dag-graph" :class="{ merge: row.hasMerge }" :style="logGraphStyle(row)">
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
                <span class="log-dag-card">
                  <span class="log-dag-title">
                    <strong>{{ row.item.summary }}</strong>
                    <small>{{ row.item.shortOid }}</small>
                  </span>
                  <span v-if="row.item.refs.length" class="commit-refs">
                    <em v-for="refName in row.item.refs" :key="refName">{{ formatRefName(refName) }}</em>
                  </span>
                  <span class="log-dag-meta">
                    {{ row.item.authorName }} · {{ formatCompactCommitTime(row.item.authorTime) }} ·
                    {{ row.item.parents.length > 1 ? `${row.item.parents.length} 父提交` : row.item.parents.length === 1 ? "1 父提交" : "根提交" }}
                  </span>
                </span>
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
              </div>
              <div v-else class="diff-empty">选择一个提交</div>
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
              <DiffViewer
                v-else
                v-memo="[activeLogDiffTab?.id, activeLogDiffTab?.diff?.text, activeLogDiffHunkIndex]"
                row-key-prefix="log"
                :rows="activeLogSideBySideDiffRows"
                :active-hunk-index="activeLogDiffHunkIndex"
                left-label="提交"
                :left-detail="activeLogDiffTab?.shortOid"
                right-label="来源"
                :right-detail="activeLogDiffTab?.subtitle"
                @scroll="syncSideBySideEditorScroll"
              />
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
          :placeholder="projectNameDialog.placeholder"
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

    <div v-if="addRemoteDialog" class="modal-backdrop" @click.self="cancelAddRemoteDialog">
      <form class="add-remote-modal" role="dialog" aria-modal="true" aria-label="定义远程" @submit.prevent="submitAddRemoteDialog">
        <header>
          <h2>定义远程</h2>
          <button
            class="icon-only-button"
            type="button"
            title="关闭"
            :disabled="addRemoteDialog.loading"
            @click="cancelAddRemoteDialog"
          >
            <X :size="14" />
          </button>
        </header>
        <label class="add-remote-field">
          <span>名称:</span>
          <input
            v-model="addRemoteDialog.name"
            autofocus
            type="text"
            :disabled="addRemoteDialog.loading"
            @input="addRemoteDialog.error = ''"
            @keydown.esc.prevent="cancelAddRemoteDialog"
          />
        </label>
        <label class="add-remote-field">
          <span>URL:</span>
          <input
            v-model="addRemoteDialog.url"
            type="text"
            :disabled="addRemoteDialog.loading"
            @input="addRemoteDialog.error = ''"
            @keydown.esc.prevent="cancelAddRemoteDialog"
          />
        </label>
        <label class="add-remote-check">
          <input v-model="addRemoteDialog.fetchAfterSave" type="checkbox" :disabled="addRemoteDialog.loading" />
          <span>抓取远程</span>
        </label>
        <p v-if="addRemoteDialog.error" class="project-name-error">{{ addRemoteDialog.error }}</p>
        <footer>
          <button class="icon-button" type="button" :disabled="addRemoteDialog.loading" @click="cancelAddRemoteDialog">
            <X :size="14" />
            <span>取消</span>
          </button>
          <button
            class="icon-button primary"
            type="submit"
            :disabled="addRemoteDialog.loading || !addRemoteDialog.name.trim() || !addRemoteDialog.url.trim()"
          >
            <LoaderCircle v-if="addRemoteDialog.loading" class="button-spinner" :size="14" />
            <Check v-else :size="14" />
            <span>确定</span>
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
      <template v-if="logRefContextMenu.kind === 'remoteDirectory'">
        <button @click="addRemoteFromLogRefContext(logRefContextMenu)">
          <span>添加远程仓库</span>
          <small>{{ nextAvailableRemoteName(logRefContextMenu.remoteName || "origin") }}</small>
        </button>
      </template>
      <template v-else>
      <button @click="showLogRefFromContext(logRefContextMenu)">
        <span>查看此引用日志</span>
      </button>
      <button :disabled="!canCheckoutLogRefContext(logRefContextMenu)" @click="checkoutLogRefFromContext(logRefContextMenu)">
        <span>{{ logRefContextMenu.kind === "tag" ? "检出标签" : logRefContextMenu.kind === "remote" ? "检出远程分支" : "切换到此分支" }}</span>
      </button>
      <button @click="createBranchFromLogRefContext(logRefContextMenu)">
        <span>从此处新建分支</span>
      </button>

      <div class="context-menu-separator" />
      <button
        v-if="logRefContextMenu.kind !== 'tag'"
        :disabled="!canMergeOrRebaseLogRefContext(logRefContextMenu)"
        @click="previewLogRefMerge(logRefContextMenu)"
      >
        <span>预览合并</span>
      </button>
      <button
        v-if="logRefContextMenu.kind !== 'tag'"
        :disabled="!canMergeOrRebaseLogRefContext(logRefContextMenu)"
        @click="mergeLogRefIntoCurrent(logRefContextMenu)"
      >
        <span>合并到当前分支</span>
      </button>
      <button
        v-if="logRefContextMenu.kind !== 'tag'"
        :disabled="!canMergeOrRebaseLogRefContext(logRefContextMenu)"
        @click="rebaseCurrentOntoLogRef(logRefContextMenu)"
      >
        <span>变基当前分支到此处</span>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'remote'"
        :disabled="!canSetLogRefContextUpstream(logRefContextMenu)"
        @click="setCurrentBranchUpstreamFromContext(logRefContextMenu)"
      >
        <span>设为当前分支上游</span>
      </button>

      <div class="context-menu-separator" />
      <button @click="toggleLogRefFavoriteFromContext(logRefContextMenu)">
        <span>{{ isLogRefContextFavorite(logRefContextMenu) ? "取消收藏" : "收藏" }}</span>
      </button>
      <button @click="copyLogRefNameFromContext(logRefContextMenu)">
        <span>复制引用名称</span>
      </button>

      <div class="context-menu-separator" />
      <button
        v-if="canRenameLogRefContext(logRefContextMenu)"
        @click="renameLogBranchFromContext(logRefContextMenu)"
      >
        <span>重命名分支</span>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'tag'"
        :disabled="branches.loading || !remote.selectedRemote"
        @click="pushLogTagFromContext(logRefContextMenu)"
      >
        <span>推送标签</span>
      </button>
      <button
        v-if="logRefContextMenu.kind === 'tag'"
        :disabled="branches.loading || !remote.selectedRemote"
        @click="deleteRemoteLogTagFromContext(logRefContextMenu)"
      >
        <span>删除远程标签</span>
      </button>
      <button
        class="danger-menu-item"
        :disabled="!canDeleteLogRefContext(logRefContextMenu)"
        @click="deleteLogRefFromContext(logRefContextMenu)"
      >
        <span>{{ logRefContextMenu.kind === "tag" ? "删除本地标签" : logRefContextMenu.kind === "remote" ? "删除远程分支" : "删除本地分支" }}</span>
      </button>
      </template>
    </div>

    <div
      v-if="logFileContextMenu"
      class="context-menu log-file-menu"
      :style="{ left: `${logFileContextMenu.x}px`, top: `${logFileContextMenu.y}px` }"
      @click.stop
    >
      <button @click="showCommitFileDiff(logFileContextMenu.row)">
        <span>显示差异</span>
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
        <span>创建补丁...</span>
      </button>
      <button @click="cherryPickLogFile(logFileContextMenu.row)">
        <span>从此修订版本获取</span>
      </button>
      <button @click="showLogFileHistory(logFileContextMenu.row)">
        <span>查看截至此处的历史记录</span>
      </button>
      <button @click="showCommitFileDiff(logFileContextMenu.row)">
        <span>显示对父项的变更</span>
      </button>
    </div>
  </div>
</template>
