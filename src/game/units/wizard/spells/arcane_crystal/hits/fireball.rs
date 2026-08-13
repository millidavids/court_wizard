//! Hit detection for fireball spells absorbed by crystals.

use super::super::setup::{
    AbsorptionBookkeeping, absorb_into_crystal, find_random_targets_in_range,
};
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::fireball_constants;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;

pub(crate) fn detect_fireball_hits(
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
    explosions: Query<(Entity, &FireballExplosion), Without<CrystalSpawn>>,
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (explosion_entity, explosion) in &explosions {
        // Visual-only explosions (the decorative sub-bubbles from
        // `spawn_explosion_bubbles`, undead-detonation VFX, etc.) carry
        // `damage_per_tick == 0.0` and are spawned WITHOUT `CrystalSpawn`. Each
        // is a fresh entity id, so without this guard every bubble triggers a
        // full absorption every frame → a geometric fireball cascade that lags
        // the game. Mirror `apply_explosion_damage`, which skips them too.
        // Skip visual-only explosions; also treat a NaN damage value as
        // visual-only (NaN <= 0.0 is false, so test it explicitly).
        if explosion.damage_per_tick.is_nan() || explosion.damage_per_tick <= 0.0 {
            continue;
        }
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            if crystal.explosions_processed.contains(&explosion_entity) {
                continue;
            }

            let distance = xz_distance(crystal.position, explosion.origin);

            if distance <= explosion.max_radius {
                // Mark before absorbing, not after: if the crystal ever refuses
                // an absorption, an unmarked explosion would be re-evaluated
                // every frame for its whole lifetime.
                crystal.explosions_processed.insert(explosion_entity);

                let rng = &mut game_rng.0;
                let Some(absorption) = absorb_into_crystal(
                    &mut commands,
                    &mut crystal,
                    &mut resonance,
                    &mut progress,
                    rng,
                    CrystalInfusion::Fireball,
                    MINI_FB_COUNT,
                    AbsorptionBookkeeping::DISCRETE,
                ) else {
                    continue;
                };

                // Emit mini fireballs at random targets
                let count = absorption.count;
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                let mini_radius =
                    fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                for (_, target_pos) in &enemies {
                    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
                    let direction = (ground_target - crystal.position).normalize();
                    let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                    let velocity = direction * speed;

                    let entity = fireball_systems::spawn_fireball_entity(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        velocity,
                        fireball_constants::DAMAGE_PER_TICK * damage_scale,
                        fireball_constants::DAMAGE_TYPE,
                        fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                        explosion.empowerment * damage_scale,
                        mini_radius,
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }
            }
        }
    }
}
