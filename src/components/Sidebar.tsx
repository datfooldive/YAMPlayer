import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { FolderOpen, Home, Music } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

interface SidebarProps {
  onIndexed: () => void;
}

export function Sidebar({ onIndexed }: SidebarProps) {
  const handleIndexFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Music Folder",
      });

      if (selected && typeof selected === "string") {
        await invoke("index_folder", { path: selected });
        onIndexed();
      }
    } catch (error) {
      console.error("Failed to index folder:", error);
    }
  };

  return (
    <div className="w-64 bg-sidebar border-r border-sidebar-border h-full flex flex-col">
      <div className="p-6">
        <h1 className="text-2xl font-bold">Music Player</h1>
      </div>

      <Separator />

      <div className="flex-1 p-4 space-y-2">
        <Button
          variant="ghost"
          className="w-full justify-start gap-3"
        >
          <Home className="w-5 h-5" />
          Home
        </Button>
        <Button
          variant="ghost"
          className="w-full justify-start gap-3"
        >
          <Music className="w-5 h-5" />
          Your Library
        </Button>
      </div>

      <Separator />

      <div className="p-4">
        <Button
          onClick={handleIndexFolder}
          className="w-full gap-2"
          variant="default"
        >
          <FolderOpen className="w-5 h-5" />
          Index Folder
        </Button>
      </div>
    </div>
  );
}

