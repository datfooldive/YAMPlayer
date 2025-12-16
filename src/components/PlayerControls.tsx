import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Play, Pause, SkipBack, SkipForward, Volume2 } from "lucide-react";

interface PlayerControlsProps {
  currentTrack: string | null;
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function PlayerControls({ currentTrack }: PlayerControlsProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [volume, setVolume] = useState([50]);
  const [currentTime, setCurrentTime] = useState(0);
  const [totalDuration, setTotalDuration] = useState<number | null>(null);

  useEffect(() => {
    const loadVolume = async () => {
      try {
        const vol = await invoke<number>("get_volume");
        setVolume([(vol * 100) as number]);
      } catch (error) {
        console.error("Failed to load volume:", error);
      }
    };
    loadVolume();
  }, []);

  useEffect(() => {
    const checkPlaying = async () => {
      try {
        const playing = await invoke<boolean>("is_playing");
        setIsPlaying(playing);
      } catch (error) {
        console.error("Failed to check playing state:", error);
      }
    };

    const updatePosition = async () => {
      try {
        const [elapsed, total] = await invoke<[number, number | null]>("get_playback_position");
        setCurrentTime(elapsed);
        if (total !== null) {
          setTotalDuration(total);
        }
      } catch (error) {
        console.error("Failed to get playback position:", error);
      }
    };

    const interval = setInterval(() => {
      checkPlaying();
      updatePosition();
    }, 500);
    return () => clearInterval(interval);
  }, [currentTrack]);

  const handlePlayPause = async () => {
    try {
      if (isPlaying) {
        await invoke("pause_music");
      } else {
        await invoke("resume_music");
      }
      setIsPlaying(!isPlaying);
    } catch (error) {
      console.error("Failed to toggle playback:", error);
    }
  };

  const handleVolumeChange = async (value: number[]) => {
    setVolume(value);
    try {
      await invoke("set_volume", { volume: value[0] / 100 });
    } catch (error) {
      console.error("Failed to set volume:", error);
    }
  };

  const handleSkipBack = () => {
  };

  const handleSkipForward = () => {
  };

  const trackName = currentTrack ? currentTrack.split("/").pop() || "Unknown" : "No track selected";

  return (
    <div className="h-24 bg-background border-t border-border flex items-center px-6 gap-6">
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium truncate">{trackName}</div>
        <div className="text-xs text-muted-foreground">
          {formatTime(currentTime)} {totalDuration !== null && ` / ${formatTime(totalDuration)}`}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          onClick={handleSkipBack}
          disabled={!currentTrack}
        >
          <SkipBack className="w-5 h-5" />
        </Button>
        <Button
          variant="default"
          size="icon"
          onClick={handlePlayPause}
          disabled={!currentTrack}
          className="w-12 h-12 rounded-full"
        >
          {isPlaying ? (
            <Pause className="w-5 h-5" />
          ) : (
            <Play className="w-5 h-5 ml-0.5" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleSkipForward}
          disabled={!currentTrack}
        >
          <SkipForward className="w-5 h-5" />
        </Button>
      </div>

      <div className="flex items-center gap-3 w-32">
        <Volume2 className="w-4 h-4 text-muted-foreground" />
        <Slider
          value={volume}
          onValueChange={handleVolumeChange}
          max={100}
          step={1}
          className="flex-1"
        />
      </div>
    </div>
  );
}

