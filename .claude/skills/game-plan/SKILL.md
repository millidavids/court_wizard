---
name: game-plan
description: >
  Plan and design new features for the Court Wizard Bevy game project. Use when the user wants to
  plan a new feature, system, unit, spell, UI screen, wizard archetype, achievement, or any gameplay
  addition. Triggers on: "plan a feature", "design a new spell", "add a new unit type", "create a
  new UI screen", "new wizard archetype", "game plan", or any request to architect a new addition to
  the Court Wizard codebase. Produces a structured implementation plan with file lists, boilerplate
  patterns, integration points, and step-by-step instructions that follow the project's established
  conventions.
---

# Game Plan - Court Wizard Feature Planning

Plan new features for the Court Wizard Bevy game by following established codebase patterns exactly. Use Context7 MCP tools for Bevy and Rust documentation when needed.

## Planning Workflow

1. **Classify** the feature into one or more categories (see below)
2. **Read** the relevant reference file(s) for boilerplate templates
3. **Identify** integration points (parent plugins, state machines, config, loading)
4. **Draft** a step-by-step implementation plan with exact file paths and code patterns
5. **Review** for code sharing opportunities -- check existing shared systems before writing new ones
6. **Present** the plan for user approval before implementing

## Feature Categories

Each category has a reference file with exact boilerplate templates:

| Category | Reference | When to use |
|----------|-----------|-------------|
| New Spell | [spells.md](references/spells.md) | Adding a spell to the wizard's arsenal |
| New Unit | [units.md](references/units.md) | Adding a defender, attacker, or special unit type |
| New UI Screen | [ui-screens.md](references/ui-screens.md) | Adding a menu, overlay, HUD element, or settings page |
| New Wizard Archetype | [archetypes.md](references/archetypes.md) | Adding a new wizard class with unique mechanics |
| New Talent Tree | [talents.md](references/talents.md) | Adding a talent tree for a spell (9 talents across 3 tiers) |
| New Achievement | [achievements.md](references/achievements.md) | Adding trackable milestones or unlockables |
| Core System | [core-systems.md](references/core-systems.md) | Adding game-wide systems (combat, pathfinding, config, etc.) |
| Shader Effect | [shader-effects.md](references/shader-effects.md) | Adding screen-space post-processing effects (distortion, color, lensing) |

For features that span multiple categories (e.g., a new wizard archetype with a custom UI and achievements), read all relevant references.

## Critical Conventions

These rules apply to ALL features:

- **Visibility**: `pub(crate)` for components/constants/resources shared across crate. `pub(in crate::game)` for systems used within the game module. `mod` (private) for plugin.rs. `pub use` only for Plugin exports.
- **Messages, not Events**: Use `#[derive(Message)]` in `messages.rs` with `Message` suffix. Never use `events.rs` or `Event` suffix.
- **Constants, not styles**: Put colors, dimensions, and styling in `constants.rs`. Never create `styles.rs` in new modules (legacy ones exist but aren't the pattern to follow).
- **Run conditions**: Every `Update` system MUST have a `run_if()` guard. Never run systems unconditionally.
- **System sets**: Use `VelocitySystemSet` for targeting/flocking (parallel, immutable queries). `MovementSystemSet` for movement application (after velocity). `PostCombatSet` for post-combat reactions.
- **Code sharing**: Check `src/game/units/systems.rs` and `src/game/shared_systems.rs` before writing new logic. Extract shared patterns into reusable functions.
- **Error handling**: No `.unwrap()`. Use `.expect("reason")` only for invariants.
- **Asset loading**: Preload in `Startup` systems. Store handles in Resource structs. Reference assets as relative paths from project root.
- **Spawn queue**: New entities that spawn per-level go through `SpawnTask` enum in `src/game/loading/spawn_queue.rs`.

## Plan Output Format

Present plans in this structure:

```
## Feature: [Name]

### Summary
One-paragraph description of the feature and its gameplay impact.

### Files to Create
- `src/path/to/mod.rs` - Module definition
- `src/path/to/plugin.rs` - Plugin registration
- ... (with one-line purpose for each)

### Files to Modify
- `src/path/to/parent/plugin.rs` - Register new plugin
- ... (with specific change description)

### Implementation Steps
1. Step with specific code patterns
2. ...

### Integration Checklist
- [ ] Plugin registered in parent
- [ ] Systems gated with run_if()
- [ ] Spawn tasks added to queue (if per-level)
- [ ] Config/save data updated (if persistent)
- [ ] Achievement triggers added (if applicable)
- [ ] Changelog entry drafted
```
