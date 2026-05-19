<script setup lang="ts">
import { FolderOpen, ListChecks, PanelLeftClose, PanelLeftOpen, Plus, X } from "@lucide/vue";
import { useRepositoriesStore } from "../stores/repositories";

defineProps<{
  collapsed: boolean;
}>();

const emit = defineEmits<{
  chooseRepository: [];
  removeRepository: [path: string];
  switchRepository: [path: string];
  toggleCollapsed: [];
}>();

const repos = useRepositoriesStore();

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
          v-for="project in repos.items"
          :key="project.path"
          class="project-row"
          :class="{ active: project.path === repos.selectedPath, uninitialized: !project.initialized }"
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
    </section>
  </aside>
</template>
