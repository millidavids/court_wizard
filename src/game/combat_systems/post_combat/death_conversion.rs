use bevy::prelude::*;

use super::super::super::units::components::{Corpse, Health, Invulnerable};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn enforce_invulnerability(
    mut query: Query<(&mut Invulnerable, &mut Health), Without<Corpse>>,
) {
    for (mut invuln, mut health) in &mut query {
        // Restore health to at least the snapshot (damage negated, heals preserved)
        health.current = health.current.max(invuln.health_snapshot).min(health.max);
        invuln.health_snapshot = health.current;
    }
}
