# New Achievement Reference

Achievements are tracked via per-achievement resources using a macro system. They persist across sessions via localStorage save data.

## Key Files

The `src/game/achievements/` module is feature-sliced:

- `src/game/achievements/resources.rs` — Achievement resource definitions (macro-generated)
- `src/game/achievements/tracking.rs` — Kill counts, milestone counters, in-run stats
- `src/game/achievements/unlocks.rs` — Unlock-condition systems (one per achievement)
- `src/game/achievements/notifications.rs` — Popup/UI announcement systems
- `src/game/achievements/messages.rs` — AchievementUnlockedMessage, ClearProgressMessage
- `src/game/achievements/plugin.rs` — System registration only
- `src/config/save/achievement_id.rs` — `AchievementId` enum (post-Phase-11). Pre-phase: `src/config/save_data.rs`.

When adding a new achievement, place its trigger system in the file matching its concern (`tracking.rs` for stat-based, `unlocks.rs` for milestone-based, `notifications.rs` for UI behavior).

## Step-by-Step

### 1. Add AchievementId

In `src/config/save/achievement_id.rs` (post-Phase-11; pre-phase: `src/config/save_data.rs`), add variant to `AchievementId` enum:
```rust
pub enum AchievementId {
    // ... existing
    YourAchievement,
}
```

Update the `id()` method to return a unique string key:
```rust
AchievementId::YourAchievement => "your_achievement",
```

Update `display_name()` and `description()` methods.

### 2. Define Achievement Resource

In `src/game/achievements/resources.rs`, use the macro:
```rust
achievement_resource!(YourAchievementAchievement, AchievementId::YourAchievement);
```

Add to `init_achievements()` function:
```rust
init!(YourAchievementAchievement);
```

Add to `reset_all_achievements()` function:
```rust
commands.insert_resource(YourAchievementAchievement(false));
```

### 3. Add Trigger System

In `src/game/achievements/systems.rs`, add a system that checks the unlock condition:

```rust
pub fn check_your_achievement(
    mut achievement: ResMut<YourAchievementAchievement>,
    mut unlock_msg: MessageWriter<AchievementUnlockedMessage>,
    // ... whatever resources/queries needed to detect the condition
) {
    if !achievement.is_locked() {
        return; // Already unlocked
    }

    if /* condition met */ {
        achievement.unlock();
        unlock_msg.write(AchievementUnlockedMessage {
            id: AchievementId::YourAchievement,
        });
    }
}
```

### 4. Register System

In `src/game/achievements/plugin.rs`:
```rust
.add_systems(
    Update,
    systems::check_your_achievement
        .run_if(achievement_locked::<YourAchievementAchievement>)
        .run_if(is_gameplay_running),
)
```

The `achievement_locked::<T>` run condition prevents the system from running after unlock, saving performance.

### 5. Achievement Categories

Group achievements logically:
- **Victory & Progression** - Level milestones (level 5, 10, 25, 50, 100)
- **Defeat & Failure** - Losing in specific ways
- **Mid-battle** - Combat-specific feats
- **Meta / Unlocks** - Wizard archetype unlocks, UI interactions
- **Unit Encounters** - First time encountering enemy types
- **Spell Unlocks** - Spells unlocked via progression

### 6. Achievement as Unlock Gate

Achievements commonly gate content. To use an achievement as a gate:

```rust
// Run condition: only run system while achievement is locked
.run_if(achievement_locked::<YourAchievementAchievement>)

// Query in system: check if unlocked
fn some_system(achievement: Res<YourAchievementAchievement>) {
    if achievement.is_locked() {
        // Still locked
    }
}
```

### 7. Achievement Checklist

- [ ] AchievementId variant added to save_data.rs with id(), display_name(), description()
- [ ] Resource defined via `achievement_resource!` macro
- [ ] Added to `init_achievements()`
- [ ] Added to `reset_all_achievements()`
- [ ] Trigger system written with unlock condition
- [ ] System registered in plugin with `achievement_locked::<T>` run condition
- [ ] Achievement popup appears via AchievementUnlockedMessage
- [ ] Content gated by achievement (if applicable) works correctly
- [ ] Compendium entry displays correctly (if shown in compendium)
