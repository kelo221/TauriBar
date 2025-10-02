<script setup lang="ts">
import { ref } from "vue";
const volume = ref(1);
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
