<script setup lang="ts">
import type { Component } from "vue";
import { FolderOpen, GitBranch, GitCommitVertical } from "@lucide/vue";

type WorkbenchMode =
  | "changes"
  | "log"
  | "project"
  | "branches"
  | "remote"
  | "operations"
  | "advanced";

defineProps<{
  mode: WorkbenchMode;
  conflictCount: number;
}>();

const emit = defineEmits<{
  "update:mode": [mode: WorkbenchMode];
}>();

const navItems: Array<{
  key: WorkbenchMode;
  label: string;
  title: string;
  icon: Component;
}> = [
  { key: "changes", label: "提交", title: "提交与变更", icon: GitCommitVertical },
  { key: "log", label: "日志", title: "提交日志", icon: GitBranch },
  { key: "project", label: "项目", title: "项目文件", icon: FolderOpen },
];
</script>

<template>
  <nav class="workbench-rail" aria-label="GitBox 工作台">
    <button
      v-for="item in navItems"
      :key="item.key"
      class="rail-button"
      :class="{ active: mode === item.key }"
      type="button"
      :title="item.title"
      @click="emit('update:mode', item.key)"
    >
      <component :is="item.icon" :size="18" />
      <span>{{ item.label }}</span>
    </button>
  </nav>
</template>
