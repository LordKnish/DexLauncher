import { useState, useEffect } from "react"
import { Sidebar } from "./sidebar"
import { HeroBanner } from "./hero-banner"
import { InstallationCard } from "./installation-card"
import { Changelog } from "./changelog"

interface VersionData {
  games: Array<{
    id: string
    name: string
    logo: string
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
  const [selectedGame, setSelectedGame] = useState("pokemon-infinite-fusion")
  const [selectedVersion, setSelectedVersion] = useState("6.7")
  const [installationStatus, setInstallationStatus] = useState<
    "not_installed" | "downloading" | "extracting" | "verifying" | "installed" | "error"
  >("not_installed")
  const [progress, setProgress] = useState(0)

  // Load version data from JSON
  useEffect(() => {
    fetch("/versions.json")
      .then((res) => res.json())
      .then((data: VersionData) => {
        setVersionData(data)
        // Set initial selected game to the first game
        if (data.games.length > 0) {
          const firstGame = data.games[0]
          setSelectedGame(firstGame.id)
          
          // Set initial selected version to the installed version, or first version if none installed
          if (firstGame.versions.length > 0) {
            const installedVersion = firstGame.versions.find((v) => v.installed)
            const versionToSelect = installedVersion ? installedVersion.version : firstGame.versions[0].version
            setSelectedVersion(versionToSelect)
            setInstallationStatus(installedVersion ? "installed" : "not_installed")
          }
        }
      })
      .catch((err) => console.error("Failed to load version data:", err))
  }, [])

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

  const handleLaunch = () => {
    console.log("Launching game:", selectedGame, selectedVersion)
    // TODO: Call Tauri command to launch game
  }

  const handleInstall = () => {
    console.log("Installing:", selectedGame, selectedVersion)
    setInstallationStatus("downloading")
    // TODO: Call Tauri command to install
    
    // Simulate progress
    let currentProgress = 0
    const interval = setInterval(() => {
      currentProgress += 5
      setProgress(currentProgress)
      if (currentProgress >= 100) {
        clearInterval(interval)
        setInstallationStatus("installed")
      }
    }, 200)
  }

  const handleUpdate = () => {
    console.log("Updating:", selectedGame, selectedVersion)
    setInstallationStatus("downloading")
    // TODO: Call Tauri command to update
    
    // Simulate progress
    let currentProgress = 0
    const interval = setInterval(() => {
      currentProgress += 5
      setProgress(currentProgress)
      if (currentProgress >= 100) {
        clearInterval(interval)
        setInstallationStatus("installed")
      }
    }, 200)
  }

  const handleDelete = () => {
    console.log("Deleting installation:", selectedGame, selectedVersion)
    setInstallationStatus("not_installed")
    setProgress(0)
    // TODO: Call Tauri command to delete
  }

  const handleCancel = () => {
    console.log("Cancelling installation...")
    setInstallationStatus("not_installed")
    setProgress(0)
  }

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
              statusMessage={
                installationStatus === "downloading"
                  ? "Downloading files..."
                  : installationStatus === "extracting"
                    ? "Extracting files..."
                    : installationStatus === "verifying"
                      ? "Verifying installation..."
                      : undefined
              }
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
                onViewFull={() => {
                  console.log("Opening full changelog...")
                  // TODO: Open full changelog in browser or dialog
                }}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  )
}