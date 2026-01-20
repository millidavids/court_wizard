use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;
use crate::game::units::components::{Health, Hitbox, MovementSpeed};

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as a triangle on the castle platform in 3D space.
pub fn setup_wizard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Define wizard hitbox (cylinder) - this determines sprite size
    let hitbox = Hitbox::new(30.0, 60.0); // radius: 30, height: 60

    // Spawn wizard as a triangle billboard sized to match the hitbox
    let wizard_width = hitbox.sprite_width();
    let wizard_height = hitbox.sprite_height();
    let wizard_triangle = Triangle2d::new(
        Vec2::new(0.0, wizard_height / 2.0), // Top vertex
        Vec2::new(-wizard_width / 2.0, -wizard_height / 2.0), // Bottom-left
        Vec2::new(wizard_width / 2.0, -wizard_height / 2.0), // Bottom-right
    );

    commands.spawn((
        Mesh3d(meshes.add(wizard_triangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WIZARD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(WIZARD_POSITION),
        hitbox,
        Health::new(100.0),
        MovementSpeed::new(0.0), // Wizard doesn't move
        Wizard,
        OnGameplayScreen,
    ));
}
