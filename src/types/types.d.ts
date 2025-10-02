type TabContent = {
    title: string;
    text: string;
};

type Tab = {
    id: string;
    label: string;
    content: TabContent;
};

type BottomStats = {
    codec: string;
    bitrate: string;
    frequency: string;
    channelType: string;
    playtime: string;
};

type ProgressBarState = {
    progress: number;
};

type Playlist = {
    id: string;
    name: string;
    songs: AudioFile[];
}

type AudioFile = {
    id: string
    track: number;
    title: string;
    artist: string;
    album: string;
    duration: string;
}
