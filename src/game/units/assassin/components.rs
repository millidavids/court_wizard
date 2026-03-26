use bevy::prelude::*;

/// Marker component for assassin units.
///
/// Assassins are fast flanking attackers that route around infantry
/// to specifically target archers. They take reduced damage from archers
/// but increased damage from infantry.
#[derive(Component)]
pub struct Assassin;
