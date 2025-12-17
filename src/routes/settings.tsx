import { Settings } from "@/components/Settings";
import { useMusicStore } from "@/store/musicStore";

export function SettingsRoute() {
  const { loadTracksFromDb } = useMusicStore();

  const handleIndexed = () => {
    loadTracksFromDb();
  };

  return <Settings onIndexed={handleIndexed} />;
}
