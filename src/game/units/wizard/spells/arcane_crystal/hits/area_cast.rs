//! Absorption for spells with no projectile to intercept.
//!
//! One reader for every ground-cast zone, instant buff, and single-target cast.
//! The five detectors alongside this file poll for moving projectiles and beams,
//! which genuinely have to be caught in flight; everything else announces itself
//! through [`SpellAreaCastMessage`] at cast completion.

use bevy::prelude::*;

use super::super::auto::crystal_aoe_burst;
use super::super::components::*;
use super::super::constants::*;
use super::super::infusions::{CrystalCharge, CrystalInfusion, CrystalWarded, apply_modifier};
use super::super::setup::{AbsorptionBookkeeping, absorb_into_crystal, destroy_crystal};
use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::messages::SpellAreaCastMessage;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use std::collections::HashSet;

/// Absorbs area and single-target casts that land on a crystal.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_area_cast_hits(
    mut commands: Commands,
    mut messages: MessageReader<SpellAreaCastMessage>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — the ghost copy of the remote
    // peer's crystal is excluded so one absorption never fires twice.
    mut crystals: Query<
        (
            Entity,
            &mut ArcaneCrystal,
            Option<&mut ResonanceCascade>,
            Option<&mut CrystalWarded>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    // A crystal despawn is deferred to the end of the frame, so without this a
    // second dispel impact in the same frame would shatter the same crystal
    // again and deal double damage.
    let mut shattered: HashSet<Entity> = HashSet::new();

    for message in messages.read() {
        let Some(charge) = CrystalInfusion::from_spell(message.spell) else {
            continue;
        };

        for (crystal_entity, mut crystal, mut resonance, mut warded) in &mut crystals {
            if crystal.permanent || shattered.contains(&crystal_entity) {
                continue;
            }
            let reach = message.radius + crystal.collision_radius;
            if xz_distance(crystal.position, message.position) > reach {
                continue;
            }

            match charge {
                CrystalCharge::Infuse(infusion) => {
                    absorb(
                        &mut commands,
                        &mut crystal,
                        &mut resonance,
                        &mut progress,
                        &mut game_rng.0,
                        infusion,
                        message.empowerment,
                    );
                }
                CrystalCharge::InfuseWithModifier(infusion, modifier) => {
                    absorb(
                        &mut commands,
                        &mut crystal,
                        &mut resonance,
                        &mut progress,
                        &mut game_rng.0,
                        infusion,
                        message.empowerment,
                    );
                    apply_modifier(&mut commands, crystal_entity, modifier);
                }
                CrystalCharge::ModifierOnly(modifier) => {
                    // Haste: keep whatever the crystal already projects and only
                    // change how fast it projects it.
                    apply_modifier(&mut commands, crystal_entity, modifier);
                    crystal.mark_absorption();
                }
                CrystalCharge::Shatter => {
                    // Guardian Circle's ward covers dispel too — the component's
                    // whole purpose is absorbing destruction attempts.
                    if let Some(ward) = warded.as_deref_mut()
                        && ward.absorb()
                    {
                        crystal.mark_absorption();
                        continue;
                    }
                    shattered.insert(crystal_entity);
                    shatter(
                        &mut commands,
                        &visual_assets,
                        crystal_entity,
                        &crystal,
                        &indicators,
                        &targets,
                        &mut health_query,
                        caster_team,
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn absorb(
    commands: &mut Commands,
    crystal: &mut ArcaneCrystal,
    resonance: &mut Option<Mut<ResonanceCascade>>,
    progress: &mut BattleTalentProgress,
    rng: &mut impl rand::Rng,
    infusion: CrystalInfusion,
    cast_empowerment: f32,
) {
    // An empowered cast charges the crystal harder, so the archetypes that scale
    // spell power get more out of the crystal too.
    let base_count =
        ((infusion.base_count() as f32 * cast_empowerment.max(0.0)).round() as usize).max(1);

    if absorb_into_crystal(
        commands,
        crystal,
        resonance,
        progress,
        rng,
        infusion,
        base_count,
        AbsorptionBookkeeping::DISCRETE,
    )
    .is_some()
    {
        // The infusion's own system fires the burst next frame.
        crystal.infusion_burst_pending = true;
    }
}

/// Dispel detonates the crystal rather than quietly deleting it. Damage scales
/// with how much lifetime was left, so shattering a fresh crystal is the payoff
/// and shattering a spent one is nearly free.
#[allow(clippy::too_many_arguments)]
fn shatter(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    crystal_entity: Entity,
    crystal: &ArcaneCrystal,
    indicators: &Query<(Entity, &CrystalRangeIndicator)>,
    targets: &Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    caster_team: Team,
) {
    let remaining = if crystal.duration > 0.0 {
        (1.0 - crystal.time_alive / crystal.duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let damage = SHATTER_BASE_DAMAGE * remaining * crystal.empowerment;

    crystal_aoe_burst(
        commands,
        visual_assets,
        crystal.position,
        crystal.range,
        damage,
        SHATTER_RADIUS,
        3.0,
        0.5,
        targets,
        health_query,
        caster_team,
    );

    destroy_crystal(commands, crystal_entity, crystal, indicators);
}
