use bevy::prelude::*;

use super::super::components::{
    FireDoT, FrostAccumulation, FrozenSolidModifier, Health, Hitbox, RemoteFireEffect,
    SlowMovementModifier, TemporaryHitPoints, apply_damage_to_unit,
};
use super::super::king::components::SpellShield;
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;
use crate::game::units::wizard::archetypes::meteorologist::constants::WET_FIRE_DOT_MULTIPLIER;

/// Updates frost accumulation: decays over time, applies proportional slow,
/// and triggers freeze at max level.
pub fn update_frost_accumulation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut FrostAccumulation,
        Option<&mut SlowMovementModifier>,
        Has<FireDoT>,
    )>,
    storms: Query<&crate::game::units::wizard::spells::squall::components::SquallStorm>,
) {
    use crate::game::units::wizard::spells::squall::constants::{
        FROST_DECAY_RATE, FROST_FREEZE_DURATION, FROST_MAX_SLOW, PERMAFROST_FREEZE_DURATION,
    };

    /// Rate at which fire melts frost accumulation (per second).
    const FIRE_MELTS_FROST_RATE: f32 = 0.3;

    let delta = time.delta_secs();
    let has_permafrost = storms.iter().any(|s| s.talent_params.permafrost);
    let freeze_duration = if has_permafrost {
        PERMAFROST_FREEZE_DURATION
    } else {
        FROST_FREEZE_DURATION
    };

    for (entity, mut frost, slow_mod, has_fire) in &mut query {
        // Fire melts frost
        if has_fire {
            frost.level -= FIRE_MELTS_FROST_RATE * delta;
        }

        // Decay logic
        frost.decay_delay -= delta;
        if frost.decay_delay <= 0.0 {
            frost.level -= FROST_DECAY_RATE * delta;
        }

        // Remove if fully thawed
        if frost.level <= 0.0 {
            commands
                .entity(entity)
                .remove::<FrostAccumulation>()
                .remove::<SlowMovementModifier>();
            continue;
        }

        // Freeze at max
        if frost.level >= 1.0 {
            commands
                .entity(entity)
                .insert(FrozenSolidModifier::new(freeze_duration))
                .remove::<FrostAccumulation>()
                .remove::<SlowMovementModifier>();
            continue;
        }

        // Update slow proportionally (refreshed each frame)
        let slow_amount = frost.level * FROST_MAX_SLOW;
        if let Some(mut slow) = slow_mod {
            slow.modifier = slow_amount;
            slow.time_remaining = 0.5; // short refresh window
        } else {
            commands
                .entity(entity)
                .insert(SlowMovementModifier::new(slow_amount, 0.5));
        }
    }
}

/// Ticks FireDoT damage on affected units and removes expired DoTs.
///
/// DoT damage is applied directly to health (does not trigger more DoT).
pub fn update_fire_dot(
    mut commands: Commands,
    time: Res<Time>,
    // Skip multiplayer ghost units — the host owns their authoritative
    // FireDoT and ticks damage there; CRDT propagates the resulting HP
    // back to the ghost. Ticking it locally would double-apply.
    mut query: Query<
        (
            Entity,
            &mut FireDoT,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Has<WetModifier>,
            Option<&FrostAccumulation>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
) {
    /// Rate at which frost quenches fire DPS (DPS reduction per second per frost level).
    const FROST_QUENCHES_FIRE_RATE: f32 = 5.0;

    let delta = time.delta_secs();

    for (entity, mut fire_dot, mut health, temp_hp, has_shield, is_wet, frost) in query.iter_mut() {
        // Frost quenches fire — reduce DPS proportional to frost level
        if let Some(frost) = frost {
            fire_dot.damage_per_tick = (fire_dot.damage_per_tick
                - FROST_QUENCHES_FIRE_RATE * frost.level * delta)
                .max(0.0);
        }

        let (tick_damage, expired) = fire_dot.update(delta);

        if let Some(damage) = tick_damage
            && !has_shield
        {
            // Storm synergy: fire DoT damage is reduced on wet units
            let effective_damage = if is_wet {
                damage * WET_FIRE_DOT_MULTIPLIER
            } else {
                damage
            };
            apply_damage_to_unit(
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                effective_damage,
            );
        }

        if expired {
            commands.entity(entity).remove::<FireDoT>();
        }
    }
}

/// Spawn fire VFX (smoke, sparks, embers) on units with an active FireDoT.
pub fn emit_burning_unit_vfx(
    mut commands: Commands,
    // Fire smoke/sparks emit for any unit currently flagged as burning —
    // host-simulated units carry the real `FireDoT`, and guest-rendered
    // ghost units carry `RemoteFireEffect` (mirrored from the host's
    // snapshot). Including both lets the burning VFX play on the guest
    // for opposing units the same way SP shows it.
    burning_units: Query<(&Transform, &Hitbox), Or<(With<FireDoT>, With<RemoteFireEffect>)>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    time: Res<Time>,
    mut smoke_timer: Local<f32>,
    mut spark_timer: Local<f32>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    *smoke_timer += delta;
    *spark_timer += delta;

    let emit_smoke = *smoke_timer >= 0.5;
    let emit_sparks = *spark_timer >= 1.5;

    if emit_smoke {
        *smoke_timer -= 0.5;
    }
    if emit_sparks {
        *spark_timer -= 1.5;
    }

    if !emit_smoke && !emit_sparks {
        return;
    }

    for (transform, hitbox) in &burning_units {
        let pos = transform.translation;
        let half_height = hitbox.height * 0.5;
        let radius = hitbox.radius * 0.5;

        if emit_smoke {
            let seed = pos.x * 0.1 + t;
            let y_offset = ((seed * 7.3).sin() * 0.5) * half_height;
            crate::game::units::wizard::spells::vfx::systems::spawn_fire_orange_smoke(
                &mut commands,
                &visual_assets,
                Vec3::new(pos.x, pos.y + y_offset, pos.z),
                radius,
                1,
                t + seed,
            );
        }
        if emit_sparks {
            let seed = pos.z * 0.1 + t;
            let y_offset = ((seed * 11.1).sin() * 0.5) * half_height;
            crate::game::units::wizard::spells::vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                Vec3::new(pos.x, pos.y + y_offset, pos.z),
                1,
                t + seed,
            );
        }
    }
}
