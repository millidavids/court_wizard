//! Utility infusions: the crystal doing a job rather than dealing damage.

use bevy::prelude::*;

use super::super::constants::*;
use super::driver::{InfusedCrystals, begin_infusion_tick};
use super::kinds::CrystalInfusion;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::units::components::{Corpse, PermanentCorpse};
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::raise_the_dead::casting::raise_corpse_as_undead;
use crate::game::units::wizard::spells::raise_the_dead::components::RaiseTheDeadTalentParams;
use crate::game::units::wizard::spells::telekinesis::systems::convert_drop_to_flying;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// Raises corpses lying inside the crystal's range.
///
/// The burst empties the field; each ongoing tick raises one more, so the value
/// of this infusion grows with the battle's attrition rather than being fixed at
/// cast time.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tick_raise_the_dead_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    corpses: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Option<Res<UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let Some(undead_assets) = undead_assets else {
        return;
    };
    let is_guest = session
        .as_deref()
        .is_some_and(|s| s.role == crate::networking::resources::PeerRole::Guest);
    let delta = time.delta_secs();

    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::RaiseTheDead,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        let limit = params.pick_count(INFUSION_BURST_COUNT * 2, 1);

        let in_range: Vec<(Entity, Vec3)> = corpses
            .iter()
            .filter(|(_, transform)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .map(|(corpse, transform)| (corpse, transform.translation))
            .take(limit)
            .collect();

        // Host-authoritative. Raising locally on the guest would mutate a corpse
        // the host owns, so the guest would see phantom undead the host has no
        // record of. The hand-cast spell forwards a `RaiseCorpse` message
        // instead, and the crystal must do the same.
        if is_guest {
            continue;
        }

        let talent_params = RaiseTheDeadTalentParams::default();
        for (corpse, position) in in_range {
            raise_corpse_as_undead(
                &mut commands,
                corpse,
                position,
                // Crystal-raised undead come up at half strength, matching
                // DAMAGE_SCALE for every other crystal echo.
                UNIT_HEALTH * params.empowerment * INFUSION_DURATION_SCALE,
                UNIT_MOVEMENT_SPEED * 0.5,
                &talent_params,
                params.empowerment,
                &undead_assets,
                &mut materials,
                Some(&mut progress),
            );
        }
    }
}

/// Collects ingredient drops lying inside the crystal's range.
///
/// Takes a handful at a time rather than emptying the field. Telekinesis cannot
/// start casting unless `find_nearest_drop` finds a drop that is not already
/// flying, so a crystal that swept up everything in its 500-unit radius every
/// frame would silently make its own parent spell uncastable — the cast never
/// begins and no mana is spent, which reads as the spell being broken.
pub(crate) fn tick_telekinesis_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    drops: Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Telekinesis,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };

        let collected = drops
            .iter()
            .filter(|(_, transform, _)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .take(params.pick_count(INFUSION_BURST_COUNT, INFUSION_ONGOING_COUNT));

        for (drop, transform, ingredient) in collected {
            convert_drop_to_flying(
                &mut commands,
                drop,
                ingredient.ingredient,
                transform.translation,
            );
        }
    }
}
