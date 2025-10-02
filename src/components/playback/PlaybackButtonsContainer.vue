<script setup lang="ts">
import PlayPauseButton from "../../components/playback/playbackButtons/PlayPauseButton.vue";
import PlayPreviousButton from "../../components/playback/playbackButtons/PlayPreviousButton.vue";
import PlayNextButton from "../../components/playback/playbackButtons/PlayNextButton.vue";
import PlayRandomButton from "../../components/playback/playbackButtons/PlayRandomButton.vue";
import PlayStopButton from "../../components/playback/playbackButtons/PlayStopButton.vue";
import { invoke } from '@tauri-apps/api/core'
import { usePlaylistStore } from '../../stores/playlistStore'



function StopPlayback() {
  console.log("Stopping playback");
  invoke("stop_audio").catch(console.error)
}

function PlayPausePlayback() {
  const playlist = usePlaylistStore()
  const selected = playlist.selectedSong.value
  // If we have a selected song and it hasn't been played yet, start it
  if (selected && selected.id && playlist.lastPlayedSongId.value !== selected.id) {
    playlist.playSelected()
    return
  }
  // Otherwise, toggle pause/play
  invoke("toggle_play_pause").catch(console.error)
}

function PlayPrevious() {
  console.log("Playing previous song");
  const playlist = usePlaylistStore()
  playlist.playPrevious()
}

function PlayNext() {
  console.log("Playing next song");
  const playlist = usePlaylistStore()
  playlist.playNext()
}

function PlayRandom() {
  console.log("Playing random song");
  const playlist = usePlaylistStore()
  const song = playlist.selectedSong.value
  if (song && song.id) {
    invoke("play_audio", { path: song.id }).catch(console.error)
  } else {
    console.warn('No selected song to play')
  }
}


</script>

<template>
  <div class="toolbar">
    <PlayPauseButton @click="PlayPausePlayback" />
    <PlayStopButton @click="StopPlayback" />
    <PlayPreviousButton @click="PlayPrevious" />
    <PlayNextButton @click="PlayNext" />
    <PlayRandomButton @click="PlayRandom" />
  </div>
</template>


<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar :deep(img) {
  width: 24px;
  height: 24px;
  cursor: pointer;
  user-select: none;
}

.toolbar :deep(img):active {
  transform: scale(0.95);
}
</style>
