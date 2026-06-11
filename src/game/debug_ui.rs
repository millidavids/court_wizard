//! Global F2 toggle that hides/shows developer-only UI affordances
//! (infinite-mana button, +10000 Insight button, hitbox cylinders).
//!
//! Default state is hidden so the trailer/release build look clean even
//! when filming on a debug binary. Pressing F2 in any state flips the
//! flag; consumers react to `DebugUiVisible` via change detection.

use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::game::components::OnGameplayScreen;
#[cfg(debug_assertions)]
use crate::game::units::components::Hitbox;
#[cfg(debug_assertions)]
use crate::state::AppState;

/// When `true`, debug-only UI elements (infinite mana button,
/// +10000 Insight button, hitbox cylinders) are visible.
#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub(crate) struct DebugUiVisible(pub bool);

#[cfg(debug_assertions)]
fn toggle_debug_ui_visible(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<DebugUiVisible>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        visible.0 = !visible.0;
    }
}

/// Generic system that drives the `Visibility` of any entity tagged with
/// marker `M` from the global F2 flag. Register one copy per marker.
#[cfg(debug_assertions)]
pub(crate) fn sync_marker_visibility<M: Component>(
    visible: Res<DebugUiVisible>,
    mut q: Query<&mut Visibility, With<M>>,
) {
    let target = if visible.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut q {
        if *vis != target {
            *vis = target;
        }
    }
}

// --- Debug hitbox visualization (driven by the F2 `DebugUiVisible` flag) ---

/// When present, debug hitbox cylinders are shown.
#[cfg(debug_assertions)]
#[derive(Resource)]
struct DebugHitboxes {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

/// Marker linking a debug cylinder to its parent unit entity.
#[cfg(debug_assertions)]
#[derive(Component)]
struct DebugHitboxMarker(Entity);

#[cfg(debug_assertions)]
fn sync_debug_hitboxes_resource(
    mut commands: Commands,
    visible: Res<DebugUiVisible>,
    existing: Option<Res<DebugHitboxes>>,
    debug_cylinders: Query<Entity, With<DebugHitboxMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if visible.0 && existing.is_none() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        let mesh = meshes.add(Cylinder::new(1.0, 1.0));
        commands.insert_resource(DebugHitboxes { material, mesh });
    } else if !visible.0 && existing.is_some() {
        commands.remove_resource::<DebugHitboxes>();
        for entity in &debug_cylinders {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(debug_assertions)]
fn update_debug_hitboxes(
    mut commands: Commands,
    debug_res: Res<DebugHitboxes>,
    units: Query<(Entity, &Transform, &Hitbox)>,
    mut cylinders: Query<(&DebugHitboxMarker, &mut Transform), Without<Hitbox>>,
    cylinder_entities: Query<(Entity, &DebugHitboxMarker), Without<Hitbox>>,
) {
    // Track which units already have a debug cylinder
    let mut has_cylinder: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (marker, mut cyl_transform) in &mut cylinders {
        if let Ok((_, unit_transform, hitbox)) = units.get(marker.0) {
            has_cylinder.insert(marker.0);
            // Position cylinder centered on the unit's position, scaled to hitbox.
            // Cylinder primitive has half_height=1 (total height=2), so Y scale = height/2.
            cyl_transform.translation = unit_transform.translation;
            cyl_transform.scale = Vec3::new(hitbox.radius, hitbox.height / 2.0, hitbox.radius);
        } else {
            // Unit despawned — will be cleaned up below
        }
    }

    // Remove orphaned debug cylinders (unit no longer exists)
    for (entity, marker) in &cylinder_entities {
        if units.get(marker.0).is_err() {
            commands.entity(entity).try_despawn();
        }
    }

    // Spawn debug cylinders for new units
    for (entity, _transform, _hitbox) in &units {
        if !has_cylinder.contains(&entity) {
            commands.spawn((
                Mesh3d(debug_res.mesh.clone()),
                MeshMaterial3d(debug_res.material.clone()),
                Transform::default(),
                DebugHitboxMarker(entity),
                OnGameplayScreen,
            ));
        }
    }
}

pub(crate) struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(debug_assertions)]
        {
            _app.init_resource::<DebugUiVisible>()
                .add_systems(Update, toggle_debug_ui_visible)
                // Debug hitbox visualization — driven by the global F2 flag.
                .add_systems(
                    Update,
                    (
                        sync_debug_hitboxes_resource,
                        update_debug_hitboxes.run_if(resource_exists::<DebugHitboxes>),
                    )
                        .run_if(in_state(AppState::InGame)),
                );
        }
    }
}
