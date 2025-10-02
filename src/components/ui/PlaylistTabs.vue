<template>
  <div class="tabs-wrapper">
    <div class="cs-tabs" @mousedown.middle.prevent="handleMiddleClickOnBar" @click="handleClickOnBar">
      <template v-if="tabs.length > 0" v-for="(tab, i) in tabs" :key="tab.id">
        <input class="radiotab" type="radio" name="tabs-cs" :id="`tab-${tab.id}`" :checked="i===activeTabIndex" @change="() => setActiveTab(i)" />
        <label class="label" :for="`tab-${tab.id}`" @dblclick="() => handleRename(tab)" @mousedown.middle.prevent.stop="() => removeTab(tab.id)">
          {{ tab.label }}
        </label>
      </template>
      <template v-else>
        <!-- Empty bar still clickable to create new tab -->
        <div style="height:21px"></div>
      </template>
    </div>
    <div class="playlist-body">
      <Playlist />
    </div>
  </div>
  
</template>

<script setup lang="ts">
import { useTabStore } from '../../stores/tabStore'
import Playlist from '../../components/playlist/Playlist.vue'

const tabStore = useTabStore()
const {
  tabs,
  activeTabIndex,
  createTab,
  removeTab,
  renameTab,
  setActiveTab
} = tabStore

function handleRename(tab: Tab) {
  const newName = prompt('Enter new name:')
  if (newName) renameTab(tab.id, newName)
}

function handleClickOnBar(event: MouseEvent) {
  // If click is on the bar area and not on a label/input/panel, create a new tab
  const target = event.target as HTMLElement
  const isLabel = target.classList.contains('label')
  const isInput = target.classList.contains('radiotab')
  const isPanel = target.classList.contains('panel')
  if (!isLabel && !isInput && !isPanel) {
    createTab()
  }
}

function handleMiddleClickOnBar(event: MouseEvent) {
  // prevent scrolling click on empty bar; do nothing else
}
</script>

<style scoped>
.tabs-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.cs-tabs { width: 100%; }

.playlist-body {
  flex: 1;
  min-height: 0;
  background: var(--playlist-bg);
}

.cs-tabs {
  min-height: 21px; /* keep clickable empty area for creating new tabs */
}
</style>
