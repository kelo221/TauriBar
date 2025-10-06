<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { ModuleRegistry, AllCommunityModule, ColDef, GridOptions, GridApi, GridReadyEvent, RowClickedEvent, RowDoubleClickedEvent } from 'ag-grid-community'
import { AgGridVue } from 'ag-grid-vue3'
import { usePlaylistStore } from '../../stores/playlistStore'
import { useSearchStore } from '../../stores/searchStore'
import { useTabStore } from '../../stores/tabStore'
// import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import type { UnlistenFn } from '@tauri-apps/api/event'
import 'ag-grid-community/styles/ag-grid.css'
import 'ag-grid-community/styles/ag-theme-alpine.css'

ModuleRegistry.registerModules([AllCommunityModule])

const playlistStore = usePlaylistStore()
const tabStore = useTabStore()
const searchStore = useSearchStore()

const columnDefs = ref<ColDef<AudioFile>[]>([
  { headerName: 'Artist/album', field: 'artist', resizable: true, flex: 1,
    valueGetter: (p) => `${p.data?.artist ?? ''} - ${p.data?.album ?? ''}` },
  { headerName: 'Track no', field: 'track', resizable: true, width: 90 },
  { headerName: 'Title / track artist', field: 'title', resizable: true, flex: 1 },
  { headerName: 'Durat…', field: 'duration', resizable: true, width: 100 },
])

const rowData = computed(() => playlistStore.currentPlayList.songs)

const defaultColDef: ColDef = {
  sortable: true,
  resizable: true,
  filter: false,
  suppressHeaderMenuButton: true,
}

const gridApi = ref<GridApi<AudioFile> | null>(null)

const matchIds = ref<string[]>([])
const currentMatchIndex = ref<number>(-1)

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

function recomputeMatches(q: string) {
  const api = gridApi.value
  if (!api) return
  const ids: string[] = []
  api.forEachNodeAfterFilterAndSort((node) => {
    const d = node.data
    if (d && dataMatchesQuery(d, q)) {
      ids.push(d.id)
    }
  })
  matchIds.value = ids
  currentMatchIndex.value = ids.length > 0 ? 0 : -1
}

function selectRowById(id: string) {
  const api = gridApi.value
  if (!api) return
  let found = false
  api.forEachNode((node) => {
    const match = node.data && node.data.id === id
    if (match) {
      node.setSelected(true, true)
      api.ensureNodeVisible(node, 'middle')
      found = true
    }
  })
  if (!found) {
    api.deselectAll()
  }
}

function selectRowByIdAndSync(id: string) {
  const api = gridApi.value
  if (!api) return
  let selectedData: AudioFile | null = null
  api.forEachNode((node) => {
    const match = node.data && node.data.id === id
    if (match) {
      node.setSelected(true, true)
      api.ensureNodeVisible(node, 'middle')
      selectedData = node.data as AudioFile
    }
  })
  if (selectedData) {
    playlistStore.setSelectedSong(selectedData)
  } else {
    api.deselectAll()
  }
}

function selectMatchAt(index: number) {
  if (index < 0 || index >= matchIds.value.length) return
  const id = matchIds.value[index]
  currentMatchIndex.value = index
  selectRowByIdAndSync(id)
}

function onGridReady(event: GridReadyEvent<AudioFile>) {
  gridApi.value = event.api
  const id = playlistStore.selectedSong?.id
  if (id) selectRowById(id)
  const q = searchStore.query.trim()
  if (q) {
    recomputeMatches(q)
  }
}

function onRowClicked(event: RowClickedEvent<AudioFile>) {
  if (event.data) {
    playlistStore.setSelectedSong(event.data)
  }
}

function onRowDoubleClicked(event: RowDoubleClickedEvent<AudioFile>) {
  if (event.data) {
    playlistStore.playSong(event.data)
  }
}

watch(() => playlistStore.selectedSong?.id, (id) => {
  if (!id) {
    gridApi.value?.deselectAll()
    return
  }
  selectRowById(id)
})

watch(() => searchStore.query, (q) => {
  const query = q.trim()
  if (!gridApi.value) return
  if (query === '') {
    matchIds.value = []
    currentMatchIndex.value = -1
    return
  }
  recomputeMatches(query)
  if (matchIds.value.length > 0) {
    selectMatchAt(0)
  }
})

