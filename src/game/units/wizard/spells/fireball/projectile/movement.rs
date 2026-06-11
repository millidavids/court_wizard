use super::super::super::super::components::{LocalWizard, Wizard};
use super::super::components::*;
use crate::config::GameConfig;
use crate::game::units::components::Hitbox;
use crate::game::units::components::Team;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::FireExplosionSphereMaterial;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Local wizard fireball casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn move_fireballs(
    time: Res<Time>,
    mut fireballs: Query<
        (&mut Transform, &Fireball),
        Without<crate::game::multiplayer::components::GhostSpellProjectile>,
    >,
) {
    for (mut transform, fireball) in &mut fireballs {
        transform.translation += fireball.velocity * time.delta_secs();
    }
}

/// Checks for fireball collisions with units or the ground.
///
/// When a fireball hits a unit or the ground, it explodes.
/// Talent effects: Cluster Bomb spawns mini-fireballs, Scorched Earth leaves burning ground.
#[allow(clippy::too_many_arguments)]
pub fn check_fireball_collisions(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    time: Res<Time>,
    #[allow(clippy::type_complexity)] fireballs: Query<
        (
            Entity,
            &Transform,
            &Fireball,
            Option<&crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellProjectile>,
    >,
    targets: Query<(&Transform, &Team, &Hitbox)>,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let t = time.elapsed_secs();

    for (fireball_entity, fireball_transform, fireball, crystal_spawn) in &fireballs {
        let fireball_pos = fireball_transform.translation;

        let explode_at = |rng: &mut dyn rand::RngCore,
                          commands: &mut Commands,
                          mats: &mut Assets<FireExplosionSphereMaterial>,
                          pos: Vec3| {
            super::spawn_explosion_with_talents(
                rng,
                commands,
                &visual_assets,
                mats,
                pos,
                fireball,
                t,
                &sfx,
                &game_config,
                crystal_spawn,
            );
        };

        // Check collision with walls
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(fireball_pos) && fireball_pos.y <= wall.height {
                explode_at(
                    &mut game_rng.0,
                    &mut commands,
                    &mut sphere_materials,
                    fireball_pos,
                );
                commands.entity(fireball_entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with rocks
        let mut hit_rock = false;
        for rock in &rocks {
            if rock.blocks_projectile(fireball_pos) {
                explode_at(
                    &mut game_rng.0,
                    &mut commands,
                    &mut sphere_materials,
                    fireball_pos,
                );
                commands.entity(fireball_entity).try_despawn();
                hit_rock = true;
                break;
            }
        }
        if hit_rock {
            continue;
        }

        // Check collision with ground (Y <= 0)
        if fireball_pos.y <= 0.0 {
            let explosion_pos = Vec3::new(fireball_pos.x, 5.0, fireball_pos.z);
            explode_at(
                &mut game_rng.0,
                &mut commands,
                &mut sphere_materials,
                explosion_pos,
            );
            commands.entity(fireball_entity).try_despawn();
            continue;
        }

        // Check collision with units (cylinder hitbox from Y=0)
        for (target_transform, _team, hitbox) in &targets {
            let hit = crate::game::units::wizard::spells::utils::sphere_intersects_cylinder(
                fireball_pos,
                fireball.radius,
                Vec3::new(
                    target_transform.translation.x,
                    0.0,
                    target_transform.translation.z,
                ),
                hitbox.radius,
                hitbox.height,
            );

            if hit {
                explode_at(
                    &mut game_rng.0,
                    &mut commands,
                    &mut sphere_materials,
                    fireball_pos,
                );
                commands.entity(fireball_entity).try_despawn();
                break;
            }
        }
    }
}

/// Despawns fireballs that travel beyond the wizard's spell range.
pub fn despawn_distant_fireballs(
    mut commands: Commands,
    fireballs: Query<
        (Entity, &Transform),
        (
            With<Fireball>,
            Without<crate::game::multiplayer::components::GhostSpellProjectile>,
        ),
    >,
    wizard_query: Query<&Wizard, (With<LocalWizard>, Without<Fireball>)>,
) {
    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    let spell_range = wizard.spell_range;

    let origin = crate::game::units::wizard::spells::utils::local_spell_origin_snapshot();
    for (entity, transform) in &fireballs {
        let distance_from_wizard = transform.translation.distance(origin);

        if distance_from_wizard > spell_range {
            commands.entity(entity).try_despawn();
        }
    }
}
