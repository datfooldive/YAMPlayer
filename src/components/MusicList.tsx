import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Play, Pause, Music } from "lucide-react";

interface MusicFile {
  path: string;
  name: string;
  artist: string | null;
  album: string | null;
  title: string | null;
}

interface MusicListProps {
  onPlay: (path: string) => Promise<void>;
  currentTrack: string | null;
}

export function MusicList({ onPlay, currentTrack }: MusicListProps) {
  const [tracks, setTracks] = useState<MusicFile[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);

  useEffect(() => {
    loadMusic();
    const interval = setInterval(checkPlaying, 500);
    return () => clearInterval(interval);
  }, []);

  const loadMusic = async () => {
    try {
      setLoading(true);
      const music = await invoke<MusicFile[]>("list_music");
      setTracks(music);
    } catch (error) {
      console.error("Failed to load music:", error);
    } finally {
      setLoading(false);
    }
  };

  const checkPlaying = async () => {
    try {
      const playing = await invoke<boolean>("is_playing");
      setIsPlaying(playing);
    } catch (error) {
      console.error("Failed to check playing state:", error);
    }
  };

  const handlePlayToggle = async (track: MusicFile) => {
    try {
      if (currentTrack === track.path) {
        const playing = await invoke<boolean>("is_playing");
        if (playing) {
          await invoke("pause_music");
          setIsPlaying(false);
        } else {
          await invoke("resume_music");
          setIsPlaying(true);
        }
      } else {
        await onPlay(track.path);
        setIsPlaying(true);
      }
    } catch (error) {
      console.error("Failed to toggle playback:", error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading music...</div>
      </div>
    );
  }

  if (tracks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
        <Music className="w-16 h-16 mb-4 opacity-50" />
        <p>No music found</p>
        <p className="text-sm mt-2">Index a folder to get started</p>
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="space-y-1 p-4">
        {tracks.map((track, index) => (
          <div
            key={track.path}
            className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
              currentTrack === track.path
                ? "bg-accent text-accent-foreground"
                : "hover:bg-accent/50"
            }`}
            onClick={() => onPlay(track.path)}
          >
            <div className="shrink-0 w-8 h-8 flex items-center justify-center">
              {currentTrack === track.path ? (
                <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
              ) : (
                <span className="text-xs text-muted-foreground">{index + 1}</span>
              )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="font-medium truncate">
                {track.artist && track.title
                  ? `${track.artist} - ${track.title}`
                  : track.title || track.name}
              </div>
              <div className="text-xs text-muted-foreground truncate">
                {track.album || track.path}
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="shrink-0"
              onClick={(e) => {
                e.stopPropagation();
                handlePlayToggle(track);
              }}
            >
              {currentTrack === track.path && isPlaying ? (
                <Pause className="w-4 h-4" />
              ) : (
                <Play className="w-4 h-4" />
              )}
            </Button>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}
