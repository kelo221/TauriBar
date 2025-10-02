<script setup lang="ts">
import {usePlaylistStore} from "../../stores/playlistStore";
import PlaylistHeader from "./PlaylistHeader.vue";
import { computed, ref } from 'vue';

const sortBy = ref("artist");
const sortDirection = ref<"asc" | "desc" | null>(null);

const playListStore = usePlaylistStore();

// Add computed property for sorted songs
const sortedSongs = computed(() => {
  
  if (!sortDirection.value) return playListStore.currentPlayList.songs;
  
  return [...playListStore.currentPlayList.songs].sort((a, b) => {
    const aValue = a[sortBy.value as keyof typeof a];
    const bValue = b[sortBy.value as keyof typeof b];
    
    
    if (sortDirection.value === "asc") {
      return aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
    } else {
      return bValue < aValue ? -1 : bValue > aValue ? 1 : 0;
    }
  });
});

function handleSort(field: string) {
  if (sortBy.value === field) {
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    sortBy.value = field;
    sortDirection.value = "asc";
  }
  
}

const lastClickTime = ref(0);

function handleSongClick(song: AudioFile) {
  const currentTime = Date.now();
  playListStore.setSelectedSong(song);
  
  if (currentTime - lastClickTime.value < 200) {
    console.log(`Playing: ${song.title}`);
  }
  lastClickTime.value = currentTime;
}
</script>

<template>
  <div class="playlist-container">
    <table class="playlist-table">
      <colgroup>
        <col />
        <col style="width:80px" />
        <col />
        <col style="width:80px" />
      </colgroup>
      <thead>
      <tr>
        <th class="col-header" @click="() => handleSort('artist')">
          <div class="header-inner">Artist/album <span class="sort" v-if="sortBy==='artist'">{{ sortDirection==='asc' ? '↑' : '↓' }}</span></div>
        </th>
        <th class="col-header col-track" @click="() => handleSort('track')">
          <div class="header-inner">Track no <span class="sort" v-if="sortBy==='track'">{{ sortDirection==='asc' ? '↑' : '↓' }}</span></div>
        </th>
        <th class="col-header" @click="() => handleSort('title')">
          <div class="header-inner">Title / track artist <span class="sort" v-if="sortBy==='title'">{{ sortDirection==='asc' ? '↑' : '↓' }}</span></div>
        </th>
        <th class="col-header col-duration" @click="() => handleSort('duration')">
          <div class="header-inner">Durat… <span class="sort" v-if="sortBy==='duration'">{{ sortDirection==='asc' ? '↑' : '↓' }}</span></div>
        </th>
      </tr>
      </thead>
      <tbody>

      <tr v-for="song in sortedSongs" 
          :key="song.id" 
          @click="() => handleSongClick(song)"
          class="row"
          :class="{ 'selected': playListStore.selectedSong?.id === song.id }"
      >
        <td class="cell">{{ song.artist }} - {{ song.album }}</td>
        <td class="cell cell-track">{{ song.track.toString().padStart(2,'0') }}</td>
        <td class="cell">{{ song.title }}</td>
        <td class="cell cell-duration">{{ song.duration }}</td>
      </tr>

      </tbody>
    </table>
  </div>
</template>

<style>
.playlist-container {
  height: 100%;
  overflow-y: auto;
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
  background: var(--secondary-bg);
  color: var(--text);
  font-weight: 600;
  text-align: left;
  border-bottom: 1px solid var(--border-dark);
}

.col-header .header-inner {
  padding: 3px 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-track { width: 80px; }
.col-duration { width: 90px; }

.row {
  cursor: default;
}

.cell {
  padding: 1px 4px;
  border-bottom: 1px solid rgba(0,0,0,0.2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row:hover .cell {
  background: rgba(255,255,255,0.04);
}

.selected .cell {
  background-color: #2f362b;
}
</style>
