<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { useSearchStore } from '../../stores/searchStore'
import { usePlaylistStore } from '../../stores/playlistStore'

const searchStore = useSearchStore()
const playlistStore = usePlaylistStore()
const inputRef = ref<HTMLInputElement | null>(null)
const focusedIndex = ref(-1)

function dataMatchesQuery(data: AudioFile, q: string): boolean {
  if (!q) return false
  const query = q.toLowerCase()
  const hay = [
    data.artist ?? '',
    data.album ?? '',
    data.title ?? '',
    String(data.track ?? ''),
    data.duration ?? ''
  ].join(' ').toLowerCase()
  return hay.includes(query)
}

const results = computed(() => {
  const q = searchStore.query.trim()
  if (!q) return [] as AudioFile[]
  return playlistStore.currentPlayList.songs.filter(s => dataMatchesQuery(s, q))
})

watch(() => searchStore.isOpen, (open) => {
  if (open) {
    requestAnimationFrame(() => {
      inputRef.value?.focus()
      inputRef.value?.select()
    })
    focusedIndex.value = results.value.length > 0 ? 0 : -1
  }
})

watch(results, (list) => {
  if (list.length === 0) focusedIndex.value = -1
  else if (focusedIndex.value < 0 || focusedIndex.value >= list.length) focusedIndex.value = 0
})

function selectResultAt(index: number) {
  if (index < 0 || index >= results.value.length) return
  const song = results.value[index]
  playlistStore.setSelectedSong(song)
  searchStore.close()
}

function onKeydown(event: KeyboardEvent) {
  if (!searchStore.isOpen) return
  if (event.key === 'Escape') {
    event.preventDefault()
    searchStore.close()
  } else if (event.key === 'Enter') {
    event.preventDefault()
    if (event.shiftKey) {
      // Also support Shift+Enter cycling when overlay open
      if (results.value.length > 0) {
        focusedIndex.value = (focusedIndex.value - 1 + results.value.length) % results.value.length
      }
    } else {
      if (focusedIndex.value >= 0) selectResultAt(focusedIndex.value)
    }
  } else if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (results.value.length > 0) {
      focusedIndex.value = (focusedIndex.value + 1) % results.value.length
    }
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (results.value.length > 0) {
      focusedIndex.value = (focusedIndex.value - 1 + results.value.length) % results.value.length
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown, { capture: true })
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown, { capture: true } as any)
})

function handleInput(e: Event) {
  const target = e.target as HTMLInputElement
  searchStore.setQuery(target.value)
}

function close() {
  searchStore.close()
}

function formatTrack(song: AudioFile): string {
  return song.track.toString().padStart(2, '0')
}
</script>

<template>
  <div v-if="searchStore.isOpen" class="overlay" @mousedown.self="close">
    <div class="panel">
      <div class="title">Playlist Search ({{ results.length }} results)</div>
      <input
        ref="inputRef"
        class="input"
        type="text"
        placeholder="Search…"
        :value="searchStore.query"
        @input="handleInput"
      />

      <div class="table-wrapper" v-if="results.length > 0">
        <table class="playlist-table">
          <colgroup>
            <col />
            <col style="width:80px" />
            <col />
            <col style="width:90px" />
          </colgroup>
          <thead>
          <tr>
            <th class="col-header"><div class="header-inner">Artist/album</div></th>
            <th class="col-header col-track"><div class="header-inner">Track no</div></th>
            <th class="col-header"><div class="header-inner">Title / track artist</div></th>
            <th class="col-header col-duration"><div class="header-inner">Durat…</div></th>
          </tr>
          </thead>
          <tbody>
          <tr v-for="(song, i) in results" :key="song.id" class="row" :class="{ focused: i===focusedIndex }" @click="() => selectResultAt(i)">
            <td class="cell">{{ song.artist }} - {{ song.album }}</td>
            <td class="cell cell-track">{{ formatTrack(song) }}</td>
            <td class="cell">{{ song.title }}</td>
            <td class="cell cell-duration">{{ song.duration }}</td>
          </tr>
          </tbody>
        </table>
      </div>

      <div class="actions">
        <button class="btn" @click="() => { if(results.length){ focusedIndex = (focusedIndex - 1 + results.length) % results.length } }">Prev</button>
        <button class="btn" @click="() => { if(results.length){ focusedIndex = (focusedIndex + 1) % results.length } }">Next</button>
        <button class="btn" @click="() => { if (focusedIndex>=0) selectResultAt(focusedIndex) }">Go</button>
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

.input {
  width: 100%;
  box-sizing: border-box;
  padding: 6px 8px;
  color: var(--text);
  background: var(--playlist-bg);
  border: 1px solid var(--border-dark);
  font-size: 16px;
}

.table-wrapper {
  margin-top: 8px;
  height: min(50vh, 420px);
  overflow: auto;
  background: var(--playlist-bg);
  border: 1px solid var(--border-dark);
  flex: 1;
}

.playlist-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

.col-header {
  position: sticky;
  top: 0;
  z-index: 20;
  background: var(--bg);
  color: var(--text);
  font-weight: 600;
  text-align: left;
  border: 1px solid;
  border-color: var(--border-light) var(--border-dark) var(--border-dark) var(--border-light);
}

.col-header .header-inner {
  padding: 3px 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-track { width: 80px; }
.col-duration { width: 90px; }

.row { cursor: default; }

.cell {
  padding: 1px 4px;
  border-bottom: 1px solid rgba(0,0,0,0.2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  background: var(--playlist-bg);
}

.row:hover .cell { background: rgba(255,255,255,0.06); }
.row.focused .cell { background: #2f362b; }

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


