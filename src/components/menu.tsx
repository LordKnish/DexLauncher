"use client"

import { useCallback } from "react"
import { Settings, FolderOpen, HelpCircle, Github } from "lucide-react"
import { getCurrentWindow } from "@tauri-apps/api/window"

import {
  Menubar,
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from "@/components/ui/menubar"

import { AboutDialog } from "./about-dialog"
import { MenuModeToggle } from "./menu-mode-toggle"
import { WindowControls } from "./window-controls"
import { Dialog, DialogTrigger } from "./ui/dialog"

export function Menu() {
  const closeWindow = useCallback(async () => {
    const appWindow = getCurrentWindow()
    await appWindow.close()
  }, [])

  const openGameFolder = useCallback(() => {
    console.log("Opening game folder...")
    // TODO: Implement open game folder
  }, [])

  const openGitHubIssue = useCallback(() => {
    window.open("https://github.com/Aegide/infinite-fusion-public/issues/new", "_blank")
  }, [])

  const startDragging = useCallback(async (e: React.MouseEvent) => {
    if (e.button === 0 && e.detail === 1) {
      const appWindow = getCurrentWindow()
      await appWindow.startDragging()
    }
  }, [])

  return (
    <div className="flex items-center justify-between border-b bg-background h-10">
      <Menubar className="rounded-none border-none pl-2 lg:pl-3">
        <MenubarMenu>
          <div className="inline-flex h-fit w-fit items-center">
            <img src="/src-tauri/icons/shard.png" alt="DexLauncher" className="h-5 w-5" />
          </div>
        </MenubarMenu>

        <MenubarMenu>
          <MenubarTrigger className="font-bold">File</MenubarTrigger>
          <MenubarContent>
            <MenubarItem onClick={openGameFolder}>
              <FolderOpen className="mr-2 h-4 w-4" />
              Open Game Folder
            </MenubarItem>
            <MenubarSeparator />
            <MenubarItem>
              Settings <MenubarShortcut>Ctrl+,</MenubarShortcut>
            </MenubarItem>
            <MenubarSeparator />
            <MenubarItem onClick={closeWindow}>
              Exit <MenubarShortcut>Alt+F4</MenubarShortcut>
            </MenubarItem>
          </MenubarContent>
        </MenubarMenu>

        <MenubarMenu>
          <MenubarTrigger>Help</MenubarTrigger>
          <Dialog modal={false}>
            <MenubarContent>
              <DialogTrigger asChild>
                <MenubarItem>About DexLauncher</MenubarItem>
              </DialogTrigger>
              <MenubarSeparator />
              <MenubarItem onClick={openGitHubIssue}>
                <Github className="mr-2 h-4 w-4" />
                Report Issue
              </MenubarItem>
            </MenubarContent>
            <AboutDialog />
          </Dialog>
        </MenubarMenu>

        <MenuModeToggle />
      </Menubar>
      <div
        className="flex-1 cursor-move h-10"
        onMouseDown={startDragging}
      />
      <WindowControls />
    </div>
  )
}
