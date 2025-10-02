<script setup lang="ts">
import PlayPauseButton from "../../components/playback/playbackButtons/PlayPauseButton.vue";
import PlayPreviousButton from "../../components/playback/playbackButtons/PlayPreviousButton.vue";
import PlayNextButton from "../../components/playback/playbackButtons/PlayNextButton.vue";
import PlayRandomButton from "../../components/playback/playbackButtons/PlayRandomButton.vue";
import PlayStopButton from "../../components/playback/playbackButtons/PlayStopButton.vue";



function StopPlayback() {
  console.log("Stopping playback");
 // StopRequest();
}

function PlayPausePlayback() {
  console.log("Pausing playback");
 //PauseRequest();
}

function PlayPrevious() {
 // Greet("test");
  console.log("Playing previous song");
}

function PlayNext() {
  console.log("Playing next song");
}

function PlayRandom() {
  console.log("Playing random song");
 // PlayRequest();
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
