<script setup lang="ts">
import { ref } from 'vue';

const progress = ref<ProgressBarState['progress']>(40);
</script>

<template>
  <div class="cs-slider progress-slider">
    <input type="range" min="0" max="1000" v-model="progress" />
  </div>
</template>

<style scoped>
.progress-slider {
  width: 100% !important;
}

.progress-slider input {
  width: 100% !important;
  height: 8px !important;
}

.progress-slider input::-webkit-slider-thumb {
  height: 20px !important;
  width: 10px !important;
}

.progress-slider input::-moz-range-thumb {
  height: 20px !important;
  width: 10px !important;
}
</style>
