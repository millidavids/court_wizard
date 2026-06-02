//! Guest snapshot application and CRDT sync.

use super::guest_visuals::pick_material;
use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::OriginalMaterial;
use crate::game::units::components::{
    Corpse, FireDoT, FrostAccumulation, Health, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, RemotePoisonEffect, RemotePolymorphEffect, Shocked,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::{King, SpellShield};
use crate::game::units::king::resources::KingAssets;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath;
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::{NetworkEntityId, NetworkEntityMap};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    CrdtSnapshot, CrdtUnitUpdate, GameSnapshot, UNRELIABLE_CRDT_SNAPSHOT, UNRELIABLE_GAME_SNAPSHOT,
    UnitFlags, u8_to_team,
};

use super::components::{GhostArrow, GhostEntity, OnMultiplayerGameScreen};
use crate::game::units::wizard::spells::polymorph::constants::SHEEP_COLOR;
use crate::game::units::wizard::spells::polymorph::systems::apply_sheep_visual;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Guest-side forwarder: when a spell on the guest applies damage to a
/// ghost unit, SP's `apply_spell_damage` inserts a `PendingDamageEffect`
/// on the target. We **poll** for any such component (deliberately not
/// `Added<>`) each frame because `apply_spell_damage` queues inserts via
/// `Commands`; the component isn't visible to an `Added` filter until
/// after the next command flush, and a ghost killed before the flush
/// would never be forwarded. Polling catches both same-frame inserts and
/// any that survive across a frame.
///
/// The host then owns status-effect bookkeeping (DoT stacks, durations,
/// snapshot flags). After forwarding, the local `PendingDamageEffect` is
/// removed so the guest doesn't *also* tick the DoT (which would double-
/// apply on top of the host-ticked damage that propagates back via CRDT).
///
/// Excremage conversion: a guest playing Excremage should turn every spell
/// hit into a Poop hit, but `process_pending_damage_effects` does that
/// lookup against the LOCAL `GameConfig.wizard_type` — and on the host
/// that's the host's wizard, not the guest's. So we do the conversion here
/// on the guest before forwarding, using the guest's own config.
pub fn forward_spell_hits_to_host(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    config: Res<crate::config::GameConfig>,
    hits: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::PendingDamageEffect,
        ),
        With<super::components::GhostEntity>,
    >,
) {
    let excremage = config.wizard_type == crate::config::WizardType::Excremage;
    for (entity, net_id, pending) in &hits {
        let damage_type = if excremage {
            crate::game::units::damage::DamageType::Poop
        } else {
            pending.damage_type
        };
        connection.outgoing_messages.push(
            crate::networking::protocol::NetworkMessage::SpellHitUnit {
                target_network_id: net_id.0,
                damage: pending.damage,
                damage_type: damage_type.to_u8(),
            },
        );
        commands
            .entity(entity)
            .remove::<crate::game::units::components::PendingDamageEffect>();
    }
}

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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
            With<super::components::GhostEntity>,
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
        With<super::components::GhostEntity>,
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

/// Marker inserted alongside each forwarded status component so the
/// forwarder doesn't re-ship the same effect every frame for as long as the
/// component lives. Component-typed so adding a new status doesn't risk
/// colliding with an existing marker. The companion system
/// `cleanup_forwarded_marker::<T>` removes this marker the frame after `T`
/// is removed from the ghost, so a re-cast of the same status on the same
/// ghost is re-forwarded instead of silently dropped.
#[derive(Component)]
pub struct StatusEffectForwarded<T: Component> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component> Default for StatusEffectForwarded<T> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Removes `StatusEffectForwarded<T>` from any ghost entity whose underlying
/// status component `T` was just removed (expired, dispelled, etc.). Run as
/// one instance per concrete `T` so each marker is paired with the exact
/// component that drives it.
pub fn cleanup_forwarded_marker<T: Component>(
    mut commands: Commands,
    forwarded: Query<Entity, (With<StatusEffectForwarded<T>>, Without<T>)>,
) {
    for entity in &forwarded {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.remove::<StatusEffectForwarded<T>>();
        }
    }
}

