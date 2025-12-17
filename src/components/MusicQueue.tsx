import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Play, Pause, Music } from "lucide-react";
import { useMusicStore } from "@/store/musicStore";

interface MusicFile {
  path: string;
  name: string;
  artist: string | null;
  album: string | null;
  title: string | null;
  thumbnail: string | null;
}

interface MusicQueueProps {
  onPlay: (path: string) => Promise<void>;
  currentTrack: string | null;
}

export function MusicQueue({ onPlay, currentTrack }: MusicQueueProps) {
  const [tracks, setTracks] = useState<MusicFile[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const { isPlaying, checkPlaying } = useMusicStore();

  useEffect(() => {
    loadMusic();
    const interval = setInterval(checkPlaying, 500);
    return () => clearInterval(interval);
  }, [checkPlaying]);

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

  const handlePlayToggle = async (track: MusicFile) => {
    try {
      if (currentTrack === track.path) {
        await onPlay(track.path);
      } else {
        await onPlay(track.path);
      }
    } catch (error) {
      console.error("Failed to toggle playback:", error);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-32">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  if (tracks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-32 text-muted-foreground">
        <Music className="w-12 h-12 mb-3 opacity-50" />
        <p className="text-sm">No music found</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {tracks.map((track) => (
        <div
          key={track.path}
          className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${
            currentTrack === track.path
              ? "bg-accent text-accent-foreground"
              : "hover:bg-accent/50"
          }`}
          onClick={() => onPlay(track.path)}
        >
           <div className="shrink-0 w-12 h-12 flex items-center justify-center bg-muted rounded-md">
            {track.thumbnail ? (
              <img src={track.thumbnail} alt="album art" className="w-full h-full object-cover rounded-md" />
            ) : (
              <Music className="w-6 h-6 text-muted-foreground" />
            )}
          </div>
          <div className="flex-1 min-w-0">
            <div className="font-medium truncate">
              {track.artist && track.title
                ? `${track.artist} - ${track.title}`
                : track.title || track.name}
            </div>
            <div className="text-xs text-muted-foreground truncate">
              {track.album || 'Unknown Album'}
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
  );
}