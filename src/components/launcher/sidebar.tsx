import { ScrollArea } from "@/components/ui/scroll-area"

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

  const handleGameSelect = (gameId: string) => {
    const game = games.find((g) => g.id === gameId)
    if (game) {
      // Select the installed version if available, otherwise the first version
      const installedVersion = game.versions.find((v) => v.installed)
      const versionToSelect = installedVersion ? installedVersion.version : game.versions[0].version
      onVersionSelect(gameId, versionToSelect)
    }
  }

  return (
    <div className="flex h-full w-60 flex-col border-r bg-[hsl(var(--sidebar-bg))]">
      {/* Games List - Simplified */}
      <ScrollArea className="flex-1 px-4 py-6">
        {games.map((game) => (
          <div key={game.id} className="mb-6">
            {/* Game Display - Clickable */}
            <button
              onClick={() => handleGameSelect(game.id)}
              className={`w-full flex flex-col items-center gap-3 rounded-lg px-3 py-4 text-center transition-all ${
                selectedGame === game.id
                  ? "bg-[hsl(var(--fusion-purple)_/_0.2)] shadow-md"
                  : "hover:bg-muted"
              }`}
            >
              <img src={game.logo} alt={game.name} className="h-16 w-21" />
              <div>
                <h3 className="text-sm font-semibold text-foreground">
                  {game.name}
                </h3>
              </div>
            </button>
          </div>
        ))}
      </ScrollArea>

      {/* App Version Footer */}
      <div className="border-t px-4 py-3 text-center">
        <p className="text-xs text-muted-foreground">
          DexLauncher v{appVersion}
        </p>
      </div>
    </div>
  )
}