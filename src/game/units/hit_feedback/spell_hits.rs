//! Turns `PendingSpellHit` markers into a colored flash and a throttled sound.
//!
//! Two independent gates, because they solve different problems:
//! - [`SpellHitCooldown`] is per-unit, so one unit can't strobe under a
//!   continuous beam (Disintegrate ticks every 0.1s; Absolute Zero every frame).
//! - [`HitSfxBudget`] is global, so a 40-target Meteor Fall plays one louder
//!   thump rather than 40 overlapping voices.

use bevy::prelude::*;

use super::super::components::{Hitbox, PendingSpellHit};
use super::super::damage::DamageType;
use super::super::wizard::spells::audio::{self, SpellSfxAssets};
use super::flash::{HIT_FLASH_VFX_DURATION, HitFlash, VFX_SCALE};
use crate::config::{GameConfig, WizardType};
use crate::game::pathfinding::StagingAttacker;

/// Gates how often one unit may produce hit feedback.
#[derive(Component)]
pub(crate) struct SpellHitCooldown {
    remaining: f32,
}

impl SpellHitCooldown {
    /// The field is private and the Warglock / Swordcerer call sites live in
    /// other modules, so they need this.
    pub(crate) const fn new(secs: f32) -> Self {
        Self { remaining: secs }
    }

    /// Ticks down; returns true once expired. Drives `update_timed_modifier`.
    pub(crate) fn update(&mut self, delta: f32) -> bool {
        self.remaining -= delta;
        self.remaining <= 0.0
    }
}

crate::game::units::components::impl_timed_modifier!(SpellHitCooldown);

/// Timestamp (`Time::elapsed_secs`) of the last hit sound.
///
/// A timestamp rather than a countdown on purpose: the system that owns it is
/// gated on `any_with_component::<PendingSpellHit>`, so it does not run on
/// quiet frames. A decremented counter would go stale and a lone hit arriving
/// after a lull would find a non-zero cooldown, skip its sound, and never
/// retry — the marker is consumed that same frame.
#[derive(Resource, Default)]
pub(crate) struct HitSfxBudget {
    last_played: f32,
}

/// Minimum gap between hit sounds, across the whole battlefield.
const HIT_SFX_MIN_INTERVAL: f32 = 0.09;
/// Per-unit gap between flashes.
const HIT_FEEDBACK_INTERVAL: f32 = 0.22;
/// Longer gate when the player has `reduce_flashes` on.
const HIT_FEEDBACK_INTERVAL_REDUCED: f32 = 0.5;
/// Ceiling on overlays spawned in one frame. Not a throughput concern — the
/// overlays are cheap — but many additive blend quads at once read as a
/// screen-wide wash, which is the actual photosensitivity risk.
const MAX_FLASHES_PER_FRAME: u32 = 24;
/// Halved ceiling when `reduce_flashes` is on.
const MAX_FLASHES_PER_FRAME_REDUCED: u32 = 12;
/// Target count at which the hit sound reaches full volume.
const FULL_VOLUME_HIT_COUNT: f32 = 30.0;

/// Volume scale for a burst that landed on `hits` units.
///
/// Takes the count of units that passed the per-unit cooldown, NOT the count
/// that actually flashed — the flash budget is a visual cap, and feeding it in
/// here would mean `reduce_flashes` (halving that cap) also quietly turned the
/// hit sound down.
fn hit_sfx_volume_scale(hits: u32) -> f32 {
    (0.4 + 0.6 * (hits as f32 / FULL_VOLUME_HIT_COUNT).sqrt()).min(1.0)
}

