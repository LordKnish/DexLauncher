# Codebase Summary

## Project Overview
Cross-platform launcher for Pokémon Infinite Fusion built with React + TypeScript + Tailwind CSS frontend and Tauri (Rust) backend.

## Current State
**Status:** Active Development - UI Framework Complete, Launcher Logic Pending  
**Version:** 0.1.0  
**Last Updated:** 2025-10-28

## Tech Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool (port 1420)
- **Tailwind CSS** - Styling
- **shadcn/ui** - Component library
- **Radix UI** - Accessible primitives

### Backend
- **Tauri 2.0 Alpha** - Desktop framework
- **Rust** - Backend language
- Plugins: app, os, shell, window

## Project Structure

### Frontend (`src/`)

**Core Files:**
- [`App.tsx`](../src/App.tsx) - Main application component with theme provider and menu
- [`main.tsx`](../src/main.tsx) - React entry point, renders App
- [`vite-env.d.ts`](../src/vite-env.d.ts) - Vite type definitions

**Components (`src/components/`):**
- [`menu.tsx`](../src/components/menu.tsx) - Top menu bar with navigation
- [`menu-mode-toggle.tsx`](../src/components/menu-mode-toggle.tsx) - Menu display toggle
- [`theme-provider.tsx`](../src/components/theme-provider.tsx) - Dark/light theme context
- [`about-dialog.tsx`](../src/components/about-dialog.tsx) - About dialog
- [`icons.tsx`](../src/components/icons.tsx) - Custom icon components
- [`tailwind-indicator.tsx`](../src/components/tailwind-indicator.tsx) - Dev mode breakpoint indicator

**UI Components (`src/components/ui/`):**
Complete shadcn/ui component library including:
- Buttons, Cards, Dialogs, Forms
- Progress bars, Sliders, Switches
- Tooltips, Popovers, Dropdowns
- Tables, Tabs, Accordions
- And 30+ more components

**Dashboard (`src/dashboard/`):**
- [`page.tsx`](../src/dashboard/page.tsx) - Main dashboard view
- [`components/`](../src/dashboard/components/) - Dashboard-specific components
  - `date-range-picker.tsx`
  - `main-nav.tsx`
  - `overview.tsx`
  - `recent-sales.tsx`
  - `search.tsx`
  - `team-switcher.tsx`
  - `user-nav.tsx`

**Utilities (`src/lib/`):**
- [`utils.ts`](../src/lib/utils.ts) - Helper functions (cn for className merging)

**Styles (`src/styles/`):**
- [`globals.css`](../src/styles/globals.css) - Global Tailwind styles and CSS variables

### Backend (`src-tauri/`)

**Rust Source (`src-tauri/src/`):**
- [`main.rs`](../src-tauri/src/main.rs) - Tauri app entry point with `greet` command

**Planned Modules (from earlier design, need to be implemented):**
- `config.rs` - Configuration management
- `github.rs` - GitHub Releases API integration
- `updater.rs` - Update download and installation logic
- `launcher.rs` - Game launching
- `wine.rs` - Wine integration for Unix systems

**Configuration:**
- [`Cargo.toml`](../src-tauri/Cargo.toml) - Rust dependencies
- [`tauri.conf.json`](../src-tauri/tauri.conf.json) - Tauri configuration
- [`capabilities/default.json`](../src-tauri/capabilities/default.json) - Permission definitions
- [`build.rs`](../src-tauri/build.rs) - Build script

**Icons (`src-tauri/icons/`):**
- Generated from [`shard.png`](../src-tauri/icons/shard.png)
- Multiple sizes for different platforms
- iOS and Android variants included

### Configuration Files (Root)

- [`package.json`](../package.json) - Node.js dependencies and scripts
- [`vite.config.ts`](../vite.config.ts) - Vite build configuration
- [`tailwind.config.js`](../tailwind.config.js) - Tailwind CSS configuration
- [`tsconfig.json`](../tsconfig.json) - TypeScript compiler options
- [`tsconfig.node.json`](../tsconfig.node.json) - TypeScript for Node scripts
- [`postcss.config.js`](../postcss.config.js) - PostCSS configuration
- [`prettier.config.cjs`](../prettier.config.cjs) - Code formatting rules
- [`components.json`](../components.json) - shadcn/ui configuration

### Assets

**Public Assets (`public/`):**
- `avatars/` - User avatar images (01.png through 05.png)

**Root Assets:**
- [`Banner_V4_Horizontal_Fade.webp`](../Banner_V4_Horizontal_Fade.webp) - Game banner artwork
- [`app-icon.png`](../app-icon.png) - Application icon

