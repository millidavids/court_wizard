# Wizard Tower Defense - Gameplay Implementation Plan

**Date**: 2026-01-17  
**Project**: the_game (Bevy 0.17.3)  
**Status**: Phase 1 Complete - Basic 3D Perspective Gameplay Working

---

## Overview

This plan details the implementation of the core gameplay for a wizard tower defense game. The player is a wizard standing on castle battlements, commanding defenders while casting spells at approaching enemy attackers.

**IMPORTANT ARCHITECTURAL CHANGE**: Migrated from 2D to 3D rendering to achieve proper perspective depth.

---

## ✅ COMPLETED: Phase 1 - Core 3D Gameplay

### What Was Implemented

#### 1. 3D Camera System (src/main.rs:56-58)
- **Changed from Camera2d to Camera3d** with perspective projection
- Position: `(0, 2000, 2000)` - far back and elevated for wide battlefield view
- Looking at: `(0, 0, 0)` - centered on origin
- Provides bird's-eye isometric-style view with automatic perspective scaling

#### 2. Component System (src/game/components.rs)
✅ All components implemented with 3D support:
```rust
- OnGameplayScreen (cleanup marker)
- Wizard
- Castle  
- Battlefield
- Defender
- Attacker
- Velocity { x, y, z }  // Added Z component for 3D movement
```

#### 3. Visual Entities (Using 3D Meshes)

**Battlefield** (src/game/systems.rs:25-38)
- `Plane3d` mesh (3000x3000)
- Position: `(0, 0, 0)` - ground level at origin
- Color: Muted green
- Provides base plane for all gameplay

**Castle Platform** (src/game/systems.rs:40-52)
- `Plane3d` mesh (400x300) 
- Position: `(-1100, 500, 1100)` - **raised 500 units above ground** in bottom-left corner
- Color: Light gray
- Floating platform where wizard stands

**Wizard** (src/game/systems.rs:54-75)
- `Triangle2d` mesh (60 pixels)
- Position: `(-1100, 501, 1100)` - on castle platform
- Color: Blue
- Static position (no spells yet)

**Defenders** (src/game/systems.rs:95-118)
- `Circle` meshes (15 pixel radius)
- Spawn position: `(-1000, 50, 1000)` - near castle in bottom-left
- Color: Yellow
- Spawn every 2 seconds

**Attackers** (src/game/systems.rs:133-156)
- `Circle` meshes (15 pixel radius)
- Spawn position: `(1200, 50, -1200)` - **top-right corner, far from castle**
- Color: Red
- Spawn every 2 seconds

#### 4. Game Systems (All Working)

**Setup System** (OnEnter(AppState::InGame))
- Spawns PointLight for 3D visibility
- Creates battlefield, castle, wizard
- Uses StandardMaterial with `unlit: true` for flat shading

**Cleanup System** (OnExit(AppState::InGame))
- Despawns all OnGameplayScreen entities

**Spawn Systems** (Update, when InGameState::Running)
- Defenders: spawn every 2 seconds near castle
- Attackers: spawn every 2 seconds in top-right

**Movement System** (Update, when InGameState::Running)
- Applies 3D velocity (x, y, z)
- Frame-rate independent (uses Time delta)
- **Perspective automatically scales distant units smaller**

**Targeting System** (Update, when InGameState::Running)
- Defenders: target nearest attacker in 3D space
- Attackers: target wizard position dynamically
- Uses 3D distance calculations

**Combat System** (Update, when InGameState::Running)
- Simple collision detection (COMBAT_RANGE = 30.0)
- Both units despawn on contact
- No health/damage system yet

#### 5. Current File Structure
```
src/
├── main.rs (Camera3d setup)
├── game/
│   ├── mod.rs (exports GamePlugin)
│   ├── plugin.rs (system registration)
│   ├── components.rs (all game components)
│   ├── systems.rs (all game logic)
│   └── styles.rs (colors, sizes, constants)
├── config/ (settings system)
├── state/ (AppState, InGameState)
└── ui/ (menus)
```

---

## Current Visual Layout

```
Camera View (from above and behind):
┌─────────────────────────────────────┐
│                                     │
│              Attackers spawn here → │ (top-right)
│                     ↘              │
│                       ↘            │
│         Battlefield    ↘           │
│         (Green)         ↘          │
│                          ↘         │
│                    Defenders meet  │
│                    Attackers       │
│                          ↙         │
│  Castle ←─ Defenders   ↙           │
│  (Gray)    spawn      ↙            │
│  + Wizard                          │
│  (Blue △)                          │
└─────────────────────────────────────┘
(bottom-left)
```

**Movement Pattern**: Attackers move diagonally from top-right to bottom-left toward wizard
**Depth Effect**: Units appear smaller when farther from camera (automatic perspective)

---

## Key Style Constants (src/game/styles.rs)

```rust
// Colors
WIZARD_COLOR: Blue (0.2, 0.4, 0.9)
CASTLE_COLOR: Gray (0.7, 0.7, 0.7)
BATTLEFIELD_COLOR: Green (0.4, 0.5, 0.35)
DEFENDER_COLOR: Yellow (0.9, 0.9, 0.2)
ATTACKER_COLOR: Red (0.9, 0.2, 0.2)

// Gameplay
UNIT_RADIUS: 15.0 pixels
UNIT_SPEED: 100.0 units/second
SPAWN_INTERVAL: 2.0 seconds
COMBAT_RANGE: 30.0 units
```

---

## ❌ What Was Removed/Changed from Original Plan

