import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getName, getVersion } from '@tauri-apps/api/app'

export const useAppStore = defineStore('app', () => {
  const appName = ref<string>('')
  const appVersion = ref<string>('')
  const isLoaded = ref(false)

  async function loadAppInfo() {
    try {
      appName.value = await getName()
      appVersion.value = await getVersion()
    } catch (e) {
      // Fallback values if Tauri API is unavailable
      appName.value = 'tauribar'
      appVersion.value = '0.1.0'
    } finally {
      isLoaded.value = true
    }
  }

  // Eagerly fetch so consumers can just read values
  // without needing to call refresh manually.
  // Safe to fire-and-forget.
  // noinspection JSIgnoredPromiseFromCall
  loadAppInfo()

  const displayNameAndVersion = computed(() => {
    if (!appName.value && !appVersion.value) return ''
    if (appName.value && appVersion.value) return `${appName.value} ${appVersion.value}`
    return appName.value || appVersion.value
  })

  return { appName, appVersion, isLoaded, displayNameAndVersion, refresh: loadAppInfo }
})


