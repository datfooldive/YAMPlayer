import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { AudioWaveformIcon, Music, Settings as SettingsIcon } from "lucide-react";
import { useLocation, useNavigate } from 'react-router';

export function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  
  const currentPath = location.pathname;
  return (
    <div className="w-64 bg-sidebar border-r border-sidebar-border h-full flex flex-col">
      <div className="p-6 flex items-center justify-center">
        <AudioWaveformIcon className="w-6 h-6 mr-1.5" />
        <h1 className="text-xl font-bold">YAMPlayer</h1>
      </div>

      <Separator />

      <div className="flex-1 p-4 space-y-2">
        <Button
          variant={currentPath === "/" || currentPath === "/now-playing" ? "secondary" : "ghost"}
          className="w-full justify-start gap-3"
          onClick={() => navigate("/")}
        >
          <Music className="w-5 h-5" />
          Your Library
        </Button>
        <Button
          variant={currentPath === "/settings" ? "secondary" : "ghost"}
          className="w-full justify-start gap-3"
          onClick={() => navigate("/settings")}
        >
          <SettingsIcon className="w-5 h-5" />
          Settings
        </Button>
      </div>
    </div>
  );
}
