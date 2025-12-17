import { NowPlayingView } from '@/components/NowPlayingView';
import { useMusicStore } from '@/store/musicStore';
import { useNavigate } from 'react-router';

export function NowPlaying() {
  const { currentTrack, trackInfo, playMusic } = useMusicStore();
  const navigate = useNavigate();

  const handleBack = () => {
    navigate('/');
  };

  const handlePlay = async (path: string) => {
    await playMusic(path);
  };

  return (
    <NowPlayingView 
      currentTrack={currentTrack} 
      trackInfo={trackInfo}
      onPlay={handlePlay} 
      onBack={handleBack} 
    />
  );
}