/// Receives the latest unit state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
///
/// Filters incoming unreliable data by type prefix byte, processing only game
/// snapshots (unit data). Spell visual snapshots are handled by `spell_sync.rs`.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn apply_state_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<KingAssets>,
    undead_assets: Res<UndeadAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut ghost_query: Query<
        (
            Entity,
            &mut Transform,
            &mut CrdtHealth,
            &mut Health,
            &mut crate::game::components::Velocity,
            Has<RemoteFireEffect>,
            Has<RemoteFrostEffect>,
            Has<RemoteElectricEffect>,
            Has<SpellShield>,
            Has<Corpse>,
            Has<crate::game::units::components::CombatAnimation>,
            Has<RemotePoisonEffect>,
            Has<ActiveMarkOfDeath>,
            Has<RemotePolymorphEffect>,
        ),
        With<GhostEntity>,
    >,
    ghost_arrows: Query<Entity, With<GhostArrow>>,
) {
    // Filter for game snapshots only (type prefix 0x00), re-queue others
    let raw_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_game_data: Option<&[u8]> = None;

    for data in &raw_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_GAME_SNAPSHOT => {
                latest_game_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    // Re-queue non-game data for other systems (spell snapshots)
    connection.incoming_unreliable = other_data;

    // Velocities are now host-authoritative (shipped in `UnitSnapshot.vx/vz`);
    // between snapshots the last written value persists, which is exactly
    // what we want — animations stay smooth during brief packet gaps and
    // wind down naturally when the host's next snapshot reports zero.
    let Some(game_bytes) = latest_game_data else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<GameSnapshot>(game_bytes) else {
        warn!(
            "Failed to deserialize game snapshot ({} bytes)",
            game_bytes.len()
        );
        return;
    };

    // Track which IDs are present in this snapshot
    let mut seen_ids = HashSet::with_capacity(snapshot.units.len());

    for unit in &snapshot.units {
        seen_ids.insert(unit.id);

        let is_corpse = unit.flags & UnitFlags::CORPSE != 0;
        let is_king = unit.flags & UnitFlags::KING != 0;
        let is_archer = unit.flags & UnitFlags::ARCHER != 0;
        let is_guard = unit.flags & UnitFlags::KINGS_GUARD != 0;
        let team = u8_to_team(unit.team);

        let pos = Vec3::new(unit.x, unit.y, unit.z);

        // NOTE on material/mesh handles: do NOT compute them
        // unconditionally outside the spawn/transition branches. For alive
        // sprite units `pick_material` calls `materials.add(...)` which
        // allocates a fresh `StandardMaterial` asset every invocation. If
        // it ran on every snapshot for every ghost, two things broke:
        //  1. The ghost's `MeshMaterial3d` would be overwritten with the
        //     new (UV = identity) handle ~60 Hz, racing with — and
        //     systematically erasing — the per-frame UV writes that
        //     `update_walking_animation` / `update_combat_animation` made
        //     to the previous material. The visible symptom was walking
        //     and attack animations restarting on every frame.
        //  2. ~60 fresh `StandardMaterial`s per ghost per second were
        //     orphaned in the asset server, leaking memory linearly with
        //     match duration.
        // The material/mesh handles are now computed lazily inside the
        // spawn branch and the corpse-transition branch only.

        // Build remote CRDT state from the snapshot
        let remote_crdt = CrdtHealth {
            max_hp: unit.max_hp,
            damage: unit.damage,
            healing: unit.healing,
        };

        let remote_fire = unit.flags & UnitFlags::FIRE_EFFECT != 0;
        let remote_frost = unit.flags & UnitFlags::FROST_EFFECT != 0;
        let remote_electric = unit.flags & UnitFlags::ELECTRIC_EFFECT != 0;
        let remote_spell_shield = unit.flags & UnitFlags::SPELL_SHIELD != 0;
        let remote_combat = unit.flags & UnitFlags::COMBAT_ANIMATION != 0;
        let remote_poison = unit.flags & UnitFlags::POISON_EFFECT != 0;
        let remote_mark = unit.flags & UnitFlags::MARK_EFFECT != 0;
        let remote_polymorph = unit.flags & UnitFlags::POLYMORPH != 0;

        if let Some(&local_entity) = entity_map.remote_to_local.get(&unit.id) {
            if let Ok((
                entity,
                mut transform,
                mut crdt_health,
                mut health,
                mut velocity,
                has_remote_fire,
                has_remote_frost,
                has_remote_electric,
                has_spell_shield,
                has_corpse,
                has_combat,
                has_remote_poison,
                has_remote_mark,
                has_remote_polymorph,
            )) = ghost_query.get_mut(local_entity)
            {
                // Use the host's AUTHORITATIVE velocity from the snapshot.
                // Synthesising velocity locally from position deltas caused
                // the walking animation to reset to idle every time the
                // host's unit briefly stopped (between targeting decisions,
                // wall-avoidance moments, etc.) — the delta was zero, the
                // synthesised speed was below the animation move threshold,
                // and `update_walking_animation` reset to frame 0. With the
                // host's actual `Velocity` shipped in the snapshot, guest
                // animations match the host's exactly.
                //
                // Defence-in-depth: reject NaN/Inf (which would propagate
                // into `speed_sq` and make every IEEE comparison return
                // false → animation ticks forever and any future system
                // that integrates this velocity into a Transform corrupts
                // position), and cap magnitude at a sane upper bound so a
                // pathological host spike (knockback math error, etc.)
                // doesn't flash through the animation for one frame.
                const MAX_REMOTE_SPEED: f32 = 400.0;
                let raw_x = if unit.vx.is_finite() { unit.vx } else { 0.0 };
                let raw_z = if unit.vz.is_finite() { unit.vz } else { 0.0 };
                let raw_speed_sq = raw_x * raw_x + raw_z * raw_z;
                let (cx, cz) = if raw_speed_sq > MAX_REMOTE_SPEED * MAX_REMOTE_SPEED {
                    let scale = MAX_REMOTE_SPEED / raw_speed_sq.sqrt();
                    (raw_x * scale, raw_z * scale)
                } else {
                    (raw_x, raw_z)
                };
                velocity.x = cx;
                velocity.z = cz;

                transform.translation = pos;

                // Merge CRDT state from host (takes max of each slot)
                crdt_health.merge(&remote_crdt);

                // Re-derive Health from converged CRDT state so damage systems see correct HP
                health.current = crdt_health.current_hp();

                // Material handling on corpse transition.
                //
                // For NON-KING units we DELIBERATELY do NOT swap the
                // material here. The existing block below (lines ~342-357)
                // inserts `Corpse` + `DyingAnimation`, and the shared SP
                // animation pipeline (`update_dying_animation` →
                // `DeathAnimationFinished` → `finalize_dying_to_corpse`)
                // mutates the entity's CURRENT material in place — first
                // setting its `base_color_texture` to the death sheet and
                // ticking the death-frame UV, then converting it to the
                // corpse appearance (corpse tint + `AlphaMode::Blend`).
                // Because the ghost's material is the per-entity allocation
                // from the spawn branch, those `get_mut` writes don't
                // corrupt anything else.
                //
                // The previous version of this branch swapped to a SHARED
                // handle cloned from `infantry_assets.defender_corpse_materials[idx]`
                // (and similar). `update_dying_animation`'s first-frame
                // `materials.get_mut(handle); mat.base_color_texture = death_sheet`
                // then mutated the SHARED asset, replacing the texture for
                // every other ghost or SP corpse holding that variant for
                // the duration of one death sequence. Removing the swap
                // here fixes that corruption.
                //
                // KINGS are special: kings have no death animation in SP
                // (the existing block skips DyingAnimation for `is_king`),
                // so `finalize_dying_to_corpse` never runs on them. They
                // also use a dedicated corpse sprite, not a tint of the
                // alive sprite. We swap directly to a king-corpse handle
                // here. At most one king per team exists, and nothing ever
                // calls `get_mut` on the king-corpse handle, so sharing
                // the handle is safe.
                if is_corpse != has_corpse {
                    let mut ec = commands.entity(entity);
                    if is_king {
                        let new_handle = pick_material(
                            &infantry_assets,
                            &archer_assets,
                            &king_assets,
                            &undead_assets,
                            &mut materials,
                            team,
                            is_corpse,
                            is_king,
                            is_archer,
                            is_guard,
                        );
                        ec.insert(MeshMaterial3d(new_handle));
                    } else if !is_corpse {
                        // Corpse → alive: Raise the Dead resurrected this corpse
                        // into an undead unit. Swap the corpse material to the alive
                        // sprite so it stops rendering as a laid-flat corpse. Uses
                        // the CURRENT snapshot team (now `Undead`), so `pick_material`
                        // returns the dedicated undead sprite. The sprite mesh is
                        // unchanged (sprite units share one quad for both states).
                        let new_handle = pick_material(
                            &infantry_assets,
                            &archer_assets,
                            &king_assets,
                            &undead_assets,
                            &mut materials,
                            team,
                            false,
                            is_king,
                            is_archer,
                            is_guard,
                        );
                        ec.insert(MeshMaterial3d(new_handle));
                        // Raising a unit mid-death-animation would otherwise leave
                        // DyingAnimation ticking death-sheet UVs over the new alive
                        // sprite, flickering it. Clear it on the corpse→alive edge.
                        ec.remove::<crate::game::units::components::DyingAnimation>();
                    }
                    // Clear stale tint state in both cases — if a tint was
                    // active on the alive sprite, the SP corpse finalizer
                    // (or, for kings, the snapshot swap above) supersedes
                    // it, and we don't want a later tint-restore to put
                    // the alive-looking material back on a corpse.
                    ec.remove::<OriginalMaterial>();
                }

                // Sync remote status effect visual markers from host
                if remote_fire && !has_remote_fire {
                    commands.entity(entity).insert(RemoteFireEffect);
                } else if !remote_fire && has_remote_fire {
                    commands.entity(entity).remove::<RemoteFireEffect>();
                }
                if remote_frost && !has_remote_frost {
                    commands.entity(entity).insert(RemoteFrostEffect);
                } else if !remote_frost && has_remote_frost {
                    commands.entity(entity).remove::<RemoteFrostEffect>();
                }
                if remote_electric && !has_remote_electric {
                    commands.entity(entity).insert(RemoteElectricEffect);
                } else if !remote_electric && has_remote_electric {
                    commands.entity(entity).remove::<RemoteElectricEffect>();
                }
                if remote_poison && !has_remote_poison {
                    commands.entity(entity).insert(RemotePoisonEffect);
                } else if !remote_poison && has_remote_poison {
                    commands.entity(entity).remove::<RemotePoisonEffect>();
                }
                // Mark of Death: insert the BARE `ActiveMarkOfDeath` marker so
                // `spawn_mark_indicators` renders the floating indicator. We do
                // NOT add `MarkedForDeathModifier`/`MarkTalentFlags`, so the
                // doom/executioner/blight gameplay systems never MATCH the ghost:
                // their `any_exist::<ActiveMarkOfDeath>` run-condition still wakes
                // them, but their queries require those absent components, so they
                // iterate nothing. Only the visual indicator renders.
                if remote_mark && !has_remote_mark {
                    commands.entity(entity).insert(ActiveMarkOfDeath);
                } else if !remote_mark && has_remote_mark {
                    commands.entity(entity).remove::<ActiveMarkOfDeath>();
                }

                // Polymorph: render the ghost as a sheep while the host's unit is
                // polymorphed (host-authoritative). Swap on the off→on edge,
                // restore the unit's normal sprite on the on→off edge. We also
                // strip any local `PolymorphedModifier` the guest's own cast left
                // on the ghost so the unit isn't excluded from `Without<
                // PolymorphedModifier>` spell-target queries forever after revert.
                if remote_polymorph && !has_remote_polymorph {
                    let mut ec = commands.entity(entity);
                    apply_sheep_visual(&mut ec, &mut materials, &spell_assets, SHEEP_COLOR);
                    ec.insert(RemotePolymorphEffect);
                } else if !remote_polymorph && has_remote_polymorph {
                    // Only restore the alive sprite if the unit is still alive. If
                    // it died AS a sheep (is_corpse), the corpse-transition branch
                    // above owns the material — swapping to a SHARED corpse handle
                    // here would let DyingAnimation's `get_mut` corrupt the corpse
                    // appearance of every other entity sharing that handle variant.
                    let restored = if is_corpse {
                        None
                    } else {
                        let material = pick_material(
                            &infantry_assets,
                            &archer_assets,
                            &king_assets,
                            &undead_assets,
                            &mut materials,
                            team,
                            false,
                            is_king,
                            is_archer,
                            is_guard,
                        );
                        let mesh = if is_king {
                            king_assets.sprite_mesh.clone()
                        } else if is_archer {
                            archer_assets.sprite_mesh.clone()
                        } else {
                            infantry_assets.sprite_mesh.clone()
                        };
                        Some((material, mesh))
                    };
                    let mut ec = commands.entity(entity);
                    if let Some((material, mesh)) = restored {
                        ec.insert(MeshMaterial3d(material));
                        ec.insert(Mesh3d(mesh));
                    }
                    ec.remove::<RemotePolymorphEffect>();
                    ec.remove::<crate::game::units::status_effects::PolymorphedModifier>();
                    // Also clear the sheep bounce so a reverted ghost doesn't keep
                    // floating (bounce_sheep_units is host-gated, but a guest-cast
                    // ghost can carry SheepBounce from its own local application).
                    ec.remove::<crate::game::units::wizard::spells::polymorph::sheep_visual::SheepBounce>();
                }

                // Sync spell shield component from host. No visual swap
                // — the king's aura sphere is the constant visual and
                // stays on regardless of shield state (matches SP, where
                // the king has no shield mechanic and the aura is just
                // always there).
                if remote_spell_shield && !has_spell_shield {
                    commands.entity(entity).insert(SpellShield);
                } else if !remote_spell_shield && has_spell_shield {
                    commands.entity(entity).remove::<SpellShield>();
                }

                // Sync corpse state so spell targeting filters work correctly.
                // On the non-corpse → corpse transition, also kick off the
                // shared `DyingAnimation` so the ghost plays the death frames
                // before settling into its corpse pose — matching the SP
                // visual where units don't pop straight from standing to
                // laid-flat. King has no death sprite sheet (instant corpse
                // swap in SP), so it's skipped here.
                if is_corpse && !has_corpse {
                    let mut ec = commands.entity(entity);
                    ec.insert(Corpse);
                    if !is_king {
                        let death_texture = if is_archer {
                            archer_assets.death_texture.clone()
                        } else {
                            infantry_assets.death_texture.clone()
                        };
                        ec.insert(crate::game::units::components::DyingAnimation::new(
                            death_texture,
                        ));
                    }
                } else if !is_corpse && has_corpse {
                    commands.entity(entity).remove::<Corpse>();
                }

                // Mirror CombatAnimation transitions from the host so the
                // ghost plays its swing/shoot animation while the host's
                // unit is mid-attack. Without this the ghost stays on the
                // idle walking frame during melee. We ONLY insert on the
                // off→on edge; we **do not** force-remove on the on→off
                // edge — the shared SP system `update_combat_animation`
                // ticks the local frames and self-removes the component
                // when finished, AND in the same path restores the
                // walking texture / resets the UV transform. A forced
                // `commands.remove` here would bypass that cleanup and
                // leave the ghost's material stuck on the combat sheet
                // with a frozen mid-swing UV until the next attack
                // overwrites it.
                if remote_combat && !has_combat && !is_corpse && !is_king {
                    let (combat_tex, walking_tex) = if is_archer {
                        (
                            archer_assets.shooting_texture.clone(),
                            archer_assets.sprite_texture.clone(),
                        )
                    } else {
                        // Infantry + king's guard share the infantry sheet.
                        // King has no combat sheet in SP (skipped above).
                        (
                            infantry_assets.attacking_texture.clone(),
                            infantry_assets.sprite_texture.clone(),
                        )
                    };
                    let anim = if is_archer {
                        crate::game::units::components::CombatAnimation::new_shooting(
                            combat_tex,
                            walking_tex,
                        )
                    } else {
                        crate::game::units::components::CombatAnimation::new_attack(
                            combat_tex,
                            walking_tex,
                        )
                    };
                    commands.entity(entity).insert(anim);
                }
            }
        } else {
            // Spawn new ghost entity with Team, Health, and CrdtHealth for spell targeting.
            // Animation components (Velocity + FacingDirection + WalkingAnimation) let the
            // shared animation systems run for ghosts the same way they run for
            // host-simulated units — Velocity starts at default and is overwritten
            // on subsequent snapshots from the host's authoritative `UnitSnapshot.vx/vz`.
            // Give the ghost the SAME `Hitbox` SP uses for this unit type
            // so SP spell-collision systems (fireball, ice, etc.) running
            // on this peer can land on it. Damage applied to the ghost's
            // `Health` flows through `sync_health_to_crdt` → CRDT slot →
            // peer snapshot → host's authoritative units lose HP. This is
            // the entire reason guest-cast spells now reach host units
            // without any per-spell network-message plumbing.
            //
            // **Known gap (latent):** today MP only spawns infantry / archer
            // / king / king's-guard, all covered by this three-way branch.
            // If MP ever spawns brutes / healers / dispellers / etc., add
            // their unit type to `UnitFlags` and a corresponding branch
            // here — otherwise their ghosts get the infantry hitbox (32u
            // vs e.g. brute's 80u) and spells will miss them.
            use crate::game::units::components::Hitbox;
            let hitbox = if is_king {
                Hitbox::new(
                    crate::game::units::king::constants::KING_RADIUS,
                    crate::game::units::king::constants::KING_HITBOX_HEIGHT,
                )
            } else if is_archer {
                Hitbox::new(
                    crate::game::units::archer::constants::ARCHER_RADIUS,
                    crate::game::constants::DEFENDER_HITBOX_HEIGHT,
                )
            } else {
                // Infantry and king's guards both use the standard unit radius.
                Hitbox::new(
                    crate::game::units::infantry::constants::UNIT_RADIUS,
                    crate::game::constants::DEFENDER_HITBOX_HEIGHT,
                )
            };

            // Sprite-based units keep `sprite_mesh` for both live and corpse states.
            let mesh_handle = if is_king {
                king_assets.sprite_mesh.clone()
            } else if is_archer {
                archer_assets.sprite_mesh.clone()
            } else {
                infantry_assets.sprite_mesh.clone()
            };

            // Compute the spawn material once. From this point on the
            // animation systems own the ghost's `MeshMaterial3d` — the
            // snapshot loop only touches it again on the alive→corpse
            // transition (see the `is_corpse != has_corpse` branch above).
            let material_handle = pick_material(
                &infantry_assets,
                &archer_assets,
                &king_assets,
                &undead_assets,
                &mut materials,
                team,
                is_corpse,
                is_king,
                is_archer,
                is_guard,
            );

            let initial_health = Health::new(remote_crdt.max_hp);
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(pos),
                    Billboard,
                    GhostEntity,
                    team,
                    NetworkEntityId(unit.id),
                    initial_health,
                    remote_crdt,
                    hitbox,
                    OnMultiplayerGameScreen,
                    crate::game::components::Velocity::default(),
                    crate::game::units::components::FacingDirection::default(),
                    // Stagger the walk cycle so a freshly-snapshot army
                    // doesn't flip animation frames in unison.
                    crate::game::units::components::WalkingAnimation::new_staggered(
                        &mut game_rng.0,
                    ),
                ))
                .id();

            // Tag the ghost king with the `King` marker AND attach the
            // SP-style aura sphere. The HP bar reads
            // `Query<&Health, With<King>>` and otherwise wouldn't match any
            // ghost on the guest. SP simulation systems that touch
            // `With<King>` are host-gated, so adding this marker on the
            // guest only enables UI queries and is safe. The aura visual
            // is the same `explosion_sphere` + `king_aura_sphere` material
            // SP uses, scaled to `KING_AURA_RADIUS` — both peers now see
            // an identical king.
            if is_king {
                commands.entity(entity).insert(King);
                crate::game::units::king::systems::spawn_king_aura_visual(
                    &mut commands,
                    entity,
                    &spell_assets,
                    OnMultiplayerGameScreen,
                );
            }

            // Mirror spell-shield component state at spawn. No visual is
            // spawned for the shield itself — the aura sphere above is the
            // king's only visual indicator; shield state is invisible to
            // the player (same as SP, which has no shield mechanic at all).
            if remote_spell_shield {
                commands.entity(entity).insert(SpellShield);
            }

            if is_corpse {
                commands.entity(entity).insert(Corpse);
            }

            // Apply effect markers / combat animation at spawn so a late-
            // join ghost doesn't wait one extra snapshot tick for visuals
            // that should be on right now. The on/off edge transitions in
            // the update branch handle subsequent flag changes; without
            // these spawn-time inserts the first frame would show the
            // ghost un-tinted and not mid-swing even when the snapshot
            // says otherwise. We intentionally skip `DyingAnimation` on
            // spawn-as-corpse — the death already happened in the past
            // from this guest's perspective; replaying its frames now
            // would be wrong (the unit just appears already laid out).
            if remote_fire {
                commands.entity(entity).insert(RemoteFireEffect);
            }
            if remote_frost {
                commands.entity(entity).insert(RemoteFrostEffect);
            }
            if remote_electric {
                commands.entity(entity).insert(RemoteElectricEffect);
            }
            if remote_poison {
                commands.entity(entity).insert(RemotePoisonEffect);
            }
            if remote_mark {
                commands.entity(entity).insert(ActiveMarkOfDeath);
            }
            if remote_polymorph {
                let mut ec = commands.entity(entity);
                apply_sheep_visual(&mut ec, &mut materials, &spell_assets, SHEEP_COLOR);
                ec.insert(RemotePolymorphEffect);
            }
            if remote_combat && !is_corpse && !is_king {
                let (combat_tex, walking_tex) = if is_archer {
                    (
                        archer_assets.shooting_texture.clone(),
                        archer_assets.sprite_texture.clone(),
                    )
                } else {
                    (
                        infantry_assets.attacking_texture.clone(),
                        infantry_assets.sprite_texture.clone(),
                    )
                };
                let anim = if is_archer {
                    crate::game::units::components::CombatAnimation::new_shooting(
                        combat_tex,
                        walking_tex,
                    )
                } else {
                    crate::game::units::components::CombatAnimation::new_attack(
                        combat_tex,
                        walking_tex,
                    )
                };
                commands.entity(entity).insert(anim);
            }

            entity_map.insert(unit.id, entity);
        }
    }

    // Despawn ghost entities whose IDs are no longer in the snapshot
    let stale_ids: Vec<u32> = entity_map
        .remote_to_local
        .keys()
        .copied()
        .filter(|id| !seen_ids.contains(id))
        .collect();

    for stale_id in stale_ids {
        if let Some(entity) = entity_map.remove_by_remote(stale_id)
            && let Ok(mut entity_commands) = commands.get_entity(entity)
        {
            entity_commands.try_despawn();
        }
    }

    // Replace all ghost arrows with fresh positions from the snapshot.
    for entity in &ghost_arrows {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }

    for arrow in &snapshot.arrows {
        commands.spawn((
            Mesh3d(archer_assets.arrow_mesh.clone()),
            MeshMaterial3d(archer_assets.arrow_material.clone()),
            Transform::from_translation(Vec3::new(arrow.x, arrow.y, arrow.z)),
            Billboard,
            GhostArrow,
            OnMultiplayerGameScreen,
        ));
    }
}

