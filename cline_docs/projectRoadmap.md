# Pokémon Infinite Fusion Launcher - Project Roadmap

## Project Vision
Build a modern, cross-platform launcher for Pokémon Infinite Fusion with a beautiful UI that makes installation, updates, and launching seamless across Windows, macOS, and Linux.

## Current Status
**Phase:** Active Development  
**Version:** 0.1.0  
**Stack:** React + TypeScript + Tailwind CSS + Tauri (Rust backend)

## High-Level Goals
- [x] Set up modern React + TypeScript + Tailwind CSS frontend
- [x] Configure Tauri with custom window decorations
- [x] Integrate shadcn/ui component library
- [ ] Implement game installation from GitHub Releases
- [ ] Add automatic update detection and installation
- [ ] Support Windows natively and macOS/Linux via Wine
- [ ] Create beautiful, game-themed UI using banner artwork
- [ ] Ensure reliability with integrity verification and rollback
- [ ] Enable launcher self-updates

## MVP Feature Checklist

### Core Functionality
- [ ] Automatic installation from GitHub Releases
- [ ] Version checking and update detection
- [ ] Full update download and extraction
- [ ] SHA256 integrity verification via manifest
- [ ] Automatic rollback on failed updates
- [ ] Game launch capability
- [ ] Progress tracking with visual feedback

### Cross-Platform Support
- [ ] Windows native execution
- [ ] macOS Wine integration
- [ ] Linux Wine integration
- [ ] Wine auto-configuration for Unix systems

### User Interface
- [x] Custom titlebar with minimize/close buttons
- [x] Modern React component architecture
- [x] Tailwind CSS styling system
- [x] shadcn/ui component library integration
- [ ] Version display (current + latest available)
- [ ] Update progress indicator
- [ ] Status messages and error display
- [ ] Settings panel
- [ ] Theme support (light/dark)

### Infrastructure
- [ ] GitHub Releases API integration
- [ ] Manifest system (JSON with version, hashes, sizes)
- [ ] Logging system for troubleshooting
- [ ] Launcher self-update via Tauri updater
- [x] Icon generation and branding

## Technical Debt
- Old vanilla JS files in `src/` (index.html, main.js, styles.css) should be removed
- Need to implement actual updater/launcher backend modules
- Custom titlebar buttons need proper Tauri window API integration
- Banner image not displaying - needs CSS/path fixes

## Next Steps
1. Fix custom titlebar minimize/close functionality
2. Make banner image visible in UI
3. Implement GitHub Releases integration
4. Build launcher UI components
5. Add update/install flow
6. Test cross-platform functionality

## Post-MVP Features (Future)
- Delta/partial updates for efficiency
- Multiple game version management
- Mod manager integration
- Custom game directory selection
- Bandwidth throttling options
- Offline mode support
- Multi-language support
- Community features integration