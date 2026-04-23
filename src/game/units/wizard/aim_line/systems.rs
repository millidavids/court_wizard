use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::spells::utils::{
    clamp_to_spell_range_ground, get_cursor_world_position,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// World-unit thickness of the aim line (Y and Z axes of the stretched cuboid).
const LINE_THICKNESS: f32 = 4.0;

const AIM_LINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.09);

/// Marker component for the aim line entity.
#[derive(Component)]
pub(super) struct AimLine;

/// Spawns a single aim-line entity for each local wizard. Starts hidden;
/// [`update_aim_line`] toggles visibility based on active input device.
pub(super) fn spawn_aim_line(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    existing: Query<(), With<AimLine>>,
    wizard_query: Query<(), (Added<Wizard>, With<LocalWizard>)>,
) {
    if !existing.is_empty() {
        return;
    }
    if wizard_query.is_empty() {
        return;
    }
    let material = materials.add(StandardMaterial {
        base_color: AIM_LINE_COLOR,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(visual_assets.unit_cuboid.clone()),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Hidden,
        AimLine,
        OnGameplayScreen,
    ));
}

/// Updates the aim line's transform to stretch from the wizard's staff tip
/// to the current ground-plane aim point. Hides the line when the active
/// input device is mouse/keyboard.
pub(super) fn update_aim_line(
    active: Res<ActiveInputDevice>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    wizard_query: Query<(&Transform, &Wizard), (With<LocalWizard>, Without<AimLine>)>,
    mut line_query: Query<(&mut Transform, &mut Visibility), With<AimLine>>,
) {
    let Ok((mut line_transform, mut line_visibility)) = line_query.single_mut() else {
        return;
    };

    if !active.is_gamepad() {
        line_visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    let Ok((wizard_transform, wizard)) = wizard_query.single() else {
        line_visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    let Some(cursor_world) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        line_visibility.set_if_neq(Visibility::Hidden);
        return;
    };

    let clamped = clamp_to_spell_range_ground(
        cursor_world,
        wizard_transform.translation,
        wizard.spell_range,
        0.0,
    );
    // Spell-spawn point — same origin every projectile/beam fires from.
    let origin = SPELL_ORIGIN;

    let diff = clamped - origin;
    let length = diff.length();
    if length < 0.001 {
        line_visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    let dir = diff / length;
    let mid = (origin + clamped) * 0.5;

    line_transform.translation = mid;
    line_transform.rotation = Quat::from_rotation_arc(Vec3::X, dir);
    line_transform.scale = Vec3::new(length, LINE_THICKNESS, LINE_THICKNESS);
    line_visibility.set_if_neq(Visibility::Inherited);
}
