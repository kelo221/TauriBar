<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from '@tauri-apps/api/core'

// Store UI volume as 0-100 for the slider, map to 0.0-1.0 for backend
const volume = ref(100);

watch(volume, (val) => {
  const v = Math.max(0, Math.min(100, Number(val)));
  // Map to 0.0 - 1.0
  const backendVol = v / 100;
  invoke('set_volume', { volume: backendVol }).catch(console.error)
});
</script>

<template>
  <div class="cs-slider" style="width:150px">
    <input
      type="range"
      min="0"
      max="100"
      step="1"
      v-model="volume"
    />
    <div class="value">
      <span>Vol</span>
      <span>{{ Math.round(Number(volume)) }}%</span>
    </div>
  </div>
</template>
