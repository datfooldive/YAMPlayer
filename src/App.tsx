import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

function App() {
  return (
    <main className="min-h-screen pt-[10vh] flex flex-col items-center justify-center text-center font-sans text-base leading-6 font-normal text-gray-900 bg-gray-50">
      <Card>
        <CardHeader>
          <CardTitle>Music Player</CardTitle>
          <CardDescription>Listen to your favorite songs</CardDescription>
        </CardHeader>
        <CardContent>
        <p>
          Welcome to the Music Player!
        </p>
        </CardContent>
      </Card>
    </main>
  );
}

export default App;
