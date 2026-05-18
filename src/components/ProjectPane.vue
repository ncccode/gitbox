<script setup lang="ts">
import { Download, FolderOpen, ListChecks, Plus, X } from "@lucide/vue";
import { useAdvancedStore } from "../stores/advanced";
import { useRepositoriesStore } from "../stores/repositories";

const emit = defineEmits<{
  chooseRepository: [];
  cloneRepository: [];
  initRepository: [];
  removeRepository: [path: string];
  switchRepository: [path: string];
}>();

const repos = useRepositoriesStore();
const advanced = useAdvancedStore();
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
          v-for="repo in repos.items"
          :key="repo.path"
          class="project-row"
          :class="{ active: repo.path === repos.path }"
        >
          <button class="project-switch" :title="repo.path" @click="emit('switchRepository', repo.path)">
            <span class="project-dot" />
            <span class="project-copy">
              <strong>{{ repos.projectName(repo.path) }}</strong>
              <small>{{ repo.path }}</small>
            </span>
          </button>
          <button class="project-remove" title="移除项目" @click="emit('removeRepository', repo.path)">
            <X :size="14" />
          </button>
        </div>
      </div>

      <button v-else class="add-project-empty" @click="emit('chooseRepository')">
        <FolderOpen :size="16" />
        <span>添加项目</span>
      </button>

      <div class="quick-create">
        <div class="branch-group-label">克隆 / 初始化</div>
        <input v-model="advanced.cloneUrl" placeholder="仓库地址" />
        <input v-model="advanced.cloneDirectory" placeholder="目标目录，例如 /Users/ncc/work/repo" />
        <input v-model.number="advanced.cloneDepth" type="number" min="0" placeholder="克隆深度，可空" />
        <button
          class="icon-button"
          :disabled="advanced.loading || !advanced.cloneUrl.trim() || !advanced.cloneDirectory.trim()"
          @click="emit('cloneRepository')"
        >
          <Download :size="14" />
          <span>克隆</span>
        </button>
        <input v-model="advanced.initDirectory" placeholder="初始化目录" />
        <div class="operation-options">
          <label title="创建裸仓库"><input v-model="advanced.initBare" type="checkbox" /> 裸仓库</label>
          <input v-model="advanced.initInitialBranch" placeholder="初始分支" />
        </div>
        <button
          class="icon-button"
          :disabled="advanced.loading || !advanced.initDirectory.trim()"
          @click="emit('initRepository')"
        >
          <Plus :size="14" />
          <span>初始化</span>
        </button>
      </div>
    </section>
  </aside>
</template>
