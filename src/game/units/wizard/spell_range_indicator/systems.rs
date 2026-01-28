use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::components::Wizard;

/// Spawns the spell range indicator circle when the wizard is created.
pub fn setup_spell_range_indicator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    wizard_query: Query<(&Transform, &Wizard), Added<Wizard>>,
) {
    for (wizard_transform, wizard) in wizard_query.iter() {
        let wizard_pos = wizard_transform.translation;
        let wizard_height = wizard_pos.y;
        let spell_range = wizard.spell_range;

        if wizard_height < spell_range {
            let circle_radius = (spell_range * spell_range - wizard_height * wizard_height).sqrt();
            spawn_range_circle(
                &mut commands,
                &mut meshes,
                &mut materials,
                wizard_pos,
                circle_radius,
            );
        }
    }
}

/// Updates the spell range circle when the wizard's spell_range changes.
pub fn update_spell_range_indicator(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    wizard_query: Query<(&Transform, &Wizard), (Changed<Wizard>, Without<SpellRangeCircle>)>,
    circle_query: Query<Entity, With<SpellRangeCircle>>,
) {
    for (wizard_transform, wizard) in wizard_query.iter() {
        let wizard_pos = wizard_transform.translation;
        let wizard_height = wizard_pos.y;
        let spell_range = wizard.spell_range;

        for entity in circle_query.iter() {
            commands.entity(entity).despawn();
        }

        if wizard_height < spell_range {
            let circle_radius = (spell_range * spell_range - wizard_height * wizard_height).sqrt();
            spawn_range_circle(
                &mut commands,
                &mut meshes,
                &mut materials,
                wizard_pos,
                circle_radius,
            );
        }
    }
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
        Transform::from_xyz(center_pos.x, 1.0, center_pos.z),
        SpellRangeCircle,
        OnGameplayScreen,
    ));
}
