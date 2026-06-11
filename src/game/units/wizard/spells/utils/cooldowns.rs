use bevy::prelude::*;

/// Trait for cooldown-style components with a `remaining: f32` countdown field.
///
/// Implementing this trait lets a component be ticked by the generic
/// [`tick_spell_cooldown`] system, eliminating the need for a per-cooldown
/// tick function.
pub(crate) trait HasCooldownRemaining {
    fn remaining_mut(&mut self) -> &mut f32;
}

/// Generic per-frame countdown system. Decrements the `remaining` field of
/// every entity's cooldown component, and removes the component when it expires.
///
/// Register one instance per cooldown type, e.g.:
/// `app.add_systems(Update, tick_spell_cooldown::<MagicMissileCooldown>)`.
pub(crate) fn tick_spell_cooldown<C>(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut C)>,
) where
    C: Component<Mutability = bevy::ecs::component::Mutable> + HasCooldownRemaining,
{
    for (entity, mut cd) in &mut cooldowns {
        let remaining = cd.remaining_mut();
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            commands.entity(entity).remove::<C>();
        }
    }
}

/// Ticks `GlobalCastCooldown` down each frame, removing the component when
/// it expires so spell input is unblocked.
pub(crate) fn tick_global_cast_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut crate::game::units::wizard::components::GlobalCastCooldown,
    )>,
) {
    for (entity, mut cd) in &mut query {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<crate::game::units::wizard::components::GlobalCastCooldown>();
        }
    }
}

/// Detects charge-spell completion (CastingState just went from `Casting`
/// to `Resting` AND mana decreased on the same frame) and inserts a global
/// cooldown on the local wizard. Channel-release returns `Channeling` →
/// `Resting` (not `Casting` → `Resting`) so channeled spells don't trigger
/// the cooldown. Cancellation (release before cast_time) doesn't consume
/// mana, so the mana check also excludes that case.
pub(crate) fn insert_global_cooldown_on_cast(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &crate::game::units::wizard::components::CastingState,
            &crate::game::units::wizard::components::Mana,
        ),
        With<crate::game::units::wizard::components::LocalWizard>,
    >,
    mut prev_state: Local<Option<crate::game::units::wizard::components::CastingState>>,
    mut prev_mana: Local<Option<f32>>,
) {
    let Ok((entity, state, mana)) = query.single() else {
        return;
    };
    let mana_decreased = prev_mana.is_some_and(|p| mana.current < p - f32::EPSILON);
    let was_casting = matches!(
        *prev_state,
        Some(crate::game::units::wizard::components::CastingState::Casting { .. })
    );
    let now_resting = matches!(
        state,
        crate::game::units::wizard::components::CastingState::Resting
    );
    if was_casting && now_resting && mana_decreased {
        commands.entity(entity).insert(
            crate::game::units::wizard::components::GlobalCastCooldown {
                remaining: crate::game::units::wizard::components::GLOBAL_CAST_COOLDOWN_SECS,
            },
        );
    }
    *prev_state = Some(*state);
    *prev_mana = Some(mana.current);
}

/// Writes `BlockSpellInput` whenever the local wizard has a
/// `GlobalCastCooldown`, so every spell's `spell_input_not_blocked`
/// run_if rejects while the cooldown is active.
pub(crate) fn block_spells_during_global_cooldown(
    query: Query<
        (),
        (
            With<crate::game::units::wizard::components::LocalWizard>,
            With<crate::game::units::wizard::components::GlobalCastCooldown>,
        ),
    >,
    mut block: MessageWriter<crate::game::input::messages::BlockSpellInput>,
) {
    if !query.is_empty() {
        block.write(crate::game::input::messages::BlockSpellInput);
    }
}
