import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { FolderOpen, RefreshCw, Trash2, CheckCircle2, AlertCircle } from "lucide-react";

interface IndexedFolder {
  id: number;
  path: string;
  last_indexed: string;
}

interface SettingsProps {
  onIndexed: () => void;
}

export function Settings({ onIndexed }: SettingsProps) {
  const [folders, setFolders] = useState<IndexedFolder[]>([]);
  const [hasChanges, setHasChanges] = useState(false);
  const [isIndexing, setIsIndexing] = useState(false);
  const [isChecking, setIsChecking] = useState(false);

  useEffect(() => {
    loadFolders();
    checkChanges();
  }, []);

  const loadFolders = async () => {
    try {
      const indexedFolders = await invoke<IndexedFolder[]>("get_indexed_folders");
      setFolders(indexedFolders);
    } catch (error) {
      console.error("Failed to load folders:", error);
    }
  };

  const checkChanges = async () => {
    try {
      setIsChecking(true);
      const changes = await invoke<boolean>("check_for_changes");
      setHasChanges(changes);
    } catch (error) {
      console.error("Failed to check for changes:", error);
    } finally {
      setIsChecking(false);
    }
  };

  const handleIndexFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Music Folder",
      });

      if (selected && typeof selected === "string") {
        setIsIndexing(true);
        await invoke("index_folder", { path: selected });
        await loadFolders();
        await checkChanges();
        onIndexed();
      }
    } catch (error) {
      console.error("Failed to index folder:", error);
    } finally {
      setIsIndexing(false);
    }
  };

  const handleReindex = async (folderPath: string) => {
    try {
      setIsIndexing(true);
      await invoke("index_folder", { path: folderPath });
      await loadFolders();
      await checkChanges();
      onIndexed();
    } catch (error) {
      console.error("Failed to reindex folder:", error);
    } finally {
      setIsIndexing(false);
    }
  };

  const handleRemoveFolder = async (folderId: number, folderPath: string) => {
    if (!confirm(`Are you sure you want to remove "${folderPath}" from indexed folders?`)) {
      return;
    }
    try {
      await invoke("remove_folder", { folderId });
      await loadFolders();
      await checkChanges();
      onIndexed();
    } catch (error) {
      console.error("Failed to remove folder:", error);
    }
  };

  const formatDate = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return date.toLocaleString();
    } catch {
      return dateString;
    }
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        <div>
          <h1 className="text-3xl font-bold mb-2">Settings</h1>
          <p className="text-muted-foreground">Manage your music library indexing</p>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Indexed Folders</CardTitle>
                <CardDescription>
                  Folders that have been indexed for music files
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                {hasChanges && (
                  <div className="flex items-center gap-2 text-amber-500">
                    <AlertCircle className="w-4 h-4" />
                    <span className="text-sm">Changes detected</span>
                  </div>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={checkChanges}
                  disabled={isChecking}
                >
                  <RefreshCw className={`w-4 h-4 mr-2 ${isChecking ? "animate-spin" : ""}`} />
                  Check Changes
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {folders.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <FolderOpen className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No folders indexed yet</p>
                <p className="text-sm mt-2">Click "Index New Folder" to get started</p>
              </div>
            ) : (
              <div className="space-y-3">
                {folders.map((folder) => (
                  <div
                    key={folder.id}
                    className="flex items-center justify-between p-4 border rounded-lg hover:bg-accent/50 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-medium truncate">{folder.path}</div>
                      <div className="text-sm text-muted-foreground mt-1">
                        Last indexed: {formatDate(folder.last_indexed)}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 ml-4">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleReindex(folder.path)}
                        disabled={isIndexing}
                        title="Reindex this folder"
                      >
                        <RefreshCw className={`w-4 h-4 ${isIndexing ? "animate-spin" : ""}`} />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleRemoveFolder(folder.id, folder.path)}
                        disabled={isIndexing}
                        title="Remove this folder"
                        className="text-destructive hover:text-destructive"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Index New Folder</CardTitle>
            <CardDescription>
              Add a new folder to scan for music files
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              onClick={handleIndexFolder}
              disabled={isIndexing}
              className="w-full"
            >
              <FolderOpen className={`w-5 h-5 mr-2 ${isIndexing ? "animate-spin" : ""}`} />
              {isIndexing ? "Indexing..." : "Index New Folder"}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

