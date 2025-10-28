# Current Task: Rebuild Launcher with Modern React Stack

## Objective
The project has been rebuilt from scratch using a modern React + TypeScript + Tailwind CSS stack. We need to implement the launcher functionality within this new architecture.

## Context
The project started with vanilla HTML/JS but has been upgraded to:
- **Frontend:** React 18 + TypeScript + Vite
- **Styling:** Tailwind CSS + shadcn/ui components
- **Backend:** Tauri 2.0 (Rust)
- **UI Library:** Radix UI primitives via shadcn/ui

### Current State
- ✅ React + TypeScript project structure set up
- ✅ Tailwind CSS configured
- ✅ shadcn/ui components integrated
- ✅ Custom titlebar structure in place
- ✅ Tauri backend initialized
- ✅ Icons generated from shard.png
- ⚠️ Custom titlebar buttons (minimize/close) not working
- ⚠️ Banner image not visible
- ❌ No launcher functionality implemented yet

### Files to Clean Up
Old vanilla JS files that should be removed:
- [`src/index.html`](../src/index.html) - Replaced by React
- [`src/main.js`](../src/main.js) - Replaced by React
- [`src/styles.css`](../src/styles.css) - Replaced by Tailwind

### Active Files
Current React application:
- [`src/App.tsx`](../src/App.tsx) - Main app component
- [`src/main.tsx`](../src/main.tsx) - React entry point
- [`src/dashboard/page.tsx`](../src/dashboard/page.tsx) - Dashboard view
- [`src/components/menu.tsx`](../src/components/menu.tsx) - Menu component
- [`src/styles/globals.css`](../src/styles/globals.css) - Global Tailwind styles

## Current Issues

### 1. Custom Titlebar Buttons Not Working
**Problem:** Minimize and close buttons don't respond
**Likely Cause:** Missing Tauri window API integration in React components
**Solution Needed:** Implement window controls in React using `@tauri-apps/api`

### 2. Banner Image Not Visible
**Problem:** [`Banner_V4_Horizontal_Fade.webp`](../Banner_V4_Horizontal_Fade.webp) not displaying
**Likely Cause:** Incorrect path or CSS not applied
**Solution Needed:** 
- Move banner to `public/` folder for Vite
- Update CSS/component to reference correct path
- Ensure background styling is applied

### 3. No Launcher Functionality
**Problem:** Backend has placeholder `greet` command, no actual launcher logic
**Solution Needed:** Implement the updater/launcher modules we designed earlier

## Next Steps

1. **Fix Custom Titlebar**
   - Add window control handlers to React components
   - Test minimize/close functionality
   - Ensure draggable region works

2. **Fix Banner Display**
   - Move banner to public folder
   - Update component to show banner
   - Style it prominently in the UI

3. **Implement Launcher Backend**
   - Port the updater.rs, launcher.rs, github.rs modules
   - Add Tauri commands for update/launch
   - Integrate with React frontend

4. **Build Launcher UI**
   - Create launcher component with version display
   - Add update/launch buttons
   - Implement progress tracking
   - Add status messages

5. **Test & Polish**
   - Test update flow
   - Test launch flow
   - Polish animations and transitions
   - Ensure cross-platform compatibility

## Questions/Blockers
None currently - clear path forward to implement launcher functionality.

## Notes
- The React + Tailwind stack is much more powerful than vanilla JS
- shadcn/ui provides beautiful, accessible components
- Need to integrate the Rust backend modules we designed
- Custom titlebar is a key UX feature that needs to work properly