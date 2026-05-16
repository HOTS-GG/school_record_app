<script setup>
import { computed } from 'vue'
import { diffChars } from 'diff'

const props = defineProps({
  before: { type: String, default: '' },
  after:  { type: String, default: '' },
})

const parts = computed(() => diffChars(props.before, props.after))
</script>

<template>
  <span class="diff-view">
    <template v-for="(part, i) in parts" :key="i">
      <span
        v-if="part.added"
        class="diff-added"
      >{{ part.value }}</span>
      <span
        v-else-if="part.removed"
        class="diff-removed"
      >{{ part.value }}</span>
      <span v-else>{{ part.value }}</span>
    </template>
  </span>
</template>

<style scoped>
.diff-view {
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.diff-added {
  background-color: var(--clr-green-border);
  color: var(--clr-green-text);
}

.diff-removed {
  background-color: var(--clr-red-border);
  color: var(--clr-red-text-light);
  text-decoration: line-through;
}
</style>