**Legacy Files (To Remove):**
- [`src/index.html`](../src/index.html) - Old vanilla JS version
- [`src/main.js`](../src/main.js) - Old vanilla JS version
- [`src/styles.css`](../src/styles.css) - Old vanilla CSS version

## Data Flow (Planned)

### Update Flow
```
User clicks "Check for Updates" (React)
    ↓
Frontend calls invoke('check_for_updates')
    ↓
Rust backend queries GitHub API
    ↓
Returns version info to frontend
    ↓
Frontend displays update available
    ↓
User clicks "Download Update"
    ↓
Backend downloads files with progress events
    ↓
Frontend updates progress bar
    ↓
Backend verifies and installs
    ↓
Frontend shows success/error
```

### Launch Flow
```
User clicks "Launch Game" (React)
    ↓
Frontend calls invoke('launch_game')
    ↓
Rust backend checks platform
    ↓
Windows: Direct execution
Unix: Launch via Wine
    ↓
Backend emits game-launched event
    ↓
Frontend updates status
```

## Component Communication

**Frontend → Backend:**
- Tauri `invoke()` function
- Type-safe command calls
- Async/await pattern

**Backend → Frontend:**
- Tauri event system
- `emit()` for progress updates
- Event listeners in React

## Styling Approach

### Tailwind Utilities
- Utility-first classes
- Responsive modifiers
- Dark mode variants
- Custom theme values

### CSS Variables
Defined in [`globals.css`](../src/styles/globals.css):
- Color palette
- Border radius
- Spacing scale
- Typography

### Component Styling
- Tailwind classes in JSX
- `cn()` utility for conditional classes
- Variant-based styling with CVA

## State Management

**Current:**
- React useState for local state
- Context API for theme
- No global state library yet

**Future Needs:**
- Update progress state
- Game installation state
- Configuration state
- Consider Zustand or Jotai if complexity grows

## Type Safety

**TypeScript Benefits:**
- Type-safe Tauri commands
- Component prop validation
- IDE autocomplete
- Compile-time error catching

**Type Definitions:**
- Tauri API types from `@tauri-apps/api`
- React types from `@types/react`
- Custom types for launcher data

## Build Process

### Development
1. Vite starts dev server on port 1420
2. Tauri watches Rust code for changes
3. Hot reload for frontend changes
4. Rust recompilation on backend changes

### Production
1. TypeScript compilation
2. Vite builds optimized frontend
3. Tailwind purges unused CSS
4. Rust compiles with optimizations
5. Tauri bundles everything
6. Platform-specific installers created

## Performance Optimizations

**Frontend:**
- React.lazy for code splitting (future)
- Vite's optimized bundling
- Tailwind CSS purging
- SWC for faster compilation

**Backend:**
- Rust's zero-cost abstractions
- Async I/O with Tokio
- Optimized release builds
- LTO and size optimizations in Cargo.toml

## Recent Changes

1. **Project Rebuilt** - Switched from vanilla JS to React + TypeScript
2. **UI Framework** - Integrated Tailwind CSS and shadcn/ui
3. **Custom Titlebar** - Added frameless window with custom controls
4. **Icons Generated** - Created from shard.png for all platforms
5. **Capabilities System** - Set up permissions for window controls
6. **Light Mode Implementation** - Added complete light mode with CSS variables and theme selector
7. **Theme Selector Fixed** - Replaced MenubarRadioItem with MenubarItem for proper functionality

## Known Issues

1. **Custom Titlebar Buttons** - Minimize/close not working (need Tauri window API integration)
2. **Banner Not Visible** - Image path or CSS issue
3. **No Launcher Logic** - Backend only has demo greet command
4. **Legacy Files** - Old vanilla JS files should be removed

## Next Implementation Steps

1. Implement Rust backend modules (updater, launcher, github)
2. Create React launcher component
3. Fix custom titlebar functionality
4. Display banner image properly
5. Add update/install flow
6. Implement progress tracking
7. Add error handling and rollback

## User Feedback Notes

- Design should be "fun and interesting and easy to use"
- Banner artwork should be prominently featured
- Modern, game-launcher aesthetic desired
- Reference designs: Cartel Tycoon, Farlight 84, Aqua Verse launchers
- Frameless window with custom controls preferred

## Technical Debt

- Remove old vanilla JS files (src/index.html, src/main.js, src/styles.css)
- Implement actual launcher backend (currently just demo code)
- Add proper error boundaries in React
- Set up proper logging system
- Create comprehensive test suite