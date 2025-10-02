import {defineStore} from 'pinia'
import {ref} from 'vue'

export const useStatStore = defineStore('stats', () => {
  const stats = ref<BottomStats>({
    codec: "WavPack",
    bitrate: "998 kbps",
    frequency: "44100 Hz",
    channelType: "stereo",
    playtime: "1:58 / 4:14",
  })

  function setStats(newStats: BottomStats) {
    stats.value = newStats
  }

  return { stats, setStats }
})
