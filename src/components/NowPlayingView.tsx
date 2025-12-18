import { MusicQueue } from "@/components/MusicQueue";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ChevronLeft } from "lucide-react";
import { useMusicStore } from "@/store/musicStore";
import { useNavigate } from "react-router";

export function NowPlayingView() {
  const navigate = useNavigate();
  const { trackInfo, playMusic } = useMusicStore();
  return (
    <div className="h-full flex flex-col">
      <div className="px-2 py-4 flex justify-between items-center">
        <Button variant="link" onClick={() => navigate("/")}>
          <ChevronLeft /> Back to Library
        </Button>
      </div>
      <div className="flex-1 flex px-6 gap-6 overflow-hidden">
        <div className="shrink-0">
          <div className="aspect-square w-80">
            {trackInfo && trackInfo.thumbnail ? (
              <img
                src={trackInfo.thumbnail}
                alt={trackInfo.album || trackInfo.title || "album art"}
                className="w-full h-full object-cover rounded-lg"
              />
            ) : (
              <div className="w-full h-full bg-muted rounded-lg flex items-center justify-center">
                <span className="text-muted-foreground">Album Art</span>
              </div>
            )}
          </div>
        </div>
        <div className="flex-1 flex flex-col overflow-hidden">
          <h2 className="text-2xl font-bold mb-4 shrink-0">Up Next</h2>
          <div className="flex-1 overflow-hidden">
            <ScrollArea className="h-full w-full">
              <div className="pr-6">
                <MusicQueue onPlay={playMusic} />
              </div>
            </ScrollArea>
          </div>
        </div>
      </div>
    </div>
  );
}
