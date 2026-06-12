//! Back-compat guard for save-data keys.
//!
//! On-disk save keys for `Spell`, `Ingredient`, `UnitType`, and `WizardType` were
//! historically produced by `format!("{:?}", variant)`. The `save_key()` methods
//! pin those strings explicitly so a future variant rename (or a change to the
//! `Debug` derive) cannot silently orphan existing player saves. These tests fail
//! loudly if `save_key()` ever diverges from the `Debug` representation that is
//! already written to disk.

#[cfg(test)]
mod tests {
    use crate::config::resources::WizardType;
    use crate::game::cauldron::brews::Ingredient;
    use crate::game::units::UnitType;
    use crate::game::units::wizard::components::Spell;

    #[test]
    fn spell_save_key_matches_debug() {
        for v in Spell::all() {
            assert_eq!(
                v.save_key(),
                format!("{v:?}"),
                "Spell save_key drifted from Debug"
            );
        }
    }

    #[test]
    fn ingredient_save_key_matches_debug() {
        for v in Ingredient::all() {
            assert_eq!(
                v.save_key(),
                format!("{v:?}"),
                "Ingredient save_key drifted from Debug"
            );
        }
        // PhilosophersStone is intentionally excluded from `Ingredient::all()`
        // (never dropped / unlocked), so the loop above doesn't reach it — pin
        // it explicitly so a rename can't silently drift its key either.
        assert_eq!(
            Ingredient::PhilosophersStone.save_key(),
            format!("{:?}", Ingredient::PhilosophersStone)
        );
    }

    #[test]
    fn unit_type_save_key_matches_debug() {
        for v in UnitType::all() {
            assert_eq!(
                v.save_key(),
                format!("{v:?}"),
                "UnitType save_key drifted from Debug"
            );
        }
    }

    #[test]
    fn wizard_type_save_key_matches_debug() {
        for v in WizardType::all() {
            assert_eq!(
                v.save_key(),
                format!("{v:?}"),
                "WizardType save_key drifted from Debug"
            );
        }
    }

    /// `AchievementId::id()` is a custom snake_case save key (not the Debug name),
    /// so the realistic save-corruption risk is a duplicate or empty id rather than
    /// drift from Debug. Guard against both — a collision would conflate two
    /// achievements' unlock state on disk.
    #[test]
    fn achievement_ids_are_unique_and_nonempty() {
        use crate::config::save_data::AchievementId;
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        for a in AchievementId::all() {
            let id = a.id();
            assert!(!id.is_empty(), "AchievementId {a:?} has an empty save id");
            assert!(seen.insert(id), "duplicate AchievementId save id: {id:?}");
        }
    }
}
