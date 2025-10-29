import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Download, RefreshCw, AlertCircle, CheckCircle2 } from "lucide-react";
import { useToast } from "@/components/ui/use-toast";

interface LauncherUpdateInfo {
  current_version: string;
  latest_version: string;
  release_notes: string;
  release_date: string;
  download_url: string;
  update_available: boolean;
}

interface LauncherUpdateProgress {
  downloaded: number;
  total: number;
  percentage: number;
  speed_bps: number;
}

type LauncherUpdateStatus =
  | { status: "Checking" }
  | { status: "Available"; data: LauncherUpdateInfo }
  | { status: "Downloading"; data: LauncherUpdateProgress }
  | { status: "Installing" }
  | { status: "ReadyToRestart" }
  | { status: "UpToDate" }
  | { status: "Error"; data: string };

export function UpdateNotificationDialog() {
  const [updateInfo, setUpdateInfo] = useState<LauncherUpdateInfo | null>(null);
  const [updateStatus, setUpdateStatus] = useState<LauncherUpdateStatus | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    // Listen for update status events from backend
    const unlisten = listen<LauncherUpdateStatus>("launcher-update-status", (event) => {
      const status = event.payload;
      setUpdateStatus(status);

      if (status.status === "Available") {
        setUpdateInfo(status.data);
        setIsDialogOpen(true);
        setIsChecking(false);
      } else if (status.status === "UpToDate") {
        setIsChecking(false);
      } else if (status.status === "Downloading") {
        setIsDownloading(true);
      } else if (status.status === "ReadyToRestart") {
        setIsDownloading(false);
        toast({
          title: "Update Ready",
          description: "The update has been installed. Restart to apply changes.",
        });
      } else if (status.status === "Error") {
        setIsChecking(false);
        setIsDownloading(false);
        toast({
          title: "Update Error",
          description: status.data,
          variant: "destructive",
        });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [toast]);

  const checkForUpdates = async () => {
    setIsChecking(true);
    try {
      const update = await invoke<LauncherUpdateInfo | null>("check_launcher_update");
      if (!update) {
        toast({
          title: "No Updates",
          description: "You're running the latest version!",
        });
      }
    } catch (error) {
      toast({
        title: "Check Failed",
        description: String(error),
        variant: "destructive",
      });
    } finally {
      setIsChecking(false);
    }
  };

  const installUpdate = async () => {
    try {
      await invoke("install_launcher_update");
    } catch (error) {
      toast({
        title: "Installation Failed",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const restartLauncher = async () => {
    try {
      await invoke("restart_launcher");
    } catch (error) {
      toast({
        title: "Restart Failed",
        description: String(error),
        variant: "destructive",
      });
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const renderDialogContent = () => {
    if (!updateStatus) return null;

    switch (updateStatus.status) {
      case "Checking":
        return (
          <div className="flex flex-col items-center justify-center py-8 space-y-4">
            <RefreshCw className="h-12 w-12 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">Checking for updates...</p>
          </div>
        );

      case "Available":
        return (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Download className="h-5 w-5" />
                Update Available
              </DialogTitle>
              <DialogDescription>
                A new version of DexLauncher is available!
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-4">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Current Version:</span>
                <Badge variant="outline">{updateInfo?.current_version}</Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Latest Version:</span>
                <Badge variant="default">{updateInfo?.latest_version}</Badge>
              </div>

              {updateInfo?.release_notes && (
                <div className="space-y-2">
                  <h4 className="text-sm font-semibold">Release Notes:</h4>
                  <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                    <div className="text-sm whitespace-pre-wrap">
                      {updateInfo.release_notes}
                    </div>
                  </ScrollArea>
                </div>
              )}
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                Later
              </Button>
              <Button onClick={installUpdate} disabled={isDownloading}>
                {isDownloading ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Downloading...
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    Update Now
                  </>
                )}
              </Button>
            </DialogFooter>
          </>
        );

      case "Downloading":
        const progress = updateStatus.data;
        return (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Download className="h-5 w-5 animate-pulse" />
                Downloading Update
              </DialogTitle>
              <DialogDescription>
                Please wait while the update is being downloaded...
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-4">
              <Progress value={progress.percentage} className="w-full" />
              <div className="flex items-center justify-between text-sm text-muted-foreground">
                <span>{progress.percentage.toFixed(1)}%</span>
                <span>
                  {formatBytes(progress.downloaded)} / {formatBytes(progress.total)}
                </span>
              </div>
            </div>
          </>
        );

      case "Installing":
        return (
          <div className="flex flex-col items-center justify-center py-8 space-y-4">
            <RefreshCw className="h-12 w-12 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">Installing update...</p>
          </div>
        );

      case "ReadyToRestart":
        return (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
                Update Ready
              </DialogTitle>
              <DialogDescription>
                The update has been installed successfully. Restart to apply changes.
              </DialogDescription>
            </DialogHeader>

            <DialogFooter className="mt-4">
              <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                Later
              </Button>
              <Button onClick={restartLauncher}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Restart Now
              </Button>
            </DialogFooter>
          </>
        );

      case "Error":
        return (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <AlertCircle className="h-5 w-5 text-destructive" />
                Update Error
              </DialogTitle>
              <DialogDescription>
                An error occurred while updating the launcher.
              </DialogDescription>
            </DialogHeader>

            <div className="py-4">
              <div className="rounded-md bg-destructive/10 p-4 text-sm text-destructive">
                {updateStatus.data}
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                Close
              </Button>
              <Button onClick={checkForUpdates}>
                <RefreshCw className="mr-2 h-4 w-4" />
                Try Again
              </Button>
            </DialogFooter>
          </>
        );

      default:
        return null;
    }
  };

  return (
    <>
      {/* Update Check Button (can be placed in menu or settings) */}
      <Button
        variant="ghost"
        size="sm"
        onClick={checkForUpdates}
        disabled={isChecking}
        className="gap-2"
      >
        {isChecking ? (
          <RefreshCw className="h-4 w-4 animate-spin" />
        ) : (
          <Download className="h-4 w-4" />
        )}
        Check for Updates
      </Button>

      {/* Update Badge (shows when update is available) */}
      {updateInfo && !isDialogOpen && (
        <Button
          variant="default"
          size="sm"
          onClick={() => setIsDialogOpen(true)}
          className="gap-2 animate-pulse"
        >
          <Download className="h-4 w-4" />
          Update Available
          <Badge variant="secondary" className="ml-1">
            {updateInfo.latest_version}
          </Badge>
        </Button>
      )}

      {/* Update Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="sm:max-w-[500px]">
          {renderDialogContent()}
        </DialogContent>
      </Dialog>
    </>
  );
}
