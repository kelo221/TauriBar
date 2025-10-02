<script setup lang="ts">
import TopButtonContainer from "./components/ui/TopButtonContainer.vue";
import PlaybackButtonsContainer from "./components/playback/PlaybackButtonsContainer.vue";
import VolumeControl from "./components/playback/VolumeControl.vue";
import ProgressBar from "./components/playback/ProgressBar.vue";
import PlaylistTabs from "./components/ui/PlaylistTabs.vue";
import BottomStats from "./components/ui/BottomStats.vue";
import { onMounted } from 'vue';
import { usePlaylistStore } from './stores/playlistStore';

const playlistStore = usePlaylistStore();

onMounted(() => {
  playlistStore.loadFromBackend();
});
</script>

<template>
  <main class="app">
    <div class="menu-bar">
      <TopButtonContainer />
    </div>

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

.menu-bar {
  padding: 1px 2px;
  border-bottom: 1px solid var(--border-dark);
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


