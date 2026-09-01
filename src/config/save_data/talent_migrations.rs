//! One-time talent-layout migrations.
//!
//! Talent picks are stored positionally — `spell_talent_selections` maps a spell
//! key to a `Vec<i8>` of tier → choice index, and nothing records which talent an
//! index actually meant. Re-tiering a talent therefore silently repoints every
//! existing pick at whatever now occupies that slot.
//!
//! The remedy is deliberately blunt: clear the tiers whose contents changed and
//! let the player choose again. Clearing is idempotent, so if the marker is ever
//! lost (an older build round-tripping the save and dropping the unknown field,
//! say) the worst case is "re-pick those rows once more" rather than a silently
//! swapped build. Talent *progress* is never touched, so anything the player had
//! unlocked stays unlocked.

use super::save_structs::UnifiedSaveFile;

/// Wall of Dirt: Terraformer moved from tier 3 choice 1 to tier 1 choice 2,
/// trading places with Quick Foundations. Tier 2 was untouched by the swap.
const WALL_OF_DIRT_RETIER: &str = "wall_of_dirt_retier_v1";

/// Every migration id known to this build, in no particular order.
const ALL_MIGRATIONS: &[&str] = &[WALL_OF_DIRT_RETIER];

/// Marks a brand-new save as already migrated.
///
/// A save created by this build is born in the current layout, so replaying the
/// migrations against it would clear picks the player made under the *new*
/// rules. Callers that construct a fresh `UnifiedSaveFile` must call this.
pub(crate) fn mark_new_save_migrated(save: &mut UnifiedSaveFile) {
    for id in ALL_MIGRATIONS {
        if !save.player.applied_migrations.iter().any(|m| m == id) {
            save.player.applied_migrations.push((*id).to_string());
        }
    }
}

/// Applies any outstanding talent-layout migrations to a save loaded from disk.
///
/// Mutates in place; persisting is left to the next ordinary save write, which
/// stores the cleared picks and the marker together in one atomic write. Do not
/// call `save_unified` from here — this runs inside the cache load path and
/// would re-enter the cache mutex.
pub(crate) fn apply_talent_migrations(save: &mut UnifiedSaveFile) {
    if !has_applied(save, WALL_OF_DIRT_RETIER) {
        clear_tiers(save, "WallOfStone", &[0, 2]);
        save.player
            .applied_migrations
            .push(WALL_OF_DIRT_RETIER.to_string());
    }
}

fn has_applied(save: &UnifiedSaveFile, id: &str) -> bool {
    save.player.applied_migrations.iter().any(|m| m == id)
}

/// Resets the given tiers of one spell's selections to "no choice" (`-1`),
/// leaving every other tier as-is. No-op when the spell has no saved entry.
fn clear_tiers(save: &mut UnifiedSaveFile, spell_key: &str, tiers: &[usize]) {
    let Some(entry) = save.player.spell_talent_selections.get_mut(spell_key) else {
        return;
    };
    for &tier in tiers {
        if let Some(slot) = entry.get_mut(tier) {
            *slot = -1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::save_data::new_unified_save;

    /// `new_unified_save` already stamps the markers, so start from a save that
    /// predates them.
    fn legacy_save(selections: Vec<i8>) -> UnifiedSaveFile {
        let mut save = new_unified_save();
        save.player.applied_migrations.clear();
        save.player
            .spell_talent_selections
            .insert("WallOfStone".to_string(), selections);
        save
    }

    fn wall_selections(save: &UnifiedSaveFile) -> Vec<i8> {
        save.player.spell_talent_selections["WallOfStone"].clone()
    }

    #[test]
    fn clears_tier_1_and_3_but_keeps_tier_2() {
        // Quarry Master / Permafrost Aura / Terraformer
        let mut save = legacy_save(vec![0, 1, 1]);
        apply_talent_migrations(&mut save);
        assert_eq!(wall_selections(&save), vec![-1, 1, -1]);
    }

    #[test]
    fn leaves_talent_progress_alone() {
        let mut save = legacy_save(vec![2, 0, 2]);
        save.player
            .spell_talent_progress
            .insert("WallOfStone".to_string(), 412);
        apply_talent_migrations(&mut save);
        assert_eq!(save.player.spell_talent_progress["WallOfStone"], 412);
    }

    #[test]
    fn is_idempotent_even_if_the_marker_is_lost() {
        let mut save = legacy_save(vec![2, 0, 1]);
        apply_talent_migrations(&mut save);
        let once = wall_selections(&save);

        // Simulate an older build round-tripping the save and dropping the
        // unknown `applied_migrations` field.
        save.player.applied_migrations.clear();
        apply_talent_migrations(&mut save);

        assert_eq!(wall_selections(&save), once);
        assert_eq!(once, vec![-1, 0, -1]);
    }

    #[test]
    fn runs_only_once_while_the_marker_survives() {
        let mut save = legacy_save(vec![1, 2, 0]);
        apply_talent_migrations(&mut save);

        // A player re-picking under the new layout must not be reset again.
        save.player
            .spell_talent_selections
            .insert("WallOfStone".to_string(), vec![2, 2, 1]);
        apply_talent_migrations(&mut save);

        assert_eq!(wall_selections(&save), vec![2, 2, 1]);
    }

    #[test]
    fn tolerates_missing_and_short_entries() {
        // No WallOfStone entry at all.
        let mut save = new_unified_save();
        save.player.applied_migrations.clear();
        apply_talent_migrations(&mut save);
        assert!(
            !save
                .player
                .spell_talent_selections
                .contains_key("WallOfStone")
        );
        assert!(has_applied(&save, WALL_OF_DIRT_RETIER));

        // Truncated vec — tier 2 index doesn't exist.
        let mut save = legacy_save(vec![2]);
        apply_talent_migrations(&mut save);
        assert_eq!(wall_selections(&save), vec![-1]);
    }

    #[test]
    fn new_saves_are_born_migrated() {
        let save = new_unified_save();
        assert!(has_applied(&save, WALL_OF_DIRT_RETIER));
    }
}
