import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface TrackInfo {
  path: string;
  name: string;
  artist: string | null;
  album: string | null;
  title: string | null;
  thumbnail: string | null;
}

interface MusicPlayerState {
  currentTrack: string | null;
  trackInfo: TrackInfo | null;
  isPlaying: boolean;
  refreshKey: number;

  // Actions
  setCurrentTrack: (track: string | null) => void;
  setTrackInfo: (info: TrackInfo | null) => void;
  setIsPlaying: (playing: boolean) => void;
  incrementRefreshKey: () => void;
  loadCurrentTrack: () => Promise<void>;
  loadTrackInfo: () => Promise<void>;
  playMusic: (path: string) => Promise<void>;
  pauseMusic: () => Promise<void>;
  resumeMusic: () => Promise<void>;
  togglePlayback: () => Promise<void>;
  checkPlaying: () => Promise<void>;
  loadTracksFromDb: () => Promise<void>;
}

export const useMusicStore = create<MusicPlayerState>((set, get) => ({
  currentTrack: null,
  trackInfo: null,
  isPlaying: false,
  refreshKey: 0,

  setCurrentTrack: (track) => {
    set({ currentTrack: track });
    get().loadTrackInfo();
  },

  setTrackInfo: (info) => set({ trackInfo: info }),

  setIsPlaying: (playing) => set({ isPlaying: playing }),

  incrementRefreshKey: () =>
    set((state) => ({ refreshKey: state.refreshKey + 1 })),

  loadCurrentTrack: async () => {
    try {
      const track = await invoke<string | null>("get_current_track");
      const prevTrack = get().currentTrack;
      set({ currentTrack: track });
      if (track && track !== prevTrack) {
        get().loadTrackInfo();
      }
    } catch (error) {
      console.error("Failed to load current track:", error);
    }
  },

  loadTrackInfo: async () => {
    const { currentTrack } = get();
    if (currentTrack) {
      try {
        const info = await invoke<TrackInfo | null>("get_current_track_info");
        set({ trackInfo: info });
      } catch (error) {
        console.error("Failed to load track info:", error);
      }
    } else {
      set({ trackInfo: null });
    }
  },

  playMusic: async (path: string) => {
    try {
      await invoke("play_music", { path });
      set({ currentTrack: path, isPlaying: true });
      get().loadTrackInfo();
    } catch (error) {
      console.error("Failed to play music:", error);
    }
  },

  pauseMusic: async () => {
    try {
      await invoke("pause_music");
      set({ isPlaying: false });
    } catch (error) {
      console.error("Failed to pause music:", error);
    }
  },

  resumeMusic: async () => {
    try {
      await invoke("resume_music");
      set({ isPlaying: true });
    } catch (error) {
      console.error("Failed to resume music:", error);
    }
  },

  togglePlayback: async () => {
    const { isPlaying, pauseMusic, resumeMusic } = get();
    if (isPlaying) {
      await pauseMusic();
    } else {
      await resumeMusic();
    }
  },

  checkPlaying: async () => {
    try {
      const playing = await invoke<boolean>("is_playing");
      set({ isPlaying: playing });
    } catch (error) {
      console.error("Failed to check playing state:", error);
    }
  },

  loadTracksFromDb: async () => {
    try {
      await invoke("load_from_db");
      get().incrementRefreshKey();
    } catch (error) {
      console.error("Failed to load tracks from database:", error);
    }
  },
}));
