use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::components::{LocalWizard, Wizard};

/// Spawns the spell range indicator circle when the wizard is created.
pub fn setup_spell_range_indicator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    wizard_query: Query<(&Transform, &Wizard), (Added<Wizard>, With<LocalWizard>)>,
) {
    for (wizard_transform, wizard) in wizard_query.iter() {
        let wizard_pos = wizard_transform.translation;
        let wizard_height = wizard_pos.y;

        // Calculate horizontal reach accounting for wizard's height
        // If wizard is at height H and spell_range is R, the ground circle radius is sqrt(R^2 - H^2)
        const BASE_RADIUS: f32 = 3000.0;
        const BASE_HEIGHT: f32 = 100.0; // Wizard's typical height on castle wall
        let base_ground_radius = (BASE_RADIUS * BASE_RADIUS - BASE_HEIGHT * BASE_HEIGHT).sqrt();

        let actual_ground_radius = if wizard.spell_range > wizard_height {
            (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
        } else {
            0.0 // Spell range is less than height, can't reach ground
        };

        let scale = actual_ground_radius / base_ground_radius;

        spawn_range_circle(
            &mut commands,
            &mut meshes,
            &mut materials,
            wizard_pos,
            base_ground_radius,
            scale,
        );
    }
}

/// Updates the spell range circle scale when the wizard's spell_range changes.
pub fn update_spell_range_indicator(
    wizard_query: Query<(&Transform, &Wizard), (Changed<Wizard>, With<LocalWizard>, Without<SpellRangeCircle>)>,
    mut circle_query: Query<&mut Transform, With<SpellRangeCircle>>,
) {
    // Only update if wizard's spell range changed
    let Some((wizard_transform, wizard)) = wizard_query.iter().next() else {
        return;
    };
    let Some(mut circle_transform) = circle_query.iter_mut().next() else {
        return;
    };

    let wizard_height = wizard_transform.translation.y;

    // Calculate ground radius accounting for height
    const BASE_RADIUS: f32 = 3000.0;
    const BASE_HEIGHT: f32 = 100.0;
    let base_ground_radius = (BASE_RADIUS * BASE_RADIUS - BASE_HEIGHT * BASE_HEIGHT).sqrt();

    let actual_ground_radius = if wizard.spell_range > wizard_height {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    let scale_factor = actual_ground_radius / base_ground_radius;
    circle_transform.scale = Vec3::splat(scale_factor);
}
/// Pulses the opacity of the spell range circle between 10% and 30%.
pub fn pulse_spell_range_indicator(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    circle_query: Query<&MeshMaterial3d<StandardMaterial>, With<SpellRangeCircle>>,
) {
    // Pulse with a 2-second period (1 second fade in, 1 second fade out)
    let pulse_frequency = 0.5; // Hz (cycles per second)
    let alpha = ((time.elapsed_secs() * pulse_frequency * std::f32::consts::TAU).sin() + 1.0) / 2.0;
    let alpha = alpha * 0.2 + 0.1; // Scale to 0.1 - 0.3 range (10% - 30%)

    for material_handle in circle_query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            let mut color = RANGE_DOT_COLOR;
            color.set_alpha(alpha);
            material.base_color = color;
            material.alpha_mode = AlphaMode::Blend;
        }
    }
}

/// Spawns a solid circle ring using a torus mesh.
fn spawn_range_circle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center_pos: Vec3,
    radius: f32,
    initial_scale: f32,
) {
    let material = materials.add(StandardMaterial {
        base_color: RANGE_DOT_COLOR.with_alpha(0.0), // Start at 0% opacity
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Create a torus (donut shape) - a thin ring on the ground
    // major_radius = distance from center to ring center = spell range radius
    // minor_radius = thickness of the ring itself
    let torus = Torus {
        major_radius: radius,
        minor_radius: 2.5, // Thin ring, 5 units wide (half of previous 10)
    };
    let torus_mesh = meshes.add(torus);

    commands.spawn((
        Mesh3d(torus_mesh),
        MeshMaterial3d(material),
        // Torus is oriented around Y-axis by default, which is vertical
        // We want it flat on the ground (XZ plane), so no rotation needed
        Transform::from_xyz(center_pos.x, 5.0, center_pos.z).with_scale(Vec3::splat(initial_scale)), // Apply initial scale
        SpellRangeCircle,
        OnGameplayScreen,
    ));
}
