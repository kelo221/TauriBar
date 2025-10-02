<script setup lang="ts">
import { ref, computed } from 'vue'
import { ModuleRegistry, AllCommunityModule, ColDef, GridOptions } from 'ag-grid-community'
import { AgGridVue } from 'ag-grid-vue3'
import { usePlaylistStore } from '../../stores/playlistStore'
import 'ag-grid-community/styles/ag-grid.css'
import 'ag-grid-community/styles/ag-theme-alpine.css'

ModuleRegistry.registerModules([AllCommunityModule])

const playlistStore = usePlaylistStore()

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

const gridOptions: GridOptions<AudioFile> = {
  defaultColDef,
  headerHeight: 24,
  rowHeight: 22,
  suppressDragLeaveHidesColumns: true,
  ensureDomOrder: true,
}

</script>

<template>
  <div class="grid-wrapper ag-theme-alpine cs-theme">
    <AgGridVue
      style="width: 100%; height: 100%"
      :columnDefs="columnDefs"
      :rowData="rowData"
      :gridOptions="gridOptions"
      :domLayout="'normal'"
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
  --ag-row-hover-color: rgba(255,255,255,0.06);
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

.ag-theme-alpine.cs-theme .ag-row-hover .ag-cell { background: rgba(255,255,255,0.04); }
.ag-theme-alpine.cs-theme .ag-row-selected .ag-cell { background: #2f362b; }
</style>