/// Sends the guest's local CRDT health state to the host for merging.
///
/// Collects CrdtHealth from all ghost entities and sends as a compact
/// CrdtSnapshot over the unreliable channel. The host merges these
/// counters into its local unit state.
pub fn send_crdt_snapshot(
    mut connection: ResMut<NetworkConnection>,
    crdt_units: Query<
        (
            &NetworkEntityId,
            &CrdtHealth,
            Has<FireDoT>,
            Has<FrostAccumulation>,
            Has<Shocked>,
        ),
        With<GhostEntity>,
    >,
) {
    let mut snapshot = CrdtSnapshot {
        units: Vec::with_capacity(crdt_units.iter().len()),
    };

    for (net_id, crdt, has_fire, has_frost, has_electric) in &crdt_units {
        // CRDT effects slot is u8 — UnitFlags constants are u16 now (after
        // COMBAT_ANIMATION moved them past the 8-bit window), so cast
        // each one back. All three fire/frost/electric flags live in the
        // low byte so the truncation is lossless.
        let mut effects = 0u8;
        if has_fire {
            effects |= UnitFlags::FIRE_EFFECT as u8;
        }
        if has_frost {
            effects |= UnitFlags::FROST_EFFECT as u8;
        }
        if has_electric {
            effects |= UnitFlags::ELECTRIC_EFFECT as u8;
        }
        snapshot.units.push(CrdtUnitUpdate {
            id: net_id.0,
            damage: crdt.damage,
            healing: crdt.healing,
            effects,
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(UNRELIABLE_CRDT_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}
