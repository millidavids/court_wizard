use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

use super::super::units::components::{Corpse, Flying, HasShadow, Hitbox, Team, UnitShadow};

// --- Shadow material ---

/// Material for ground shadows that prevents overlapping shadows from compounding.
///
/// Uses depth writing with strict `Greater` comparison so the first shadow drawn
/// at a pixel claims the depth — subsequent shadows at the same depth fail the
/// test and are discarded. This ensures overlapping shadows produce a single
/// uniform darkened area instead of stacking.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct ShadowMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for ShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shadow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Enable depth writing so the first shadow at a pixel "claims" it.
        // Use strict Greater (not GreaterEqual) so shadows at the same depth
        // don't stack — only the first one drawn passes.
        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.depth_write_enabled = true;
            depth_stencil.depth_compare = CompareFunction::Greater;
        }
        Ok(())
    }
}

// --- Unit shadow system ---

/// Shared shadow mesh and material for all units.
#[derive(Resource)]
pub struct ShadowAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<ShadowMaterial>,
}

/// Base shadow circle radius (half of sprite width / 2).
const SHADOW_BASE_RADIUS: f32 = super::super::units::constants::DEFAULT_SPRITE_WIDTH / 4.0;
/// Extra scale factor for flying unit shadows (simulates height spreading the shadow).
const FLYING_SHADOW_HEIGHT_SCALE: f32 = 1.3;
/// Y position of the shadow (just above ground to avoid z-fighting).
const SHADOW_Y: f32 = 1.5;

pub fn preload_shadow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShadowMaterial>>,
) {
    let mesh = meshes.add(Circle::new(SHADOW_BASE_RADIUS));
    let material = materials.add(ShadowMaterial {
        color: LinearRgba::new(0.0, 0.0, 0.0, 0.35),
    });
    commands.insert_resource(ShadowAssets { mesh, material });
}

/// Spawns a static ground shadow at the given XZ position.
/// Used by terrain objects (flora, trees, bushes, boulders).
pub fn spawn_terrain_shadow(
    commands: &mut Commands,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
) {
    commands.spawn((
        Mesh3d(shadow_assets.mesh.clone()),
        MeshMaterial3d(shadow_assets.material.clone()),
        Transform::from_xyz(x, SHADOW_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(scale)),
        super::super::components::OnGameplayScreen,
    ));
}

/// Spawns shadow entities for any unit that doesn't have one yet.
/// Shadow scale is derived from the unit's hitbox radius so bosses (which use
/// large meshes rather than large transform scales) get proportionally larger shadows.
/// Flying units get slightly larger shadows to simulate height spreading.
pub fn spawn_unit_shadows(
    mut commands: Commands,
    shadow_assets: Res<ShadowAssets>,
    units: Query<
        (Entity, &Transform, &Hitbox, Has<Flying>),
        (With<Team>, Without<HasShadow>, Without<Corpse>),
    >,
) {
    /// Default hitbox radius for regular infantry units.
    const BASE_HITBOX_RADIUS: f32 = 8.0 * crate::game::constants::UNIT_SCALE;

    for (entity, transform, hitbox, is_flying) in &units {
        // Scale shadow proportionally to hitbox radius relative to a standard infantry unit.
        // Use the larger of transform scale or hitbox ratio to avoid double-counting
        // (e.g. brute has both large scale AND large hitbox for the same size increase).
        let hitbox_ratio = hitbox.radius / BASE_HITBOX_RADIUS;
        let scale_factor = transform.scale.x.max(transform.scale.z);
        let mut shadow_scale = hitbox_ratio.max(scale_factor);
        if is_flying {
            shadow_scale *= FLYING_SHADOW_HEIGHT_SCALE;
        }
        commands.spawn((
            Mesh3d(shadow_assets.mesh.clone()),
            MeshMaterial3d(shadow_assets.material.clone()),
            Transform::from_xyz(transform.translation.x, SHADOW_Y, transform.translation.z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(shadow_scale)),
            super::super::components::OnGameplayScreen,
            UnitShadow { owner: entity },
        ));
        commands.entity(entity).insert(HasShadow);
    }
}

/// Syncs shadow positions to their owning unit's XZ, always at ground level.
/// Despawns shadows whose owner no longer exists or is a corpse.
pub fn update_unit_shadows(
    mut commands: Commands,
    mut shadows: Query<(Entity, &UnitShadow, &mut Transform)>,
    units: Query<(&Transform, Has<Corpse>), (With<Team>, Without<UnitShadow>)>,
) {
    for (shadow_entity, shadow, mut shadow_transform) in &mut shadows {
        if let Ok((unit_transform, is_corpse)) = units.get(shadow.owner) {
            if is_corpse {
                commands.entity(shadow_entity).try_despawn();
                continue;
            }
            shadow_transform.translation.x = unit_transform.translation.x;
            shadow_transform.translation.z = unit_transform.translation.z;
            shadow_transform.translation.y = SHADOW_Y;
        } else {
            commands.entity(shadow_entity).try_despawn();
        }
    }
}
