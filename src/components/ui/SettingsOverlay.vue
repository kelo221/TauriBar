<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

function onKeydown(event: KeyboardEvent) {
  if (!settingsStore.isOpen) return
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
</script>

<template>
  <div v-if="settingsStore.isOpen" class="overlay" @mousedown.self="close">
    <div class="panel">
      <div class="title">Settings</div>

      <div class="content">
        <!-- Placeholder settings; extend as needed -->
        <div class="section">
          <div class="section-title">General</div>
          <div class="row">
            <label><input type="checkbox" /> Enable notifications</label>
          </div>
          <div class="row">
            <label><input type="checkbox" /> Start playback on launch</label>
          </div>
        </div>

        <div class="section">
          <div class="section-title">Appearance</div>
          <div class="row">
            <label><input type="checkbox" /> High contrast theme</label>
          </div>
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
</style>
