import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "@/components/Sidebar";
import { MusicList } from "@/components/MusicList";
import { PlayerControls } from "@/components/PlayerControls";

function App() {
  const [currentTrack, setCurrentTrack] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    loadCurrentTrack();
  }, []);

  const loadCurrentTrack = async () => {
    try {
      const track = await invoke<string | null>("get_current_track");
      setCurrentTrack(track);
    } catch (error) {
      console.error("Failed to load current track:", error);
    }
  };

  const handlePlay = async (path: string) => {
    try {
      await invoke("play_music", { path });
      setCurrentTrack(path);
    } catch (error) {
      console.error("Failed to play music:", error);
    }
  };

  const handleIndexed = () => {
    setRefreshKey((prev) => prev + 1);
  };

  return (
    <div className="h-screen w-screen flex flex-col bg-background dark">
      <div className="flex flex-1 overflow-hidden">
        <Sidebar onIndexed={handleIndexed} />
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-hidden bg-gradient-to-b from-background to-background/95">
            <div className="h-full p-6">
              <h2 className="text-3xl font-bold mb-6">Your Library</h2>
              <MusicList key={refreshKey} onPlay={handlePlay} currentTrack={currentTrack} />
            </div>
          </div>
          <PlayerControls currentTrack={currentTrack} />
        </div>
      </div>
    </div>
  );
}

export default App;
