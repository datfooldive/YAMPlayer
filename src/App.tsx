import { useEffect } from "react";
import { Outlet } from "react-router";
import { Sidebar } from "@/components/Sidebar";
import { PlayerControls } from "@/components/PlayerControls";
import { useMusicStore } from "@/store/musicStore";
import { useLocation } from "react-router";

function App() {
  const location = useLocation();
  const { currentTrack, trackInfo, loadCurrentTrack } = useMusicStore();

  useEffect(() => {
    loadCurrentTrack();
  }, []);

  const isSettingsRoute = location.pathname === "/settings";

  return (
    <div className="h-screen w-screen flex flex-col bg-background dark">
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <Outlet />
          </div>
          {!isSettingsRoute && (
            <PlayerControls
              currentTrack={currentTrack}
              trackInfo={trackInfo}
              loadCurrentTrack={loadCurrentTrack}
            />
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
