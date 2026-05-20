<script setup lang="ts">
import {
  GitBranch,
  Monitor,
  Moon,
  Sun,
} from "@lucide/vue";
import { useSettingsStore } from "../stores/settings";
import type { ThemeMode } from "../stores/settings";

defineProps<{
  brandSubtitle: string;
  hasRepository: boolean;
  currentBranch: string;
  ahead: number;
  behind: number;
}>();

const settings = useSettingsStore();
const themeModes: Array<{ key: ThemeMode; label: string; title: string }> = [
  { key: "dark", label: "深色", title: "使用深色主题" },
  { key: "light", label: "浅色", title: "使用浅色主题" },
  { key: "system", label: "系统", title: "跟随系统外观" },
];
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
