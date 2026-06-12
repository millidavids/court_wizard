use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;

use super::marker::StatusEffectForwarded;

/// Generic status-effect forwarder. Polls every frame for status components
/// freshly added to ghost units (from the guest's local spell casts), ships
/// `ApplyStatusEffect` messages with the relevant duration/magnitude/flags,
/// then tags each forwarded ghost-component with `StatusEffectForwarded` so
/// we don't ship a duplicate message next frame.
///
/// The local component is intentionally LEFT on the ghost — local visual
/// systems (sleep Z, polymorph swap, mind-control tint, banish visibility)
/// read it for immediate guest-side feedback while the host catches up. The
/// host snapshot will overwrite ghost position/material on subsequent ticks.
///
/// Adding a new status spell? Add the relevant component query arm here AND
/// a matching arm to `apply_status_to_entity` in `host_systems.rs`. The
/// spell's casting code stays unchanged.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn forward_status_effects_to_host(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    // Each status component is queried separately with Added<>; a ghost may
    // have multiple statuses applied in the same frame.
    sleep: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::SleepModifier,
            // Sleep talent companion components — packed into the
            // ApplyStatusEffect.flags so the host re-creates them.
            Has<crate::game::units::status_effects::NightTerrors>,
            Has<crate::game::units::status_effects::Comatose>,
            Has<crate::game::units::status_effects::NarcolepticWave>,
            Has<crate::game::units::status_effects::Sleepwalking>,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::SleepModifier>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::SleepModifier>>,
        ),
    >,
    root: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::RootedModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            // Poll (not `Added<>`): a `Commands`-queued insert from the entangle
            // cast isn't visible to `Added` until after the command flush, so
            // `Added` could miss the root entirely (this caused guest entangle's
            // "vines but no root"). The `StatusEffectForwarded` marker is queued
            // the same frame we forward and flushes before the next poll, so this
            // stays single-shot. Mirrors `forward_spell_hits_to_host`.
            Without<StatusEffectForwarded<crate::game::units::status_effects::RootedModifier>>,
        ),
    >,
    mind: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::MindControlled,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            // Poll (not `Added<>`) — see the `root` arm; keeps guest-cast mind
            // control reliable even when the insert lands via deferred commands.
            Without<StatusEffectForwarded<crate::game::units::components::MindControlled>>,
        ),
    >,
    banish: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::BanishedModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::BanishedModifier>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::BanishedModifier>>,
        ),
    >,
    mark: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::MarkedForDeathModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::MarkedForDeathModifier>,
            Without<
                StatusEffectForwarded<crate::game::units::status_effects::MarkedForDeathModifier>,
            >,
        ),
    >,
    haste: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::HasteModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::HasteModifier>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::HasteModifier>>,
        ),
    >,
    battle_hymn: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::BattleHymnModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::BattleHymnModifier>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::BattleHymnModifier>>,
        ),
    >,
    berserker: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::BerserkerRageModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::BerserkerRageModifier>,
            Without<
                StatusEffectForwarded<crate::game::units::status_effects::BerserkerRageModifier>,
            >,
        ),
    >,
    temp_hp: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::TemporaryHitPoints,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::components::TemporaryHitPoints>,
            Without<StatusEffectForwarded<crate::game::units::components::TemporaryHitPoints>>,
        ),
    >,
    slow: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::SlowMovementModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::components::SlowMovementModifier>,
            Without<StatusEffectForwarded<crate::game::units::components::SlowMovementModifier>>,
        ),
    >,
    stun: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::Stunned,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::Stunned>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::Stunned>>,
        ),
    >,
    fog_evasion: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::FogEvasionModifier,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            Added<crate::game::units::status_effects::FogEvasionModifier>,
            Without<StatusEffectForwarded<crate::game::units::status_effects::FogEvasionModifier>>,
        ),
    >,
    polymorphed: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::status_effects::PolymorphedModifier,
            // Talent markers the guest applied locally — packed into the flags so
            // the host re-creates them (otherwise guest-cast Explosive/Contagious/
            // Pig/Permanent/Dire sheep behave as a vanilla sheep on the host).
            Has<crate::game::units::wizard::spells::polymorph::components::ExplosiveSheep>,
            Option<&crate::game::units::wizard::spells::polymorph::components::ContagiousBaas>,
            Has<crate::game::units::wizard::spells::polymorph::components::PigForm>,
            Has<crate::game::units::wizard::spells::polymorph::components::PermanentLivestock>,
            Has<crate::game::units::wizard::spells::polymorph::components::DireSheep>,
        ),
        (
            With<super::super::super::components::GhostEntity>,
            // Poll (not `Added<>`) — see the `root` arm; keeps guest-cast
            // polymorph reliable even when the insert lands via deferred commands.
            Without<StatusEffectForwarded<crate::game::units::status_effects::PolymorphedModifier>>,
        ),
    >,
    // Polled (not `Added<>`): chain-lightning / telekinesis / meteor insert
    // Knockback via deferred Commands, which `Added<>` can miss (same reason the
    // root/mind/polymorph arms poll). The loop below removes the component after
    // forwarding instead of tagging `StatusEffectForwarded` — see there for why.
    knockback: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::Knockback,
        ),
        With<super::super::super::components::GhostEntity>,
    >,
) {
    use crate::networking::protocol::{NetworkMessage, StatusEffectKind};

    let push = |connection: &mut NetworkConnection,
                net_id: u32,
                kind: StatusEffectKind,
                duration: f32,
                magnitude: f32,
                flags: u32| {
        connection
            .outgoing_messages
            .push(NetworkMessage::ApplyStatusEffect {
                target_network_id: net_id,
                kind: kind.to_u8(),
                duration,
                magnitude,
                flags,
            });
    };

    for (e, id, m, has_terrors, has_coma, has_wave, has_walk) in &sleep {
        use crate::networking::protocol::status_flags as sf;
        let mut flags: u32 = 0;
        if has_terrors {
            flags |= sf::SLEEP_NIGHT_TERRORS;
        }
        if has_coma {
            flags |= sf::SLEEP_COMATOSE;
        }
        if has_wave {
            flags |= sf::SLEEP_NARCOLEPTIC_WAVE;
        }
        if has_walk {
            flags |= sf::SLEEP_DREAMWALKER;
        }
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Sleep,
            m.time_remaining,
            m.bonus_damage_multiplier,
            flags,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::SleepModifier,
        >::default());
    }
    for (e, id, m) in &root {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Root,
            m.time_remaining,
            0.0,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::RootedModifier,
        >::default());
    }
    for (e, id, m) in &mind {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::MindControl,
            m.wear_off_duration,
            m.damage_multiplier,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::components::MindControlled,
        >::default());
    }
    for (e, id, m) in &banish {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Banish,
            m.time_remaining,
            0.0,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::BanishedModifier,
        >::default());
    }
    for (e, id, m) in &mark {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Mark,
            m.time_remaining,
            m.damage_amplification,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::MarkedForDeathModifier,
        >::default());
    }
    for (e, id, m) in &haste {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Haste,
            m.time_remaining,
            m.modifier,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::HasteModifier,
        >::default());
    }
    for (e, id, m) in &battle_hymn {
        let attack_speed_bits = (m.attack_speed.clamp(0.0, 6.5535) * 10_000.0) as u32;
        push(
            &mut connection,
            id.0,
            StatusEffectKind::BattleHymn,
            m.time_remaining,
            m.damage_bonus,
            attack_speed_bits & 0xFFFF,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::BattleHymnModifier,
        >::default());
    }
    for (e, id, m) in &berserker {
        let vuln_bits = (m.damage_vulnerability.clamp(0.0, 6.5535) * 10_000.0) as u32;
        push(
            &mut connection,
            id.0,
            StatusEffectKind::BerserkerRage,
            m.time_remaining,
            m.damage_bonus,
            vuln_bits & 0xFFFF,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::BerserkerRageModifier,
        >::default());
    }
    for (e, id, m) in &temp_hp {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::GuardianTempHp,
            m.time_remaining,
            m.amount,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::components::TemporaryHitPoints,
        >::default());
    }
    for (e, id, m) in &slow {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Slow,
            m.time_remaining,
            m.modifier,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::components::SlowMovementModifier,
        >::default());
    }
    for (e, id, m) in &stun {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Stun,
            m.time_remaining,
            0.0,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::Stunned,
        >::default());
    }
    for (e, id, m) in &fog_evasion {
        push(
            &mut connection,
            id.0,
            StatusEffectKind::FogEvasion,
            m.time_remaining,
            m.evasion_chance,
            0,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::FogEvasionModifier,
        >::default());
    }
    for (e, id, m, explosive, contagious, pig, permanent, dire) in &polymorphed {
        use crate::networking::protocol::status_flags as sf;
        let mut flags: u32 = 0;
        if explosive {
            flags |= sf::POLYMORPH_EXPLOSIVE;
        }
        if contagious.is_some() {
            flags |= sf::POLYMORPH_CONTAGIOUS;
        }
        if pig {
            flags |= sf::POLYMORPH_PIG;
        }
        if permanent {
            flags |= sf::POLYMORPH_PERMANENT;
        }
        if dire {
            flags |= sf::POLYMORPH_DIRE;
        }
        // Empowerment (used by the Contagious spread) rides in `magnitude`.
        let empowerment = contagious.map_or(1.0, |c| c.empowerment);
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Polymorph,
            m.time_remaining,
            empowerment,
            flags,
        );
        commands.entity(e).insert(StatusEffectForwarded::<
            crate::game::units::status_effects::PolymorphedModifier,
        >::default());
    }
    for (e, id, m) in &knockback {
        let dir_x_bits = ((m.direction_x * 1000.0) as i16) as u16 as u32;
        let dir_z_bits = ((m.direction_z * 1000.0) as i16) as u16 as u32;
        let flags = dir_x_bits | (dir_z_bits << 16);
        push(
            &mut connection,
            id.0,
            StatusEffectKind::Knockback,
            m.duration,
            m.speed,
            flags,
        );
        // Drop the ghost's local Knockback instead of tagging it forwarded: it
        // has no guest-side visual role and is never ticked on the guest, so
        // removing it lets the next cast re-insert and re-forward a fresh pull.
        commands
            .entity(e)
            .remove::<crate::game::units::components::Knockback>();
    }
}
