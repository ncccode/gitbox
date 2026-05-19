<script setup lang="ts">
withDefaults(defineProps<{ size?: number | string }>(), {
  size: 18,
});
</script>

<template>
  <!-- Apache 2.0 licensed icon path. -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 16 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <circle cx="4.5" cy="4" r="2" stroke="currentColor" />
    <path d="M4.5 11.5H8.5C9.60457 11.5 10.5 10.6046 10.5 9.5V8" stroke="currentColor" />
    <path d="M4.5 6.5L4.5 14.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" />
    <circle cx="10.5" cy="6" r="2" stroke="currentColor" />
  </svg>
</template>
