<script setup lang="ts">
import { FolderOpen, ListChecks, Plus, X } from "@lucide/vue";
import { useRepositoriesStore } from "../stores/repositories";

const emit = defineEmits<{
  chooseRepository: [];
  removeRepository: [path: string];
  switchRepository: [path: string];
}>();

const repos = useRepositoriesStore();
</script>

<template>
  <aside class="project-pane">
    <section class="projects-section">
      <div class="section-heading">
        <div class="section-title">
          <ListChecks :size="16" />
          <span>项目</span>
        </div>
        <button class="icon-only-button" title="添加项目" @click="emit('chooseRepository')">
          <Plus :size="15" />
        </button>
      </div>

      <div v-if="repos.items.length" class="project-list">
        <div
          v-for="project in repos.items"
          :key="project.path"
          class="project-row"
          :class="{ active: project.path === repos.selectedPath, uninitialized: !project.initialized }"
        >
          <button class="project-switch" :title="project.path" @click="emit('switchRepository', project.path)">
            <span class="project-dot" />
            <span class="project-copy">
              <strong>{{ repos.projectName(project.path) }}</strong>
              <small>{{ project.path }}</small>
            </span>
          </button>
          <button class="project-remove" title="移除项目" @click="emit('removeRepository', project.path)">
            <X :size="14" />
          </button>
        </div>
      </div>

      <button v-else class="add-project-empty" @click="emit('chooseRepository')">
        <FolderOpen :size="16" />
        <span>添加项目</span>
      </button>
    </section>
  </aside>
</template>
