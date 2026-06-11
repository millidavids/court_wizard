use bevy::prelude::*;

use crate::game::units::archer::Archer;
use crate::game::units::archer::components::Arrow;
use crate::game::units::components::{
    Corpse, FireDoT, FrostAccumulation, Health, KingsGuard, Shocked, Team,
};
use crate::game::units::king::components::{King, SpellShield};
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{ArrowSnapshot, GameSnapshot, SnapshotTick, build_unit_snapshot};

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
        // Grouped into a nested tuple to stay within Bevy's query-data arity
        // limit (the flat tuple was already at the maximum).
        (
            Has<crate::game::units::components::CombatAnimation>,
            Has<crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath>,
            Has<crate::game::units::status_effects::PoisonedModifier>,
            Has<crate::game::units::status_effects::PolymorphedModifier>,
            Has<crate::game::units::status_effects::SmellyModifier>,
            Has<crate::game::units::wizard::archetypes::swordcerer::components::SwordcererAvatar>,
            Has<crate::game::units::components::InMelee>,
        ),
    )>,
    arrows: Query<&Transform, With<Arrow>>,
    kill_stats: Res<crate::game::resources::KillStats>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
        arrows: Vec::with_capacity(arrows.iter().len()),
        // Ship the host's authoritative match clock so the guest's HUD clock
        // matches exactly instead of free-running on its own timer.
        host_elapsed_secs: kill_stats.elapsed_time,
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
        (
            has_combat_animation,
            has_mark,
            has_poison,
            has_polymorph,
            has_smelly,
            has_swordcerer_avatar,
            has_in_melee,
        ),
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
            has_mark,
            has_poison,
            has_polymorph,
            has_smelly,
            has_swordcerer_avatar,
            has_in_melee,
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
