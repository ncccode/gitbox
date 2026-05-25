<script setup lang="ts">
import type { SideBySideDiffRow } from "../composables/useDiffRows";

defineProps<{
  rows: SideBySideDiffRow[];
  activeHunkIndex: number | null;
  leftLabel: string;
  leftDetail?: string | null;
  rightLabel: string;
  rightDetail?: string | null;
  rowKeyPrefix?: string;
}>();

const emit = defineEmits<{
  scroll: [event: Event];
}>();
</script>

<template>
  <div class="side-by-side-diff">
    <div class="side-by-side-file-header">
      <div class="side-by-side-title">
        <strong>{{ leftLabel }}</strong>
        <span>{{ leftDetail }}</span>
      </div>
      <div class="side-by-side-title">
        <strong>{{ rightLabel }}</strong>
        <span>{{ rightDetail }}</span>
      </div>
    </div>
    <div class="side-by-side-editors">
      <div class="side-by-side-column old" @scroll="emit('scroll', $event)">
        <div class="side-by-side-column-lines">
          <div
            v-for="row in rows"
            :key="`${rowKeyPrefix ?? 'diff'}-old-${row.id}`"
            class="side-by-side-line"
            :class="[
              row.type,
              { active: row.hunkIndex !== null && row.hunkIndex === activeHunkIndex },
            ]"
            :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
          >
            <div class="diff-cell old" :class="row.old.type">
              <span class="line-number">{{ row.old.lineNumber ?? "" }}</span>
              <span class="line-content"><template
                v-for="(token, tokenIndex) in row.old.tokens"
                :key="tokenIndex"
              ><span
                v-if="token.kind || token.diff"
                :class="[token.kind ? `syntax-${token.kind}` : '', token.diff ? 'word-diff-fragment' : '']"
              >{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
            </div>
          </div>
        </div>
      </div>
      <div class="side-by-side-column new" @scroll="emit('scroll', $event)">
        <div class="side-by-side-column-lines">
          <div
            v-for="row in rows"
            :key="`${rowKeyPrefix ?? 'diff'}-new-${row.id}`"
            class="side-by-side-line"
            :class="[
              row.type,
              { active: row.hunkIndex !== null && row.hunkIndex === activeHunkIndex },
            ]"
            :data-hunk-anchor="row.anchorHunkIndex ?? undefined"
          >
            <div class="diff-cell new" :class="row.new.type">
              <span class="line-number">{{ row.new.lineNumber ?? "" }}</span>
              <span class="line-content"><template
                v-for="(token, tokenIndex) in row.new.tokens"
                :key="tokenIndex"
              ><span
                v-if="token.kind || token.diff"
                :class="[token.kind ? `syntax-${token.kind}` : '', token.diff ? 'word-diff-fragment' : '']"
              >{{ token.text }}</span><template v-else>{{ token.text }}</template></template></span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
