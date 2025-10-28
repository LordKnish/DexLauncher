import { ChevronDown } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu"
import { Badge } from "@/components/ui/badge"
import { Check } from "lucide-react"

interface Version {
  version: string
  installed: boolean
  isBeta?: boolean
  date?: string
}

interface VersionSelectorProps {
  versions: Version[]
  currentVersion: string
  onVersionSelect: (version: string) => void
}

export function VersionSelector({
  versions,
  currentVersion,
  onVersionSelect,
}: VersionSelectorProps) {
  const currentVersionData = versions.find((v) => v.version === currentVersion)
  const installedVersions = versions.filter((v) => v.installed)
  const availableVersions = versions.filter((v) => !v.installed)

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          className="gap-2"
        >
          <span className="font-mono font-semibold">v{currentVersion}</span>
          <ChevronDown className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-56">
        {/* Installed Versions */}
        {installedVersions.length > 0 && (
          <>
            <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
              Installed
            </div>
            {installedVersions.map((version) => (
              <DropdownMenuItem
                key={version.version}
                onClick={() => onVersionSelect(version.version)}
                className="flex items-center justify-between cursor-pointer"
              >
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm">v{version.version}</span>
                  {version.isBeta && (
                    <Badge variant="secondary" className="text-xs">
                      Beta
                    </Badge>
                  )}
                </div>
                {currentVersion === version.version && (
                  <Check className="h-4 w-4 text-green-400" />
                )}
              </DropdownMenuItem>
            ))}
          </>
        )}

        {/* Available Versions */}
        {availableVersions.length > 0 && (
          <>
            {installedVersions.length > 0 && <DropdownMenuSeparator />}
            <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
              Available
            </div>
            {availableVersions.map((version) => (
              <DropdownMenuItem
                key={version.version}
                onClick={() => onVersionSelect(version.version)}
                className="flex items-center justify-between cursor-pointer"
              >
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm">v{version.version}</span>
                  {version.isBeta && (
                    <Badge variant="secondary" className="text-xs">
                      Beta
                    </Badge>
                  )}
                </div>
              </DropdownMenuItem>
            ))}
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}