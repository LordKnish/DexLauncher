import { Minus, X } from "lucide-react"
import { getCurrentWindow } from "@tauri-apps/api/window"

export function WindowControls() {
  const appWindow = getCurrentWindow()

  const handleMinimize = async () => {
    await appWindow.minimize()
  }

  const handleClose = async () => {
    await appWindow.close()
  }

  return (
    <div className="flex h-8 items-center gap-1">
      <button
        onClick={handleMinimize}
        className="flex h-8 w-12 items-center justify-center transition-colors hover:bg-muted"
        aria-label="Minimize"
      >
        <Minus className="h-4 w-4" />
      </button>
      <button
        onClick={handleClose}
        className="flex h-8 w-12 items-center justify-center transition-colors hover:bg-destructive hover:text-destructive-foreground"
        aria-label="Close"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  )
}