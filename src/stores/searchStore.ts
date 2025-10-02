import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSearchStore = defineStore('search', () => {
  const isOpen = ref(false)
  const query = ref('')
  const navToken = ref(0)
  const navDirection = ref<'next' | 'prev'>('next')

  function open() {
    isOpen.value = true
  }

  function close() {
    isOpen.value = false
    query.value = ''
  }

  function setQuery(q: string) {
    query.value = q
  }

  function next() {
    navDirection.value = 'next'
    navToken.value++
  }

  function prev() {
    navDirection.value = 'prev'
    navToken.value++
  }

  return { isOpen, query, navToken, navDirection, open, close, setQuery, next, prev }
})


