<script setup lang="ts">
import PlaybackButtonsContainer from "./components/playback/PlaybackButtonsContainer.vue";
import VolumeControl from "./components/playback/VolumeControl.vue";
import ProgressBar from "./components/playback/ProgressBar.vue";
import PlaylistTabs from "./components/ui/PlaylistTabs.vue";
import BottomStats from "./components/ui/BottomStats.vue";
import SearchOverlay from "./components/ui/SearchOverlay.vue";
import SettingsOverlay from "./components/ui/SettingsOverlay.vue";
import { onMounted, onBeforeUnmount, onUnmounted, watch, unref } from 'vue';
import { usePlaylistStore } from './stores/playlistStore';
import { useSearchStore } from './stores/searchStore';
import { useSettingsStore } from './stores/settingsStore';
import { doesEventMatchCombo, comboToAccelerator } from './utils/keyboard';
import { invoke } from '@tauri-apps/api/core'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'

const playlistStore = usePlaylistStore();
const searchStore = useSearchStore();
const settingsStore = useSettingsStore();

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

  // Skip when Settings/Search overlays are capturing/active
  if (settingsStore.isOpen || searchStore.isOpen) return;

  // Playback shortcuts
  const s = settingsStore.shortcuts;
  if (doesEventMatchCombo(event, s.playPause)) {
    event.preventDefault();
    // If we have a selected song and it hasn't been played yet, start it; otherwise toggle
    const selected = unref((playlistStore as any).selectedSong) as AudioFile | undefined
    const lastId = unref((playlistStore as any).lastPlayedSongId) as string | null | undefined
    if (selected && selected.id) {
      if (lastId && lastId === selected.id) {
        invoke('toggle_play_pause').catch(console.error)
      } else {
        playlistStore.playSelected();
      }
    } else {
      invoke('toggle_play_pause').catch(console.error)
    }
    return;
  }
  if (doesEventMatchCombo(event, s.stop)) {
    event.preventDefault();
    invoke('stop_audio').catch(console.error);
    return;
  }
  if (doesEventMatchCombo(event, s.previous)) {
    event.preventDefault();
    playlistStore.playPrevious();
    return;
  }
  if (doesEventMatchCombo(event, s.next)) {
    event.preventDefault();
    playlistStore.playNext();
    return;
  }
  if (doesEventMatchCombo(event, s.random)) {
    event.preventDefault();
    if (typeof (playlistStore as any).playRandom === 'function') {
      (playlistStore as any).playRandom();
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

// Global shortcut registration
async function refreshGlobalShortcuts() {
  try {
    await unregisterAll()
  } catch {}
  if (!settingsStore.globalShortcutsEnabled) return
  const s = settingsStore.shortcuts
  const entries: Array<[string, (ev: any) => void]> = []
  const make = (combo: KeyCombo | null | undefined, handler: () => void) => {
    const accel = comboToAccelerator(combo)
    if (accel) entries.push([accel, () => handler()])
  }
  make(s.playPause, () => invoke('toggle_play_pause').catch(console.error))
  make(s.stop, () => invoke('stop_audio').catch(console.error))
  make(s.previous, () => (playlistStore as any).playPrevious())
  make(s.next, () => (playlistStore as any).playNext())
  make(s.random, () => {
    if (typeof (playlistStore as any).playRandom === 'function') (playlistStore as any).playRandom()
  })
  for (const [accel, handler] of entries) {
    try {
      await register(accel, (event) => {
        if (event.state === 'Pressed') handler()
      })
    } catch (e) {
      console.error('Failed to register global shortcut', accel, e)
    }
  }
}

watch(() => [settingsStore.globalShortcutsEnabled, settingsStore.shortcuts], () => {
  refreshGlobalShortcuts()
}, { deep: true })

onMounted(() => {
  refreshGlobalShortcuts()
})

onUnmounted(async () => {
  try { await unregisterAll() } catch {}
})
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


