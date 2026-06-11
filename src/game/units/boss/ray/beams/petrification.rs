use bevy::prelude::*;

use super::super::constants::PETRIFY_KING_DAMAGE_PER_SECOND;
use crate::game::units::components::Petrified;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::King;

// ===== Petrified Unit Effects =====

pub fn update_petrified_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<crate::game::units::king::components::SpellShield>,
        ),
        (With<Petrified>, With<King>),
    >,
) {
    let delta = time.delta_secs();
    let damage = PETRIFY_KING_DAMAGE_PER_SECOND * delta;

    for (entity, mut health, temp_hp, has_shield) in &mut query {
        apply_spell_damage(
            &mut commands,
            entity,
            &mut health,
            temp_hp.map(|t| t.into_inner()),
            damage,
            DamageType::Force,
            has_shield,
        );
    }
}