/// Reads every `PendingSpellHit`, flashes the unit, and plays at most one
/// throttled sound for the whole batch.
///
/// This system is the *sole* remover of `PendingSpellHit`, which is what makes
/// it safe to leave unordered: whenever the deferred insert lands, this sees it
/// exactly once. See the `PendingSpellHit` docs for why reusing
/// `PendingDamageEffect` instead does not work.
pub(crate) fn drive_spell_hit_feedback(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    sfx: Res<SpellSfxAssets>,
    mut budget: ResMut<HitSfxBudget>,
    // No `Without<Corpse>`: a unit killed by a spell becomes a Corpse in
    // PostCombatSet the same frame the damage lands, so filtering corpses out
    // would mean the killing blow — the hit most worth seeing — never flashes.
    hits: Query<(
        Entity,
        &PendingSpellHit,
        &Transform,
        Option<&Hitbox>,
        Has<SpellHitCooldown>,
        Has<StagingAttacker>,
    )>,
) {
    let reduced = config.reduce_flashes;
    let interval = if reduced {
        HIT_FEEDBACK_INTERVAL_REDUCED
    } else {
        HIT_FEEDBACK_INTERVAL
    };
    let max_flashes = if reduced {
        MAX_FLASHES_PER_FRAME_REDUCED
    } else {
        MAX_FLASHES_PER_FRAME
    };
    // `process_pending_damage_effects` reskins every damage type to Poop for an
    // Excremage; match it so the flash isn't orange while the kit is brown.
    let excremage = config.wizard_type == WizardType::Excremage;

    let listener = audio::audio_origin();
    let mut nearest = Vec3::ZERO;
    let mut nearest_d2 = f32::MAX;
    // Flashes actually spawned, bounded by `max_flashes`.
    let mut flashed = 0_u32;
    // Hits that passed the per-unit gate, unbounded. Kept separate from
    // `flashed` so the visual cap — and therefore `reduce_flashes` — cannot
    // quietly turn the hit sound down.
    let mut audible_hits = 0_u32;

    for (entity, pending, transform, hitbox, on_cooldown, is_staging) in &hits {
        // Consume the marker FIRST, unconditionally. Nothing else removes it,
        // so a marker skipped here would linger forever and keep this system's
        // `any_with_component` run condition true for the rest of the match.
        //
        // `try_remove` / `try_insert` rather than `remove` / `insert`: the
        // target may be despawned between this query read and the command
        // flush, where plain `insert` panics and plain `remove` warns.
        // `Commands::get_entity` is not a guard against that — it only checks
        // entity metadata, which still holds queued-but-unapplied despawns.
        let mut entity_commands = commands.entity(entity);
        entity_commands.try_remove::<PendingSpellHit>();

        if on_cooldown || is_staging {
            continue;
        }

        audible_hits += 1;

        // Nearest-to-listener rather than a centroid: for a beam sweeping two
        // ends of the field the mean position is somewhere nothing happened.
        let d2 = transform.translation.distance_squared(listener);
        if d2 < nearest_d2 {
            nearest_d2 = d2;
            nearest = transform.translation;
        }

        if flashed >= max_flashes {
            continue;
        }

        let damage_type = if excremage {
            DamageType::Poop
        } else {
            pending.0
        };
        entity_commands.try_insert((
            HitFlash {
                timer: HIT_FLASH_VFX_DURATION,
                damage_type,
                // Resolved now, while the target still has its Hitbox — death
                // strips it the same frame, and a boss killing blow would
                // otherwise render at the infantry-sized fallback.
                base_scale: hitbox.map_or(VFX_SCALE, |h| h.radius),
            },
            SpellHitCooldown::new(interval),
        ));
        flashed += 1;
    }

    if audible_hits == 0 {
        return;
    }

    // One sound per burst, louder with more targets — the same philosophy as
    // `update_battle_ambience`, which scales a single loop by melee-unit count
    // instead of playing a sound per swing.
    let now = time.elapsed_secs();
    if now - budget.last_played < HIT_SFX_MIN_INTERVAL {
        return;
    }
    let volume_scale = hit_sfx_volume_scale(audible_hits);

    // Stamp the budget only when a sound actually starts. `play_sfx_scaled`
    // drops fully-attenuated sounds, and claiming the window for an inaudible
    // hit would silence an audible one arriving right behind it.
    if audio::sfx_would_be_audible(nearest, &config, volume_scale) {
        // `play_sfx_scaled`, NOT `play_impact_sfx_scaled` — the impact variant
        // substitutes `grease_cast` (a full-length cast sound) for an
        // Excremage, which at hit frequency would be unlistenable.
        audio::play_sfx_scaled(
            &mut commands,
            &sfx.spell_hit,
            nearest,
            &config,
            volume_scale,
        );
        budget.last_played = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A single hit must still be clearly audible — the whole point of the
    /// feature is telling the player one spell connected.
    #[test]
    fn a_lone_hit_is_audible_but_not_full_volume() {
        let one = hit_sfx_volume_scale(1);
        assert!(one >= 0.4, "a single hit should not be near-silent: {one}");
        assert!(one < 1.0, "a single hit should leave headroom: {one}");
    }

    /// Louder with more targets, and it must actually *reach* full volume.
    #[test]
    fn volume_rises_with_target_count_and_saturates() {
        assert!(hit_sfx_volume_scale(1) < hit_sfx_volume_scale(10));
        assert!(hit_sfx_volume_scale(10) < hit_sfx_volume_scale(30));
        assert_eq!(hit_sfx_volume_scale(FULL_VOLUME_HIT_COUNT as u32), 1.0);
        assert_eq!(hit_sfx_volume_scale(200), 1.0);
    }

    /// Regression: the curve is driven by hits that passed the per-unit
    /// cooldown, not by the per-frame *flash* cap. If someone re-plumbs it to
    /// the flash budget, full volume becomes unreachable — and turning on
    /// `reduce_flashes` (a visual accessibility setting) would silently quieten
    /// the game's audio too.
    #[test]
    fn full_volume_is_reachable_past_both_flash_caps() {
        // Full volume sits above both caps, so if the curve were ever fed the
        // flash count it could never reach 1.0 — these two assertions are what
        // detect that mistake.
        assert!(hit_sfx_volume_scale(MAX_FLASHES_PER_FRAME) < 1.0);
        assert!(hit_sfx_volume_scale(MAX_FLASHES_PER_FRAME_REDUCED) < 1.0);
        // ...and with the uncapped audio count it does.
        assert_eq!(hit_sfx_volume_scale(FULL_VOLUME_HIT_COUNT as u32), 1.0);
    }
}
