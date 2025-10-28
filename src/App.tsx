import { Menu } from "@/components/menu"
import { TailwindIndicator } from "./components/tailwind-indicator"
import { ThemeProvider } from "./components/theme-provider"
import { LauncherPage } from "./components/launcher/launcher-page"

function App() {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
      <div className="h-screen overflow-clip">
        <Menu />
        <div className="h-[calc(100vh-40px)]">
          <LauncherPage />
        </div>
      </div>
      <TailwindIndicator />
    </ThemeProvider>
  )
}

export default App
