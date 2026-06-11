use super::super::components::{FogCloudZone, PhantomFogZone, PhantomUnit};
use super::super::constants;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::multiplayer::components::{GhostEntity, GhostSpellEffect};
use crate::game::units::components::{AttackTiming, Effectiveness, Health, Hitbox, Stunned, Team};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::systems::create_default_sprite_material;
use bevy::prelude::*;

/// Tier 3: Phantom Fog — periodically spawns phantom decoy units inside fog zones.
pub fn spawn_phantom_units(
    time: Res<Time>,
    mut commands: Commands,
    mut zones: Query<(Entity, &FogCloudZone, &mut PhantomFogZone), Without<GhostSpellEffect>>,
    existing_phantoms: Query<&PhantomUnit>,
    infantry_assets: Res<InfantryAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let mut phantom_count: Option<usize> = None;

    for (_zone_entity, zone, mut phantom_zone) in &mut zones {
        phantom_zone.spawn_timer += dt;
        if phantom_zone.spawn_timer < constants::PHANTOM_SPAWN_INTERVAL {
            continue;
        }
        phantom_zone.spawn_timer -= constants::PHANTOM_SPAWN_INTERVAL;

        // Count phantoms lazily (only when a zone is ready to spawn)
        let count = *phantom_count.get_or_insert_with(|| existing_phantoms.iter().count());
        if count >= constants::PHANTOM_MAX_TOTAL {
            continue;
        }

        // Spawn phantom at a random position within the zone
        let seed = t * 7.1 + zone.origin.x * 3.3;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5;
        let r_frac = 0.3 + 0.5 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let r = zone.radius * r_frac;
        let x = zone.origin.x + angle.cos() * r;
        let z = zone.origin.z + angle.sin() * r;

        // Ghostly translucent infantry sprite
        let material = create_default_sprite_material(
            &mut materials,
            infantry_assets.sprite_texture.clone(),
            Color::srgba(0.7, 0.75, 0.85, 0.3),
        );

        commands.spawn((
            Mesh3d(infantry_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(x, constants::PHANTOM_HITBOX_HEIGHT * 0.5, z)),
            Billboard,
            Hitbox::new(
                constants::PHANTOM_HITBOX_RADIUS,
                constants::PHANTOM_HITBOX_HEIGHT,
            ),
            Health::new(1.0),
            Team::Defenders,
            AttackTiming::new(),
            Effectiveness::new(),
            Stunned {
                time_remaining: f32::MAX,
            },
            PhantomUnit,
            OnGameplayScreen,
        ));
    }
}

/// Despawns phantom units when they die or when no phantom fog zones remain.
#[allow(clippy::type_complexity)]
pub fn cleanup_phantom_units(
    mut commands: Commands,
    phantoms: Query<(Entity, &Health), (With<PhantomUnit>, Without<GhostEntity>)>,
    zones: Query<&PhantomFogZone, Without<GhostSpellEffect>>,
) {
    let zones_exist = !zones.is_empty();
    for (entity, health) in &phantoms {
        if health.current <= 0.0 || !zones_exist {
            commands.entity(entity).try_despawn();
        }
    }
}
