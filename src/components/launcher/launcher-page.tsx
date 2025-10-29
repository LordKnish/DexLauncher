import { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { open as openUrl } from "@tauri-apps/plugin-shell"
import { Sidebar } from "./sidebar"
import { HeroBanner } from "./hero-banner"
import { InstallationCard } from "./installation-card"
import { Changelog } from "./changelog"
import { InstallDirectoryDialog } from "./install-directory-dialog"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"

// Tauri backend types
interface Installation {
  id: number
  game_id: string
  version: string
  install_path: string
  installed_at: string
  last_played: string | null
  size_bytes: number | null
  integrity_hash: string | null
  is_valid: boolean
}

interface InstallProgress {
  operation_id: string
  phase: string
  percentage: number
  message: string
}

interface InstallComplete {
  installation_id: number
  game_id: string
  version: string
  install_path: string
}

interface InstallError {
  operation_id: string
  error: string
  can_retry: boolean
}

interface VersionData {
  games: Array<{
    id: string
    name: string
    logo: string
    save_files_path?: string
    versions: Array<{
      version: string
      date: string
      installed: boolean
      isBeta: boolean
      announcement: string
      changelogUrl?: string
      changelog: {
        categories: Array<{
          name: string
          changes: string[]
        }>
      }
    }>
  }>
}

export function LauncherPage() {
  const [versionData, setVersionData] = useState<VersionData | null>(null)
  const [installations, setInstallations] = useState<Installation[]>([])
  const [selectedGame, setSelectedGame] = useState("pokemon-infinite-fusion")
  const [selectedVersion, setSelectedVersion] = useState("6.7")
  const [currentInstallationId, setCurrentInstallationId] = useState<number | null>(null)
  const [installationStatus, setInstallationStatus] = useState<
    "not_installed" | "downloading" | "extracting" | "verifying" | "installed" | "error"
  >("not_installed")
  const [progress, setProgress] = useState(0)
  const [statusMessage, setStatusMessage] = useState<string>("")
  const [currentOperationId, setCurrentOperationId] = useState<string | null>(null)
  const [showInstallDialog, setShowInstallDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [pendingInstallVersion, setPendingInstallVersion] = useState<string>("")

  // Load installations from database
  const loadInstallations = useCallback(async () => {
    try {
      const installs = await invoke<Installation[]>("get_installations")
      setInstallations(installs)
      
      // Check if current game/version is installed
      const currentInstall = installs.find(
        (i) => i.game_id === selectedGame && i.version === selectedVersion
      )
      
      if (currentInstall) {
        setCurrentInstallationId(currentInstall.id)
        setInstallationStatus("installed")
      } else {
        setCurrentInstallationId(null)
        setInstallationStatus("not_installed")
      }
    } catch (err) {
      console.error("Failed to load installations:", err)
    }
  }, [selectedGame, selectedVersion])

  // Load version data from JSON and installations from database
  useEffect(() => {
    // Load version data
    fetch("/versions.json")
      .then((res) => res.json())
      .then((data: VersionData) => {
        setVersionData(data)
        if (data.games.length > 0) {
          const firstGame = data.games[0]
          setSelectedGame(firstGame.id)
          
          if (firstGame.versions.length > 0) {
            setSelectedVersion(firstGame.versions[0].version)
          }
        }
      })
      .catch((err) => console.error("Failed to load version data:", err))

    // Load installations
    loadInstallations()
  }, [])

  // Reload installations when game/version changes
  useEffect(() => {
    loadInstallations()
  }, [selectedGame, selectedVersion, loadInstallations])

  // Listen for installation progress events
  useEffect(() => {
    const unlistenProgress = listen<InstallProgress>("install-progress", (event) => {
      const { phase, percentage, message } = event.payload
      setProgress(Math.round(percentage))
      setStatusMessage(message)
      
      if (phase === "downloading") {
        setInstallationStatus("downloading")
      } else if (phase === "extracting") {
        setInstallationStatus("extracting")
      } else if (phase === "verifying") {
        setInstallationStatus("verifying")
      }
    })

    const unlistenComplete = listen<InstallComplete>("install-complete", (event) => {
      setInstallationStatus("installed")
      setProgress(100)
      setStatusMessage("Installation complete!")
      setCurrentInstallationId(event.payload.installation_id)
      setCurrentOperationId(null)
      // Reload installations
      loadInstallations()
    })

    const unlistenError = listen<InstallError>("install-error", (event) => {
      setInstallationStatus("error")
      setStatusMessage(event.payload.error)
      setCurrentOperationId(null)
    })

    return () => {
      unlistenProgress.then((fn) => fn())
      unlistenComplete.then((fn) => fn())
      unlistenError.then((fn) => fn())
    }
  }, [loadInstallations])

  const handleVersionSelect = (gameId: string, version: string) => {
    setSelectedGame(gameId)
    setSelectedVersion(version)
    // Check if this version is installed
    const game = versionData?.games.find((g) => g.id === gameId)
    const versionInfo = game?.versions.find((v) => v.version === version)
    setInstallationStatus(versionInfo?.installed ? "installed" : "not_installed")
  }

  const handleVersionSelectFromCard = (version: string) => {
    setSelectedVersion(version)
    // Check if this version is installed
    const game = versionData?.games.find((g) => g.id === selectedGame)
    const versionInfo = game?.versions.find((v) => v.version === version)
    setInstallationStatus(versionInfo?.installed ? "installed" : "not_installed")
  }

  const handleLaunch = async () => {
    if (currentInstallationId === null) {
      console.error("No installation ID")
      return
    }

    try {
      await invoke("launch_game", { installationId: currentInstallationId })
    } catch (err) {
      console.error("Failed to launch game:", err)
      setInstallationStatus("error")
      setStatusMessage(`Failed to launch: ${err}`)
    }
  }

  const handleInstall = () => {
    // Show installation directory dialog
    setPendingInstallVersion(selectedVersion)
    setShowInstallDialog(true)
  }

  const handleInstallConfirm = async (
    installPath: string,
    createStartMenu: boolean,
    createDesktop: boolean,
    addToSteam: boolean
  ) => {
    try {
      // Generate operation ID
      const operationId = `install-${selectedGame}-${pendingInstallVersion}-${Date.now()}`
      setCurrentOperationId(operationId)
      setInstallationStatus("downloading")
      setProgress(0)
      setStatusMessage("Preparing installation...")

      // Start installation
      const installationId = await invoke<number>("install_game", {
        operationId,
        gameId: selectedGame,
        version: pendingInstallVersion,
        installPath,
        createStartMenu,
        createDesktop,
        addToSteam,
      })

      console.log("Installation started with ID:", installationId)
    } catch (err) {
      console.error("Failed to install:", err)
      setInstallationStatus("error")
      setStatusMessage(`Installation failed: ${err}`)
      setCurrentOperationId(null)
    }
  }

  const handleUpdate = async () => {
    // For now, update is the same as install (git pull)
    await handleInstall()
  }

  const handleDelete = () => {
    // Show confirmation dialog
    setShowDeleteDialog(true)
  }

  const handleDeleteConfirm = async () => {
    if (currentInstallationId === null) {
      console.error("No installation ID")
      return
    }

    try {
      await invoke("delete_installation", { installationId: currentInstallationId })
      setInstallationStatus("not_installed")
      setProgress(0)
      setCurrentInstallationId(null)
      setStatusMessage("")
      // Reload installations
      await loadInstallations()
    } catch (err) {
      console.error("Failed to delete installation:", err)
      setStatusMessage(`Failed to delete: ${err}`)
    }
  }

  const handleCancel = async () => {
    if (currentOperationId) {
      try {
        await invoke("cancel_operation", { operationId: currentOperationId })
        console.log("Cancellation requested for:", currentOperationId)
      } catch (err) {
        console.error("Failed to cancel:", err)
      }
    }
    
    setInstallationStatus("not_installed")
    setProgress(0)
    setCurrentOperationId(null)
    setStatusMessage("")
  }

  const handleViewFullChangelog = useCallback(async () => {
    const game = versionData?.games.find((g) => g.id === selectedGame)
    const version = game?.versions.find((v) => v.version === selectedVersion)
    const changelogUrl = version?.changelogUrl
    if (changelogUrl) {
      await openUrl(changelogUrl)
    }
  }, [versionData, selectedGame, selectedVersion])

  if (!versionData) {
    return (
      <div className="flex h-screen items-center justify-center bg-[hsl(var(--launcher-bg))]">
        <div className="text-center">
          <div className="mb-4 text-lg text-muted-foreground">Loading...</div>
        </div>
      </div>
    )
  }

  const currentGame = versionData.games.find((g) => g.id === selectedGame)
  const currentVersionData = currentGame?.versions.find((v) => v.version === selectedVersion)
  const isLatestVersion = currentGame?.versions[0].version === selectedVersion

  return (
    <div className="flex h-screen overflow-hidden bg-[hsl(var(--launcher-bg))]">
      {/* Sidebar */}
      <Sidebar
        games={versionData.games}
        selectedGame={selectedGame}
        selectedVersion={selectedVersion}
        onVersionSelect={handleVersionSelect}
        appVersion="0.1.0"
      />

      {/* Main Content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Hero Banner */}
        <HeroBanner />

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto scrollbar-custom">
          <div className="mx-auto max-w-5xl space-y-6 p-8">
            {/* Installation Card */}
            <InstallationCard
              version={selectedVersion}
              versions={currentGame?.versions || []}
              status={installationStatus}
              progress={progress}
              isLatestVersion={isLatestVersion}
              statusMessage={statusMessage || undefined}
              installPath={
                currentInstallationId !== null
                  ? installations.find((i) => i.id === currentInstallationId)?.install_path
                  : undefined
              }
              saveFilesPath={currentGame?.save_files_path}
              onLaunch={handleLaunch}
              onInstall={handleInstall}
              onUpdate={handleUpdate}
              onDelete={handleDelete}
              onCancel={handleCancel}
              onVersionSelect={handleVersionSelectFromCard}
            />

            {/* Changelog */}
            {currentVersionData && (
              <Changelog
                version={currentVersionData.version}
                date={currentVersionData.date}
                isBeta={currentVersionData.isBeta}
                entries={currentVersionData.changelog.categories.map((cat) => ({
                  category: cat.name,
                  changes: cat.changes,
                }))}
                onViewFull={handleViewFullChangelog}
              />
            )}
          </div>
        </div>
      </div>

      {/* Installation Directory Dialog */}
      <InstallDirectoryDialog
        open={showInstallDialog}
        onOpenChange={setShowInstallDialog}
        gameName={currentGame?.name || "Pokemon Infinite Fusion"}
        onConfirm={handleInstallConfirm}
      />

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Installation?</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete Pokemon Infinite Fusion? This will remove all game files and shortcuts.
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteConfirm} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
