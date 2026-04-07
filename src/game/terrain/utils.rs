use bevy::prelude::*;

use super::wind_sway::material::WindSwayMaterial;
use crate::game::units::systems::create_sprite_material;
use crate::game::units::components::FireDoT;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::{MeteorExplosion, MeteorGroundFire};
use crate::game::units::wizard::spells::utils::{distance_to_line_segment_xz, xz_distance};
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Tint color applied to burning vegetation (trees, bushes).
pub(crate) const BURNING_VEGETATION_TINT: Color = Color::srgba(1.0, 0.55, 0.25, 1.0);

/// Loads a horizontal sprite sheet and creates one material per sprite variant.
///
/// Returns a `(mesh, materials)` tuple where the mesh is a rectangle sized to
/// a single sprite and each material samples the correct UV region.
pub(crate) fn preload_sprite_sheet<const N: usize>(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    asset_path: &'static str,
    sprite_width: f32,
    sprite_height: f32,
) -> (Handle<Mesh>, [Handle<StandardMaterial>; N]) {
    let texture: Handle<Image> = asset_server.load(asset_path);
    let mesh = meshes.add(Rectangle::new(sprite_width, sprite_height));

    let u_width = 1.0 / N as f32;
    let sprite_materials: [Handle<StandardMaterial>; N] = std::array::from_fn(|i| {
        let u_offset = i as f32 * u_width;
        create_sprite_material(
            materials,
            texture.clone(),
            Color::WHITE,
            Vec2::new(u_width, 1.0),
            Vec2::new(u_offset, 0.0),
        )
    });

    (mesh, sprite_materials)
}

/// Loads a horizontal sprite sheet with wind sway materials.
///
/// Same as [`preload_sprite_sheet`] but creates [`WindSwayMaterial`] instances
/// with per-variant phase offsets for organic wind animation.
pub(crate) fn preload_wind_sway_sprite_sheet<const N: usize>(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<WindSwayMaterial>,
    asset_server: &AssetServer,
    asset_path: &'static str,
    sprite_width: f32,
    sprite_height: f32,
    sway_amplitude: f32,
    sway_speed: f32,
) -> (Handle<Mesh>, [Handle<WindSwayMaterial>; N]) {
    let texture: Handle<Image> = asset_server.load(asset_path);
    let mesh = meshes.add(Rectangle::new(sprite_width, sprite_height));

    let u_width = 1.0 / N as f32;
    let sprite_materials: [Handle<WindSwayMaterial>; N] = std::array::from_fn(|i| {
        let u_offset = i as f32 * u_width;
        materials.add(WindSwayMaterial {
            base_color: LinearRgba::WHITE,
            uv_params: Vec4::new(u_width, 1.0, u_offset, 0.0),
            sway_params: Vec4::new(
                0.0,                  // time (updated each frame)
                sway_amplitude,
                sway_speed,
                i as f32 * 1.37,      // phase offset per variant
            ),
            base_texture: texture.clone(),
        })
    });

    (mesh, sprite_materials)
}

/// Checks if a terrain object at `center` with `radius` should be ignited by active fire sources.
#[allow(clippy::too_many_arguments)]
pub(crate) fn should_ignite_from_fire(
    center: Vec3,
    radius: f32,
    explosions: &Query<&FireballExplosion>,
    meteor_explosions: &Query<&MeteorExplosion>,
    beams: &Query<&DisintegrateBeam>,
    walls: &Query<&WallOfFireEffect>,
    ground_fires: &Query<&MeteorGroundFire>,
) -> bool {
    for explosion in explosions.iter() {
        if explosion.damage_per_tick > 0.0
            && xz_distance(explosion.origin, center) <= explosion.current_radius() + radius
        {
            return true;
        }
    }

    for explosion in meteor_explosions.iter() {
        if !explosion.damage_applied
            && xz_distance(explosion.origin, center) <= explosion.max_radius + radius
        {
            return true;
        }
    }

    for beam in beams.iter() {
        if beam.contains_point_with_radius(center, radius) {
            return true;
        }
    }

    for wall in walls.iter() {
        if distance_to_line_segment_xz(center, wall.start, wall.end) <= wall.half_width + radius {
            return true;
        }
    }

    for fire in ground_fires.iter() {
        if xz_distance(fire.origin, center) <= fire.radius + radius {
            return true;
        }
    }

    false
}

/// Applies or stacks FireDoT on a unit from burning terrain heat.
/// Respects spell shield — shielded units are unaffected.
pub(crate) fn apply_heat_zone_fire_dot(
    commands: &mut Commands,
    entity: Entity,
    existing_dot: Option<Mut<FireDoT>>,
    has_spell_shield: bool,
    spell_damage: f32,
) {
    if has_spell_shield {
        return;
    }
    if let Some(mut dot) = existing_dot {
        dot.stack(spell_damage);
    } else {
        let new_dot = FireDoT::new(spell_damage);
        commands
            .entity(entity)
            .queue_silenced(move |mut e: EntityWorldMut| {
                e.insert(new_dot);
            });
    }
}

/// Emits fire smoke and spark VFX at random heights along a burning terrain sprite.
pub(crate) fn emit_burning_vfx(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    center_x: f32,
    center_z: f32,
    sprite_height: f32,
    radius: f32,
    smoke_count: usize,
    spark_count: usize,
    t: f32,
    emit_smoke: bool,
    emit_sparks: bool,
) {
    if emit_smoke {
        for i in 0..smoke_count {
            let seed = center_x * 0.1 + t + i as f32 * 1.37;
            let y = ((seed * 7.3).sin() * 0.5 + 0.5) * sprite_height * vfx::constants::BURNING_VFX_HEIGHT_FRACTION;
            vfx::systems::spawn_fire_orange_smoke(
                commands,
                visual_assets,
                Vec3::new(center_x, y, center_z),
                radius,
                1,
                t + seed,
            );
        }
    }
    if emit_sparks {
        for i in 0..spark_count {
            let seed = center_z * 0.1 + t + i as f32 * 2.13;
            let y = ((seed * 11.1).sin() * 0.5 + 0.5) * sprite_height * vfx::constants::BURNING_VFX_HEIGHT_FRACTION;
            vfx::systems::spawn_fire_sparks(
                commands,
                visual_assets,
                Vec3::new(center_x, y, center_z),
                2,
                t + seed,
            );
        }
    }
    // Heat shimmer at ground level
    if emit_smoke {
        vfx::systems::spawn_heat_shimmer(
            commands,
            visual_assets,
            Vec3::new(center_x, 0.0, center_z),
            1,
            t,
        );
    }
}
