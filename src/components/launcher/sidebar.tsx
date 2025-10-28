import { Check } from "lucide-react"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
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

  return (
    <div className="flex h-full w-60 flex-col border-r bg-[hsl(var(--sidebar-bg))]">

      {/* Games List */}
      <ScrollArea className="flex-1 px-3 py-4">
        {games.map((game) => (
          <div key={game.id} className="mb-4">
            {/* Game Header */}
            <div className="mb-2 flex items-center gap-2 px-3">
              <img src={game.logo} alt={game.name} className="h-6 w-6" />
              <span className="text-sm font-semibold text-foreground">
                {game.name}
              </span>
            </div>

            {/* Version List */}
            <div className="space-y-1">
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
          </div>
        ))}
      </ScrollArea>
    </div>
  )
}