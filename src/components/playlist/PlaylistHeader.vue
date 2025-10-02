<template>
  <th
    class="group cursor-pointer select-none bg-base-200 hover:bg-base-300 transition-colors !bg-opacity-100"
    @click="sortable ? onSort() : () => {}"
  >
    <div class="flex items-center gap-2">
      <slot />
      <div v-if="sortable" class="text-xs opacity-50 group-hover:opacity-100">
        <span v-if="sortDirection === 'asc'">↑</span>
        <span v-else-if="sortDirection === 'desc'">↓</span>
      </div>
    </div>
  </th>
</template>

<script setup lang="ts">
import { defineProps } from 'vue'

defineProps<{
  sortable: boolean
  sortDirection: 'asc' | 'desc' | null
  onSort: () => void
}>()
</script>

<style scoped>
th {
  position: sticky;
  top: 0;
  z-index: 20;
  padding: 0.75rem 1rem;
  font-weight: 600;
  background-color: hsl(var(--b2));
  box-shadow: 0 1px 0 0 hsl(var(--bc)/0.1);
}
</style>
