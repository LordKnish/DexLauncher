import { FileText, ExternalLink } from "lucide-react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

interface ChangelogEntry {
  category: string
  changes: string[]
}

interface ChangelogProps {
  version: string
  date: string
  isBeta?: boolean
  entries: ChangelogEntry[]
  onViewFull?: () => void
  className?: string
}

export function Changelog({
  version,
  date,
  isBeta = false,
  entries,
  onViewFull,
  className,
}: ChangelogProps) {
  return (
    <Card className={cn("animate-fade-in-up backdrop-blur-sm", className)}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <FileText className="h-5 w-5 text-[hsl(var(--fusion-purple))]" />
            <CardTitle className="text-xl">
              Changelog - v{version}
            </CardTitle>
          </div>
          {isBeta && (
            <Badge variant="secondary" className="bg-yellow-500/20 text-yellow-400">
              BETA
            </Badge>
          )}
        </div>
        <p className="text-sm text-muted-foreground">{date}</p>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[280px] pr-4 scrollbar-custom">
          <div className="space-y-6">
            {entries.map((entry, index) => (
              <div key={index} className="space-y-2">
                <h4 className="font-semibold text-foreground">
                  {entry.category}
                </h4>
                <ul className="space-y-1.5 text-sm text-muted-foreground">
                  {entry.changes.map((change, changeIndex) => (
                    <li key={changeIndex} className="flex gap-2">
                      <span className="text-[hsl(var(--fusion-purple))]">•</span>
                      <span>{change}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </ScrollArea>

        {onViewFull && (
          <Button
            variant="ghost"
            className="mt-4 w-full text-[hsl(var(--fusion-purple))] hover:bg-[hsl(var(--fusion-purple)_/_0.1)] hover:text-[hsl(var(--fusion-purple))]"
            onClick={onViewFull}
          >
            View Full Changelog
            <ExternalLink className="ml-2 h-4 w-4" />
          </Button>
        )}
      </CardContent>
    </Card>
  )
}