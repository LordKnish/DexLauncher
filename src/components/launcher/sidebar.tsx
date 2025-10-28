import { useState } from "react"
import { Check, ChevronDown } from "lucide-react"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

interface Version {
  version: string
  installed: boolean
  date?: string
}

interface Game {
  id: string
  name: string
  logo: string
  versions: Version[]
}

interface SidebarProps {
  games: Game[]
  selectedGame: string
  selectedVersion: string
  onVersionSelect: (gameId: string, version: string) => void
  appVersion: string
}

export function Sidebar({
  games,
  selectedGame,
  selectedVersion,
  onVersionSelect,
  appVersion,
}: SidebarProps) {
  const currentGame = games.find((g) => g.id === selectedGame)
  const [expandedGames, setExpandedGames] = useState<Set<string>>(new Set([selectedGame]))

  const toggleGameExpanded = (gameId: string) => {
    const newExpanded = new Set(expandedGames)
    if (newExpanded.has(gameId)) {
      newExpanded.delete(gameId)
    } else {
      newExpanded.add(gameId)
    }
    setExpandedGames(newExpanded)
  }

  const handleGameNameClick = (gameId: string) => {
    // If clicking on the game name/icon, navigate to the installed version or first version
    const game = games.find((g) => g.id === gameId)
    if (game) {
      const installedVersion = game.versions.find((v) => v.installed)
      const versionToSelect = installedVersion ? installedVersion.version : game.versions[0].version
      onVersionSelect(gameId, versionToSelect)
    }
  }

  return (
    <div className="flex h-full w-60 flex-col border-r bg-[hsl(var(--sidebar-bg))]">

      {/* Games List */}
      <ScrollArea className="flex-1 px-3 py-4">
        {games.map((game) => {
          const isExpanded = expandedGames.has(game.id)
          const installedVersion = game.versions.find((v) => v.installed)
          const displayVersion = installedVersion ? installedVersion.version : game.versions[0].version

          return (
            <div key={game.id} className="mb-4">
              {/* Game Header - Collapsible */}
              <div className="flex items-center gap-2 px-2 mb-2">
                <button
                  onClick={() => toggleGameExpanded(game.id)}
                  className="flex items-center justify-center p-2 hover:bg-muted transition-colors rounded"
                  title="Expand/collapse versions"
                >
                  <ChevronDown
                    className={cn(
                      "h-4 w-4 transition-transform",
                      isExpanded ? "rotate-0" : "-rotate-90"
                    )}
                  />
                </button>
                <button
                  onClick={() => handleGameNameClick(game.id)}
                  className="flex items-center gap-2 flex-1 rounded-lg px-2 py-2 hover:bg-muted transition-colors"
                  title="Go to latest installed version"
                >
                  <img src={game.logo} alt={game.name} className="h-5 w-5" />
                  <span className="text-sm font-semibold text-foreground truncate">
                    {game.name}
                  </span>
                </button>
              </div>

              {/* Version List - Expandable */}
              {isExpanded && (
                <div className="space-y-1 ml-2">
                  {game.versions.map((version) => (
                    <button
                      key={version.version}
                      onClick={() => onVersionSelect(game.id, version.version)}
                      className={cn(
                        "group relative flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-all",
                        selectedGame === game.id && selectedVersion === version.version
                          ? "bg-[hsl(var(--fusion-purple)_/_0.2)] text-foreground shadow-sm"
                          : "text-muted-foreground hover:bg-muted hover:text-foreground"
                      )}
                    >
                      <div className="flex items-center gap-2">
                        {selectedGame === game.id && selectedVersion === version.version && (
                          <div className="absolute left-0 h-8 w-1 rounded-r-full bg-[hsl(var(--fusion-purple))]" />
                        )}
                        <span className="ml-6 font-mono text-xs">{version.version}</span>
                      </div>
                      {version.installed && (
                        <Check className="h-3 w-3 text-green-400" />
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )
        })}
      </ScrollArea>
    </div>
  )
}