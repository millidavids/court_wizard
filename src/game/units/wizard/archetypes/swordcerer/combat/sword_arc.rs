use super::super::components::*;
use super::super::constants::*;
use super::super::resources::{SwordcererAssets, SwordcererPhase, SwordcererState};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;

/// Handles RT / left-click sword swing from the swordcerer avatar.
///
/// Swings in the avatar's current facing direction (most recent movement vector).
#[allow(clippy::too_many_arguments)]
pub(crate) fn sword_swing(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            &mut crate::game::components::Velocity,
            Option<&SwordcererSwordCooldown>,
            Option<&SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    state: Res<SwordcererState>,
    swordcerer_assets: Res<SwordcererAssets>,
    mut pending: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    if state.phase != SwordcererPhase::OnField {
        return;
    }
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok((avatar_entity, avatar_transform, mut velocity, cooldown, facing)) =
        avatar_query.single_mut()
    else {
        return;
    };

    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let avatar_pos = avatar_transform.translation;

    let direction = facing.copied().unwrap_or_default().0.normalize_or_zero();

    let arc_pos = Vec3::new(avatar_pos.x, 2.0, avatar_pos.z);

    spawn_sword_arc(
        &mut commands,
        &mut meshes,
        &mut materials,
        arc_pos,
        direction,
        false,
    );

    // Replicate the swing to the opponent (visual only — damage crosses via CRDT).
    crate::game::multiplayer::spell_sync::emit_cast_event(
        &mut pending,
        crate::networking::snapshot::CastEventKind::SwordArc,
        0,
        arc_pos,
        [direction.x, direction.y, 0.0, 0.0],
    );

    // Lunge the avatar toward the cursor via velocity impulse
    velocity.x += direction.x * SWORD_LUNGE_SPEED;
    velocity.z += direction.y * SWORD_LUNGE_SPEED;

    // Trigger attack animation
    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_attack(
            swordcerer_assets.attacking_texture.clone(),
            swordcerer_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(SwordcererSwordCooldown {
            remaining: SWORD_COOLDOWN,
        });
}

/// Updates sword arc visuals (rapid grow + fade) and checks collisions with
/// enemies on the first frame. Staging attackers (not yet activated at their
/// rally point) are excluded.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn update_sword_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arc_query: Query<(
        Entity,
        &mut SwordArc,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Has<GhostSwordArc>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<SwordArc>,
            Without<Corpse>,
            Without<SwordcererAvatar>,
            Without<Wizard>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let dt = time.delta_secs();
    let cos_half_angle = SWORD_ARC_HALF_ANGLE.cos();
    for (arc_entity, mut arc, mut arc_transform, mat_handle, is_ghost) in &mut arc_query {
        // Ghost arcs (the opponent's replicated swing) are visual-only.
        let just_spawned = arc.is_added() && !is_ghost;
        arc.time_alive += dt;

        let grow_t = (arc.time_alive / SWORD_ARC_GROW_DURATION).clamp(0.0, 1.0);
        let grow_eased = 1.0 - (1.0 - grow_t).powi(3);
        arc_transform.scale = Vec3::splat(grow_eased.max(SWORD_ARC_MIN_SCALE));

        let fade_t = (arc.time_alive / arc.duration).clamp(0.0, 1.0);
        let alpha = (1.0 - fade_t).powi(2);
        let new_color = Color::srgba(1.0, 1.0, 1.0, alpha);
        if let Some(mat) = materials.get_mut(&mat_handle.0)
            && mat.base_color != new_color
        {
            mat.base_color = new_color;
        }

        // Friendly fire by design — `Without<SwordcererAvatar>` and
        // `Without<Wizard>` filters in the query keep the avatar and the
        // hidden wizard out of the damage set.
        if just_spawned {
            let arc_pos = arc_transform.translation;
            for (target_entity, target_transform, mut health, temp_hp) in &mut targets {
                let diff = target_transform.translation - arc_pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                if dist > SWORD_ARC_RADIUS {
                    continue;
                }
                let target_angle = Vec2::new(diff.x, diff.z).normalize_or_zero();
                if arc.direction.dot(target_angle) > cos_half_angle {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        SWORD_DAMAGE,
                        DamageType::Force,
                        false,
                    );
                    commands.entity(target_entity).insert(
                        crate::game::units::hit_flash::HitFlash {
                            timer: SWORD_HIT_FLASH_DURATION,
                        },
                    );
                }
            }
        }

        if arc.time_alive >= arc.duration {
            commands.entity(arc_entity).try_despawn();
        }
    }
}

/// Builds the curved strip mesh for one sword swing. Per-vertex colors
/// encode the right-to-left alpha gradient (trailing edge transparent,
/// leading edge opaque) — `StandardMaterial` picks up `ATTRIBUTE_COLOR`
/// automatically when present.
fn build_arc_strip_mesh(
    direction: Vec2,
    radius: f32,
    half_angle: f32,
    thickness: f32,
    segments: u32,
) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};

    let base_angle = direction.y.atan2(direction.x);
    let inner_r = (radius - thickness * 0.5).max(0.0);
    let outer_r = radius + thickness * 0.5;

    let n = (segments + 1) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n * 2);

    for i in 0..=segments {
        let frac = i as f32 / segments as f32;
        let angle = base_angle - half_angle + frac * 2.0 * half_angle;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        positions.push([inner_r * cos_a, 0.0, inner_r * sin_a]);
        positions.push([outer_r * cos_a, 0.0, outer_r * sin_a]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([frac, 0.0]);
        uvs.push([frac, 1.0]);

        // `frac.powf(2.5)` gives a fast trailing-edge fall-off so the back
        // of the slash thins out into a fine line rather than a hard cut.
        let alpha = frac.powf(2.5);
        colors.push([1.0, 1.0, 1.0, alpha]);
        colors.push([1.0, 1.0, 1.0, alpha]);
    }

    let mut indices: Vec<u32> = Vec::with_capacity((segments * 6) as usize);
    for i in 0..segments {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        let i2 = (i + 1) * 2;
        let i3 = (i + 1) * 2 + 1;
        indices.push(i0);
        indices.push(i2);
        indices.push(i1);
        indices.push(i1);
        indices.push(i2);
        indices.push(i3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Spawns a sword-swing arc (grow + fade visual). Shared by the local
/// `sword_swing` (`ghost = false`) and the opponent's replicated swing
/// (`ghost = true`, tagged `GhostSwordArc` so it deals no damage).
pub(crate) fn spawn_sword_arc(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    direction: Vec2,
    ghost: bool,
) {
    let arc_mesh = meshes.add(build_arc_strip_mesh(
        direction,
        SWORD_ARC_RADIUS,
        SWORD_ARC_HALF_ANGLE,
        SWORD_ARC_THICKNESS,
        SWORD_ARC_SEGMENTS,
    ));
    // Per-instance material — alpha fades over the arc's lifetime.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mut entity = commands.spawn((
        Mesh3d(arc_mesh),
        MeshMaterial3d(material),
        Transform::from_translation(pos).with_scale(Vec3::splat(SWORD_ARC_MIN_SCALE)),
        SwordArc {
            time_alive: 0.0,
            duration: SWORD_ARC_DURATION,
            direction,
        },
        OnGameplayScreen,
    ));
    if ghost {
        entity.insert(GhostSwordArc);
    }
}
