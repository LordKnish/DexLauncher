# DexLauncher v0.2.0 - Initial Release

**Release Date:** January 29, 2025  
**Developer:** Bernie M (LordKnish)

![DexLauncher Banner](https://i.imgur.com/bUzeVHg.png)

## 🎉 Welcome to DexLauncher

I'm excited to introduce **DexLauncher** - a modern, cross-platform launcher for Pokemon fan games that I've built from the ground up using cutting-edge technology. This initial release represents months of development work to create a polished, production-ready experience for managing and playing Pokemon Infinite Fusion.

## ✨ Core Features

### 🎮 Game Management
- **Multi-Version Support** - Install and manage Pokemon Infinite Fusion versions 6.4, 6.5, 6.6, and 6.7
- **Git-Based Installation** - Direct cloning from GitHub repositories with submodule support
- **Real-Time Progress Tracking** - Live progress updates with carriage-return delimited frame parsing
- **Version Switching** - Seamlessly switch between installed game versions
- **Automatic Updates** - Built-in update checking system (coming soon)

### 🚀 Installation System
- **Production-Ready Installer** - Complete installation workflow with proper Windows integration
- **Desktop Shortcuts** - Automatic creation of desktop shortcuts with custom icons
- **Start Menu Integration** - Proper Windows Start Menu entries
- **Custom Install Locations** - Choose where to install your games
- **Disk Space Verification** - Pre-installation disk space checks
- **Installation Verification** - Post-installation integrity checks

### 🎨 Modern User Interface
- **Beautiful Design** - Gradient accents and polished UI components
- **Dark & Light Themes** - Full theme support with smooth transitions
- **Custom Window Controls** - Frameless window with native-looking controls
- **Responsive Layout** - Optimized for various screen sizes
- **Hero Banner** - Eye-catching game artwork display
- **Changelog Viewer** - Detailed release notes for each version

### 🎯 Steam Integration
- **Automatic Steam Detection** - Finds Steam installation via Windows registry
- **Non-Steam Game Addition** - Adds Pokemon Infinite Fusion to your Steam library
- **Multi-Account Support** - Automatically adds to all Steam user accounts
- **Grid Art Support** - Custom library artwork (foundation laid)
- **Smart Process Management** - Gracefully handles Steam restart when needed
- **Manual Retry Option** - Add to Steam later if skipped during installation

### 🛠️ Technical Excellence
- **Lightweight Bundle** - Only ~2.5MB on Windows
- **Rust Backend** - High-performance Tauri v2.9.1 backend
- **React Frontend** - Modern React 18 with TypeScript
- **Database Integration** - SQLite for persistent state management
- **Error Recovery** - Robust error handling with automatic backups
- **Cross-Platform Foundation** - Windows support with Linux/macOS groundwork

## 🔧 Technical Stack

### Frontend
- **React 18.2** - Latest React with concurrent features
- **TypeScript 5.1** - Type-safe development
- **Tailwind CSS 3.3** - Utility-first styling
- **shadcn/ui** - Beautiful, accessible UI components
- **Radix UI** - Unstyled, accessible component primitives
- **Lucide React** - Modern icon library
- **Vite 4.4** - Lightning-fast build tool

### Backend
- **Tauri v2.9.1** - Secure, lightweight desktop framework
- **Rust 1.70+** - Memory-safe systems programming
- **SQLite** - Embedded database for state management
- **sysinfo** - Cross-platform system information
- **steam-shortcuts-util** - Steam VDF file manipulation
- **tokio** - Async runtime for concurrent operations

## 📦 Installation & Usage

### System Requirements
- **Windows 10/11** (primary support)
- **4GB RAM** minimum
- **500MB** free disk space for launcher
- **5-10GB** per game installation
- **Internet connection** for downloads

### Quick Start
1. Download `DexLauncher-Setup.exe` from the releases page
2. Run the installer and follow the setup wizard
3. Launch DexLauncher from your desktop or Start Menu
4. Select a game version and click "Install"
5. Choose your installation directory
6. Optionally add to Steam library
7. Wait for installation to complete
8. Click "Play" to launch the game!

## 🎯 Development Journey - Key Milestones

### Git-Based Installation System (Commit: 76305fc)
Replaced the traditional download/extract approach with direct git cloning. This was a major architectural decision that:
- Enables real-time progress tracking with carriage-return frame parsing
- Properly initializes submodules for complete game files
- Significantly speeds up the installation process
- Provides better error recovery and resume capabilities

### Complete Steam Integration (Commit: 0106c42)
Built a comprehensive Steam integration system from scratch:
- Automatic detection of Steam installation via Windows registry
- Smart handling of running Steam processes with graceful shutdown
- Multi-user account support for households with multiple Steam users
- VDF file backup and verification to prevent data loss
- Manual retry functionality for users who skip during installation

### Production-Ready Installer (Commit: 600200c)
Implemented proper Windows integration:
- Custom icon support for desktop shortcuts
- Start Menu integration with proper categorization
- Installation verification system to ensure game integrity
- Clean uninstallation support

### UI Polish & Theme System (Commits: 33d7b00, b1b60c5)
Refined the user experience:
- Complete light mode implementation with proper contrast
- Improved theme switching with smooth transitions
- Larger grab bar for better window dragging experience
- Replaced SVG logo with high-quality PNG for better rendering

### Critical Bug Fixes
- **React Hooks Error** (Commit: a98ac3b) - Fixed state management issues in launcher page
- **URL Opener Fix** (Commit: a6672d4) - Migrated to Tauri shell API for secure external links
- **Git Progress Tracking** (Commit: ca36eec) - Fixed carriage-return frame parsing for accurate progress
- **Feature Flag Cleanup** (Commit: a25cf7b) - Removed invalid Tauri plugin configurations

## 🚧 Current Limitations

- **Windows Only** - Primary support is Windows; Linux/macOS support is in the roadmap
- **Single Game** - Currently supports Pokemon Infinite Fusion only (multi-game architecture is ready)
- **Steam Grid Art** - Grid art installation is implemented but not yet activated
- **Auto-Updates** - Update checking system is built but not yet enabled

## 🔮 Roadmap

### v0.3.0 (Next Release)
- Enable auto-update functionality
- Add support for additional Pokemon fan games
- Activate Steam grid art installation
- Enhanced error reporting and logging
- Performance optimizations

### v0.4.0 (Future)
- Linux support
- macOS support
- Cloud save synchronization
- Mod management system
- Community features and integrations

## 🐛 Bug Reports & Feedback

Found a bug or have a suggestion? Please report it on the [GitHub Issues](https://github.com/LordKnish/DexLauncher/issues) page. I actively monitor and respond to all feedback.

## 📄 License

DexLauncher is released under the MIT License. See the LICENSE file for details.

## 🙏 Acknowledgments

- **Pokemon Infinite Fusion Team** - For creating an incredible fan game that inspired this project
- **Tauri Team** - For the excellent desktop framework that made this possible
- **shadcn** - For the beautiful UI component library
- **Radix UI Team** - For accessible component primitives
- **The Pokemon Community** - For the continued support and enthusiasm

## 📊 Project Statistics

- **Total Commits:** 20+
- **Lines of Code:** 10,000+
- **Development Time:** 3+ months of solo development
- **Bundle Size:** ~2.5MB (Windows)
- **Supported Versions:** 4 (Pokemon Infinite Fusion 6.4-6.7)
- **Languages:** Rust, TypeScript, React

## 🔗 Links

- **GitHub Repository:** https://github.com/LordKnish/DexLauncher
- **Issue Tracker:** https://github.com/LordKnish/DexLauncher/issues
- **Discussions:** https://github.com/LordKnish/DexLauncher/discussions

---

**Thank you for trying DexLauncher!** This is just the beginning - I have many exciting features planned for future releases. Your feedback and support mean everything as I continue to improve and expand this project.

*DexLauncher is a fan-made tool and is not affiliated with or endorsed by Nintendo, Game Freak, or The Pokemon Company.*
