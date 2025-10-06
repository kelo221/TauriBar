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

    return { currentPlayList, setPlaylist, selectedSong, setSelectedSong, loadFromBackend, loadFromPaths, lastPlayedSongId, playSong, playSelected, playNext, playPrevious }
  })
