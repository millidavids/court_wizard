use bevy::prelude::*;

use crate::game::units::archer::ArcherAssets;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::resources::KingAssets;
use crate::game::units::undead::resources::UndeadAssets;

/// Receives the latest unit state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
///
/// Filters incoming unreliable data by type prefix byte, processing only game
/// snapshots (unit data). Spell visual snapshots are handled by `spell_sync.rs`.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn pick_material(
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    team: crate::game::units::components::Team,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
) -> Handle<StandardMaterial> {
    use crate::game::units::components::{CORPSE_MATERIAL_VARIANTS, Team};
    use crate::game::units::systems::{corpse_material_for_team, create_default_sprite_material};

    if is_corpse {
        let idx = rand::random_range(0..CORPSE_MATERIAL_VARIANTS);
        if is_king {
            king_assets.corpse_materials[idx].clone()
        } else if is_archer {
            corpse_material_for_team(
                &archer_assets.defender_corpse_materials,
                &archer_assets.attacker_corpse_materials,
                &archer_assets.undead_corpse_materials,
                team,
                idx,
            )
        } else {
            corpse_material_for_team(
                &infantry_assets.defender_corpse_materials,
                &infantry_assets.attacker_corpse_materials,
                &infantry_assets.undead_corpse_materials,
                team,
                idx,
            )
        }
    } else if team == Team::Undead {
        // Raised undead always use the dedicated undead walking sprite (purple
        // tint), regardless of the unit type they were raised from. Checked before
        // the type branches so a resurrected archer / king's-guard still reads as
        // undead. Without this the guest rendered raised units as living infantry.
        use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            undead_assets.sprite_texture.clone(),
            UNDEAD_SPRITE_TINT,
        )
    } else if is_king {
        use crate::game::units::king::constants::KING_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            king_assets.sprite_texture.clone(),
            KING_SPRITE_TINT,
        )
    } else if is_archer {
        let tint = crate::game::units::systems::archer_sprite_tint_for_team(team);
        create_default_sprite_material(materials, archer_assets.sprite_texture.clone(), tint)
    } else if is_guard {
        use crate::game::units::infantry::constants::KINGS_GUARD_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            infantry_assets.sprite_texture.clone(),
            KINGS_GUARD_SPRITE_TINT,
        )
    } else {
        let tint = crate::game::units::systems::sprite_tint_for_team(team);
        create_default_sprite_material(materials, infantry_assets.sprite_texture.clone(), tint)
    }
}
