<script setup lang="ts">
import PlaybackButtonsContainer from "./components/playback/PlaybackButtonsContainer.vue";
import VolumeControl from "./components/playback/VolumeControl.vue";
import ProgressBar from "./components/playback/ProgressBar.vue";
import PlaylistTabs from "./components/ui/PlaylistTabs.vue";
import BottomStats from "./components/ui/BottomStats.vue";
import SearchOverlay from "./components/ui/SearchOverlay.vue";
import SettingsOverlay from "./components/ui/SettingsOverlay.vue";
import { onMounted, onBeforeUnmount } from 'vue';
import { usePlaylistStore } from './stores/playlistStore';
import { useSearchStore } from './stores/searchStore';

const playlistStore = usePlaylistStore();
const searchStore = useSearchStore();

function handleGlobalKeydown(event: KeyboardEvent) {
  const isCtrlF = (event.ctrlKey || event.metaKey) && (event.key === 'f' || event.key === 'F');
  const isF3 = event.key === 'F3' || event.key === 'F3'.toLowerCase();
  if (isCtrlF) {
    event.preventDefault();
    event.stopPropagation();
    searchStore.open();
    return;
  }
  if (isF3) {
    event.preventDefault();
    event.stopPropagation();
    if (event.shiftKey) {
      searchStore.prev();
    } else {
      searchStore.next();
    }
    return;
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeydown, { capture: true });
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleGlobalKeydown, { capture: true } as any);
});
</script>

<template>
  <main class="app">

    <div class="toolbar-row">
      <div class="left">
        <PlaybackButtonsContainer />
      </div>
      <div class="right">
        <VolumeControl />
      </div>
    </div>

    <div class="seek-row">
      <ProgressBar />
    </div>

    <div class="tabs-row">
      <PlaylistTabs />
    </div>

    <BottomStats />
    <SearchOverlay />
    <SettingsOverlay />
  </main>
  
</template>

<style>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  -webkit-user-select: none;
     -moz-user-select: none;
          user-select: none;
}



.toolbar-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-dark);
}

.seek-row {
  padding: 8px 8px;
  border-bottom: 1px solid var(--border-dark);
}

.tabs-row {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--playlist-bg);
}
</style>



<!--<script setup lang="ts">-->
<!--//import { ref } from "vue";-->
<!--// import { invoke } from "@tauri-apps/api/core";-->
<!--//-->
<!--// const greetMsg = ref("");-->
<!--// const name = ref("");-->
<!--//-->
<!--// async function greet() {-->
<!--//   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/-->
<!--//   greetMsg.value = await invoke("greet", { name: name.value });-->
<!--// }-->
<!--</script>-->

<!--<template>-->

<!--</template>-->