watch(() => searchStore.navToken, () => {
  const query = searchStore.query.trim()
  if (!gridApi.value || query === '') return
  if (matchIds.value.length === 0) {
    recomputeMatches(query)
  }
  if (matchIds.value.length === 0) return
  const dir = searchStore.navDirection
  if (currentMatchIndex.value === -1) {
    selectMatchAt(0)
    return
  }
  let nextIndex = currentMatchIndex.value
  if (dir === 'next') {
    nextIndex = (currentMatchIndex.value + 1) % matchIds.value.length
  } else {
    nextIndex = (currentMatchIndex.value - 1 + matchIds.value.length) % matchIds.value.length
  }
  selectMatchAt(nextIndex)
})

const gridOptions: GridOptions<AudioFile> = {
  defaultColDef,
  headerHeight: 24,
  rowHeight: 22,
  suppressDragLeaveHidesColumns: true,
  ensureDomOrder: true,
  theme: 'legacy',
  rowSelection: { mode: 'singleRow', checkboxes: false },
  suppressRowClickSelection: false,
  suppressRowDeselection: false,
  getRowId: (params) => params.data?.id ?? '',
}

// Handle OS-level file/folder drops (Tauri v1/v2 compatible via event API)
let fileDropUnlisten: UnlistenFn | null = null
let fileDropHoverUnlisten: UnlistenFn | null = null
let fileDropCancelledUnlisten: UnlistenFn | null = null
let fileDropWebviewUnlisten: UnlistenFn | null = null

function setupFileDropHandler() {
  console.log('[ui] registering tauri file-drop listeners')
  // v2: preferred API
  try {
    const win = getCurrentWebviewWindow()
    if (win && typeof (win as any).onDragDropEvent === 'function') {
      ;(win as any).onDragDropEvent(async (event: any) => {
        console.log('[ui] onDragDropEvent:', event)
        const payload = event?.payload
        const type = payload?.type
        const paths: string[] | undefined = Array.isArray(payload?.paths) ? payload.paths : undefined
        if (type !== 'drop' || !paths || paths.length === 0) return
        try {
          const label = paths.length === 1 ? paths[0].split(/[\\/]/).pop() : `Dropped (${paths.length})`
          console.log('[ui] loading playlist from dropped paths (onDragDropEvent):', paths)
          await playlistStore.loadFromPaths(paths, label)
          // Mirror playlist into active tab so it persists when switching tabs
          tabStore.setActiveTabPlaylist(playlistStore.currentPlayList)
        } catch (e) {
          console.error('Failed to load from dropped paths', e)
        }
      }).then((unlisten: UnlistenFn) => { fileDropWebviewUnlisten = unlisten }).catch(() => {})
    }
  } catch {}
  // Drop
  listen('tauri://file-drop', async (ev) => {
    console.log('[ui] tauri://file-drop event:', ev)
    const payload: any = ev.payload as any
    let paths: string[] | undefined
    if (Array.isArray(payload)) {
      // v1: payload is string[]
      paths = payload as string[]
    } else if (payload && Array.isArray(payload.paths)) {
      // v2: payload has shape { type, paths }
      if (payload.type && payload.type !== 'drop') return
      paths = payload.paths
    }
    if (!paths || paths.length === 0) return
    try {
      const label = paths.length === 1 ? paths[0].split(/[\\/]/).pop() : `Dropped (${paths.length})`
      console.log('[ui] loading playlist from dropped paths:', paths)
      await playlistStore.loadFromPaths(paths, label)
      // Mirror playlist into active tab so new tabs can keep their own data if needed later
      tabStore.setActiveTabPlaylist(playlistStore.currentPlayList)
    } catch (e) {
      console.error('Failed to load from dropped paths', e)
    }
  }).then((unlisten) => { fileDropUnlisten = unlisten }).catch(() => {})

  // Hover (optional logging)
  listen('tauri://file-drop-hover', (ev) => {
    console.log('[ui] tauri://file-drop-hover:', ev)
  }).then((unlisten) => { fileDropHoverUnlisten = unlisten }).catch(() => {})

  // Cancelled (optional logging)
  listen('tauri://file-drop-cancelled', (ev) => {
    console.log('[ui] tauri://file-drop-cancelled:', ev)
  }).then((unlisten) => { fileDropCancelledUnlisten = unlisten }).catch(() => {})
}

