use super::super::super::super::components::Spell;
use super::super::constants;
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::messages::IngredientCollectedMessage;
use crate::game::units::wizard::spells::telekinesis::components::TransmutationStacks;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Computed talent configuration for a single Telekinesis cast.
pub(super) struct TelekinesisConfig {
    pub(super) pickup_radius: f32,
    pub(super) cast_time: f32,
    pub(super) mana_cost: f32,
    pub(super) is_storm: bool,
    pub(super) has_harvest: bool,
    pub(super) has_shockwave: bool,
}

/// Builds a TelekinesisConfig from the active talent selections.
pub(super) fn compute_telekinesis_config(
    active_talents: Option<&ActiveTalents>,
) -> TelekinesisConfig {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 2));

    // T1: Auto-Target removes the pickup radius constraint
    let pickup_radius = match t1 {
        Some(0) => f32::MAX,
        _ => constants::PICKUP_RADIUS,
    };

    let cast_time = match t1 {
        Some(1) => constants::QUICK_GRAB_CAST_TIME,
        _ => constants::CAST_TIME,
    };

    let mana_cost = match t1 {
        Some(2) => constants::MANA_COST * constants::MANA_EFFICIENCY_COST_MULT,
        _ => constants::MANA_COST,
    };

    let has_harvest = t2 == Some(1);

    let is_storm = t3 == Some(0);
    let has_shockwave = t3 == Some(2);

    TelekinesisConfig {
        pickup_radius,
        cast_time,
        mana_cost,
        is_storm,
        has_harvest,
        has_shockwave,
    }
}

/// T2: Magnetic Pull — passively drifts ingredient drops toward the wizard.
pub(crate) fn magnetic_pull_ingredients(
    time: Res<Time>,
    mut drops: Query<&mut Transform, (With<IngredientDrop>, Without<FlyingToWizard>)>,
) {
    let wizard_pos = crate::game::constants::WIZARD_POSITION;
    let pull_radius_sq = constants::MAGNETIC_PULL_RADIUS * constants::MAGNETIC_PULL_RADIUS;

    for mut transform in drops.iter_mut() {
        let diff = wizard_pos - transform.translation;
        let dist_sq = diff.x * diff.x + diff.z * diff.z;

        if dist_sq <= pull_radius_sq && dist_sq > 1.0 {
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize();
            let move_dist = constants::MAGNETIC_PULL_SPEED * time.delta_secs();
            transform.translation.x += direction.x * move_dist;
            transform.translation.z += direction.z * move_dist;
        }
    }
}

/// T3: Transmutation — increments stacks when ingredients are collected.
pub(crate) fn track_transmutation_stacks(
    mut collected: MessageReader<IngredientCollectedMessage>,
    mut stacks: ResMut<TransmutationStacks>,
) {
    for _ in collected.read() {
        stacks.count += 1;
    }
}

pub(crate) fn init_transmutation_stacks(mut commands: Commands) {
    commands.init_resource::<TransmutationStacks>();
}

pub(crate) fn cleanup_transmutation_stacks(mut commands: Commands) {
    commands.remove_resource::<TransmutationStacks>();
}
