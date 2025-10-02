import { defineStore } from 'pinia'
import { ref } from 'vue'

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

    function setPlaylist(newStats: Playlist) {
      currentPlayList.value = newStats
    }

    function setSelectedSong(newSelection: AudioFile) {
      selectedSong.value = newSelection
    }

    return { currentPlayList, setPlaylist, selectedSong, setSelectedSong }
  })
