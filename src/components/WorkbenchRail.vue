<script setup lang="ts">
import {
  ArchiveRestore,
  Download,
  GitBranch,
  GitCommitVertical,
  ListChecks,
  RotateCcw,
} from "@lucide/vue";

type WorkbenchMode = "changes" | "log" | "branches" | "remote" | "operations" | "advanced";

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
  icon: typeof ListChecks;
}> = [
  { key: "changes", label: "变更", title: "变更与提交", icon: ListChecks },
  { key: "log", label: "日志", title: "提交日志", icon: GitCommitVertical },
  { key: "branches", label: "分支", title: "分支与标签", icon: GitBranch },
  { key: "remote", label: "远程", title: "远程与同步", icon: Download },
  { key: "operations", label: "操作", title: "合并、变基与冲突", icon: RotateCcw },
  { key: "advanced", label: "高级", title: "高级 Git 工具", icon: ArchiveRestore },
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
      <small v-if="item.key === 'operations' && conflictCount > 0">{{ conflictCount }}</small>
    </button>
  </nav>
</template>
