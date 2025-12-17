import { useEffect } from 'react';
import { MusicList } from '@/components/MusicList';
import { useMusicStore } from '@/store/musicStore';
import { useNavigate } from 'react-router';

export function Library() {
  const { currentTrack, refreshKey, playMusic, loadCurrentTrack } = useMusicStore();
  const navigate = useNavigate();

  useEffect(() => {
    loadCurrentTrack();
  }, [loadCurrentTrack]);

  const handlePlay = async (path: string) => {
    if (currentTrack === path) {
      navigate('/now-playing');
    } else {
      await playMusic(path);
      navigate('/now-playing');
    }
  };

  return (
    <div className="h-full p-6">
      <MusicList key={refreshKey} onPlay={handlePlay} currentTrack={currentTrack} />
    </div>
  );
}
