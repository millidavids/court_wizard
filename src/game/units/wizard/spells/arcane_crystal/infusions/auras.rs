//! Support infusions: the crystal projecting a buff across its range.
//!
//! Per the game's design rule that magic is indiscriminate, these reach every
//! unit in range regardless of team. A Battle Hymn crystal placed badly arms the
//! enemy, which is the trade that makes placement matter.
//!
//! Each reuses its parent spell's own `apply_*` helper, so retuning the spell
//! retunes the crystal's version with it.

use bevy::prelude::*;

use super::super::constants::*;
use super::driver::{InfusedCrystals, begin_infusion_tick};
use super::kinds::CrystalInfusion;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::{
    BattleHymnModifier, BerserkerRageModifier, Corpse, HasteModifier, Team, TemporaryHitPoints,
};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::battle_hymn::systems::apply_battle_hymn_buff;
use crate::game::units::wizard::spells::berserker_rage::components::BerserkerRageTalentParams;
use crate::game::units::wizard::spells::berserker_rage::systems::buff_application::apply_berserker_rage_buff;
use crate::game::units::wizard::spells::guardian_circle::constants as gc_constants;
use crate::game::units::wizard::spells::guardian_circle::systems::apply_guardian_circle_buff;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

/// Sustains a war-hymn aura over the crystal's range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn tick_battle_hymn_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BattleHymnModifier>,
            Option<&mut TemporaryHitPoints>,
            Option<&mut HasteModifier>,
        ),
        // Ghosts excluded here, unlike the other status infusions — not a
        // choice, but a requirement: `apply_battle_hymn_buff` declares this
        // exact filter in its own signature, so the crystal path cannot include
        // ghosts without changing the helper the hand-cast spell also uses.
        // That means a Battle Hymn crystal buffs nothing on the guest, matching
        // whatever the hand-cast spell already does there. Fixing it properly
        // means widening the helper, which is a change to Battle Hymn itself.
        (
            Without<Wizard>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut progress: Option<ResMut<BattleTalentProgress>>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::BattleHymn,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        apply_battle_hymn_buff(
            &mut commands,
            params.position,
            params.range,
            params.empowerment * INFUSION_DURATION_SCALE,
            &mut targets,
            &mut progress,
            active_talents.as_deref(),
        );
    }
}

/// Sustains a rage aura over the crystal's range.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_berserker_rage_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BerserkerRageModifier>,
        ),
        (
            Without<Wizard>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::BerserkerRage,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        apply_berserker_rage_buff(
            &mut commands,
            params.position,
            params.range,
            params.empowerment * INFUSION_DURATION_SCALE,
            &BerserkerRageTalentParams::default(),
            &mut targets,
        );
    }
}

/// Refreshes a temporary-hit-point ward over the crystal's range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn tick_guardian_circle_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    mut targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut attacker_hit_msg: MessageWriter<GuardianCircleHitAttackerMessage>,
    mut progress: Option<ResMut<BattleTalentProgress>>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::GuardianCircle,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        // The burst hands out a full ward; ongoing refreshes are half-strength so
        // a parked crystal cannot stack an army to invulnerability.
        //
        // Deliberately *not* multiplied by empowerment here — the helper applies
        // it internally, and doing it twice made an empowered crystal hand out
        // empowerment-squared temporary HP.
        let temp_hp = gc_constants::TEMP_HP_AMOUNT * params.pick(1.0, INFUSION_DURATION_SCALE);
        apply_guardian_circle_buff(
            &mut commands,
            params.position,
            params.range,
            temp_hp,
            gc_constants::TEMP_HP_DURATION,
            params.empowerment,
            &mut targets,
            &mut attacker_hit_msg,
            &mut progress,
            active_talents.as_deref(),
        );
    }
}
