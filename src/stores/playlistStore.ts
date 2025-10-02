import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const usePlaylistStore = defineStore('playlists',
  () => {

    const currentPlayList = ref<Playlist>({
      id: "1",
      name: "My Playlist",
      songs: [
        {
          id: "rsatarstaro",
          track: 1,
          title: "Song 1",
          artist: "Artist 1",
          album: "Album 1",
          duration: "3:45",
        },
        {
          id: "arstarst",
          track: 2,
          title: "Song 2",
          artist: "Artist 2",
          album: "Album 2",
          duration: "2:50",
        },
      ]
    })

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

    return { currentPlayList, setPlaylist, selectedSong, setSelectedSong, loadFromBackend, lastPlayedSongId, playSong, playSelected }
  })
