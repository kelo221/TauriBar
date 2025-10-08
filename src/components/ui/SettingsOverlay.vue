<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import { useSettingsStore } from '../../stores/settingsStore'
import { comboToString, eventToCombo } from '../../utils/keyboard'

const settingsStore = useSettingsStore()

type ShortcutAction = 'playPause' | 'stop' | 'previous' | 'next' | 'random'

function onKeydown(event: KeyboardEvent) {
  if (!settingsStore.isOpen) return
  // If capturing, assign on any non-modifier; Esc cancels capture
  if (settingsStore.captureTarget) {
    event.preventDefault()
    event.stopPropagation()
    if (event.key === 'Escape') {
      settingsStore.beginCapture(null)
      return
    }
    const combo = eventToCombo(event)
    if (combo) {
      settingsStore.assignShortcut(settingsStore.captureTarget, combo)
      settingsStore.beginCapture(null)
    }
    return
  }
  if (event.key === 'Escape') {
    event.preventDefault()
    settingsStore.close()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown, { capture: true })
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown, { capture: true } as any)
})

function close() {
  settingsStore.close()
}

function startCapture(action: ShortcutAction) {
  settingsStore.beginCapture(action)
}

function clear(action: ShortcutAction) {
  settingsStore.clearShortcut(action)
}

function formatShortcut(action: ShortcutAction): string {
  const combo = settingsStore.shortcuts[action]
  return comboToString(combo) || 'Unassigned'
}
</script>

<template>
  <div v-if="settingsStore.isOpen" class="overlay" @mousedown.self="close">
    <div class="panel">
      <div class="title">Settings</div>

      <div class="content">
        <div class="section">
          <div class="row">
            <label class="checkbox">
              <input type="checkbox" v-model="settingsStore.globalShortcutsEnabled" />
              <span>Enable global media shortcuts</span>
            </label>
          </div>
        </div>
        <div class="section">
          <div class="section-title">Keyboard Shortcuts</div>
          <table class="shortcuts">
            <thead>
              <tr>
                <th>Action</th>
                <th>Shortcut</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>Play / Pause</td>
                <td class="keys">
                  <button class="capture" :class="{ active: settingsStore.captureTarget==='playPause' }" @click="startCapture('playPause')">
                    {{ settingsStore.captureTarget==='playPause' ? 'Listening…' : formatShortcut('playPause') }}
                  </button>
                  <button class="clear" @click="clear('playPause')">Clear</button>
                </td>
              </tr>
              <tr>
                <td>Stop</td>
                <td class="keys">
                  <button class="capture" :class="{ active: settingsStore.captureTarget==='stop' }" @click="startCapture('stop')">
                    {{ settingsStore.captureTarget==='stop' ? 'Listening…' : formatShortcut('stop') }}
                  </button>
                  <button class="clear" @click="clear('stop')">Clear</button>
                </td>
              </tr>
              <tr>
                <td>Previous track</td>
                <td class="keys">
                  <button class="capture" :class="{ active: settingsStore.captureTarget==='previous' }" @click="startCapture('previous')">
                    {{ settingsStore.captureTarget==='previous' ? 'Listening…' : formatShortcut('previous') }}
                  </button>
                  <button class="clear" @click="clear('previous')">Clear</button>
                </td>
              </tr>
              <tr>
                <td>Next track</td>
                <td class="keys">
                  <button class="capture" :class="{ active: settingsStore.captureTarget==='next' }" @click="startCapture('next')">
                    {{ settingsStore.captureTarget==='next' ? 'Listening…' : formatShortcut('next') }}
                  </button>
                  <button class="clear" @click="clear('next')">Clear</button>
                </td>
              </tr>
              <tr>
                <td>Play random track</td>
                <td class="keys">
                  <button class="capture" :class="{ active: settingsStore.captureTarget==='random' }" @click="startCapture('random')">
                    {{ settingsStore.captureTarget==='random' ? 'Listening…' : formatShortcut('random') }}
                  </button>
                  <button class="clear" @click="clear('random')">Clear</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="actions">
        <button class="btn" @click="close">Close</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.25);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 40px;
  z-index: 9999;
}

.panel {
  width: clamp(640px, 70vw, 1100px);
  max-width: 90vw;
  background: var(--bg);
  color: var(--text);
  border: 1px solid var(--border-dark);
  box-shadow: 0 4px 16px rgba(0,0,0,0.4);
  padding: 8px;
  display: flex;
  flex-direction: column;
  max-height: 80vh;
}

.title {
  font-size: 12px;
  margin-bottom: 6px;
  opacity: 0.9;
}

.content {
  background: var(--playlist-bg);
  border: 1px solid var(--border-dark);
  padding: 8px;
  overflow: auto;
  max-height: 60vh;
}

.section { margin-bottom: 12px; }
.section-title { font-weight: 600; margin-bottom: 6px; }
.row { margin: 4px 0; }

.actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
  margin-top: 8px;
}

.btn {
  padding: 4px 8px;
  background: var(--secondary-bg);
  color: var(--text);
  border: 1px solid var(--border-dark);
}

/* Shortcuts table */
.shortcuts {
  width: 100%;
  border-collapse: collapse;
}
.shortcuts th,
.shortcuts td {
  padding: 4px 6px;
  border-bottom: 1px solid rgba(0,0,0,0.2);
  text-align: left;
}
.shortcuts .keys kbd {
  display: inline-block;
  padding: 1px 6px;
  border: 1px solid var(--border-dark);
  border-radius: 3px;
  background: var(--bg);
}

.shortcuts .keys {
  display: flex;
  gap: 6px;
}

.capture {
  padding: 2px 8px;
  background: var(--bg);
  color: var(--text);
  border: 1px solid var(--border-dark);
  border-radius: 4px;
}
.capture.active {
  outline: 1px dashed var(--border-light);
}
.clear {
  padding: 2px 6px;
  background: transparent;
  color: var(--text);
  border: 1px solid var(--border-dark);
  border-radius: 4px;
}
</style>
