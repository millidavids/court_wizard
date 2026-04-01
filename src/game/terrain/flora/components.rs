use bevy::prelude::*;

/// Marker component for battlefield flora (flowers/plants).
///
/// Stores the saved ID so trampled flora can be removed from the save data.
#[derive(Component)]
pub struct Flora {
    pub saved_id: u32,
}
