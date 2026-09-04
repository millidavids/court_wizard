use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

use super::team::Team;
use crate::game::units::damage::DamageType;

/// Health component for all units.
///
/// Tracks the current and maximum health of a unit.
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    /// Extra vulnerability to spell damage (0.0 = normal, 0.3 = +30% spell damage taken).
    /// Used by the Psychopath archetype to amplify spell damage against defenders.
    pub spell_vulnerability: f32,
    /// Healing reduction (0.0 = normal, 0.5 = 50% less healing received).
    /// Applied inside heal() so all heal sources are affected automatically.
    pub healing_reduction: f32,
}

impl Health {
    /// Creates a new Health component with the given maximum health.
    ///
    /// Current health starts at the maximum value.
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            spell_vulnerability: 0.0,
            healing_reduction: 0.0,
        }
    }

    /// Returns true if the unit is dead (current health <= 0).
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Takes damage, reducing current health.
    ///
    /// Current health is clamped to not go below 0.
    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// Heals the unit, increasing current health.
    ///
    /// Current health is clamped to not exceed max health.
    pub fn heal(&mut self, amount: f32) {
        let effective = amount * (1.0 - self.healing_reduction).max(0.0);
        self.current = (self.current + effective).min(self.max);
    }
}

/// Temporary hit points that absorb damage before real health.
///
/// Expires after a duration and is consumed before health when taking damage.
/// Multiple applications of temporary HP do not stack - only the maximum is kept.
#[derive(Component)]
pub struct TemporaryHitPoints {
    /// Current amount of temporary HP.
    pub amount: f32,
    /// Time remaining before temp HP expires (in seconds).
    pub time_remaining: f32,
}

impl TemporaryHitPoints {
    /// Creates new temporary hit points with a duration.
    pub const fn new(amount: f32, duration: f32) -> Self {
        Self {
            amount,
            time_remaining: duration,
        }
    }

