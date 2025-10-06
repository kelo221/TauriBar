import {defineStore} from 'pinia'
import {ref, computed, watch} from 'vue'

export const useTabStore = defineStore('tabs', () => {
  const STORAGE_KEY = 'tabState'
  const tabs = ref<Tab[]>([
    {
      id: "tab1",
      label: "First Tab",
      content: {
        title: "Welcome",
        text: "This is the first tab content",
      },
      playlist: { id: 'empty', name: 'Empty', songs: [] }
    }
  ])

  const activeTabIndex = ref(0)

  function restoreFromStorage() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (!raw) return
      const parsed = JSON.parse(raw)
      if (!parsed || typeof parsed !== 'object') return
      const savedTabs = Array.isArray(parsed.tabs) ? parsed.tabs as Tab[] : null
      const savedIndex = Number.isInteger(parsed.activeTabIndex) ? parsed.activeTabIndex as number : 0
      if (savedTabs && savedTabs.length > 0) {
        tabs.value = savedTabs
        activeTabIndex.value = Math.min(Math.max(0, savedIndex), savedTabs.length - 1)
      }
    } catch (e) {
      // ignore corrupt data
      console.warn('[tabs] failed to restore state', e)
    }
  }

  function saveToStorage() {
    try {
      const payload = JSON.stringify({ tabs: tabs.value, activeTabIndex: activeTabIndex.value })
      localStorage.setItem(STORAGE_KEY, payload)
    } catch (e) {
      console.warn('[tabs] failed to persist state', e)
    }
  }

  const activeTab = computed(() => tabs.value[activeTabIndex.value])

  function createTab() {
    tabs.value.push({
      id: `tab${tabs.value.length + 1}`,
      label: `Tab ${tabs.value.length + 1}`,
      content: {
        title: "Welcome",
        text: `This is tab ${tabs.value.length + 1} content`,
      },
      playlist: { id: 'empty', name: 'Empty', songs: [] }
    })
    activeTabIndex.value = tabs.value.length - 1
  }

  function removeTab(id: string) {
    const index = tabs.value.findIndex((tab) => tab.id === id)
    if (index !== -1) {
      tabs.value.splice(index, 1)
      // Adjust or recreate tab area if empty
      if (tabs.value.length === 0) {
        // keep active index at 0 so the tab bar empty space remains clickable
        activeTabIndex.value = 0
      } else if (activeTabIndex.value >= tabs.value.length) {
        activeTabIndex.value = Math.max(0, tabs.value.length - 1)
      }
    }
  }

  function renameTab(id: string, newName: string) {
    const tab = tabs.value.find((tab) => tab.id === id)
    if (tab) {
      tab.label = newName
    }
  }

  function setActiveTab(index: number) {
    activeTabIndex.value = index
  }

  function setActiveTabPlaylist(pl: Playlist) {
    const idx = activeTabIndex.value
    if (idx >= 0 && idx < tabs.value.length) {
      tabs.value[idx].playlist = pl
    }
  }

  // initial restore
  restoreFromStorage()

  // persist on change
  watch(tabs, saveToStorage, { deep: true })
  watch(activeTabIndex, saveToStorage)

  return {
    tabs,
    activeTabIndex,
    activeTab,
    createTab,
    removeTab,
    renameTab,
    setActiveTab,
    setActiveTabPlaylist
  }
})
