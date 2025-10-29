import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { FolderOpen } from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Checkbox } from "@/components/ui/checkbox"

interface InstallDirectoryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gameName: string
  onConfirm: (path: string, createStartMenu: boolean, createDesktop: boolean, addToSteam: boolean) => void
}

export function InstallDirectoryDialog({
  open,
  onOpenChange,
  gameName,
  onConfirm,
}: InstallDirectoryDialogProps) {
  // Default path structure: C:\DexLauncher\Games\{Game Name}
  // Using C:\ instead of Program Files to avoid UAC permission issues
  const defaultPath = `C:\\DexLauncher\\Games\\${gameName}`
  const [installPath, setInstallPath] = useState(defaultPath)
  const [createStartMenu, setCreateStartMenu] = useState(true)
  const [createDesktop, setCreateDesktop] = useState(true)
  const [addToSteam, setAddToSteam] = useState(true)
  const [repositorySize, setRepositorySize] = useState<number | null>(null)
  const [loadingSize, setLoadingSize] = useState(false)

  // Fetch repository size when dialog opens
  useEffect(() => {
    if (open) {
      setInstallPath(defaultPath)
      setLoadingSize(true)
      invoke<number>("get_repository_size")
        .then((size) => {
          setRepositorySize(size)
        })
        .catch((err) => {
          console.error("Failed to fetch repository size:", err)
          setRepositorySize(null)
        })
        .finally(() => {
          setLoadingSize(false)
        })
    }
  }, [open, defaultPath])

  // Format bytes to human-readable size
  const formatSize = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024)
    const mb = bytes / (1024 * 1024)
    if (gb >= 1) {
      return `${gb.toFixed(2)} GB`
    }
    return `${mb.toFixed(0)} MB`
  }

  const handleBrowse = async () => {
    try {
      const selected = await invoke<string | null>("select_install_directory")
      if (selected) {
        // Append game name to selected path
        setInstallPath(`${selected}\\DexLauncher\\Games\\${gameName}`)
      }
    } catch (err) {
      console.error("Failed to select directory:", err)
    }
  }

  const handleConfirm = () => {
    onConfirm(installPath, createStartMenu, createDesktop, addToSteam)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Choose Installation Location</DialogTitle>
          <DialogDescription>
            Select where you want to install {gameName}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Installation Path */}
          <div className="space-y-2">
            <Label htmlFor="install-path">Installation Path</Label>
            <div className="flex gap-2">
              <Input
                id="install-path"
                value={installPath}
                onChange={(e) => setInstallPath(e.target.value)}
                className="flex-1"
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleBrowse}
                title="Browse for folder"
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
            <p className="text-sm text-muted-foreground">
              Game will be installed to: <span className="font-mono">{installPath}</span>
            </p>
          </div>

          {/* Shortcuts */}
          <div className="space-y-3 rounded-lg border p-4">
            <Label>Shortcuts</Label>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="start-menu"
                checked={createStartMenu}
                onCheckedChange={(checked) => setCreateStartMenu(checked as boolean)}
              />
              <label
                htmlFor="start-menu"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Add to Start Menu
              </label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="desktop"
                checked={createDesktop}
                onCheckedChange={(checked) => setCreateDesktop(checked as boolean)}
              />
              <label
                htmlFor="desktop"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Create Desktop Shortcut
              </label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="add-to-steam"
                checked={addToSteam}
                onCheckedChange={(checked) => setAddToSteam(checked as boolean)}
              />
              <label
                htmlFor="add-to-steam"
                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Add to Steam
              </label>
            </div>
          </div>

          {/* Size Info */}
          <div className="rounded-lg bg-muted p-3 text-sm">
            <p className="text-muted-foreground">
              <span className="font-semibold">Size required:</span>{" "}
              {loadingSize ? (
                "Loading..."
              ) : repositorySize ? (
                formatSize(repositorySize)
              ) : (
                "~1.5 GB"
              )}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleConfirm}>
            Install
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