    /// Absorbs damage from temporary HP, returning overflow damage.
    ///
    /// Returns the amount of damage that wasn't absorbed (overflow to real HP).
    pub fn absorb_damage(&mut self, damage: f32) -> f32 {
        if self.amount >= damage {
            self.amount -= damage;
            0.0 // All damage absorbed
        } else {
            let overflow = damage - self.amount;
            self.amount = 0.0;
            overflow // This much damage overflows to real HP
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0 || self.amount <= 0.0
    }
}

/// Invulnerability status effect — prevents all damage while attached.
/// Health is snapshotted each frame; any damage taken is undone.
#[derive(Component)]
pub struct Invulnerable {
    /// Health value to restore to each frame (damage negation).
    pub health_snapshot: f32,
}

/// Marker component for units that have been damaged by a spell.
/// Used to detect friendly fire for the achievement system.
#[derive(Component)]
pub struct SpellDamaged;

/// Marker component for units that have been damaged by residual fire (ground fire).
/// Used to detect deaths from fireball ground fire for the Scorched Earth achievement.
#[derive(Component)]
pub struct ResidualFireDamaged;

/// One-frame marker requesting hit feedback (colored flash + throttled sound)
/// for a unit that just took spell damage. Carries the damage type so the flash
/// can be tinted with that element's color.
///
/// Deliberately separate from [`PendingDamageEffect`], which carries the same
/// data but has three competing removers (`process_pending_damage_effects`,
/// `forward_spell_hits_to_host`, and corpse cleanup). Because inserts and
/// removes are both deferred through `Commands`, a reader ordered against those
/// consumers would see markers only on some frames — an auto-inserted
/// `ApplyDeferred` barrier flushes the spell systems' inserts past the reader
/// and into the consumer. `drive_spell_hit_feedback` is the sole remover of
/// this marker, so it observes every one exactly once with no ordering edges.
#[derive(Component)]
pub struct PendingSpellHit(pub DamageType);

/// Marker component inserted by `apply_spell_damage` to defer persistent effect stacking.
///
/// A central system (`process_pending_damage_effects`) reads these each frame and
/// creates/stacks the real `FireDoT`, `SlowMovementModifier`, or `Shocked` components.
#[derive(Component)]
pub struct PendingDamageEffect {
    pub damage_type: DamageType,
    pub damage: f32,
    /// Team that cast the spell, when known. Used so a King's `SpellShield` blocks
    /// only ENEMY spell DoTs (`source_team != king_team`) and lets the King's own
    /// team's friendly-fire through. `None` (most call sites) = treated as enemy,
    /// preserving the old block-all behavior.
    pub source_team: Option<Team>,
}

/// Process-wide flag that, while set, makes every unit immune to damage. Driven
/// by [`sync_setup_immunity_flag`] from the multiplayer setup stage. Damage
/// helpers and the few direct-`take_damage` sites early-return when this is set.
///
/// A `static` (rather than a Bevy resource) is used because the damage helpers
/// below are plain free functions called from ~50 sites with no `World`/`Res`
/// access. Only one game runs per process, and the flag is reset on match exit.
static SETUP_IMMUNITY_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Sets the global setup-stage damage-immunity flag.
pub(crate) fn set_setup_immunity(active: bool) {
    SETUP_IMMUNITY_ACTIVE.store(active, Ordering::Relaxed);
}

/// Returns true while units are immune to damage (multiplayer setup stage).
pub(crate) fn is_setup_immune() -> bool {
    SETUP_IMMUNITY_ACTIVE.load(Ordering::Relaxed)
}

/// Updates [`SETUP_IMMUNITY_ACTIVE`] each frame from the multiplayer setup-stage
/// condition. Registered under `resource_exists::<MultiplayerSession>` so it only
/// runs in multiplayer; the flag is additionally reset on match exit so it never
/// leaks into a later single-player game.
pub fn sync_setup_immunity_flag(
    kill_stats: Res<crate::game::resources::KillStats>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    // Only a VERSUS session has the opening setup-immunity stage. Co-op runs on
    // the SP endless/roguelite battlefield and must never grant blanket immunity.
    let active = session.is_some_and(|s| !s.is_coop())
        && kill_stats.elapsed_time < crate::game::run_conditions::MP_SETUP_DURATION;
    set_setup_immunity(active);
}

/// Applies damage to a unit, absorbing with temporary HP first.
///
/// This function should be used instead of directly calling `health.take_damage()`
/// when temporary hit points should be respected. Damage is first absorbed by
/// temporary HP (if present), and any overflow damage is applied to real health.
///
/// # Arguments
///
/// * `health` - The unit's Health component
/// * `temp_hp` - Optional TemporaryHitPoints component
/// * `damage` - Amount of damage to apply
pub fn apply_damage_to_unit(
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
) {
    // Multiplayer setup stage: units are immune to all damage.
    if is_setup_immune() {
        return;
    }
    let overflow = if let Some(temp) = temp_hp {
        temp.absorb_damage(damage)
    } else {
        damage
    };

    health.take_damage(overflow);
}

/// Applies spell damage to a unit and inserts a `PendingDamageEffect` marker
/// so the persistent effect system can stack the appropriate effect.
///
/// This replaces the pattern of calling `apply_damage_to_unit()` + inserting
/// `SpellDamaged` manually at each spell call site.
pub fn apply_spell_damage(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
    damage_type: DamageType,
    has_spell_shield: bool,
) {
    apply_spell_damage_inner(
        commands,
        entity,
        health,
        temp_hp,
        damage,
        damage_type,
        has_spell_shield,
        None,
    );
}

/// Team-aware spell damage: a King's `SpellShield` blocks the hit ONLY when the
/// spell comes from the enemy team (`caster_team != target_team`). The King's own
/// team's friendly fire still lands. Records `caster_team` on the
/// `PendingDamageEffect` so its damage-over-time survives the shield check in
/// `process_pending_damage_effects` too. Use this at area spells that can hit a
/// same-team King; single-target enemy-seeking spells can keep `apply_spell_damage`.
#[allow(clippy::too_many_arguments)]
pub fn apply_spell_damage_with_team(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
    damage_type: DamageType,
    has_spell_shield: bool,
    caster_team: Team,
    target_team: Team,
) {
    let blocked = has_spell_shield && caster_team != target_team;
    apply_spell_damage_inner(
        commands,
        entity,
        health,
        temp_hp,
        damage,
        damage_type,
        blocked,
        Some(caster_team),
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_spell_damage_inner(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    temp_hp: Option<&mut TemporaryHitPoints>,
    damage: f32,
    damage_type: DamageType,
    blocked: bool,
    source_team: Option<Team>,
) {
    if blocked {
        return;
    }
    // Multiplayer setup stage: skip damage AND its downstream DoT/tally effects so
    // no damage-over-time is pre-applied to the frozen, immune armies.
    if is_setup_immune() {
        return;
    }
    let modified_damage = damage * (1.0 + health.spell_vulnerability);
    apply_damage_to_unit(health, temp_hp, modified_damage);
    commands.entity(entity).insert((
        SpellDamaged,
        PendingDamageEffect {
            damage_type,
            damage: modified_damage,
            source_team,
        },
        // Drives the colored hit flash + throttled hit sound.
        PendingSpellHit(damage_type),
    ));
    // One-frame marker for the multiplayer score screen's per-wizard spell-damage
    // tally. Only `apply_spell_damage` inserts this, so on each peer it captures
    // exactly the local wizard's output. `accumulate_wizard_spell_stats` sums and
    // removes it every frame (a pure cleanup pass in single-player). `entry` +
    // `and_modify` accumulates multiple hits on the same unit within one frame
    // (e.g. overlapping Meteor/Squall explosions) instead of overwriting.
    commands
        .entity(entity)
        .entry::<crate::game::units::spell_stats::SpellDamageTally>()
        .and_modify(move |mut tally| tally.0 += modified_damage)
        .or_insert(crate::game::units::spell_stats::SpellDamageTally(
            modified_damage,
        ));
}
