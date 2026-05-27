//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send unit state snapshots to the guest every frame.

use bevy::prelude::*;

use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::Archer;
use crate::game::units::archer::components::Arrow;
use crate::game::units::components::{
    Corpse, FireDoT, FrostAccumulation, Health, KingsGuard, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, Shocked, Team,
};
use crate::game::units::king::components::{King, SpellShield};
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    ArrowSnapshot, CrdtSnapshot, GameSnapshot, SnapshotTick, UNRELIABLE_CRDT_SNAPSHOT, UnitFlags,
    build_unit_snapshot,
};
use crate::state::MultiplayerGameState;

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// or `NetworkedSpellEffect` but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    new_units: Query<Entity, (With<Health>, With<Team>, Without<NetworkEntityId>)>,
    new_effects: Query<Entity, (With<NetworkedSpellEffect>, Without<NetworkEntityId>)>,
) {
    for entity in new_units.iter().chain(new_effects.iter()) {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}

/// Serializes unit state and sends it over the unreliable channel.
///
/// Runs every frame (~60Hz). Queries all entities with a `NetworkEntityId` and
/// builds a compact `GameSnapshot` serialized with `bincode`, prefixed with
/// a type byte so the guest can distinguish it from spell visual snapshots.
#[allow(clippy::type_complexity)]
pub fn send_state_snapshots(
    mut tick: ResMut<SnapshotTick>,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(
        &NetworkEntityId,
        &Transform,
        &crate::game::components::Velocity,
        &Team,
        &Health,
        Option<&CrdtHealth>,
        Has<Corpse>,
        Has<King>,
        Has<Archer>,
        Has<KingsGuard>,
        Has<FireDoT>,
        Has<FrostAccumulation>,
        Has<Shocked>,
        Has<SpellShield>,
        Has<crate::game::units::components::CombatAnimation>,
    )>,
    arrows: Query<&Transform, With<Arrow>>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
        arrows: Vec::with_capacity(arrows.iter().len()),
    };

    for (
        net_id,
        transform,
        velocity,
        team,
        health,
        crdt_health,
        is_corpse,
        is_king,
        is_archer,
        is_guard,
        has_fire,
        has_frost,
        has_electric,
        has_spell_shield,
        has_combat_animation,
    ) in &units
    {
        snapshot.units.push(build_unit_snapshot(
            net_id,
            transform,
            velocity,
            team,
            health,
            crdt_health,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
            has_fire,
            has_frost,
            has_electric,
            has_spell_shield,
            has_combat_animation,
        ));
    }

    for transform in &arrows {
        snapshot.arrows.push(ArrowSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(crate::networking::snapshot::UNRELIABLE_GAME_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}

/// Checks for King death and triggers game-over transition.
///
/// Host = Defenders, Guest = Attackers. When a King becomes a corpse:
/// - Defender King dies → guest wins
/// - Attacker King dies → host wins
pub fn check_mp_king_death(
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    dead_kings: Query<&Team, (With<King>, With<Corpse>)>,
) {
    if let Some(team) = dead_kings.iter().next() {
        let result = match team {
            Team::Defenders => GameOverResult::GuestWins,
            Team::Attackers | Team::Undead => GameOverResult::HostWins,
        };

        *game_outcome = match result {
            GameOverResult::HostWins => GameOutcome::Victory,
            GameOverResult::GuestWins => GameOutcome::DefeatKingDied,
        };

        connection
            .outgoing_messages
            .push(NetworkMessage::GameOver(result));
        next_state.set(MultiplayerGameState::ScoreScreen);
    }
}

/// Receives `TeleportUnits` messages from the guest and executes the teleport on the host.
///
/// Unit positions are host-authoritative, so when the guest casts Teleport it sends
/// a message with source/dest/radius. The host applies the actual position changes.
pub fn receive_teleport_message(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units_query: Query<
        (
            Entity,
            &Transform,
            Option<&crate::game::units::components::Team>,
        ),
        (
            With<crate::game::units::components::Teleportable>,
            Without<
                crate::game::units::wizard::spells::teleport::components::TeleportDestinationCircle,
            >,
            Without<crate::game::units::wizard::spells::teleport::components::TeleportSourceCircle>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::TeleportUnits {
                source_x,
                source_z,
                dest_x,
                dest_z,
                radius,
            } => {
                let source = Vec3::new(source_x, 0.0, source_z);
                let dest = Vec3::new(dest_x, 0.0, dest_z);
                crate::game::units::wizard::spells::teleport::systems::teleport_units_with_radius(
                    &mut rand::rng(),
                    source,
                    dest,
                    radius,
                    &units_query,
                    &mut commands,
                );
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives `SpellHitUnit` messages from the guest and inserts the standard
/// `PendingDamageEffect` on the matching authoritative unit. From there the
/// host runs SP's full status-effect pipeline (`process_pending_damage_effects`
/// → `FireDoT` / `Shocked` / etc. → `update_fire_dot` → CRDT damage tick),
/// and the resulting status flag is shipped back to the guest in the next
/// state snapshot for visual rendering — the guest never owns status state
/// itself.
pub fn receive_spell_hit_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(Entity, &NetworkEntityId), With<Health>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::SpellHitUnit {
                target_network_id,
                damage,
                damage_type,
            } => {
                // HP damage flows via the CRDT pipeline (guest's local
                // `apply_spell_damage` decrements ghost Health →
                // `sync_health_to_crdt` records into guest's CRDT slot →
                // CRDT snapshot → `receive_crdt_snapshot` merges →
                // host's Health.current re-derives). This message ONLY
                // carries the status-type so the host can stack the
                // right DoT/status — `apply_spell_damage` on the guest
                // already accounted for the immediate hit via CRDT.
                let Some(local_entity) = units
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                else {
                    continue;
                };
                if let Ok(mut ec) = commands.get_entity(local_entity) {
                    ec.insert(crate::game::units::components::PendingDamageEffect {
                        damage,
                        damage_type: crate::game::units::damage::DamageType::from_u8(damage_type),
                    });
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives `ApplyStatusEffect` messages from the guest and applies the
/// corresponding component(s) to the host's authoritative unit. This is the
/// universal guest→host wire for all non-damage status effects — sleep, root,
/// polymorph, mind control, banish, mark, haste, battle hymn, berserker rage,
/// guardian temp-HP, slow, stun, fog evasion, knockback.
///
/// HP damage continues to flow via the CRDT pipeline; this system handles
/// everything else.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn receive_apply_status_effect(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(Entity, &NetworkEntityId), With<Health>>,
    // Read-only access to the target's current rendering + spawn state so the
    // Polymorph handler can construct a proper `PolymorphedModifier` that
    // captures what to restore when the spell wears off.
    polymorph_targets: Query<(
        &Transform,
        &crate::game::units::components::Team,
        &Health,
        &MeshMaterial3d<StandardMaterial>,
        &Mesh3d,
    )>,
    // MindControl needs the defender's spawn position so the control-wear-off
    // path can rally the unit back to its origin.
    mind_control_targets: Query<&crate::game::pathfinding::FlowFieldInfluence>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::ApplyStatusEffect {
                target_network_id,
                kind,
                duration,
                magnitude,
                flags,
            } => {
                let Some(kind) =
                    crate::networking::protocol::StatusEffectKind::from_u8(kind)
                else {
                    warn!("[MP] Unknown StatusEffectKind ordinal {}", kind);
                    continue;
                };
                let Some(local_entity) = units
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                else {
                    continue;
                };
                // Polymorph needs to capture the target's current visual state
                // for the restore-on-expiry path. Look it up here while we still
                // have read-only query access; everything else gets `None`.
                let polymorph_capture = if matches!(
                    kind,
                    crate::networking::protocol::StatusEffectKind::Polymorph
                ) {
                    polymorph_targets
                        .get(local_entity)
                        .ok()
                        .map(|(_, team, hp, mat, mesh)| {
                            (*team, hp.current, hp.max, mat.0.clone(), mesh.0.clone())
                        })
                } else {
                    None
                };
                // MindControl needs the unit's defender spawn position so its
                // control-wear-off path can rally back. Only defenders have
                // a spawn position; everyone else gets None and the rally
                // logic does nothing on wear-off (acceptable for attackers).
                let mc_spawn_pos = if matches!(
                    kind,
                    crate::networking::protocol::StatusEffectKind::MindControl
                ) {
                    mind_control_targets.get(local_entity).ok().and_then(|infl| {
                        if let crate::game::pathfinding::FlowFieldInfluence::Defender { spawn_pos } =
                            infl
                        {
                            Some(*spawn_pos)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };
                apply_status_to_entity(
                    &mut commands,
                    local_entity,
                    kind,
                    duration,
                    magnitude,
                    flags,
                    polymorph_capture,
                    mc_spawn_pos,
                );
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Inserts the SP component(s) for a status-effect kind on the target entity.
/// Centralised here so the message handler and any future direct callers
/// share the same mapping.
///
/// **Coverage note:** Phase 1 wires the *base* effects for the cleanly
/// constructable status components. Polymorph and MindControl have specific
/// per-instance state (original material/mesh/team for Polymorph, spawn
/// position for MindControl) that needs to be captured authoritatively on the
/// host from the unit being targeted — those branches construct sensible
/// defaults so the unit at least *enters* the state; talent + visual variants
/// land in Phase 3.
#[allow(clippy::too_many_arguments)]
fn apply_status_to_entity(
    commands: &mut Commands,
    entity: Entity,
    kind: crate::networking::protocol::StatusEffectKind,
    duration: f32,
    magnitude: f32,
    flags: u32,
    polymorph_capture: Option<(
        crate::game::units::components::Team,
        f32, // current HP
        f32, // max HP
        Handle<StandardMaterial>,
        Handle<Mesh>,
    )>,
    mc_spawn_pos: Option<Vec2>,
) {
    use crate::game::units::components as comp;
    use crate::game::units::status_effects as sfx;
    use crate::networking::protocol::StatusEffectKind as K;
    use crate::networking::protocol::status_flags as sf;

    let Ok(mut ec) = commands.get_entity(entity) else {
        return;
    };

    match kind {
        K::Sleep => {
            use crate::game::units::wizard::spells::sleep::constants as sleep_constants;
            let bonus_mult = if magnitude > 0.0 { magnitude } else { 2.0 };
            ec.insert(sfx::SleepModifier::new(duration, bonus_mult));
            if flags & sf::SLEEP_NIGHT_TERRORS != 0 {
                // Use the SP constant, not magnitude — magnitude carries
                // bonus_damage_multiplier (sleep wake-up damage scaling),
                // NOT the per-second tick damage value.
                ec.insert(sfx::NightTerrors::new(sleep_constants::NIGHT_TERRORS_DPS));
            }
            if flags & sf::SLEEP_COMATOSE != 0 {
                ec.insert(sfx::Comatose::new(0.30));
            }
            if flags & sf::SLEEP_NARCOLEPTIC_WAVE != 0 {
                // Use the SP constants, not the previously-hardcoded
                // (0.5, 80.0) which made the wave 6x faster and 33% wider
                // on the host vs SP.
                ec.insert(sfx::NarcolepticWave::new(
                    sleep_constants::NARCOLEPTIC_SPREAD_DELAY,
                    sleep_constants::NARCOLEPTIC_SPREAD_RADIUS,
                ));
            }
            if flags & sf::SLEEP_DREAMWALKER != 0 {
                ec.insert(sfx::Sleepwalking::new(0.5));
            }
            // SLEEP_ETERNAL_SLUMBER bit is documented in protocol.rs but
            // never set by the guest forwarder: Eternal Slumber kills
            // units by setting HP=0 in the guest's local cast path; the
            // CRDT health pipeline already propagates the kill to the
            // host. No status-effect message is needed for it.
        }
        K::Root => {
            ec.insert(sfx::RootedModifier::new(duration));
            let _ = (flags, magnitude);
        }
        K::Polymorph => {
            // Real Polymorph requires the unit's current material/mesh/team
            // for the revert path. If the caller threaded the capture
            // through (`polymorph_capture`), construct the full
            // `PolymorphedModifier`; otherwise fall back to Sleep so the
            // unit still can't act on the host.
            if let Some((team, hp_current, hp_max, material, mesh)) = polymorph_capture {
                ec.insert(sfx::PolymorphedModifier::new(
                    duration, hp_current, hp_max, material, mesh, team,
                ));
            } else {
                ec.insert(sfx::SleepModifier::new(duration, 1.0));
            }
            let _ = (flags, magnitude);
        }
        K::MindControl => {
            // MindControlled has no `new`; construct with defaults plus the
            // defender spawn position (if the target was a defender), so
            // the host's wear-off path correctly rallies the unit back to
            // its origin instead of leaving it wandering.
            ec.insert(comp::MindControlled {
                time_elapsed: 0.0,
                wear_off_duration: duration,
                original_spawn_pos: mc_spawn_pos,
                damage_multiplier: if magnitude > 0.0 { magnitude } else { 1.0 },
            });
            let _ = flags;
        }
        K::Banish => {
            ec.insert(sfx::BanishedModifier::new(duration));
            ec.insert(Visibility::Hidden);
            let _ = (flags, magnitude);
        }
        K::Mark => {
            let amp = if magnitude > 0.0 { magnitude } else { 0.5 };
            ec.insert(sfx::MarkedForDeathModifier::new(amp, duration));
            let _ = flags;
        }
        K::Haste => {
            let speed_bonus = if magnitude > 0.0 { magnitude } else { 0.5 };
            ec.insert(sfx::HasteModifier::new(speed_bonus, duration));
            let _ = flags;
        }
        K::BattleHymn => {
            // magnitude = damage_bonus, encode attack_speed in flags low 16
            // as percent (0..=10000 → 0.0..=1.0).
            let damage_bonus = if magnitude > 0.0 { magnitude } else { 0.4 };
            let attack_speed = ((flags & 0xFFFF) as f32) / 10_000.0;
            ec.insert(sfx::BattleHymnModifier::new(damage_bonus, attack_speed, duration));
        }
        K::BerserkerRage => {
            // magnitude = damage_bonus, vulnerability in flags low 16 as percent.
            let damage_bonus = if magnitude > 0.0 { magnitude } else { 1.0 };
            let vulnerability = ((flags & 0xFFFF) as f32) / 10_000.0;
            ec.insert(sfx::BerserkerRageModifier::new(damage_bonus, vulnerability, duration));
        }
        K::GuardianTempHp => {
            ec.insert(comp::TemporaryHitPoints::new(magnitude, duration));
            let _ = flags;
        }
        K::Slow => {
            ec.insert(comp::SlowMovementModifier::new(magnitude, duration));
            let _ = flags;
        }
        K::Knockback => {
            // magnitude = speed, duration = duration;
            // flags low 16 = (dir_x * 1000) as i16, high 16 = (dir_z * 1000) as i16.
            let dir_x = ((flags & 0xFFFF) as i16) as f32 / 1000.0;
            let dir_z = (((flags >> 16) & 0xFFFF) as i16) as f32 / 1000.0;
            ec.insert(comp::Knockback::new(
                Vec3::new(dir_x, 0.0, dir_z),
                magnitude,
                duration,
            ));
        }
        K::Stun => {
            ec.insert(sfx::Stunned::new(duration));
            let _ = (magnitude, flags);
        }
        K::FogEvasion => {
            ec.insert(sfx::FogEvasionModifier::new(magnitude, duration));
            let _ = flags;
        }
    }
}

/// Receives `RaiseCorpse` messages: convert a specific corpse on the host
/// into an Undead unit. Uses the shared SP `resurrect_corpse_as_infantry`
/// helper with the undead asset set, so the resulting unit looks and behaves
/// identically to one raised by the SP code path.
///
/// The new undead unit gets a `NetworkEntityId` next frame via
/// `assign_network_ids`, and its existence then propagates to the guest via
/// the regular unit snapshot — no extra message needed.
pub fn receive_raise_corpse_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    corpses: Query<
        (Entity, &NetworkEntityId, &Transform),
        With<crate::game::units::components::Corpse>,
    >,
    undead_assets: Option<Res<crate::game::units::undead::resources::UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let Some(undead_assets) = undead_assets else {
        // Undead assets unavailable (e.g. preloaded by spell plugin not
        // initialised yet) — drop the message; the guest will retry on
        // the next cast.
        return;
    };

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::RaiseCorpse {
                target_network_id,
                flags,
                empowerment,
            } => {
                let Some((corpse_entity, _, transform)) = corpses
                    .iter()
                    .find(|(_, id, _)| id.0 == target_network_id)
                else {
                    continue;
                };
                use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
                use crate::game::units::components::{Effectiveness, Team};
                use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
                use crate::game::units::wizard::spells::raise_the_dead::{
                    components as raise_components, constants as raise_constants,
                };
                use crate::networking::protocol::status_flags as sf;

                // HP multiplier — match SP's `resurrect_nearest_corpse` flow:
                // stack EMPOWERED_UNDEAD and REVENANT_LORD multipliers, then
                // multiply by empowerment for the final HP.
                let mut hp_mult = 1.0_f32;
                if flags & sf::RAISE_EMPOWERED_UNDEAD != 0 {
                    hp_mult *= raise_constants::EMPOWERED_UNDEAD_HP_MULT;
                }
                if flags & sf::RAISE_REVENANT_LORD != 0 {
                    hp_mult *= raise_constants::REVENANT_HP_MULT;
                }
                let health = UNIT_HEALTH * empowerment.max(0.01) * hp_mult;
                let speed = UNIT_MOVEMENT_SPEED * 0.5 * empowerment.max(0.01);

                crate::game::units::systems::resurrect_corpse_as_infantry(
                    &mut commands,
                    corpse_entity,
                    transform.translation,
                    Team::Undead,
                    health,
                    speed,
                    UNDEAD_SPRITE_TINT,
                    undead_assets.sprite_texture.clone(),
                    undead_assets.sprite_mesh.clone(),
                    &mut materials,
                    Some(undead_assets.death_texture.clone()),
                );

                // Apply talent marker components matching the SP path's
                // `apply_talent_components` so all behaviors run on the
                // host's authoritative new undead. Includes Effectiveness
                // so damage bonuses from Empowered Undead / Revenant Lord
                // / empowerment are actually applied (was previously
                // missing — undead dealt base damage regardless of talent).
                if let Ok(mut ec) = commands.get_entity(corpse_entity) {
                    ec.insert(raise_components::RaisedUndead);

                    let mut damage_bonus = if empowerment > 1.0 { 0.25 } else { 0.0 };
                    if flags & sf::RAISE_EMPOWERED_UNDEAD != 0 {
                        damage_bonus += raise_constants::EMPOWERED_UNDEAD_DAMAGE_MULT - 1.0;
                    }
                    if flags & sf::RAISE_REVENANT_LORD != 0 {
                        damage_bonus += raise_constants::REVENANT_DAMAGE_MULT - 1.0;
                    }
                    if damage_bonus > 0.0 {
                        let mut effectiveness = Effectiveness::new();
                        effectiveness.spell_bonus = damage_bonus;
                        ec.insert(effectiveness);
                    }

                    if flags & sf::RAISE_PLAGUE_BEARER != 0 {
                        ec.insert(raise_components::PlagueBearerAura::new(
                            raise_constants::PLAGUE_BEARER_DPS,
                            raise_constants::PLAGUE_BEARER_RADIUS,
                            raise_constants::PLAGUE_BEARER_TICK_INTERVAL,
                        ));
                    }
                    if flags & sf::RAISE_PERPETUAL_UNREST != 0 {
                        ec.insert(raise_components::PerpetualUnrest {
                            raise_radius: raise_constants::PERPETUAL_UNREST_RADIUS,
                        });
                    }
                    if flags & sf::RAISE_REVENANT_LORD != 0 {
                        ec.insert(raise_components::RevenantLord {
                            raise_radius: raise_constants::REVENANT_RAISE_RADIUS,
                            raise_interval: raise_constants::REVENANT_RAISE_INTERVAL,
                            raise_timer: 0.0,
                        });
                    }
                    if flags & sf::RAISE_UNDEAD_DETONATION != 0 {
                        ec.insert(raise_components::UndeadDetonation {
                            damage: raise_constants::UNDEAD_DETONATION_DAMAGE,
                            radius: raise_constants::UNDEAD_DETONATION_RADIUS,
                        });
                    }
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives Dispel-driven messages from the guest: despawn a spell-effect
/// entity by network ID, or strip SpellShield from a unit.
pub fn receive_dispel_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    spell_effects: Query<
        (Entity, &NetworkEntityId),
        With<crate::game::multiplayer::components::NetworkedSpellEffect>,
    >,
    units_with_shield: Query<
        (Entity, &NetworkEntityId),
        With<crate::game::units::king::components::SpellShield>,
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::DispelSpellEffect { target_network_id } => {
                if let Some(entity) = spell_effects
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                    && let Ok(mut ec) = commands.get_entity(entity)
                {
                    ec.try_despawn();
                }
            }
            NetworkMessage::DispelShield { target_network_id } => {
                if let Some(entity) = units_with_shield
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                    && let Ok(mut ec) = commands.get_entity(entity)
                {
                    ec.remove::<crate::game::units::king::components::SpellShield>();
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives CRDT health updates from the guest and merges into local unit state.
///
/// The guest sends its CRDT counters (guest spell damage/healing) over the
/// unreliable channel. The host merges these using element-wise max so that
/// guest spell effects are reflected in the host's simulation. Critically
/// for healing: the CRDT path lets the guest cast a heal spell (which
/// touches `Health.current` upward), have that propagate to the host's
/// authoritative unit, and have the host's HP converge — without needing
/// per-spell heal messages.
pub fn receive_crdt_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut units: Query<(
        Entity,
        &NetworkEntityId,
        &mut CrdtHealth,
        &mut Health,
        Has<RemoteFireEffect>,
        Has<RemoteFrostEffect>,
        Has<RemoteElectricEffect>,
    )>,
) {
    let raw_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_crdt_data: Option<&[u8]> = None;

    for data in &raw_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_CRDT_SNAPSHOT => {
                latest_crdt_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    connection.incoming_unreliable = other_data;

    let Some(crdt_bytes) = latest_crdt_data else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<CrdtSnapshot>(crdt_bytes) else {
        warn!(
            "Failed to deserialize CRDT snapshot ({} bytes)",
            crdt_bytes.len()
        );
        return;
    };

    let mut update_map = std::collections::HashMap::with_capacity(snapshot.units.len());
    for (i, update) in snapshot.units.iter().enumerate() {
        update_map.insert(update.id, i);
    }

    for (
        entity,
        net_id,
        mut crdt_health,
        mut health,
        has_remote_fire,
        has_remote_frost,
        has_remote_electric,
    ) in &mut units
    {
        if let Some(&idx) = update_map.get(&net_id.0) {
            let update = &snapshot.units[idx];
            let remote = CrdtHealth {
                max_hp: crdt_health.max_hp,
                damage: update.damage,
                healing: update.healing,
            };
            crdt_health.merge(&remote);
            health.current = crdt_health.current_hp();

            // CRDT effects slot is u8 (lower 8 bits) — cast u16 constants to u8.
            let remote_fire = update.effects & (UnitFlags::FIRE_EFFECT as u8) != 0;
            let remote_frost = update.effects & (UnitFlags::FROST_EFFECT as u8) != 0;
            let remote_electric = update.effects & (UnitFlags::ELECTRIC_EFFECT as u8) != 0;

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
        }
    }
}
