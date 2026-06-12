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
}
