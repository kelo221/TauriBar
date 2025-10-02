<script setup lang="ts">
import {useStatStore} from "../../stores/statStore";
import {computed} from "vue";

const statStore = useStatStore();
const currentBottomStats = computed(() => statStore.stats);
</script>

<template>
  <footer class="statusbar">
    <span>{{ currentBottomStats.codec }} | {{ currentBottomStats.bitrate }} | {{ currentBottomStats.frequency }} | {{ currentBottomStats.channelType }} | {{ currentBottomStats.playtime }}</span>
  </footer>
</template>

<style>
.statusbar {
  padding: 2px 6px;
  border-top: 1px solid var(--border-dark);
  background: var(--bg);
  font-size: 0.85rem;
}
</style>
