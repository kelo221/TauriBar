<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core'
import { usePlaylistStore } from '../../stores/playlistStore'
import { storeToRefs } from 'pinia'

// UI state
const sliderMax = 1000
const progress = ref<ProgressBarState['progress']>(0);
const isDragging = ref(false)

// Backend-derived state
const durationSeconds = ref(0)
const isPlaying = ref(false)

const playlist = usePlaylistStore()
const { lastPlayedSongId } = storeToRefs(playlist)
let timer: number | null = null

async function pollState() {
  try {
    const st = await invoke<{ position_seconds: number; duration_seconds: number; is_playing: boolean; finished: boolean }>('get_playback_state')
    durationSeconds.value = st.duration_seconds || 0
    isPlaying.value = !!st.is_playing
    if (!isDragging.value && durationSeconds.value > 0) {
      // map position to slider range
      const ratio = Math.max(0, Math.min(1, st.position_seconds / durationSeconds.value))
      progress.value = Math.round(ratio * sliderMax)
    }
    // Auto-advance when finished and not playing
    if (st.finished && !st.is_playing) {
      const { playNext } = playlist
      if (typeof playNext === 'function') {
        playNext()
      }
    }
  } catch (e) {
    // ignore transient errors
  }
}

function startPolling() {
  stopPolling()
  timer = window.setInterval(pollState, 500) as unknown as number
}

function stopPolling() {
  if (timer !== null) {
    window.clearInterval(timer as unknown as number)
    timer = null
  }
}

onMounted(() => {
  startPolling()
})

onBeforeUnmount(() => {
  stopPolling()
})

// Reset slider on song change
watch(lastPlayedSongId, () => {
  progress.value = 0
})

function handleInput() {
  // user is dragging, avoid UI being overridden by poll
  isDragging.value = true
}

function handleChange(e: Event) {
  const input = e.target as HTMLInputElement
  const val = Number(input.value)
  if (!Number.isFinite(val)) return
  const ratio = Math.max(0, Math.min(1, val / sliderMax))
  const seconds = (durationSeconds.value > 0 ? durationSeconds.value * ratio : val)
  invoke('seek_to', { seconds }).catch(console.error)
  // allow poll to resume control after a short delay
  setTimeout(() => { isDragging.value = false }, 150)
}
</script>

<template>
  <div class="cs-slider progress-slider">
    <input type="range" min="0" :max="sliderMax" v-model="progress" @input="handleInput" @change="handleChange" />
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
