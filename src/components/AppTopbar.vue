<script setup lang="ts">
import {
  Columns3,
  FolderOpen,
  GitBranch,
  Monitor,
  Moon,
  RefreshCw,
  RotateCcw,
  Sun,
} from "@lucide/vue";
import { useSettingsStore } from "../stores/settings";
import type { LayoutPanelKey, ThemeMode } from "../stores/settings";

defineProps<{
  brandSubtitle: string;
  hasRepository: boolean;
  currentBranch: string;
  ahead: number;
  behind: number;
}>();

const emit = defineEmits<{
  addRepository: [];
  refresh: [];
}>();

const settings = useSettingsStore();
const themeModes: Array<{ key: ThemeMode; label: string; title: string }> = [
  { key: "dark", label: "暗黑", title: "使用暗黑主题" },
  { key: "light", label: "浅色", title: "使用浅色主题" },
  { key: "system", label: "系统", title: "跟随系统外观" },
];
const layoutPanels: Array<{ key: LayoutPanelKey; label: string }> = [
  { key: "project", label: "项目栏" },
  { key: "repo", label: "仓库上下文" },
  { key: "changes", label: "工作区上下文" },
];

function setPanelVisibility(panel: LayoutPanelKey, event: Event) {
  settings.setPanelVisible(panel, (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <header class="topbar">
    <div class="brand">
      <img class="brand-mark" src="../assets/gitbox-logo.svg" alt="" />
      <div class="brand-copy">
        <strong>GitBox</strong>
        <span>{{ brandSubtitle }}</span>
      </div>
      <div v-if="hasRepository" class="topbar-state">
        <GitBranch :size="14" />
        <span>{{ currentBranch }}</span>
        <small>领先 {{ ahead }} / 落后 {{ behind }}</small>
      </div>
    </div>

    <div class="toolbar">
      <button class="tool-button primary" title="添加项目" @click="emit('addRepository')">
        <FolderOpen :size="16" />
        <span>添加</span>
      </button>
      <button class="tool-button" title="刷新状态" :disabled="!hasRepository" @click="emit('refresh')">
        <RefreshCw :size="16" />
        <span>刷新</span>
      </button>
      <details class="layout-menu">
        <summary class="tool-button layout-summary" title="栏位设置">
          <Columns3 :size="16" />
          <span>栏位</span>
        </summary>
        <div class="layout-popover">
          <label v-for="panel in layoutPanels" :key="panel.key" class="layout-option">
            <input
              type="checkbox"
              :checked="settings.panelVisibility[panel.key]"
              @change="setPanelVisibility(panel.key, $event)"
            />
            <span>{{ panel.label }}</span>
          </label>
          <button class="layout-reset" type="button" @click="settings.resetLayout">
            <RotateCcw :size="14" />
            <span>重置栏位</span>
          </button>
        </div>
      </details>
      <div class="theme-switch" role="group" aria-label="主题">
        <button
          v-for="mode in themeModes"
          :key="mode.key"
          class="theme-option"
          :class="{ active: settings.themeMode === mode.key }"
          type="button"
          :title="mode.title"
          @click="settings.setThemeMode(mode.key)"
        >
          <Moon v-if="mode.key === 'dark'" :size="14" />
          <Sun v-else-if="mode.key === 'light'" :size="14" />
          <Monitor v-else :size="14" />
          <span>{{ mode.label }}</span>
        </button>
      </div>
    </div>
  </header>
</template>
