import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const usePlaylistStore = defineStore('playlists',
  () => {

    const currentPlayList = ref<Playlist>({ id: "empty", name: "Empty", songs: [] })

    const selectedSong = ref<AudioFile>()
    const lastPlayedSongId = ref<string | null>(null)

    async function loadFromBackend() {
      try {
        const files = await invoke<Pick<AudioFile, 'id' | 'track' | 'title' | 'artist' | 'album' | 'duration'>[]>("list_media_files")
        const playlist: Playlist = {
          id: "testfiles",
          name: "Test Files",
          songs: files.map((f, idx) => ({
            id: f.id ?? String(idx),
            track: typeof f.track === 'number' ? f.track : Number(f.track) || 0,
            title: f.title || '',
            artist: f.artist || '',
            album: f.album || '',
            duration: f.duration || '',
          }))
        }
        currentPlayList.value = playlist
        if (playlist.songs.length > 0) {
          selectedSong.value = playlist.songs[0]
        }
      } catch (e) {
        console.error('Failed to load media files', e)
      }
    }

    async function loadFromPaths(paths: string[], label?: string) {
      try {
        console.log('[store] loadFromPaths called with', paths)
        const files = await invoke<Pick<AudioFile, 'id' | 'track' | 'title' | 'artist' | 'album' | 'duration'>[]>(
          "list_media_files_from_paths",
          { paths }
        )
        console.log('[store] backend returned', files.length, 'items')
        const playlist: Playlist = {
          id: "dropped",
          name: label || "Dropped",
          songs: files.map((f, idx) => ({
            id: f.id ?? String(idx),
            track: typeof f.track === 'number' ? f.track : Number(f.track) || 0,
            title: f.title || '',
            artist: f.artist || '',
            album: f.album || '',
            duration: f.duration || '',
          }))
        }
        currentPlayList.value = playlist
        if (playlist.songs.length > 0) {
          selectedSong.value = playlist.songs[0]
        }
      } catch (e) {
        console.error('Failed to load media files from paths', e)
      }
    }

    function removeSongsByIds(ids: string[]) {
      try {
        if (!ids || ids.length === 0) return
        const idSet = new Set(ids)
        const oldList = currentPlayList.value.songs
        const oldSelectedId = selectedSong.value?.id ?? null
        const oldSelectedIndex = oldSelectedId ? oldList.findIndex(s => s.id === oldSelectedId) : -1

        const newList = oldList.filter(song => !idSet.has(song.id))

        // Replace playlist object to ensure reactivity across stores/tabs
        currentPlayList.value = {
          ...currentPlayList.value,
          songs: newList,
        }

        // Recompute selection to a sensible neighbor
        if (newList.length === 0) {
          selectedSong.value = undefined
          return
        }

        // If the previous selected song remains, keep it
        if (oldSelectedId && newList.some(s => s.id === oldSelectedId)) {
          const keep = newList.find(s => s.id === oldSelectedId)!
          selectedSong.value = keep
          return
        }

        // Otherwise choose a neighbor near the previous index
        let nextIndex = oldSelectedIndex >= 0 ? oldSelectedIndex : 0
        if (nextIndex >= newList.length) nextIndex = newList.length - 1
        selectedSong.value = newList[nextIndex]
      } catch (e) {
        console.error('Failed to remove songs', e)
      }
    }

    function setPlaylist(newStats: Playlist) {
      currentPlayList.value = newStats
    }

    function setSelectedSong(newSelection: AudioFile) {
      selectedSong.value = newSelection
    }

    function playSong(song: AudioFile) {
      selectedSong.value = song
      if (song && song.id) {
        invoke('play_audio', { path: song.id }).catch(console.error)
        lastPlayedSongId.value = song.id
      }
    }

    function playSelected() {
      const song = selectedSong.value
      if (song && song.id) {
        invoke('play_audio', { path: song.id }).catch(console.error)
        lastPlayedSongId.value = song.id
      }
    }

    function playNext() {
      const list = currentPlayList.value.songs
      if (!list || list.length === 0) return
      const currentId = selectedSong.value?.id
      const idx = currentId ? list.findIndex(s => s.id === currentId) : -1
      const nextIndex = idx >= 0 ? (idx + 1) % list.length : 0
      const next = list[nextIndex]
      if (next) {
        playSong(next)
      }
    }

    function playPrevious() {
      const list = currentPlayList.value.songs
      if (!list || list.length === 0) return
      const currentId = selectedSong.value?.id
      const idx = currentId ? list.findIndex(s => s.id === currentId) : -1
      const prevIndex = idx >= 0 ? (idx - 1 + list.length) % list.length : 0
      const prev = list[prevIndex]
      if (prev) {
        playSong(prev)
      }
    }

    function playRandom() {
      const list = currentPlayList.value.songs
      if (!list || list.length === 0) return
      const currentId = selectedSong.value?.id || null
      let index = Math.floor(Math.random() * list.length)
      if (list.length > 1 && currentId) {
        const currentIndex = list.findIndex(s => s.id === currentId)
        if (currentIndex === index) {
          index = (index + 1) % list.length
        }
      }
      const next = list[index]
      if (next) {
        playSong(next)
      }
    }

    return { currentPlayList, setPlaylist, selectedSong, setSelectedSong, loadFromBackend, loadFromPaths, removeSongsByIds, lastPlayedSongId, playSong, playSelected, playNext, playPrevious, playRandom }
  })
