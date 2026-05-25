<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { FolderOpen, ListChecks, PanelLeftClose, PanelLeftOpen, Plus, Star, X } from "@lucide/vue";
import { openProjectDirectory, openProjectTerminal } from "../lib/gitboxCommands";
import { useRepositoriesStore } from "../stores/repositories";

defineProps<{
  collapsed: boolean;
}>();

const emit = defineEmits<{
  chooseRepository: [];
  removeRepository: [path: string];
  switchRepository: [path: string];
  togglePinned: [path: string];
  toggleCollapsed: [];
}>();

const repos = useRepositoriesStore();
const projectContextMenu = ref<ProjectContextMenu | null>(null);

type ProjectContextMenu = {
  path: string;
  x: number;
  y: number;
};

const avatarPalettes = [
  { backgroundColor: "#3f6ea5", color: "#ffffff" },
  { backgroundColor: "#2f8f6f", color: "#ffffff" },
  { backgroundColor: "#c56f2d", color: "#ffffff" },
  { backgroundColor: "#7a68c6", color: "#ffffff" },
  { backgroundColor: "#b84f67", color: "#ffffff" },
  { backgroundColor: "#2f8fa3", color: "#ffffff" },
  { backgroundColor: "#6f8f32", color: "#ffffff" },
  { backgroundColor: "#a15c99", color: "#ffffff" },
];

function projectInitial(path: string) {
  const name = repos.projectName(path).trim();
  return Array.from(name)[0]?.toUpperCase() ?? "?";
}

function projectTitle(path: string) {
  return `${repos.projectName(path)}\n${path}`;
}

function projectAvatarStyle(path: string) {
  let hash = 0;
  for (const char of path) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return avatarPalettes[hash % avatarPalettes.length];
}

function openProjectContextMenu(path: string, event: MouseEvent) {
  const menuWidth = 260;
  const menuHeight = 128;
  projectContextMenu.value = {
    path,
    x: Math.max(8, Math.min(event.clientX, window.innerWidth - menuWidth - 8)),
    y: Math.max(8, Math.min(event.clientY, window.innerHeight - menuHeight - 8)),
  };
}

function closeProjectContextMenu() {
  projectContextMenu.value = null;
}

function closeProjectContextMenuOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") {
    closeProjectContextMenu();
  }
}

async function runProjectContextAction(action: (path: string) => Promise<unknown>) {
  const menu = projectContextMenu.value;
  if (!menu) return;

  const { path } = menu;
  closeProjectContextMenu();
  repos.error = "";
  try {
    await action(path);
  } catch (error) {
    repos.error = String(error);
  }
}

async function openProjectDirectoryFromContext(path: string) {
  await runProjectContextAction(() => openProjectDirectory(path));
}

async function openProjectTerminalFromContext(path: string) {
  await runProjectContextAction(() => openProjectTerminal(path));
}

async function copyProjectPathFromContext(path: string) {
  closeProjectContextMenu();
  try {
    await navigator.clipboard.writeText(path);
  } catch {
    window.prompt("复制项目路径", path);
  }
}

onMounted(() => {
  window.addEventListener("click", closeProjectContextMenu);
  window.addEventListener("blur", closeProjectContextMenu);
  window.addEventListener("keydown", closeProjectContextMenuOnEscape);
});

onUnmounted(() => {
  window.removeEventListener("click", closeProjectContextMenu);
  window.removeEventListener("blur", closeProjectContextMenu);
  window.removeEventListener("keydown", closeProjectContextMenuOnEscape);
});
</script>

<template>
  <aside class="project-pane" :class="{ collapsed }">
    <section class="projects-section">
      <div class="section-heading">
        <div v-if="!collapsed" class="section-title">
          <ListChecks :size="16" />
          <span>项目</span>
        </div>
        <div class="project-heading-actions">
          <button
            class="icon-only-button project-collapse-button"
            type="button"
            :title="collapsed ? '展开项目栏' : '收起项目栏'"
            :aria-label="collapsed ? '展开项目栏' : '收起项目栏'"
            :aria-pressed="collapsed"
            @click="emit('toggleCollapsed')"
          >
            <PanelLeftOpen v-if="collapsed" :size="15" />
            <PanelLeftClose v-else :size="15" />
          </button>
          <button class="icon-only-button" type="button" title="添加项目" @click="emit('chooseRepository')">
            <Plus :size="15" />
          </button>
        </div>
      </div>

      <div v-if="repos.items.length" class="project-list">
        <div
          v-for="project in repos.quickSwitchItems"
          :key="project.path"
          class="project-row"
          :class="{ active: project.path === repos.selectedPath, uninitialized: !project.initialized, pinned: project.pinned }"
          @contextmenu.prevent.stop="openProjectContextMenu(project.path, $event)"
        >
          <button
            class="project-switch"
            :title="projectTitle(project.path)"
            :aria-label="`切换到项目 ${repos.projectName(project.path)}`"
            :aria-current="project.path === repos.selectedPath ? 'page' : undefined"
            @click="emit('switchRepository', project.path)"
          >
            <span class="project-avatar" :style="projectAvatarStyle(project.path)">
              {{ projectInitial(project.path) }}
            </span>
            <span v-if="!collapsed" class="project-copy">
              <strong>{{ repos.projectName(project.path) }}</strong>
              <small>{{ project.path }}</small>
            </span>
          </button>
          <button
            v-if="!collapsed"
            class="project-pin"
            type="button"
            :title="project.pinned ? '取消固定' : '固定项目'"
            :aria-pressed="Boolean(project.pinned)"
            @click="emit('togglePinned', project.path)"
          >
            <Star :size="13" :fill="project.pinned ? 'currentColor' : 'none'" />
          </button>
          <button
            v-if="!collapsed"
            class="project-remove"
            type="button"
            title="移除项目"
            @click="emit('removeRepository', project.path)"
          >
            <X :size="14" />
          </button>
        </div>
      </div>

      <button v-else class="add-project-empty" type="button" title="添加项目" @click="emit('chooseRepository')">
        <FolderOpen :size="16" />
        <span v-if="!collapsed">添加项目</span>
      </button>

      <div
        v-if="projectContextMenu"
        class="context-menu project-context-menu"
        :style="{ left: `${projectContextMenu.x}px`, top: `${projectContextMenu.y}px` }"
        @click.stop
        @contextmenu.prevent.stop
      >
        <button type="button" @click="openProjectDirectoryFromContext(projectContextMenu.path)">
          <span>在系统文件管理器打开</span>
        </button>
        <button type="button" @click="openProjectTerminalFromContext(projectContextMenu.path)">
          <span>在系统终端打开</span>
        </button>
        <button type="button" @click="copyProjectPathFromContext(projectContextMenu.path)">
          <span>复制路径</span>
        </button>
        <button type="button" @click="emit('togglePinned', projectContextMenu.path); closeProjectContextMenu()">
          <span>{{ repos.items.find((item) => item.path === projectContextMenu?.path)?.pinned ? "取消固定" : "固定项目" }}</span>
        </button>
      </div>
    </section>
  </aside>
</template>
