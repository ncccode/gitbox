<script setup lang="ts">
import {
  Monitor,
  Moon,
  Search,
  Sun,
} from "@lucide/vue";
import VcsIcon from "./icons/VcsIcon.vue";
import { useSettingsStore } from "../stores/settings";
import type { ThemeMode } from "../stores/settings";

defineProps<{
  brandSubtitle: string;
  hasRepository: boolean;
  currentBranch: string;
  remoteBranch?: string | null;
  ahead: number;
  behind: number;
}>();

const emit = defineEmits<{
  openCommandPalette: [];
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
        <VcsIcon :size="14" />
        <span class="topbar-branch-local" :title="currentBranch">{{ currentBranch }}</span>
        <small class="topbar-branch-remote" :title="remoteBranch ? `远程 ${remoteBranch}` : '未设置远程分支'">
          远程 {{ remoteBranch || "未设置" }}
        </small>
        <small class="topbar-sync-state">领先 {{ ahead }} / 落后 {{ behind }}</small>
      </div>
    </div>

    <div class="toolbar">
      <button class="command-open-button" type="button" title="命令面板" @click="emit('openCommandPalette')">
        <Search :size="14" />
      </button>
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
