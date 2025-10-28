# Design Concepts & UI/UX Planning

## Design Philosophy

**Core Principles:**
1. **Fun & Engaging** - Game launcher should feel exciting, not boring
2. **Banner Showcase** - Pokémon Infinite Fusion artwork is the centerpiece
3. **Modern & Clean** - Contemporary game launcher aesthetic
4. **Easy to Use** - Minimal clicks, clear actions
5. **Cross-Platform** - Consistent experience everywhere

## Design Inspiration

Reference launchers that inspired the design:
1. **Cartel Tycoon Launcher** - Frameless window, custom controls, prominent artwork
2. **Farlight 84 Launcher** - Clean UI with game art background
3. **Aqua Verse Launcher** - Modern dark theme with vibrant accents

## Visual Design

### Window Design

**Frameless Window:**
- No Windows/macOS chrome
- Custom titlebar with minimize/close buttons
- 900x600px fixed size
- Centered on screen
- Non-resizable for consistent experience

**Custom Titlebar:**
```
┌─────────────────────────────────────────────────────┐
│ Pokémon Infinite Fusion          [─] [✕]           │
└─────────────────────────────────────────────────────┘
```
- Height: 32px
- Dark background with blur
- Draggable region for window movement
- Minimize button (hover: subtle highlight)
- Close button (hover: red background)

### Layout Structure

```
┌─────────────────────────────────────────────────────┐
│ [Custom Titlebar]                        [─] [✕]   │
├─────────────────────────────────────────────────────┤
│                                                     │
│         [Banner Image - Full Width]                 │
│         Pokémon Infinite Fusion artwork             │
│         (280px height, fades to dark at bottom)     │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │  Current: v5.2.0    │    Latest: v5.2.1    │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  [████████████░░░░░░░░] 65% - Downloading...       │
│                                                     │
│  [Check for Updates]  [Launch Game]                │
│                                                     │
│  ✓ Ready to play                                   │
│                                                     │
│  [Settings]  [Logs]  [About]                       │
└─────────────────────────────────────────────────────┘
```

### Color Palette

**Primary Colors:**
- Pokémon Red: `#FF1C1C`
- Pokémon Yellow: `#FFDE00`
- Fire Orange: `#FF9741`
- Water Blue: `#6890F0`
- Grass Green: `#78C850`
- Electric Yellow: `#F7D02C`

**UI Colors:**
- Background: `#0a0a0f` (very dark blue-black)
- Card Background: `rgba(20, 20, 30, 0.85)` (dark with transparency)
- Text Primary: `#ffffff`
- Text Secondary: `#e0e0e0`
- Text Muted: `#a0a0a0`

**Semantic Colors:**
- Success: Grass Green
- Error: Pokémon Red
- Warning: Pokémon Yellow
- Info: Water Blue

### Typography

**Font Stack:**
```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 
             'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 
             'Fira Sans', 'Droid Sans', 'Helvetica Neue', 
             sans-serif;
```

**Sizes:**
- Title: 3rem (48px), bold, gradient
- Subtitle: 1.25rem (20px), uppercase
- Version: 1.5rem (24px), monospace
- Buttons: 1.125rem (18px), bold, uppercase
- Status: 0.9375rem (15px)
- Links: 0.8125rem (13px), uppercase

### Button Design

**Primary Button (Launch Game):**
- Gradient: Red → Orange
- White text
- Bold, uppercase
- Large padding (16px 24px)
- Rounded corners (12px)
- Hover: Lift up, scale slightly, glow effect
- Active: Press down
- Disabled: 50% opacity

**Secondary Button (Check for Updates):**
- Transparent background with blur
- Yellow border (2px)
- Yellow text
- Hover: Yellow glow, lift up
- Same size as primary

**Link Buttons (Settings, Logs, About):**
- Transparent background
- Muted text color
- Small size
- Uppercase
- Hover: Yellow text, subtle background

### Progress Indicator