### Original 2D Approach (Deprecated)
- ~~Camera2d~~ → Changed to **Camera3d**
- ~~Sprite components~~ → Changed to **3D Meshes** (Plane3d, Triangle2d, Circle)
- ~~ColorMaterial~~ → Changed to **StandardMaterial**
- ~~Z-index layering~~ → Changed to **actual 3D depth (Y-axis)**
- ~~2D positioning~~ → Changed to **3D coordinates (x, y, z)**

### Why The Change?
User requested:
1. Perspective depth with horizon
2. Units that scale based on distance from camera
3. Isometric-style view looking over battlefield

This required true 3D perspective, which Sprites (2D-only) cannot provide.

---

## 🔧 How to Continue Tomorrow

### Quick Start
1. `cd /home/david/code/the_game`
2. `cargo run` - Test native build
3. `./build_wasm.sh` - Build for browser
4. Open `web/index.html` in browser

### Current State
- ✅ Core gameplay loop working
- ✅ 3D perspective rendering
- ✅ Unit spawning and movement
- ✅ Combat system functional
- ✅ Pause/resume works
- ✅ WASM build working

### Camera Position (Adjustable)
Current: `Transform::from_xyz(0.0, 2000.0, 2000.0).looking_at(Vec3::ZERO, Vec3::Y)`
- To zoom in: Reduce Y and Z values
- To change angle: Adjust looking_at target
- To rotate view: Change X position

---

## 🎯 Next Steps (Not Yet Implemented)

### Priority 1: Camera Refinement
- [ ] Zoom camera closer for better gameplay view
- [ ] Fine-tune camera angle for optimal perspective
- [ ] Consider adding camera controls (zoom in/out)

### Priority 2: Visual Improvements
- [ ] Add shadows to 3D meshes
- [ ] Improve unit visibility (make circles more distinct)
- [ ] Add visual effects for combat
- [ ] Consider adding grid lines to battlefield

### Priority 3: Gameplay Features (Deferred)

#### Wizard Spells
- [ ] Click-to-cast mechanics
- [ ] Spell types (AOE, single target, projectiles)
- [ ] Mana/cooldown system
- [ ] Visual effects for spells

#### Unit Health & Damage
- [ ] Health component for units
- [ ] Damage system (instead of instant despawn)
- [ ] Health bars above units
- [ ] Death animations

#### Win/Lose Conditions
- [ ] Castle health (attackers damage it)
- [ ] Wave completion tracking
- [ ] GameOver state transitions
- [ ] Victory/defeat screens
- [ ] Score system

#### Additional Unit Types
- [ ] Multiple defender types (melee, ranged, tank)
- [ ] Multiple attacker types (fast, strong, flying)
- [ ] Special abilities for units
- [ ] Unit upgrade system

#### Waves & Difficulty
- [ ] Wave system with breaks between waves
- [ ] Increasing difficulty curve
- [ ] Boss units
- [ ] Resource economy (spend resources on defenders)

---

## 📝 Important Notes for Tomorrow

### 3D Rendering in Bevy
- **Sprites don't work with Camera3d** - must use Mesh3d
- StandardMaterial required for 3D (not ColorMaterial)
- PointLight needed for visibility (unless `unlit: true`)
- Perspective is automatic once using Camera3d

### Current Positions (3D Coordinates)
```rust
// World origin: (0, 0, 0)
Battlefield: (0, 0, 0)        // Ground level
Castle: (-1100, 500, 1100)    // Raised platform, bottom-left
Wizard: (-1100, 501, 1100)    // On castle
Defender spawn: (-1000, 50, 1000)    // Near castle
Attacker spawn: (1200, 50, -1200)    // Top-right, far away
```

### Coordinate System
- **X-axis**: Left (negative) to Right (positive)
- **Y-axis**: Down (negative) to Up (positive) - height above ground
- **Z-axis**: Back (negative) to Front (positive) - depth from camera

### Perspective Scaling
Units automatically scale based on distance from camera:
- Attackers start small (far away at Z = -1200)
- As they move toward wizard, they appear larger
- This creates natural depth perception

### State Management
- Game only updates when `InGameState::Running`
- Pause menu sets state to `InGameState::Paused` (stops all game systems)
- Escape key toggles pause
- All entities cleaned up on exit to main menu

### Building
- **Native**: `cargo run` or `cargo build --release`
- **WASM**: `./build_wasm.sh` (builds to `web/` directory)
- No warnings in current build

---

## 🐛 Known Issues / Considerations

### None Currently
Everything is working as intended for Phase 1.

### Future Considerations
1. **Performance**: May need optimization with many units
2. **Unit Overlap**: Multiple units can stack at same position
3. **Pathfinding**: Units move in straight lines (could collide with obstacles later)
4. **Balance**: Spawn rates and speeds not tuned for difficulty

---

## 📚 Reference: Original vs Current Implementation

| Feature | Original Plan | Current Implementation |
|---------|--------------|----------------------|
| Camera | Camera2d | **Camera3d with perspective** |
| Rendering | 2D Sprites | **3D Meshes (Plane3d, Triangle2d, Circle)** |
| Materials | ColorMaterial | **StandardMaterial** |
| Depth | Z-index (fake) | **True 3D Y-coordinates** |
| Perspective | None | **Automatic camera perspective** |
| Movement | 2D (x, y) | **3D (x, y, z)** |
| Castle | Rectangle sprite | **Raised Plane3d platform** |
| Battlefield | Rectangle sprite | **Plane3d ground** |
| Lighting | Not needed | **PointLight added** |

---

## End of Updated Plan

**Status**: Phase 1 complete and working. Core gameplay loop functional with 3D perspective rendering.

**Ready for**: Camera refinement and additional gameplay features.

**Last Updated**: 2026-01-17
