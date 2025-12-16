import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Play, Music } from "lucide-react";

interface MusicFile {
  path: string;
  name: string;
}

interface MusicListProps {
  onPlay: (path: string) => void;
  currentTrack: string | null;
}

export function MusicList({ onPlay, currentTrack }: MusicListProps) {
  const [tracks, setTracks] = useState<MusicFile[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadMusic();
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
            <div className="flex-shrink-0 w-8 h-8 flex items-center justify-center">
              {currentTrack === track.path ? (
                <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
              ) : (
                <span className="text-xs text-muted-foreground">{index + 1}</span>
              )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="font-medium truncate">{track.name}</div>
              <div className="text-xs text-muted-foreground truncate">{track.path}</div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="flex-shrink-0"
              onClick={(e) => {
                e.stopPropagation();
                onPlay(track.path);
              }}
            >
              <Play className="w-4 h-4" />
            </Button>
          </div>
        ))}
      </div>
    </ScrollArea>
  );
}

