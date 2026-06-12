//! Hit detection for meteor spells absorbed by crystals.

use super::super::setup::{
    find_random_targets_in_range, increment_resonance, scaled_count, spell_echo_multiplier,
};
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::meteor_fall::casting::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::meteor_fall_constants;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;

/// Detects meteors hitting crystals and emits mini meteors.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_meteor_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — the ghost copy of the
    // remote peer's crystal is excluded so the same absorption never fires
    // twice across the network.
    mut crystals: Query<
        (&mut ArcaneCrystal, Option<&mut ResonanceCascade>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    meteors: Query<(Entity, &Transform, &MeteorProjectile)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (meteor_entity, meteor_transform, meteor) in &meteors {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = xz_distance(crystal.position, meteor_transform.translation);

            // Check if meteor is near the crystal's XZ position and falling through it
            if distance <= crystal.collision_radius
                && meteor_transform.translation.y <= crystal.position.y + CRYSTAL_HEIGHT
                && meteor_transform.translation.y >= 0.0
            {
                // Absorb the meteor
                commands.entity(meteor_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::Meteor);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(2, crystal.count_mult) * echo_mult;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                // Emit mini meteors at random targets
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                for (_, target_pos) in &enemies {
                    let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
                    let damage = meteor.damage * damage_scale;
                    let explosion_radius = meteor.explosion_radius * SIZE_SCALE;

                    let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                        &mut commands,
                        &visual_assets,
                        spawn_pos,
                        Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                        damage,
                        explosion_radius,
                        meteor.empowerment,
                        mini_radius,
                        MeteorProjectileTalentFlags::default(),
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}
