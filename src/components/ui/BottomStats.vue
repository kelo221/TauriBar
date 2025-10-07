<script setup lang="ts">
import {useStatStore} from "../../stores/statStore";
import {computed, onMounted, onBeforeUnmount} from "vue";
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from "../../stores/settingsStore";

const statStore = useStatStore();
const settingsStore = useSettingsStore();
const currentBottomStats = computed(() => statStore.stats);

const displayLine = computed(() => {
  const s = currentBottomStats.value
  if (!s) return ''
  if (s.playtime === 'Playback stopped') return 'Playback stopped'
  const parts = [s.codec, s.bitrate, s.frequency, s.channelType, s.playtime]
    .filter((p) => !!p && String(p).trim() !== '')
  return parts.join(' | ')
})

let timer: number | null = null

async function pollStats() {
  try {
    const st = await invoke<{ position_seconds: number; duration_seconds: number; is_playing: boolean; codec: string; bitrate_kbps: number; sample_rate_hz: number; channels: number; finished: boolean }>('get_playback_state')
    const pos = Math.max(0, Math.floor(st.position_seconds || 0))
    const dur = Math.max(0, Math.floor(st.duration_seconds || 0))
    const mmss = (s: number) => `${Math.floor(s/60)}:${(s%60).toString().padStart(2,'0')}`
    const playtime = dur > 0 ? `${mmss(pos)} / ${mmss(dur)}` : (st.is_playing ? '' : 'Playback stopped')
    const codec = st.codec || ''
    const bitrate = st.bitrate_kbps ? `${st.bitrate_kbps} kbps` : ''
    const frequency = st.sample_rate_hz ? `${st.sample_rate_hz} Hz` : ''
    const channelType = st.channels ? (st.channels === 1 ? 'mono' : 'stereo') : ''
    statStore.setStats({ codec, bitrate, frequency, channelType, playtime })
  } catch (e) {
    // ignore
  }
}

onMounted(() => {
  timer = window.setInterval(pollStats, 500) as unknown as number
})

onBeforeUnmount(() => {
  if (timer !== null) {
    window.clearInterval(timer as unknown as number)
    timer = null
  }
})
</script>

<template>
  <footer class="statusbar">
    <span>{{ displayLine }}</span>
    <button class="settings-btn" @click="settingsStore.open()" title="Settings">
      <img src="../../assets/gear_high_contrast.svg" alt="Settings" />
    </button>
  </footer>
</template>

<style>
.statusbar {
  padding: 2px 6px;
  border-top: 1px solid var(--border-dark);
  background: var(--bg);
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-btn {
  margin-left: auto;
  background: transparent;
  color: var(--text);
  border: 0;
  padding: 2px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
}

.settings-btn img {
  width: 16px;
  height: 16px;
}
</style>
