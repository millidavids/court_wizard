use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::{
    Corpse, Health, RemoteBattleHymnEffect, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, RemoteHasteEffect, RemoteHealingEffect, RemotePoisonEffect,
    RemotePolymorphEffect, RemoteRageEffect, RemoteTempHpEffect,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::SpellShield;
use crate::game::units::king::resources::KingAssets;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{UnitFlags, u8_to_team};

use super::super::super::components::{GhostArrow, GhostEntity};

use super::arrows::sync_ghost_arrows;
use super::despawn::despawn_stale_ghosts;
use super::effect_flags::{GhostMarkerState, RemoteEffectFlags};
use super::packet::{filter_latest_game_snapshot, parse_game_snapshot};
use super::spawn::spawn_ghost_entity;
use super::update::update_ghost_entity;

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
    swordcerer_assets: Res<
        crate::game::units::wizard::archetypes::swordcerer::resources::SwordcererAssets,
    >,
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
            // Nested tuple: the flat tuple is at Bevy's query-data arity limit.
            (
                Has<RemoteRageEffect>,
                Has<RemoteBattleHymnEffect>,
                Has<RemoteTempHpEffect>,
                Has<RemoteHasteEffect>,
                Has<RemoteHealingEffect>,
            ),
        ),
        With<GhostEntity>,
    >,
    ghost_arrows: Query<Entity, With<GhostArrow>>,
    // Separate query for the smelly tint so the main ghost query stays under
    // Bevy's query-data arity limit.
    smelly_ghosts: Query<Entity, With<crate::game::units::status_effects::SmellyModifier>>,
    // Separate query so the guest can mirror the host's `InMelee` flag onto ghosts
    // (drives the battle-ambience melee sound) without widening the main query.
    melee_ghosts: Query<Entity, With<crate::game::units::components::InMelee>>,
    mut kill_stats: ResMut<crate::game::resources::KillStats>,
) {
    let Some(game_bytes) = filter_latest_game_snapshot(&mut connection) else {
        return;
    };

    // Velocities are now host-authoritative (shipped in `UnitSnapshot.vx/vz`);
    // between snapshots the last written value persists, which is exactly
    // what we want — animations stay smooth during brief packet gaps and
    // wind down naturally when the host's next snapshot reports zero.
    let Some(snapshot) = parse_game_snapshot(&game_bytes) else {
        return;
    };

    // Mirror the host's authoritative match clock so the guest's HUD clock
    // matches the host exactly. This system runs only on the guest, so it never
    // touches the host's own KillStats.
    kill_stats.elapsed_time = snapshot.host_elapsed_secs;

    // Track which IDs are present in this snapshot
    let mut seen_ids = HashSet::with_capacity(snapshot.units.len());

    for unit in &snapshot.units {
        seen_ids.insert(unit.id);

        let is_corpse = unit.flags & UnitFlags::CORPSE != 0;
        let is_king = unit.flags & UnitFlags::KING != 0;
        let is_archer = unit.flags & UnitFlags::ARCHER != 0;
        let is_guard = unit.flags & UnitFlags::KINGS_GUARD != 0;
        let is_swordcerer_avatar = unit.flags & UnitFlags::SWORDCERER_AVATAR != 0;
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

        let remote = RemoteEffectFlags::from_flags(unit.flags);

        let existing_local = entity_map.remote_to_local.get(&unit.id).copied();

        if let Some(local_entity) = existing_local {
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
                (
                    has_remote_rage,
                    has_remote_battle_hymn,
                    has_remote_temp_hp,
                    has_remote_haste,
                    has_remote_healing,
                ),
            )) = ghost_query.get_mut(local_entity)
            {
                let state = GhostMarkerState {
                    fire: has_remote_fire,
                    frost: has_remote_frost,
                    electric: has_remote_electric,
                    spell_shield: has_spell_shield,
                    corpse: has_corpse,
                    combat: has_combat,
                    poison: has_remote_poison,
                    mark: has_remote_mark,
                    polymorph: has_remote_polymorph,
                    rage: has_remote_rage,
                    battle_hymn: has_remote_battle_hymn,
                    temp_hp: has_remote_temp_hp,
                    haste: has_remote_haste,
                    healing: has_remote_healing,
                };
                update_ghost_entity(
                    &mut commands,
                    entity,
                    &mut transform,
                    &mut crdt_health,
                    &mut health,
                    &mut velocity,
                    &state,
                    &smelly_ghosts,
                    &melee_ghosts,
                    remote_crdt,
                    pos,
                    unit.vx,
                    unit.vz,
                    is_corpse,
                    is_king,
                    is_archer,
                    is_guard,
                    is_swordcerer_avatar,
                    &remote,
                    team,
                    &infantry_assets,
                    &archer_assets,
                    &king_assets,
                    &undead_assets,
                    &spell_assets,
                    &swordcerer_assets,
                    &mut materials,
                );
            }
        } else {
            let spawned = spawn_ghost_entity(
                &mut commands,
                unit.id,
                remote_crdt,
                pos,
                is_corpse,
                is_king,
                is_archer,
                is_guard,
                is_swordcerer_avatar,
                &remote,
                team,
                &infantry_assets,
                &archer_assets,
                &king_assets,
                &undead_assets,
                &spell_assets,
                &swordcerer_assets,
                &mut materials,
                &mut game_rng,
            );
            entity_map.insert(unit.id, spawned);
        }
    }

    despawn_stale_ghosts(&mut commands, &mut entity_map, &seen_ids);

    sync_ghost_arrows(
        &mut commands,
        &ghost_arrows,
        &archer_assets,
        &snapshot.arrows,
    );
}
