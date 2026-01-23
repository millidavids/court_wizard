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
            spawn_dotted_circle(
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
            spawn_dotted_circle(
                &mut commands,
                &mut meshes,
                &mut materials,
                wizard_pos,
                circle_radius,
            );
        }
    }
}

/// Rotates the spell range circle over time.
pub fn rotate_spell_range_indicator(
    time: Res<Time>,
    mut circle_query: Query<&mut Transform, With<SpellRangeCircle>>,
) {
    let rotation_delta = ROTATION_SPEED * time.delta_secs();
    for mut transform in circle_query.iter_mut() {
        transform.rotate_y(rotation_delta);
    }
}

/// Spawns a dotted circle made of individual plane meshes.
fn spawn_dotted_circle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center_pos: Vec3,
    radius: f32,
) {
    let material = materials.add(StandardMaterial {
        base_color: RANGE_DOT_COLOR,
        unlit: true,
        ..default()
    });

    let dot_plane = Plane3d::default().mesh().size(DOT_SIZE, DOT_SIZE);
    let dot_mesh = meshes.add(dot_plane);
    let angle_per_dot = std::f32::consts::TAU / NUM_DOTS as f32;

    let parent_id = commands
        .spawn((
            Transform::from_xyz(center_pos.x, 1.0, center_pos.z),
            SpellRangeCircle,
            OnGameplayScreen,
        ))
        .id();

    for i in 0..NUM_DOTS {
        let angle = i as f32 * angle_per_dot;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        let dot_id = commands
            .spawn((
                Mesh3d(dot_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::new(x, 0.0, z)),
                SpellRangeDash,
            ))
            .id();

        commands.entity(parent_id).add_child(dot_id);
    }
}
