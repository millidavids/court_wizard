use bevy::prelude::*;

use crate::game::units::components::Health;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;

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
    // `Without<PolymorphedModifier>`: if a unit is ALREADY a sheep when a second
    // polymorph message arrives (host-cast + guest-cast overlap, or a duplicated
    // message), capturing its current mesh/material would store the SHEEP as the
    // "original" to restore — leaving it a sheep permanently. Excluding already-
    // polymorphed units makes the capture None, so the receiver no-ops on them.
    polymorph_targets: Query<
        (
            &Transform,
            &crate::game::units::components::Team,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
        ),
        Without<crate::game::units::status_effects::PolymorphedModifier>,
    >,
    // MindControl needs the defender's spawn position so the control-wear-off
    // path can rally the unit back to its origin.
    mind_control_targets: Query<&crate::game::pathfinding::FlowFieldInfluence>,
    // Polymorph needs to swap the unit's mesh/material to the sheep sprite on the
    // host (so it stops attacking and the snapshot flag renders a sheep on the guest).
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
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
                let Some(kind) = crate::networking::protocol::StatusEffectKind::from_u8(kind)
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
                    polymorph_targets.get(local_entity).ok().map(
                        |(transform, team, hp, mat, mesh)| {
                            (
                                *team,
                                hp.current,
                                hp.max,
                                mat.0.clone(),
                                mesh.0.clone(),
                                transform.translation.y,
                            )
                        },
                    )
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
                    mind_control_targets
                        .get(local_entity)
                        .ok()
                        .and_then(|infl| {
                            if let crate::game::pathfinding::FlowFieldInfluence::Defender {
                                spawn_pos,
                            } = infl
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
                    &mut materials,
                    &visual_assets,
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
        f32, // unit Y (sheep bounce baseline)
    )>,
    mc_spawn_pos: Option<Vec2>,
    materials: &mut Assets<StandardMaterial>,
    visual_assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
) {
    use crate::game::units::components as comp;
    use crate::game::units::status_effects as sfx;
    use crate::game::units::wizard::spells::polymorph::{
        components as poly_comp,
        constants::{self as poly_const, SHEEP_COLOR, SHEEP_HP},
        sheep_visual::SheepBounce,
        systems::apply_sheep_visual,
    };
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
            // Full sheep transform on the host. Combat is NOT gated on
            // `PolymorphedModifier` — a unit stops attacking because its
            // `AttackTiming` is removed and it's rendered as a sheep, exactly like
            // the SP cast path. Inserting only the modifier (the old behaviour)
            // left the unit fighting → "guest's polymorph does nothing".
            // `tick_polymorphed_units` (host-only) reverts using the captured
            // original mesh/material/health/team, and the POLYMORPH snapshot flag
            // makes the guest render the sheep.
            if let Some((team, hp_current, hp_max, material, mesh, base_y)) = polymorph_capture {
                let explosive = flags & sf::POLYMORPH_EXPLOSIVE != 0;
                let contagious = flags & sf::POLYMORPH_CONTAGIOUS != 0;
                let permanent = flags & sf::POLYMORPH_PERMANENT != 0;
                let dire = flags & sf::POLYMORPH_DIRE != 0;
                let pig = flags & sf::POLYMORPH_PIG != 0;
                // Contagious spread empowerment rides in magnitude.
                let empowerment = if magnitude > 0.0 { magnitude } else { 1.0 };
                // Fragile HP isn't forwarded (no marker component to read on the
                // guest), so a fragile sheep is base-HP on the host.
                let (sheep_hp, color) = if dire {
                    (poly_const::DIRE_SHEEP_HP, poly_const::DIRE_SHEEP_COLOR)
                } else if pig {
                    (SHEEP_HP, poly_const::PIG_COLOR)
                } else {
                    (SHEEP_HP, SHEEP_COLOR)
                };
                ec.insert(sfx::PolymorphedModifier::new(
                    duration, hp_current, hp_max, material, mesh, team,
                ));
                ec.insert(comp::Health::new(sheep_hp));
                ec.insert(SheepBounce {
                    base_y,
                    elapsed: 0.0,
                });
                apply_sheep_visual(&mut ec, materials, visual_assets, color);
                if explosive {
                    ec.insert(poly_comp::ExplosiveSheep);
                }
                if permanent {
                    ec.insert(poly_comp::PermanentLivestock);
                }
                if pig {
                    ec.insert(poly_comp::PigForm);
                }
                if contagious {
                    let talent_params = poly_comp::PolymorphTalentParams {
                        explosive,
                        contagious: true,
                        pig_form: pig,
                        permanent,
                        dire,
                        ..Default::default()
                    };
                    ec.insert(poly_comp::ContagiousBaas {
                        empowerment,
                        talent_params,
                    });
                }
                if dire {
                    // Dire Sheep is a friendly combatant: keep AttackTiming so it can
                    // headbutt, and put it on the Defenders' side (matches SP cast).
                    ec.insert((
                        poly_comp::DireSheep::new(),
                        comp::Team::Defenders,
                        comp::AttackTiming::new(),
                    ));
                } else {
                    ec.remove::<comp::AttackTiming>();
                }
            } else {
                ec.insert(sfx::SleepModifier::new(duration, 1.0));
            }
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
            // ActiveMarkOfDeath drives the floating doom-skull indicator
            // (`spawn_mark_indicators` / `update_mark_indicators` both query
            // `With<ActiveMarkOfDeath>`). The SP cast path inserts it via
            // `apply_mark_of_death`; the MP forwarded path bypasses that
            // helper, so we add it here so the host renders the indicator.
            ec.insert(
                crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath,
            );
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
            ec.insert(sfx::BattleHymnModifier::new(
                damage_bonus,
                attack_speed,
                duration,
            ));
        }
        K::BerserkerRage => {
            // magnitude = damage_bonus, vulnerability in flags low 16 as percent.
            let damage_bonus = if magnitude > 0.0 { magnitude } else { 1.0 };
            let vulnerability = ((flags & 0xFFFF) as f32) / 10_000.0;
            ec.insert(sfx::BerserkerRageModifier::new(
                damage_bonus,
                vulnerability,
                duration,
            ));
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
