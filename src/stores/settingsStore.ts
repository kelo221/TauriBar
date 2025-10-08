import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { deserializeCombo, serializeCombo } from '../utils/keyboard'

export const useSettingsStore = defineStore('settings', () => {
  const isOpen = ref(false)
  const captureTarget = ref<null | 'playPause' | 'stop' | 'previous' | 'next' | 'random'>(null)
  const globalShortcutsEnabled = ref(true)
  const replayGainMode = ref<'off' | 'track' | 'album'>('off')

  // Defaults
  const shortcuts = ref<ShortcutMap>({
    playPause: { key: 'Space', alt: false, ctrl: false, meta: false, shift: false },
    stop: { key: 'S', alt: false, ctrl: false, meta: false, shift: false },
    previous: { key: 'ArrowLeft', alt: false, ctrl: false, meta: false, shift: false },
    next: { key: 'ArrowRight', alt: false, ctrl: false, meta: false, shift: false },
    random: { key: 'R', alt: false, ctrl: false, meta: false, shift: false },
  })

  // Load from localStorage if present
  try {
    const globalEnabled = localStorage.getItem('tb.globalShortcutsEnabled')
    if (globalEnabled != null) {
      globalShortcutsEnabled.value = globalEnabled === '1' || globalEnabled === 'true'
    }
    const rg = localStorage.getItem('tb.replayGainMode')
    if (rg === 'off' || rg === 'track' || rg === 'album') {
      replayGainMode.value = rg
    }
    const stored = localStorage.getItem('tb.shortcuts')
    if (stored) {
      const obj = JSON.parse(stored) as Partial<Record<keyof ShortcutMap, string>>
      const next: any = { ...shortcuts.value }
      for (const k of Object.keys(next) as (keyof ShortcutMap)[]) {
        const raw = obj[k]
        const parsed = deserializeCombo(raw as any)
        if (parsed) next[k] = parsed
      }
      shortcuts.value = next
    }
  } catch {}

  // Persist
  watch(shortcuts, (val) => {
    try {
      const payload: Record<string, string> = {}
      for (const [k, v] of Object.entries(val)) payload[k] = serializeCombo(v as any)
      localStorage.setItem('tb.shortcuts', JSON.stringify(payload))
    } catch {}
  }, { deep: true })

  watch(globalShortcutsEnabled, (val) => {
    try {
      localStorage.setItem('tb.globalShortcutsEnabled', val ? '1' : '0')
    } catch {}
  })

  watch(replayGainMode, (val) => {
    try {
      localStorage.setItem('tb.replayGainMode', val)
    } catch {}
  })

  function open() {
    isOpen.value = true
  }

  function close() {
    isOpen.value = false
  }

  function beginCapture(target: 'playPause' | 'stop' | 'previous' | 'next' | 'random' | null) {
    captureTarget.value = target
  }

  function assignShortcut(action: 'playPause' | 'stop' | 'previous' | 'next' | 'random', combo: KeyCombo) {
    shortcuts.value = { ...shortcuts.value, [action]: combo }
  }

  function clearShortcut(action: 'playPause' | 'stop' | 'previous' | 'next' | 'random') {
    const existing = shortcuts.value[action]
    shortcuts.value = { ...shortcuts.value, [action]: { ...existing, key: '' } }
  }

  return { isOpen, open, close, shortcuts, captureTarget, beginCapture, assignShortcut, clearShortcut, globalShortcutsEnabled, replayGainMode }
})
