import {defineStore} from 'pinia'
import {ref, computed} from 'vue'

export const useTabStore = defineStore('tabs', () => {
  const tabs = ref<Tab[]>([
    {
      id: "tab1",
      label: "First Tab",
      content: {
        title: "Welcome",
        text: "This is the first tab content",
      },
    }
  ])

  const activeTabIndex = ref(0)

  const activeTab = computed(() => tabs.value[activeTabIndex.value])

  function createTab() {
    tabs.value.push({
      id: `tab${tabs.value.length + 1}`,
      label: `Tab ${tabs.value.length + 1}`,
      content: {
        title: "Welcome",
        text: `This is tab ${tabs.value.length + 1} content`,
      },
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

  return {
    tabs,
    activeTabIndex,
    activeTab,
    createTab,
    removeTab,
    renameTab,
    setActiveTab
  }
})
