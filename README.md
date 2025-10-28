
# DexLauncher
A modern, cross-platform launcher for Pokemon fan games built with Tauri and React.
![enter image description here](https://i.imgur.com/bUzeVHg.png)
## Features

- 🎮 **Multi-Game Support** - Designed to support multiple Pokemon fan games
- 🎨 **Modern UI** - Fenix-inspired design with Pokemon-themed colors
- 📦 **Version Management** - Easy installation and switching between game versions
- 📝 **Changelog Display** - View detailed release notes for each version
- 🌙 **Dark Theme** - Optimized dark mode with gradient accents
- 🪟 **Custom Window Controls** - Frameless window with native-looking controls
- ⚡ **Lightweight** - Optimized bundle size (~2.5MB on Windows)
- 🔄 **Auto-Updates** - Built-in update checking (coming soon)

## Currently Supported Games

- **Pokemon Infinite Fusion** - Versions 6.7, 6.6, 6.5, 6.4

## Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS
- **UI Components**: shadcn/ui, Radix UI
- **Icons**: Lucide React, Radix Icons
- **Backend**: Tauri v2.9.1 (Rust)
- **Build**: Vite, SWC

## Development

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Project Structure

```
├── src/                      # React frontend
│   ├── components/
│   │   ├── launcher/        # Main launcher components
│   │   │   ├── sidebar.tsx
│   │   │   ├── hero-banner.tsx
│   │   │   ├── installation-card.tsx
│   │   │   └── changelog.tsx
│   │   └── ui/              # shadcn/ui components
│   └── styles/              # Global styles
├── src-tauri/               # Tauri backend
│   ├── src/
│   │   └── main.rs
│   └── icons/               # App icons
└── public/
    ├── versions.json        # Version metadata
    └── banner.png          # Game artwork
```

## Configuration

Version data is stored in `public/versions.json`. To add a new game or version:

```json
{
  "games": [
    {
      "id": "game-id",
      "name": "Game Name",
      "logo": "/path/to/logo.png",
      "versions": [
        {
          "version": "1.0.0",
          "date": "2024-01-01",
          "installed": false,
          "isBeta": false,
          "announcement": "Release announcement",
          "changelog": {
            "categories": [
              {
                "name": "Category Name",
                "changes": ["Change 1", "Change 2"]
              }
            ]
          }
        }
      ]
    }
  ]
}
```

## Building

### Development Build
```bash
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```

Builds will be available in `src-tauri/target/release/bundle/`

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

This launcher is a fan-made tool for Pokemon fan games. Pokemon and all related properties are trademarks of Nintendo, Game Freak, and The Pokemon Company. This project is not affiliated with or endorsed by these entities.

## Acknowledgments

- Pokemon Infinite Fusion team for the amazing game
- shadcn for the beautiful UI components
- Tauri team for the excellent framework
