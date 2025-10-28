"use client"

import * as React from "react"
import { LaptopIcon, MoonIcon, SunIcon, CheckIcon } from "@radix-ui/react-icons"
import { useTheme } from "next-themes"

import {
  MenubarContent,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarTrigger,
} from "@/components/ui/menubar"

export function MenuModeToggle() {
  const { setTheme, theme } = useTheme()

  return (
    <MenubarMenu>
      <MenubarTrigger>Theme</MenubarTrigger>
      <MenubarContent forceMount>
        <MenubarItem onClick={() => setTheme("light")} className="flex items-center justify-between">
          <div className="flex items-center">
            <SunIcon className="mr-2 h-4 w-4" />
            <span>Light</span>
          </div>
          {theme === "light" && <CheckIcon className="ml-2 h-4 w-4" />}
        </MenubarItem>
        <MenubarItem onClick={() => setTheme("dark")} className="flex items-center justify-between">
          <div className="flex items-center">
            <MoonIcon className="mr-2 h-4 w-4" />
            <span>Dark</span>
          </div>
          {theme === "dark" && <CheckIcon className="ml-2 h-4 w-4" />}
        </MenubarItem>
        <MenubarSeparator />
        <MenubarItem onClick={() => setTheme("system")} className="flex items-center justify-between">
          <div className="flex items-center">
            <LaptopIcon className="mr-2 h-4 w-4" />
            <span>System</span>
          </div>
          {theme === "system" && <CheckIcon className="ml-2 h-4 w-4" />}
        </MenubarItem>
      </MenubarContent>
    </MenubarMenu>
  )
}