onMounted(() => {
  setupFileDropHandler()
  // Add DOM listeners just to confirm browser-level drop reaches us
  window.addEventListener('dragover', (e) => {
    e.preventDefault()
  })
  window.addEventListener('drop', (e) => {
    e.preventDefault()
    try {
      const dt = e.dataTransfer
      if (!dt) return
      console.log('[ui] DOM drop: files=', dt.files?.length, 'items=', dt.items?.length)
      const paths: string[] = []
      // Some environments expose a path on File (not standard). Log it if present.
      for (let i = 0; i < dt.files.length; i++) {
        const f: any = dt.files[i]
        if (f && typeof f.path === 'string') paths.push(f.path)
      }
      if (paths.length > 0) {
        const label = paths.length === 1 ? paths[0].split(/[\\/]/).pop() : `Dropped (${paths.length})`
        playlistStore.loadFromPaths(paths, label)
        tabStore.setActiveTabPlaylist(playlistStore.currentPlayList)
      }
    } catch (err) {
      console.error('[ui] DOM drop handler error', err)
    }
  })
})

onBeforeUnmount(() => {
  if (fileDropUnlisten) { try { fileDropUnlisten() } catch {} fileDropUnlisten = null }
  if (fileDropHoverUnlisten) { try { fileDropHoverUnlisten() } catch {} fileDropHoverUnlisten = null }
  if (fileDropCancelledUnlisten) { try { fileDropCancelledUnlisten() } catch {} fileDropCancelledUnlisten = null }
  if (fileDropWebviewUnlisten) { try { fileDropWebviewUnlisten() } catch {} fileDropWebviewUnlisten = null }
})

</script>

<template>
  <div class="grid-wrapper ag-theme-alpine cs-theme">
    <AgGridVue
      style="width: 100%; height: 100%"
      :columnDefs="columnDefs"
      :rowData="rowData"
      :gridOptions="gridOptions"
      :domLayout="'normal'"
      @grid-ready="onGridReady"
      @row-clicked="onRowClicked"
      @row-double-clicked="onRowDoubleClicked"
    />
  </div>
  
</template>

<style>
.grid-wrapper {
  height: 100%;
  width: 100%;
}

/* CS16 theme overrides for AG Grid */
.ag-theme-alpine.cs-theme {
  --ag-foreground-color: var(--text);
  --ag-background-color: var(--playlist-bg);
  --ag-header-background-color: var(--bg);
  --ag-header-foreground-color: var(--text);
  --ag-border-color: var(--border-dark);
  --ag-secondary-border-color: var(--border-light);
  --ag-row-hover-color: transparent;
  --ag-selected-row-background-color: #2f362b;
  --ag-odd-row-background-color: var(--playlist-bg);
  --ag-row-border-color: var(--border-dark);
  --ag-header-height: 24px;
  --ag-row-height: 22px;
}

.ag-theme-alpine.cs-theme .ag-root-wrapper,
.ag-theme-alpine.cs-theme .ag-root-wrapper-body,
.ag-theme-alpine.cs-theme .ag-root {
  background: var(--playlist-bg);
}

.ag-theme-alpine.cs-theme .ag-header {
  border-bottom: 1px solid var(--border-dark);
  background: var(--bg);
  color: var(--text);
}

/* Ensure header surfaces inherit our background and text color */
.ag-theme-alpine.cs-theme .ag-header-viewport,
.ag-theme-alpine.cs-theme .ag-header-row,
.ag-theme-alpine.cs-theme .ag-header-cell {
  background-color: var(--bg) !important;
  color: var(--text) !important;
}

.ag-theme-alpine.cs-theme .ag-header-cell-text {
  color: var(--text) !important;
}

.ag-theme-alpine.cs-theme .ag-header-cell {
  border-right: 1px solid var(--border-dark);
}

.ag-theme-alpine.cs-theme .ag-header-cell-label {
  padding: 3px 4px;
}

.ag-theme-alpine.cs-theme .ag-cell {
  padding: 1px 4px;
  color: var(--text);
  background: var(--playlist-bg);
}

/* Column separators to mimic table borders */
.ag-theme-alpine.cs-theme .ag-center-cols-container, 
.ag-theme-alpine.cs-theme .ag-center-cols-viewport {
  background: var(--playlist-bg);
}

.ag-theme-alpine.cs-theme .ag-cell:not(:last-child) {
  border-right: 1px solid var(--border-dark);
}

.ag-theme-alpine.cs-theme .ag-row-hover .ag-cell { background: var(--playlist-bg); }
.ag-theme-alpine.cs-theme .ag-row-selected .ag-cell { background: #2f362b; }
</style>


