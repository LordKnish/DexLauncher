# Technology Stack & Architecture

## Current Stack (React + Tauri)

### Frontend Technologies

**React 18** - UI framework
- Component-based architecture
- Hooks for state management
- Fast rendering with Virtual DOM

**TypeScript** - Type-safe JavaScript
- Compile-time type checking
- Better IDE support
- Reduced runtime errors

**Vite** - Build tool and dev server
- Lightning-fast HMR (Hot Module Replacement)
- Optimized production builds
- Native ES modules support
- Runs on port 1420 for Tauri integration

**Tailwind CSS** - Utility-first CSS framework
- Rapid UI development
- Consistent design system
- Small production bundle (purged unused styles)
- Custom configuration in [`tailwind.config.js`](../tailwind.config.js)

**shadcn/ui** - Component library
- Built on Radix UI primitives
- Accessible by default
- Customizable with Tailwind
- Copy-paste components (not npm package)
- Components in [`src/components/ui/`](../src/components/ui/)

**Radix UI** - Headless UI primitives
- Accessible components
- Unstyled, fully customizable
- Keyboard navigation
- ARIA attributes

**Additional Libraries:**
- `lucide-react` - Icon library
- `next-themes` - Theme management (light/dark)
- `react-hook-form` + `zod` - Form handling and validation
- `date-fns` - Date utilities
- `recharts` - Charts (for future analytics)
- `tauri-controls` - Custom window controls

### Backend Technologies

**Tauri 2.0 Alpha** - Desktop app framework
- Rust backend for performance and security
- Small bundle size (~3-5MB)
- Native system integration
- Built-in updater
- Cross-platform (Windows, macOS, Linux)

**Tauri Plugins:**
- `tauri-plugin-app` - App metadata and lifecycle
- `tauri-plugin-os` - OS information
- `tauri-plugin-shell` - Shell command execution
- `tauri-plugin-window` - Window management

**Rust Dependencies (Planned):**
- `reqwest` - HTTP client for GitHub API
- `tokio` - Async runtime
- `serde` + `serde_json` - JSON serialization
- `sha2` - SHA256 hashing
- `zip` - Archive extraction
- `directories` - Cross-platform paths
- `anyhow` - Error handling
- `tracing` - Logging

## Project Structure

```
pokemon-infinite-fusion-launcher/
├── src/                          # React frontend
│   ├── App.tsx                   # Main app component
│   ├── main.tsx                  # React entry point
│   ├── components/               # React components
│   │   ├── ui/                   # shadcn/ui components
│   │   ├── menu.tsx              # Top menu bar
│   │   ├── theme-provider.tsx   # Theme context
│   │   └── ...
│   ├── dashboard/                # Dashboard view
│   │   ├── page.tsx              # Main dashboard
│   │   └── components/           # Dashboard components
│   ├── lib/                      # Utilities
│   │   └── utils.ts              # Helper functions
│   └── styles/                   # Global styles
│       └── globals.css           # Tailwind + custom CSS
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   └── main.rs               # Tauri entry point
│   ├── capabilities/             # Permission definitions
│   │   └── default.json          # Default permissions
│   ├── icons/                    # App icons
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── public/                       # Static assets
│   └── avatars/                  # Avatar images
├── cline_docs/                   # Documentation
├── .github/workflows/            # CI/CD (to be set up)
├── package.json                  # Node dependencies
├── vite.config.ts                # Vite configuration
├── tailwind.config.js            # Tailwind configuration
├── tsconfig.json                 # TypeScript configuration
└── components.json               # shadcn/ui configuration
```

## Development Workflow

### Running the App
```bash
npm run dev          # Start Vite dev server
npm run tauri dev    # Start Tauri with Vite (recommended)
```

### Building
```bash
npm run build        # Build frontend
npm run tauri build  # Build complete app for current platform
```

### Code Quality
```bash
npm run format       # Format code with Prettier
npm run taze         # Update dependencies
```

## Configuration Files

### [`vite.config.ts`](../vite.config.ts)
- React plugin with SWC (faster than Babel)
- Path aliases (`@/` → `src/`)
- Port 1420 for Tauri integration
- Optimized for Tauri development

### [`tailwind.config.js`](../tailwind.config.js)
- Custom theme configuration
- Dark mode support
- Content paths for purging
- Plugin integrations

### [`tsconfig.json`](../tsconfig.json)
- TypeScript compiler options
- Path mappings
- Strict type checking

### [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json)
- Window configuration (900x600, frameless)
- Bundle settings
- Plugin configuration
- Security capabilities
- `withGlobalTauri: true` for window.__TAURI__ access

### [`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json)
- Permission definitions
- Window control permissions
- Event system permissions

## Design System

### Tailwind Configuration
- Custom color palette
- Typography scale
- Spacing system
- Border radius values
- Shadow utilities

### Component Architecture
- Atomic design principles
- Reusable UI components
- Composition over inheritance
- Props-based customization

### Theme Support
- Light and dark modes
- System preference detection
- Persistent theme selection
- CSS variables for theming

## Security Considerations

1. **Tauri Security Model**
   - Capabilities-based permissions
   - No arbitrary code execution
   - Sandboxed environment

2. **File Integrity** (to implement)
   - SHA256 verification
   - Manifest validation
   - Rollback on corruption

3. **Network Security** (to implement)
   - HTTPS only
   - GitHub API rate limiting
   - Retry logic with backoff

## Performance Targets

- App startup: < 1 second
- UI responsiveness: 60 FPS
- Memory usage: < 150MB
- Bundle size: < 10MB
- Update check: < 2 seconds

## Known Issues

1. Custom titlebar minimize/close buttons not functional
2. Banner image not displaying
3. No launcher functionality implemented yet (only demo greet command)
4. Old vanilla JS files still present (should be removed)

## Dependencies to Add

For launcher functionality, we need to add to [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml):
```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
sha2 = "0.10"
hex = "0.4"
directories = "5"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
zip = "0.6"
```

And for Unix Wine support:
```toml
[target.'cfg(unix)'.dependencies]
which = "6"