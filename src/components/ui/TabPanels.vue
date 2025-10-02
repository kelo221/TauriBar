<template>
  <div class="tab-content-container">
    <div
      v-for="(tab, i) in tabs"
      :key="tab.id"
      role="tabpanel"
      class="tab-content"
      :class="{ active: i === activeIndex }"
    >
      <Playlist />
    </div>
  </div>
</template>

<script setup lang="ts">
// Update this import path to match your project structure
import Playlist from '../../components/playlist/Playlist.vue'

defineProps<{
  tabs: Tab[]
  activeIndex: number
}>()
</script>

<style scoped>
.tab-content-container {
  flex: 1;
  min-height: 0;
  position: relative;
}

.tab-content {
  display: none;
  height: 100%;
}

.tab-content.active {
  display: block;
}
</style>
