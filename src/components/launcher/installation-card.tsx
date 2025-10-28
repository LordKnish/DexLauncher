import { useState } from "react"
import { Rocket, Trash2, Download, Loader2, Settings } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import { VersionSelector } from "./version-selector"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

type InstallationStatus =
  | "not_installed"
  | "downloading"
  | "extracting"
  | "verifying"
  | "installed"
  | "error"

interface Version {
  version: string
  installed: boolean
  isBeta?: boolean
  date?: string
}

interface InstallationCardProps {
  version: string
  versions: Version[]
  status: InstallationStatus
  progress?: number
  statusMessage?: string
  isLatestVersion?: boolean
  onLaunch?: () => void
  onInstall?: () => void
  onUpdate?: () => void
  onDelete?: () => void
  onCancel?: () => void
  onVersionSelect?: (version: string) => void
  className?: string
}

export function InstallationCard({
  version,
  versions,
  status,
  progress = 0,
  statusMessage,
  isLatestVersion = true,
  onLaunch,
  onInstall,
  onUpdate,
  onDelete,
  onCancel,
  onVersionSelect,
  className,
}: InstallationCardProps) {
  const [settingsOpen, setSettingsOpen] = useState(false)
  const isInstalled = status === "installed"
  const isInstalling =
    status === "downloading" || status === "extracting" || status === "verifying"
  const hasError = status === "error"

  return (
    <Card className={cn("animate-fade-in-up backdrop-blur-sm", className)}>
      <CardHeader>
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-3 flex-1">
            {onVersionSelect && (
              <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
                <DialogTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    title="Settings"
                  >
                    <Settings className="h-4 w-4" />
                  </Button>
                </DialogTrigger>
                <DialogContent className="max-w-md">
                  <DialogHeader>
                    <DialogTitle>Game Settings</DialogTitle>
                    <DialogDescription>
                      Select a version to play
                    </DialogDescription>
                  </DialogHeader>
                  <VersionSelector
                    versions={versions}
                    currentVersion={version}
                    onVersionSelect={(v) => {
                      onVersionSelect(v)
                      setSettingsOpen(false)
                    }}
                  />
                </DialogContent>
              </Dialog>
            )}
            <CardTitle className="text-2xl">
              Pokémon Infinite Fusion
            </CardTitle>
          </div>
          <div className="flex flex-col items-end gap-2">
            {isInstalled && (
              <Badge className="bg-green-500/20 text-green-400 hover:bg-green-500/30">
                ✓ Installed
              </Badge>
            )}
            {hasError && (
              <Badge variant="destructive">Error</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Progress Bar (shown during installation) */}
        {isInstalling && (
          <div className="space-y-2">
            <div className="relative">
              <Progress
                value={progress}
                className="h-3 bg-gradient-fusion animate-shimmer"
              />
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-xs font-bold text-white drop-shadow-lg">
                  {progress}%
                </span>
              </div>
            </div>
            <p className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              {statusMessage || "Installing..."}
            </p>
          </div>
        )}

        {/* Error Message */}
        {hasError && statusMessage && (
          <div className="rounded-lg border border-destructive bg-destructive/10 p-4">
            <p className="text-sm text-destructive">{statusMessage}</p>
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex gap-3">
          {isInstalled && (
            <>
              <Button
                size="lg"
                className="flex-1 bg-gradient-pokemon text-lg font-bold uppercase shadow-lg transition-all hover:scale-105 hover:shadow-xl animate-pulse-glow"
                onClick={onLaunch}
              >
                <Rocket className="mr-2 h-5 w-5" />
                Launch Game
              </Button>
              {!isLatestVersion && onUpdate && (
                <Button
                  size="lg"
                  className="flex-1 bg-gradient-pokemon text-lg font-bold uppercase shadow-lg transition-all hover:scale-105 hover:shadow-xl"
                  onClick={onUpdate}
                >
                  <Download className="mr-2 h-5 w-5" />
                  Update
                </Button>
              )}
              <Button
                size="lg"
                variant="outline"
                className="border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground"
                onClick={onDelete}
                title="Delete installation"
              >
                <Trash2 className="h-5 w-5" />
              </Button>
            </>
          )}

          {status === "not_installed" && (
            <Button
              size="lg"
              className="flex-1 bg-gradient-pokemon text-lg font-bold uppercase shadow-lg transition-all hover:scale-105 hover:shadow-xl"
              onClick={onInstall}
            >
              <Download className="mr-2 h-5 w-5" />
              Install
            </Button>
          )}

          {isInstalling && (
            <Button
              size="lg"
              variant="outline"
              className="flex-1"
              onClick={onCancel}
            >
              Cancel
            </Button>
          )}

          {hasError && (
            <Button
              size="lg"
              className="flex-1 bg-gradient-pokemon text-lg font-bold uppercase"
              onClick={onInstall}
            >
              <Download className="mr-2 h-5 w-5" />
              Retry Install
            </Button>
          )}
        </div>

        {/* Additional Info */}
        {isInstalled && (
          <div className="flex items-center justify-between text-sm text-muted-foreground">
            <span>Installation complete</span>
            <span>Ready to play</span>
          </div>
        )}
      </CardContent>
    </Card>
  )
}