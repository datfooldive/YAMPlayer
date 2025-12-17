import { createBrowserRouter, RouterProvider } from 'react-router';
import App from './App';
import { Library } from './routes/library';
import { NowPlaying } from './routes/now-playing';
import { SettingsRoute } from './routes/settings';

const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      {
        index: true,
        element: <Library />,
      },
      {
        path: 'now-playing',
        element: <NowPlaying />,
      },
      {
        path: 'settings',
        element: <SettingsRoute />,
      },
    ],
  },
]);

export function Router() {
  return <RouterProvider router={router} />;
}
