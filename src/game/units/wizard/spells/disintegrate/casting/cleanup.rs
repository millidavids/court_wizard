use crate::game::units::wizard::components::{CastingState, LocalWizard};
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::ChannelingSfx;
use crate::game::units::wizard::spells::disintegrate::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam, DisintegrateParticle,
};
use bevy::prelude::*;

/// System that despawns beams when wizard is not actively casting/channeling disintegrate.
///
/// Checks CastingState directly to avoid deferred command timing issues.
/// Excludes crystal-spawned beams (those with CrystalSpawn) — they're managed by the crystal.
#[allow(clippy::too_many_arguments)]
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
    beam_query: Query<Entity, (With<DisintegrateBeam>, Without<CrystalSpawn>)>,
    glow_query: Query<(Entity, &BeamGlow)>,
    flare_query: Query<(Entity, &BeamOriginFlare)>,
    particle_query: Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: Query<(Entity, &BeamEclipse)>,
    channeling_sfx_query: Query<Entity, With<ChannelingSfx>>,
) {
    if let Ok(casting_state) = wizard_query.single()
        && matches!(casting_state, CastingState::Resting)
    {
        // Collect wizard beam entities for filtering visuals
        let wizard_beams: Vec<Entity> = beam_query.iter().collect();
        for entity in &wizard_beams {
            commands.entity(*entity).try_despawn();
        }
        // Only despawn visuals that belong to wizard beams
        for (entity, glow) in &glow_query {
            if wizard_beams.contains(&glow.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for (entity, flare) in &flare_query {
            if wizard_beams.contains(&flare.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for entity in particle_query.iter() {
            commands.entity(entity).try_despawn();
        }
        for (entity, eclipse) in &eclipse_query {
            if wizard_beams.contains(&eclipse.beam_entity) {
                commands.entity(entity).try_despawn();
            }
        }
        for entity in channeling_sfx_query.iter() {
            commands.entity(entity).try_despawn();
        }
    }
}
