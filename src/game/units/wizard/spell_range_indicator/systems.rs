use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::spells::utils::ground_projected_range;

/// Reference dimensions the spell-range circle mesh is authored at; the live
/// circle is scaled from this base to the wizard's actual spell range.
const BASE_RADIUS: f32 = 3000.0;
const BASE_HEIGHT: f32 = 100.0;

/// Spawns the spell range indicator circle when the wizard is created.
pub fn setup_spell_range_indicator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    wizard_query: Query<(&Transform, &Wizard), (Added<Wizard>, With<LocalWizard>)>,
) {
    for (wizard_transform, wizard) in wizard_query.iter() {
        let wizard_pos = wizard_transform.translation;

        let base_ground_radius = ground_projected_range(BASE_RADIUS, BASE_HEIGHT);
        let actual_ground_radius = ground_projected_range(wizard.spell_range, wizard_pos.y);

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
    wizard_query: Query<
        (&Transform, &Wizard),
        (
            Changed<Wizard>,
            With<LocalWizard>,
            Without<SpellRangeCircle>,
        ),
    >,
    mut circle_query: Query<&mut Transform, With<SpellRangeCircle>>,
) {
    // Only update if wizard's spell range changed
    let Ok((wizard_transform, wizard)) = wizard_query.single() else {
        return;
    };
    let Ok(mut circle_transform) = circle_query.single_mut() else {
        return;
    };

    let base_ground_radius = ground_projected_range(BASE_RADIUS, BASE_HEIGHT);
    let actual_ground_radius =
        ground_projected_range(wizard.spell_range, wizard_transform.translation.y);

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
            // alpha_mode is set to AlphaMode::Blend at spawn and never changes
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
        depth_bias: -100.0,
        ..default()
    });

    // Create a torus (donut shape) - a thin ring on the ground
    // major_radius = distance from center to ring center = spell range radius
    // minor_radius = thickness of the ring itself
    let torus = Torus {
        major_radius: radius,
        minor_radius: 2.5, // Thin ring, 5 units wide
    };
    let torus_mesh = meshes.add(torus);

    commands.spawn((
        Mesh3d(torus_mesh),
        MeshMaterial3d(material),
        // Torus is oriented around Y-axis by default, which is vertical
        // We want it flat on the ground (XZ plane), so no rotation needed
        Transform::from_xyz(center_pos.x, 1.0, center_pos.z).with_scale(Vec3::splat(initial_scale)),
        SpellRangeCircle,
        OnGameplayScreen,
    ));
}