**Rainbow Progress Bar:**
- Height: 40px
- Animated gradient: Red → Orange → Yellow → Green → Blue → Pink
- Shimmer animation (3s loop)
- Glowing effect
- Percentage text overlay
- Status text below

**States:**
- Downloading: Rainbow shimmer
- Extracting: Same animation
- Verifying: Same animation
- Complete: Green fill

### Banner Integration

**Banner Display:**
- Full width at top of window
- 280px height
- Positioned below titlebar
- Gradient fade to dark at bottom
- No blur or darkening (show artwork clearly)
- Mask gradient for smooth transition

**Banner Artwork:**
- [`Banner_V4_Horizontal_Fade.webp`](../Banner_V4_Horizontal_Fade.webp)
- Colorful Pokémon fusion sprites
- Official game logo in center
- Vibrant, eye-catching

## Animations & Transitions

**Page Load:**
- Staggered fade-in for elements
- Slide up from bottom
- 0.6s duration with easing

**Button Interactions:**
- Hover: 200ms lift and scale
- Click: Ripple effect
- Loading: Spin animation

**Progress:**
- Smooth width transition (250ms)
- Shimmer animation (3s loop)
- Glow pulse effect

**Theme Transitions:**
- 200ms ease for color changes
- Smooth dark/light mode switch

## Accessibility

**Keyboard Navigation:**
- Tab through all interactive elements
- Enter/Space to activate
- Escape to close dialogs
- Focus visible indicators

**Screen Reader:**
- Proper ARIA labels
- Status announcements
- Progress updates
- Error messages

**Motion:**
- Respect `prefers-reduced-motion`
- Disable animations when requested
- Instant transitions as fallback

## Responsive Behavior

**Window States:**
- Fixed 900x600px size
- Non-resizable
- Centered on screen
- Remember position (future)

**Mobile Considerations:**
- Not applicable (desktop app only)
- But components are responsive-ready

## Theme Support

**Dark Mode (Default):**
- Dark backgrounds
- Vibrant accent colors
- High contrast for readability

**Light Mode (Future):**
- Light backgrounds
- Adjusted color palette
- Maintained contrast ratios

**System Theme:**
- Detect OS preference
- Auto-switch on change
- User override option

## Component Hierarchy

```
App (Theme Provider)
├── Menu (Top navigation)
│   ├── Mode Toggle
│   └── Theme Toggle
└── Dashboard
    ├── Version Display
    ├── Progress Bar
    ├── Action Buttons
    ├── Status Message
    └── Footer Links
```

## User Flows

### First Launch
1. App opens with custom titlebar
2. Banner displays prominently
3. Shows "Not Installed"
4. "Check for Updates" button available
5. User clicks → Detects no installation
6. Prompts to install latest version
7. Downloads with progress
8. Installs and enables "Launch Game"

### Update Flow
1. User clicks "Check for Updates"
2. Queries GitHub API
3. If update available: Shows version number
4. User confirms download
5. Progress bar shows download/extract/verify
6. Success message
7. Version number updates

### Launch Flow
1. User clicks "Launch Game"
2. Button shows loading state
3. Game launches (Wine on Unix)
4. Status shows "Game running"
5. Launcher stays open or minimizes
6. When game closes, status updates

## Error Handling UI

**Error States:**
- Network error: Retry button
- Verification failed: Auto-retry, then rollback
- Disk space: Clear message with required space
- Wine missing: Install instructions
- Game not found: Reinstall option

**Error Display:**
- Red border and background
- Clear error message
- Actionable buttons
- Link to logs

## Platform-Specific UI

**Windows:**
- Native-feeling controls
- Windows 11 rounded corners

**macOS:**
- macOS-style window controls (future)
- Menu bar integration (future)

**Linux:**
- Respect desktop environment
- System tray icon (future)

## Future Enhancements

- Settings dialog with preferences
- Changelog viewer
- Mod manager integration
- Multiple theme options
- Custom background selection
- Achievement/stats display
- Community news feed