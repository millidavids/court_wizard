use bevy::math::Affine2;
use bevy::prelude::*;

use super::constants::{
    ELECTRIC_ARC_CHANCE_PER_DAMAGE, ELECTRIC_ARC_CHANCE_PER_HIT, ELECTRIC_ARC_COOLDOWN,
    ELECTRIC_ARC_DURATION, ELECTRIC_ARC_MAX_CHANCE, FIRE_DOT_DAMAGE_RATIO, FIRE_DOT_DURATION,
    FIRE_DOT_MAX_DPS, FIRE_DOT_TICK_INTERVAL,
};
use super::damage::DamageType;

/// All trackable unit types in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitType {
    // Defenders (default unlocked)
    Infantry,
    Archer,
    King,
    KingsGuard,
    // Attackers (unlocked on encounter)
    Brute,
    Elite,
    Commander,
    Healer,
    Dispeller,
    Shielder,
    Assassin,
    Aerialist,
    // Bosses
    Hag,
    Ogre,
    Lich,
}

impl UnitType {
    /// Returns all unit type variants.
    pub fn all() -> &'static [UnitType] {
        &[
            UnitType::Infantry,
            UnitType::Archer,
            UnitType::King,
            UnitType::KingsGuard,
            UnitType::Brute,
            UnitType::Elite,
            UnitType::Commander,
            UnitType::Healer,
            UnitType::Dispeller,
            UnitType::Shielder,
            UnitType::Assassin,
            UnitType::Aerialist,
            UnitType::Hag,
            UnitType::Ogre,
            UnitType::Lich,
        ]
    }

    /// Display name for the UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            UnitType::Infantry => "Infantry",
            UnitType::Archer => "Archer",
            UnitType::King => "King",
            UnitType::KingsGuard => "King's Guard",
            UnitType::Brute => "Brute",
            UnitType::Elite => "Elite",
            UnitType::Commander => "Commander",
            UnitType::Healer => "Healer",
            UnitType::Dispeller => "Dispeller",
            UnitType::Shielder => "Shielder",
            UnitType::Assassin => "Assassin",
            UnitType::Aerialist => "Aerialist",
            UnitType::Hag => "Hag",
            UnitType::Ogre => "Ogre",
            UnitType::Lich => "Lich",
        }
    }

    /// Short description of the unit.
    pub const fn description(&self) -> &'static str {
        match self {
            UnitType::Infantry => "Melee fighters forming the front line of defense.",
            UnitType::Archer => "Ranged soldiers picking off targets from afar.",
            UnitType::King => "The leader you must protect at all costs.",
            UnitType::KingsGuard => "Elite warriors sworn to defend the King.",
            UnitType::Brute => "Heavy-hitting melee fighters that hit like a truck.",
            UnitType::Elite => "Enhanced soldiers with bonus health, damage, and speed.",
            UnitType::Commander => "Officers that buff nearby allies with damage and speed auras.",
            UnitType::Healer => "Support units that restore health to wounded allies.",
            UnitType::Dispeller => "Anti-magic units that remove your spell effects.",
            UnitType::Shielder => "Support units that shield allies from your spells.",
            UnitType::Assassin => "Fast flankers that slip past infantry to strike archers.",
            UnitType::Aerialist => "Flying attackers that swoop over walls and strike from above.",
            UnitType::Hag => "Ancient witches with devastating magical abilities.",
            UnitType::Ogre => "A massive beast that grows stronger as the fight goes on.",
            UnitType::Lich => "An undead sorcerer who grows stronger from death itself.",
        }
    }

    /// Flavor text shown in the compendium.
    pub const fn flavor_text(&self) -> &'static str {
        match self {
            UnitType::Infantry => "They signed up for this. Probably.",
            UnitType::Archer => "They never miss. Except when they do.",
            UnitType::King => {
                "Heavy is the head that wears the crown. Heavier when fireballs are involved."
            }
            UnitType::KingsGuard => {
                "Sworn to protect, trained to intimidate, paid to stand very still."
            }
            UnitType::Brute => "What they lack in strategy, they make up for in sheer mass.",
            UnitType::Elite => {
                "Better than regular soldiers in every measurable way. They won't let you forget it."
            }
            UnitType::Commander => {
                "Barking orders from behind the front line, as tradition demands."
            }
            UnitType::Healer => "The only unit the enemy army actually values. Unfortunately.",
            UnitType::Dispeller => "Your spells mean nothing to them. Take it personally.",
            UnitType::Shielder => {
                "Handing out magical umbrellas like party favors. How thoughtful."
            }
            UnitType::Assassin => {
                "They don't fight fair. That's the whole point."
            }
            UnitType::Aerialist => "Death from above. Way, way above.",
            UnitType::Hag => "Three sisters who share one terrible disposition.",
            UnitType::Ogre => "Started the fight angry. It only gets worse from there.",
            UnitType::Lich => "Every fallen soldier is just another name on his roster.",
        }
    }

    /// Whether this unit is unlocked by default (defenders).
    pub const fn is_default_unlocked(&self) -> bool {
        matches!(
            self,
            UnitType::Infantry | UnitType::Archer | UnitType::King | UnitType::KingsGuard
        )
    }

    /// Team label for display.
    pub const fn team_label(&self) -> &'static str {
        match self {
            UnitType::Infantry | UnitType::Archer | UnitType::King | UnitType::KingsGuard => {
                "Defender"
            }
            UnitType::Brute
            | UnitType::Elite
            | UnitType::Commander
            | UnitType::Healer
            | UnitType::Dispeller
            | UnitType::Shielder
            | UnitType::Assassin
            | UnitType::Aerialist => "Attacker",
            UnitType::Hag | UnitType::Ogre | UnitType::Lich => "Boss",
        }
    }

    /// Locked description hint shown when the unit hasn't been encountered.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            UnitType::Infantry => "A common defender.",
            UnitType::Archer => "A ranged defender.",
            UnitType::King => "The one you protect.",
            UnitType::KingsGuard => "Royal bodyguards.",
            UnitType::Brute => "Something big is coming...",
            UnitType::Elite => "The enemy is adapting.",
            UnitType::Commander => "Someone is giving orders out there.",
            UnitType::Healer => "The wounded keep getting back up.",
            UnitType::Dispeller => "Your magic feels weaker somehow.",
            UnitType::Shielder => "Something is protecting the enemy.",
            UnitType::Assassin => "Shadows move faster than they should.",
            UnitType::Aerialist => "Something circles overhead.",
            UnitType::Hag => "Dark magic stirs in the distance.",
            UnitType::Ogre => "The ground trembles.",
            UnitType::Lich => "The dead whisper of a master.",
        }
    }
}

/// Trait for modifier components with a timed duration that can expire.
///
/// Implementing this trait allows use of the generic `update_timed_modifier::<T>` system
/// which automatically ticks and removes expired modifiers.
pub trait TimedModifier {
    /// Tick the modifier's timer by `delta` seconds. Returns `true` if expired.
    fn tick(&mut self, delta: f32) -> bool;
}

macro_rules! impl_timed_modifier {
    ($($ty:ty),* $(,)?) => {
        $(impl TimedModifier for $ty {
            fn tick(&mut self, delta: f32) -> bool {
                self.update(delta)
            }
        })*
    };
}

impl_timed_modifier!(
    SlowMovementModifier,
    FrostEffectMarker,
    TemporaryHitPoints,
    RootedModifier,
    HasteModifier,
    MarkedForDeathModifier,
    SleepModifier,
    // BattleHymnModifier has a custom tick system (handles EchoingSong)
    BerserkerRageModifier,
    FogEvasionModifier,
    FrozenSolidModifier,
    Stunned,
);

/// Team component for all units.
///
/// Determines which side a unit is on. Units attack members of opposing teams.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Defenders,
    Attackers,
    Undead,
}

impl Team {
    /// Returns true if units on these two teams are hostile to each other.
    /// Undead are hostile to everyone (including other Undead is false).
    pub fn is_enemy(&self, other: &Team) -> bool {
        match (self, other) {
            (Team::Undead, Team::Undead) => false,
            (Team::Undead, _) | (_, Team::Undead) => true,
            _ => self != other,
        }
    }
}

/// Health component for all units.
///
/// Tracks the current and maximum health of a unit.
#[derive(Component)]
#[allow(dead_code)]
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

/// Movement speed component for all units.
///
/// Determines how fast a unit moves in units per second.
#[derive(Component, Clone, Copy)]
pub struct MovementSpeed(pub f32);

/// Damage bonus as a percentage.
///
/// Used by special units and buffs to modify damage output.
/// Examples: 0.5 = +50% damage, 1.0 = +100% damage (double), -0.4 = -40% damage.
/// Combat system applies this as: damage * (1.0 + percentage).
#[derive(Component)]
pub struct DamageMultiplier(pub f32);

/// Movement speed modifier from Commander aura as a percentage.
///
/// Applied to units within a Commander's aura range.
/// Examples: 0.25 = +25% speed from commander aura.
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct CommanderAuraSpeedModifier(pub f32);

/// Movement speed modifier from rough terrain as a percentage.
///
/// Applied to units walking over corpses.
/// Examples: -0.6 = -60% speed (0.4x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct RoughTerrainModifier(pub f32);

/// Unified movement speed slow modifier.
///
/// Replaces the separate FrostSlowModifier, SpikeGrowthSlowModifier, and
/// GreaseSlipModifier. Uses strongest-wins semantics: when a new slow is
/// applied, the stronger modifier and longer duration are kept.
#[derive(Component)]
pub struct SlowMovementModifier {
    /// Speed reduction as a percentage (negative value, e.g., -0.4 = 40% slower).
    pub modifier: f32,
    /// Time remaining before the slow effect expires (in seconds).
    pub time_remaining: f32,
}

impl SlowMovementModifier {
    /// Creates a new slow modifier with the given strength and duration.
    pub const fn new(modifier: f32, duration: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    /// Apply a new slow. Keeps the stronger modifier and longer duration.
    pub fn apply(&mut self, modifier: f32, duration: f32) {
        if modifier < self.modifier {
            self.modifier = modifier;
        }
        if duration > self.time_remaining {
            self.time_remaining = duration;
        }
    }
}

/// Marker component for the frost visual tint effect.
///
/// Separate from SlowMovementModifier because the frost blue tint should only
/// appear for frost-sourced slows, not for spike growth or grease slows.
#[derive(Component)]
pub struct FrostEffectMarker {
    /// Time remaining before the visual effect expires (in seconds).
    pub time_remaining: f32,
}

impl FrostEffectMarker {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    /// Refresh the marker, keeping the longer duration.
    pub fn apply(&mut self, duration: f32) {
        if duration > self.time_remaining {
            self.time_remaining = duration;
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Attack timing component for all units.
///
/// Tracks when in the global attack cycle a unit can attack.
/// Units attack at a specific time offset (0.0 to cycle_duration) and can only
/// attack again when the global timer cycles back to that offset. This naturally
/// staggers attacks across all units.
#[derive(Component)]
pub struct AttackTiming {
    /// The time offset in the cycle when this unit last attacked, or None if never attacked
    pub last_attack_time: Option<f32>,
}

impl AttackTiming {
    /// Creates a new AttackTiming with no attack scheduled.
    pub const fn new() -> Self {
        Self {
            last_attack_time: None,
        }
    }

    /// Returns true if the unit can attack at the current cycle time.
    /// Units can attack if they haven't attacked yet, or if the cycle has come back
    /// around to their attack time.
    pub fn can_attack(&self, current_time: f32, last_time: f32) -> bool {
        match self.last_attack_time {
            None => true, // Never attacked, can attack immediately
            Some(attack_time) => {
                // Check if we've cycled past the attack time
                // Handle wrap-around: if current < last, we wrapped around
                if current_time < last_time {
                    // We wrapped, check if attack_time is in the wrapped portion
                    attack_time >= last_time || attack_time <= current_time
                } else {
                    // Normal case: check if we're in the window since last update
                    attack_time >= last_time && attack_time <= current_time
                }
            }
        }
    }

    /// Records that the unit attacked at this time offset in the cycle.
    pub fn record_attack(&mut self, current_time: f32) {
        self.last_attack_time = Some(current_time);
    }
}

/// Hitbox component for all units.
///
/// Represents a cylindrical collision volume for the unit.
/// The cylinder's radius determines the width of the billboard sprite,
/// and the height determines the sprite's height. The cylinder provides
/// depth for 3D collision detection while the billboard renders at the center.
#[derive(Component, Clone, Copy)]
pub struct Hitbox {
    /// Radius of the cylinder (determines sprite width).
    pub radius: f32,
    /// Height of the cylinder (determines sprite height).
    pub height: f32,
}

#[allow(dead_code)]
impl Hitbox {
    /// Creates a new Hitbox with the given radius and height.
    pub const fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }

    /// Returns the sprite width (diameter of the cylinder).
    pub fn sprite_width(&self) -> f32 {
        self.radius * 2.0
    }

    /// Returns the sprite height.
    pub fn sprite_height(&self) -> f32 {
        self.height
    }

    /// Checks if this hitbox overlaps with another hitbox in 3D space.
    ///
    /// Uses cylinder-cylinder collision detection:
    /// - Checks distance in XZ plane (horizontal) against combined radii
    /// - Checks overlap in Y axis (vertical) based on heights and positions
    pub fn overlaps(&self, self_pos: Vec3, other: &Hitbox, other_pos: Vec3) -> bool {
        // Check horizontal distance (XZ plane)
        let horizontal_dist_sq =
            (self_pos.x - other_pos.x).powi(2) + (self_pos.z - other_pos.z).powi(2);
        let combined_radius = self.radius + other.radius;

        if horizontal_dist_sq > combined_radius.powi(2) {
            return false; // Too far apart horizontally
        }

        // Check vertical overlap (Y axis)
        // Assuming positions are at the base of the cylinders
        let self_top = self_pos.y + self.height;
        let self_bottom = self_pos.y;
        let other_top = other_pos.y + other.height;
        let other_bottom = other_pos.y;

        // Cylinders overlap vertically if one's bottom is below the other's top
        self_bottom < other_top && other_bottom < self_top
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

#[allow(dead_code)]
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
    let overflow = if let Some(temp) = temp_hp {
        temp.absorb_damage(damage)
    } else {
        damage
    };

    health.take_damage(overflow);
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

/// Stores the original shared material handle before persistent effect tinting.
///
/// Inserted when a persistent damage effect (FireDoT, FrostEffectMarker, ElectricCharge)
/// is first applied to a unit. The unit's MeshMaterial3d is replaced with a cloned
/// per-entity material that can be safely tinted without affecting other units.
/// When all effects expire, the original material is restored and this component is removed.
#[derive(Component)]
pub struct OriginalMaterial(pub Handle<StandardMaterial>);

/// Visual-only markers for status effects active on the remote peer.
///
/// These are inserted/removed based on network snapshot flags so that
/// `update_persistent_effect_visuals` can tint units without creating
/// damage-ticking DoT components (which would double-count damage via CRDT).
#[derive(Component)]
pub struct RemoteFireEffect;

#[derive(Component)]
pub struct RemoteFrostEffect;

#[derive(Component)]
pub struct RemoteElectricEffect;

/// Marker component inserted by `apply_spell_damage` to defer persistent effect stacking.
///
/// A central system (`process_pending_damage_effects`) reads these each frame and
/// creates/stacks the real `FireDoT`, `SlowMovementModifier`, or `ElectricCharge` components.
#[derive(Component)]
pub struct PendingDamageEffect {
    pub damage_type: DamageType,
    pub damage: f32,
}

/// Fire damage-over-time effect that stacks with repeated fire hits.
///
/// Each fire hit adds a percentage of spell damage as extra DoT DPS.
/// Duration resets on each new fire hit.
#[derive(Component)]
pub struct FireDoT {
    /// Accumulated DoT DPS (grows with each fire hit).
    pub damage_per_tick: f32,
    /// Time remaining before the DoT expires (resets on each fire hit).
    pub time_remaining: f32,
    /// Accumulator for tick timing.
    pub tick_timer: f32,
}

impl FireDoT {
    /// Creates a new FireDoT from the initial fire damage.
    pub fn new(spell_damage: f32) -> Self {
        let dps = (spell_damage * FIRE_DOT_DAMAGE_RATIO).min(FIRE_DOT_MAX_DPS);
        Self {
            damage_per_tick: dps,
            time_remaining: FIRE_DOT_DURATION,
            tick_timer: 0.0,
        }
    }

    /// Stacks additional fire damage and resets the duration.
    pub fn stack(&mut self, spell_damage: f32) {
        self.damage_per_tick =
            (self.damage_per_tick + spell_damage * FIRE_DOT_DAMAGE_RATIO).min(FIRE_DOT_MAX_DPS);
        self.time_remaining = FIRE_DOT_DURATION;
    }

    /// Ticks the DoT timer, returning damage to apply this frame (if any).
    /// Returns `None` if no tick happened, `Some(damage)` if a tick occurred.
    /// Also returns `true` in the second tuple element if the DoT has expired.
    pub fn update(&mut self, delta: f32) -> (Option<f32>, bool) {
        self.time_remaining -= delta;
        if self.time_remaining <= 0.0 {
            return (None, true);
        }

        self.tick_timer += delta;
        if self.tick_timer >= FIRE_DOT_TICK_INTERVAL {
            self.tick_timer -= FIRE_DOT_TICK_INTERVAL;
            let tick_damage = self.damage_per_tick * FIRE_DOT_TICK_INTERVAL;
            (Some(tick_damage), false)
        } else {
            (None, false)
        }
    }
}

/// Electric charge effect that builds arc chance with repeated electric hits.
///
/// Each electric hit adds arc chance. When the charge arcs, it deals damage
/// to nearby enemies and builds charge on them too.
#[derive(Component)]
pub struct ElectricCharge {
    /// Chance per tick to arc (0.0–1.0).
    pub arc_chance: f32,
    /// Time remaining before the charge expires (resets on each electric hit).
    pub time_remaining: f32,
    /// Cooldown timer preventing arcing every frame.
    pub arc_cooldown: f32,
}

impl ElectricCharge {
    /// Creates a new ElectricCharge from the initial electric damage.
    pub fn new(spell_damage: f32) -> Self {
        let chance = (ELECTRIC_ARC_CHANCE_PER_HIT + spell_damage * ELECTRIC_ARC_CHANCE_PER_DAMAGE)
            .min(ELECTRIC_ARC_MAX_CHANCE);
        Self {
            arc_chance: chance,
            time_remaining: ELECTRIC_ARC_DURATION,
            arc_cooldown: ELECTRIC_ARC_COOLDOWN,
        }
    }

    /// Stacks additional electric charge and resets the duration.
    pub fn stack(&mut self, spell_damage: f32) {
        self.arc_chance = (self.arc_chance
            + ELECTRIC_ARC_CHANCE_PER_HIT
            + spell_damage * ELECTRIC_ARC_CHANCE_PER_DAMAGE)
            .min(ELECTRIC_ARC_MAX_CHANCE);
        self.time_remaining = ELECTRIC_ARC_DURATION;
    }

    /// Updates timers. Returns `true` if the charge has expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        if self.time_remaining <= 0.0 {
            return true;
        }
        self.arc_cooldown = (self.arc_cooldown - delta).max(0.0);
        false
    }

    /// Returns `true` if arc is off cooldown.
    pub fn can_arc(&self) -> bool {
        self.arc_cooldown <= 0.0
    }

    /// Resets the arc cooldown after a successful arc.
    pub fn reset_arc_cooldown(&mut self) {
        self.arc_cooldown = ELECTRIC_ARC_COOLDOWN;
    }
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
    if has_spell_shield {
        return;
    }
    let modified_damage = damage * (1.0 + health.spell_vulnerability);
    apply_damage_to_unit(health, temp_hp, modified_damage);
    commands.entity(entity).insert(SpellDamaged);
    commands.entity(entity).insert(PendingDamageEffect {
        damage_type,
        damage: modified_damage,
    });
}

/// Which direction a unit faces, determining which sprite texture to show.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacingDirection {
    #[default]
    Forward = 0,
    Back = 1,
    Left = 2,
    Right = 3,
}

/// Minimum velocity squared to count as "moving" for animation purposes (5.0 units/sec).
pub const ANIMATION_MOVE_THRESHOLD_SQ: f32 = 25.0;

/// Hysteresis factor for direction switching. The current direction's axis gets
/// this multiplier bonus, preventing rapid flipping near diagonal angles.
/// 1.3 ≈ 8° overlap per side around the 45° boundary.
pub(super) const DIRECTION_HYSTERESIS_FACTOR: f32 = 1.3;

/// Sign hysteresis threshold: within an axis, the dot product must cross this
/// magnitude past zero before flipping direction (e.g., Forward→Back).
/// Prevents flickering when velocity is nearly perpendicular to the axis.
pub(super) const SIGN_HYSTERESIS_THRESHOLD: f32 = 2.0;

// Combined sprite sheet constants (shared by infantry and archer sheets).
pub const SPRITE_SHEET_IMAGE_WIDTH: f32 = 832.0;
pub const SPRITE_SHEET_IMAGE_HEIGHT: f32 = 256.0;
pub const SPRITE_FRAME_SIZE: f32 = 64.0;
pub const SPRITE_COLUMNS: usize = 9;
pub const ATTACK_SPRITE_COLUMNS: usize = 6;
pub const SHOOTING_SPRITE_COLUMNS: usize = 12;
pub const CASTING_SPRITE_COLUMNS: usize = 7;
pub const DEATH_SPRITE_COLUMNS: usize = 6;
pub const DEATH_SHEET_IMAGE_HEIGHT: f32 = 64.0;
/// Maps FacingDirection [Forward, Back, Left, Right] to sprite sheet rows.
/// Sheet row order: Away(0), Left(1), Forward(2), Right(3).
pub const SPRITE_DIRECTION_ROWS: [usize; 4] = [0, 2, 1, 3];

/// Calculates the UV size of a single frame within a sprite sheet.
pub fn sprite_frame_uv(sheet_height: f32) -> Vec2 {
    Vec2::new(
        SPRITE_FRAME_SIZE / SPRITE_SHEET_IMAGE_WIDTH,
        SPRITE_FRAME_SIZE / sheet_height,
    )
}

/// Number of pre-generated corpse material variants per unit type/team.
pub const CORPSE_MATERIAL_VARIANTS: usize = 3;

/// Walking animation state for sprite-sheet-animated units.
///
/// Uses a single combined sprite sheet with columns = animation frames
/// and rows = facing directions. The `direction_rows` array maps each
/// `FacingDirection` variant to the correct sheet row.
#[derive(Component)]
pub struct WalkingAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
    /// Number of animation frames (columns) per direction.
    pub columns: usize,
    /// UV size of a single frame: (frame_width / image_width, frame_height / image_height).
    pub frame_uv: Vec2,
    /// Maps `FacingDirection` enum index to the sprite sheet row.
    /// Index order: [Forward, Back, Left, Right].
    pub direction_rows: [usize; 4],
}

impl Default for WalkingAnimation {
    fn default() -> Self {
        Self {
            current_frame: 0,
            elapsed: rand::random::<f32>() * Self::FRAME_DURATION, // stagger
            columns: SPRITE_COLUMNS,
            frame_uv: sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT),
            direction_rows: SPRITE_DIRECTION_ROWS,
        }
    }
}

impl WalkingAnimation {
    const FRAME_DURATION: f32 = 0.125;

    /// Advance animation by `delta` seconds. Returns `true` if the frame changed.
    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= Self::FRAME_DURATION {
            self.elapsed -= Self::FRAME_DURATION;
            let old = self.current_frame;
            self.current_frame = (self.current_frame + 1) % self.columns;
            old != self.current_frame
        } else {
            false
        }
    }

    /// UV offset for the current frame and facing direction.
    pub fn uv_offset(&self, facing: FacingDirection) -> Vec2 {
        let col = self.current_frame as f32;
        let row = self.direction_rows[facing as usize] as f32;
        Vec2::new(col * self.frame_uv.x, row * self.frame_uv.y)
    }

    /// Returns the `Affine2` UV transform for the current frame and facing direction.
    pub fn uv_transform(&self, facing: FacingDirection) -> Affine2 {
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, self.uv_offset(facing))
    }

    /// UV transform for frame 0 in the given direction (idle/stationary pose).
    pub fn idle_uv_transform(facing: FacingDirection) -> Affine2 {
        let frame_uv = sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT);
        let row = SPRITE_DIRECTION_ROWS[facing as usize] as f32;
        let offset = Vec2::new(0.0, row * frame_uv.y);
        Affine2::from_scale_angle_translation(frame_uv, 0.0, offset)
    }
}

/// One-shot combat animation (melee attack or ranged shooting).
/// Temporarily overrides the walking texture, then restores it when finished.
#[derive(Component)]
pub struct CombatAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
    pub columns: usize,
    pub frame_uv: Vec2,
    pub direction_rows: [usize; 4],
    pub combat_texture: Handle<Image>,
    pub walking_texture: Handle<Image>,
    pub started: bool,
}

impl CombatAnimation {
    const FRAME_DURATION: f32 = 0.1;

    fn new(columns: usize, combat_texture: Handle<Image>, walking_texture: Handle<Image>) -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
            columns,
            frame_uv: sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT),
            direction_rows: SPRITE_DIRECTION_ROWS,
            combat_texture,
            walking_texture,
            started: false,
        }
    }

    pub fn new_attack(combat_texture: Handle<Image>, walking_texture: Handle<Image>) -> Self {
        Self::new(ATTACK_SPRITE_COLUMNS, combat_texture, walking_texture)
    }

    pub fn new_shooting(combat_texture: Handle<Image>, walking_texture: Handle<Image>) -> Self {
        Self::new(SHOOTING_SPRITE_COLUMNS, combat_texture, walking_texture)
    }

    pub fn new_casting(combat_texture: Handle<Image>, walking_texture: Handle<Image>) -> Self {
        Self::new(CASTING_SPRITE_COLUMNS, combat_texture, walking_texture)
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= Self::FRAME_DURATION {
            self.elapsed -= Self::FRAME_DURATION;
            let old = self.current_frame;
            self.current_frame += 1;
            old != self.current_frame
        } else {
            false
        }
    }

    pub fn finished(&self) -> bool {
        self.current_frame >= self.columns
    }

    pub fn uv_offset(&self, facing: FacingDirection) -> Vec2 {
        let col = self.current_frame.min(self.columns - 1) as f32;
        let row = self.direction_rows[facing as usize] as f32;
        Vec2::new(col * self.frame_uv.x, row * self.frame_uv.y)
    }

    pub fn uv_transform(&self, facing: FacingDirection) -> Affine2 {
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, self.uv_offset(facing))
    }
}

/// Death animation that plays when a unit dies. Non-directional single row.
/// Freezes on the last frame, then the entity is converted to a permanent corpse.
#[derive(Component)]
pub struct DyingAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
    pub columns: usize,
    pub frame_uv: Vec2,
    pub death_texture: Handle<Image>,
    pub started: bool,
}

impl DyingAnimation {
    const FRAME_DURATION: f32 = 0.15;

    pub fn new(death_texture: Handle<Image>) -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
            columns: DEATH_SPRITE_COLUMNS,
            frame_uv: sprite_frame_uv(DEATH_SHEET_IMAGE_HEIGHT),
            death_texture,
            started: false,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        if self.finished() {
            return false;
        }
        self.elapsed += delta;
        if self.elapsed >= Self::FRAME_DURATION {
            self.elapsed -= Self::FRAME_DURATION;
            let old = self.current_frame;
            self.current_frame += 1;
            old != self.current_frame
        } else {
            false
        }
    }

    pub fn finished(&self) -> bool {
        self.current_frame >= self.columns
    }

    pub fn uv_offset(&self) -> Vec2 {
        let col = self.current_frame.min(self.columns - 1) as f32;
        Vec2::new(col * self.frame_uv.x, 0.0)
    }

    pub fn uv_transform(&self) -> Affine2 {
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, self.uv_offset())
    }

    /// UV transform for the final (last) frame, used for the permanent corpse.
    pub fn last_frame_uv_transform(&self) -> Affine2 {
        let col = (self.columns - 1) as f32;
        let offset = Vec2::new(col * self.frame_uv.x, 0.0);
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, offset)
    }
}

/// Marker inserted when a `DyingAnimation` finishes, signaling
/// `finalize_dying_to_corpse` to lay the corpse flat.
#[derive(Component)]
pub struct DeathAnimationFinished;

/// Marker component for dead units (corpses).
///
/// Dead units remain on the battlefield as corpses that affect living units.
/// Corpses don't move, attack, or collide, but they slow down units walking over them.
#[derive(Component)]
pub struct Corpse;

/// Marker component for permanent corpses that cannot be resurrected.
///
/// Applied to undead corpses to prevent them from being raised again.
#[derive(Component)]
pub struct PermanentCorpse;

/// Marker component for units that can be teleported.
///
/// Applied to all combat units (defenders, attackers, undead) but not the wizard.
#[derive(Component)]
pub struct Teleportable;

/// Marker component for flying units that render above the battlefield.
///
/// Flying units ignore wall obstacles (no wall avoidance, collision, or LOS suppression)
/// and cannot be targeted by melee ground units (infantry, king, kingsguard).
/// Archers can still target and hit flying units.
#[derive(Component)]
pub struct Flying;

/// Shadow entity that tracks a unit's XZ position at ground level.
#[derive(Component)]
pub struct UnitShadow {
    pub owner: Entity,
}

/// Marker on a unit that already has a shadow spawned for it.
#[derive(Component)]
pub struct HasShadow;

/// Component that slows units walking over rough terrain (corpses).
///
/// Applied to corpses to create a movement penalty for living units that walk over them.
#[derive(Component)]
pub struct RoughTerrain {
    /// Movement speed multiplier (0.0 = no movement, 1.0 = full speed).
    /// For example, 0.6 means units move at 60% of their normal speed.
    pub slowdown_factor: f32,
}

/// Effectiveness coefficient applied to movement speed and attack damage.
///
/// Dynamically calculated based on:
/// - Number of allies in melee range (positive effect)
/// - Number of enemies in melee range (negative effect)
/// - Spell-based modifiers (future feature)
///
/// Units with allies nearby become more effective; units surrounded by enemies
/// become less effective. This creates strategic depth and rewards good positioning.
#[derive(Component, Clone, Copy)]
pub struct Effectiveness {
    /// Current effectiveness multiplier (applied to speed and damage).
    pub current: f32,
    /// Base effectiveness before any modifiers (always 1.0).
    pub base: f32,
    /// Bonus from spell effects (future feature).
    pub spell_bonus: f32,
}

impl Effectiveness {
    /// Creates a new Effectiveness component with default values.
    pub const fn new() -> Self {
        Self {
            current: 1.0,
            base: 1.0,
            spell_bonus: 0.0,
        }
    }

    /// Recalculates effectiveness based on proximity modifiers and spell bonuses.
    ///
    /// Formula: current = clamp(base + proximity_modifier + spell_bonus, MIN, MAX)
    ///
    /// # Arguments
    /// * `ally_count` - Number of allies in melee range
    /// * `enemy_count` - Number of enemies in melee range
    pub fn recalculate(&mut self, ally_count: i32, enemy_count: i32) {
        use crate::game::constants::{
            EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT,
            EFFECTIVENESS_MAX, EFFECTIVENESS_MIN,
        };

        let proximity_modifier = (ally_count as f32 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT)
            + (enemy_count as f32 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT);

        self.current = (self.base + proximity_modifier + self.spell_bonus)
            .clamp(EFFECTIVENESS_MIN, EFFECTIVENESS_MAX);
    }

    /// Returns the current effectiveness multiplier.
    pub fn multiplier(&self) -> f32 {
        self.current
    }
}

impl Default for Effectiveness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::constants::{
        EFFECTIVENESS_ALLY_BONUS_PER_UNIT, EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT, EFFECTIVENESS_MAX,
        EFFECTIVENESS_MIN,
    };

    #[test]
    fn test_effectiveness_base_values() {
        let eff = Effectiveness::new();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_default() {
        let eff = Effectiveness::default();
        assert_eq!(eff.current, 1.0);
        assert_eq!(eff.base, 1.0);
        assert_eq!(eff.spell_bonus, 0.0);
    }

    #[test]
    fn test_effectiveness_ally_bonus() {
        let mut eff = Effectiveness::new();
        eff.recalculate(3, 0); // 3 allies, 0 enemies
        assert_eq!(eff.current, 1.0 + 3.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT);
    }

    #[test]
    fn test_effectiveness_enemy_penalty() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 2); // 0 allies, 2 enemies
        assert_eq!(
            eff.current,
            1.0 + 2.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT
        );
    }

    #[test]
    fn test_effectiveness_mixed() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1); // 2 allies, 1 enemy
        let expected = 1.0
            + 2.0 * EFFECTIVENESS_ALLY_BONUS_PER_UNIT
            + 1.0 * EFFECTIVENESS_ENEMY_PENALTY_PER_UNIT;
        assert_eq!(eff.current, expected);
    }

    #[test]
    fn test_effectiveness_clamping_min() {
        let mut eff = Effectiveness::new();
        eff.recalculate(0, 10); // Many enemies
        assert_eq!(eff.current, EFFECTIVENESS_MIN);
    }

    #[test]
    fn test_effectiveness_clamping_max() {
        let mut eff = Effectiveness::new();
        eff.recalculate(20, 0); // Many allies
        assert_eq!(eff.current, EFFECTIVENESS_MAX);
    }

    #[test]
    fn test_effectiveness_with_spell_bonus() {
        let mut eff = Effectiveness::new();
        eff.spell_bonus = 0.5;
        eff.recalculate(0, 0); // No proximity modifiers
        assert_eq!(eff.current, 1.0 + 0.5);
    }

    #[test]
    fn test_effectiveness_multiplier() {
        let mut eff = Effectiveness::new();
        eff.recalculate(2, 1);
        assert_eq!(eff.multiplier(), eff.current);
    }
}

/// Stun effect that prevents both movement and attacking.
///
/// General-purpose CC component — any spell can insert this with a duration.
/// A dedicated system zeros velocity on stunned units each frame.
#[derive(Component)]
pub struct Stunned {
    /// Time remaining before the stun expires (in seconds).
    pub time_remaining: f32,
}

impl Stunned {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Frozen solid effect that prevents both movement and attacking.
///
/// Applied by Squall's Permafrost and Absolute Zero talents.
/// Unlike RootedModifier, frozen units cannot attack either.
#[derive(Component)]
pub struct FrozenSolidModifier {
    /// Time remaining before the freeze expires (in seconds).
    pub time_remaining: f32,
}

impl FrozenSolidModifier {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Movement speed modifier from being rooted (unable to move).
///
/// Applied to units hit by the Entangle spell.
/// Rooted units cannot move but can still attack.
#[derive(Component)]
pub struct RootedModifier {
    /// Time remaining before the root effect expires (in seconds).
    pub time_remaining: f32,
}

impl RootedModifier {
    /// Creates a new rooted modifier with the given duration.
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Movement speed modifier from haste effect as a percentage.
///
/// Applied to units affected by the Haste spell.
/// Examples: 0.5 = +50% speed (1.5x multiplier).
/// Movement systems apply this as: speed * (1.0 + sum_of_all_modifiers).
#[derive(Component)]
pub struct HasteModifier {
    /// Speed increase as a percentage (positive value).
    pub modifier: f32,
    /// Time remaining before the haste effect expires (in seconds).
    pub time_remaining: f32,
    /// Attack speed bonus (e.g., 0.2 = +20% attack speed). From Adrenaline Surge / Time Warp.
    pub attack_speed: f32,
}

impl HasteModifier {
    /// Creates a new haste modifier with the given strength and duration.
    pub const fn new(modifier: f32, duration: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
            attack_speed: 0.0,
        }
    }

    /// Creates a new haste modifier with attack speed bonus.
    pub const fn with_attack_speed(modifier: f32, duration: f32, attack_speed: f32) -> Self {
        Self {
            modifier,
            time_remaining: duration,
            attack_speed,
        }
    }

    /// Updates the timer, returning true if expired.
    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    /// Refreshes the duration (used when reapplying the haste).
    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Damage amplification modifier from Mark of Death.
///
/// Marked units take increased damage from ALL sources.
/// Combat system applies: damage * (1.0 + damage_amplification).
#[derive(Component)]
pub struct MarkedForDeathModifier {
    /// Damage amplification (e.g., 0.5 = +50% damage taken).
    pub damage_amplification: f32,
    /// Time remaining before the mark expires (in seconds).
    pub time_remaining: f32,
}

impl MarkedForDeathModifier {
    pub const fn new(amplification: f32, duration: f32) -> Self {
        Self {
            damage_amplification: amplification,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Sleep effect from Sleep spell.
///
/// Sleeping units cannot move or attack. First damage hit deals bonus damage
/// and wakes them (removes this effect).
///
/// Talent-specific behaviors are separate components: [`NightTerrors`],
/// [`Comatose`], [`NarcolepticWave`], [`Sleepwalking`].
#[derive(Component)]
pub struct SleepModifier {
    /// Time remaining before the effect expires (in seconds).
    pub time_remaining: f32,
    /// Bonus damage multiplier on first hit (e.g., 2.0 = double damage).
    pub bonus_damage_multiplier: f32,
    /// Full duration this modifier was created with (for narcoleptic wave inheritance).
    pub full_duration: f32,
}

impl SleepModifier {
    pub fn new(duration: f32, bonus_multiplier: f32) -> Self {
        Self {
            time_remaining: duration,
            bonus_damage_multiplier: bonus_multiplier,
            full_duration: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Night Terrors talent: sleeping units take minor DPS.
#[derive(Component)]
pub struct NightTerrors {
    pub dps: f32,
    pub tick_accumulator: f32,
}

impl NightTerrors {
    pub fn new(dps: f32) -> Self {
        Self {
            dps,
            tick_accumulator: 0.0,
        }
    }
}

/// Comatose talent: sleeping units only wake if a single hit exceeds a fraction of max HP.
#[derive(Component)]
pub struct Comatose {
    /// Fraction of max HP that a single hit must exceed to wake (e.g., 0.3 = 30%).
    pub wake_threshold: f32,
}

impl Comatose {
    pub fn new(threshold: f32) -> Self {
        Self {
            wake_threshold: threshold,
        }
    }
}

/// Narcoleptic Wave talent: after a delay, sleep spreads to nearby awake enemies.
/// Removed from the entity once it has spread.
#[derive(Component)]
pub struct NarcolepticWave {
    /// Timer counting down before sleep spreads.
    pub timer: f32,
    /// Radius for spreading sleep.
    pub radius: f32,
}

impl NarcolepticWave {
    pub fn new(delay: f32, radius: f32) -> Self {
        Self { timer: delay, radius }
    }
}

/// Dreamwalker talent: sleeping units sleepwalk back toward spawn instead of being immobilized.
#[derive(Component)]
pub struct Sleepwalking {
    pub speed_mult: f32,
}

impl Sleepwalking {
    pub fn new(speed_mult: f32) -> Self {
        Self { speed_mult }
    }
}

/// Battle Hymn buff granting damage and attack speed bonuses.
///
/// Combat system adds damage_bonus to outgoing damage and scales attack timing.
/// Talent-specific behaviors are separate components: [`EchoingSong`], [`AnthemResilience`].
#[derive(Component)]
pub struct BattleHymnModifier {
    /// Damage bonus as a percentage (e.g., 0.4 = +40% damage).
    pub damage_bonus: f32,
    /// Attack speed bonus as a percentage (e.g., 0.3 = 30% faster attacks).
    pub attack_speed: f32,
    /// Time remaining before the buff expires (in seconds).
    pub time_remaining: f32,
}

impl BattleHymnModifier {
    pub const fn new(damage_bonus: f32, attack_speed: f32, duration: f32) -> Self {
        Self {
            damage_bonus,
            attack_speed,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Echoing Song talent: when BattleHymnModifier expires, re-apply at reduced duration.
#[derive(Component)]
pub struct EchoingSong {
    /// Duration to re-apply when the buff expires.
    pub echo_duration: f32,
}

impl EchoingSong {
    pub fn new(duration: f32) -> Self {
        Self {
            echo_duration: duration,
        }
    }
}

/// Anthem of Resilience talent: damage reduction while Battle Hymn is active.
#[derive(Component)]
pub struct AnthemResilience {
    /// Damage reduction percentage (e.g., 0.3 = 30% less damage taken).
    pub damage_reduction: f32,
}

impl AnthemResilience {
    pub fn new(reduction: f32) -> Self {
        Self {
            damage_reduction: reduction,
        }
    }
}

/// Berserker Rage buff granting damage bonus but increasing damage taken.
///
/// Risk/reward buff: +damage dealt, +damage taken.
#[derive(Component)]
pub struct BerserkerRageModifier {
    /// Damage bonus as a percentage (e.g., 0.8 = +80% damage dealt).
    pub damage_bonus: f32,
    /// Damage vulnerability as a percentage (e.g., 0.5 = +50% damage taken).
    pub damage_vulnerability: f32,
    /// Time remaining before the buff expires (in seconds).
    pub time_remaining: f32,
}

impl BerserkerRageModifier {
    pub const fn new(damage_bonus: f32, vulnerability: f32, duration: f32) -> Self {
        Self {
            damage_bonus,
            damage_vulnerability: vulnerability,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Fog evasion effect from Fog Cloud.
///
/// Units inside the fog have a chance to evade incoming attacks.
#[derive(Component)]
pub struct FogEvasionModifier {
    /// Evasion chance (0.0–1.0, e.g., 0.4 = 40% dodge chance).
    pub evasion_chance: f32,
    /// Time remaining before the evasion expires (in seconds).
    pub time_remaining: f32,
}

impl FogEvasionModifier {
    pub const fn new(chance: f32, duration: f32) -> Self {
        Self {
            evasion_chance: chance,
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn refresh(&mut self, duration: f32) {
        self.time_remaining = duration;
    }
}

/// Banishment effect that removes a unit from the battlefield temporarily.
///
/// Banished units are hidden, untargetable, and cannot act.
/// When the effect expires, the unit reappears.
#[derive(Component)]
pub struct BanishedModifier {
    /// Time remaining before the unit returns (in seconds).
    pub time_remaining: f32,
}

impl BanishedModifier {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Permanent marker preventing a unit from being banished again.
#[derive(Component)]
pub struct WasBanished;

/// Sickened effect that prevents a unit from moving or acting.
///
/// Applied by certain spells. The unit is incapacitated for the duration.
/// When the effect expires, the component is removed.
#[derive(Component)]
pub struct SickenedModifier {
    /// Time remaining before the effect expires (in seconds).
    pub time_remaining: f32,
}

impl SickenedModifier {
    pub const fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Poison debuff that reduces unit effectiveness over time.
///
/// Stacks up to a cap. If total accumulated poison reaches the sickened threshold,
/// the unit becomes sickened (stops moving) and then smelly (allies flee).
#[derive(Component)]
pub struct PoisonedModifier {
    /// Accumulated effectiveness penalty (negative, grows with stacking).
    pub effectiveness_penalty: f32,
    /// Time remaining before poison expires (resets on each stack).
    pub time_remaining: f32,
    /// Timer for periodic effectiveness penalty ticks.
    pub tick_timer: f32,
    /// Total accumulated penalty for sickened threshold check.
    pub total_accumulated: f32,
    /// Total penalty applied to spell_bonus (for accurate cleanup).
    pub applied_to_spell_bonus: f32,
}

impl PoisonedModifier {
    pub fn new(penalty_per_stack: f32, duration: f32) -> Self {
        Self {
            effectiveness_penalty: penalty_per_stack,
            time_remaining: duration,
            tick_timer: 0.0,
            total_accumulated: penalty_per_stack.abs(),
            applied_to_spell_bonus: 0.0,
        }
    }

    pub fn stack(&mut self, penalty_per_stack: f32, duration: f32, cap: f32) {
        self.effectiveness_penalty = (self.effectiveness_penalty + penalty_per_stack).max(cap);
        self.time_remaining = duration;
        self.total_accumulated += penalty_per_stack.abs();
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }

    pub fn is_sickened(&self, threshold: f32) -> bool {
        self.total_accumulated >= threshold
    }
}

/// Smelly debuff that causes allied units to flee.
///
/// Applied after sickened expires or directly by Poop damage.
/// Other units on the same team avoid the smelly unit.
#[derive(Component)]
pub struct SmellyModifier {
    /// Time remaining before the smell fades.
    pub time_remaining: f32,
}

impl SmellyModifier {
    pub fn new(duration: f32) -> Self {
        Self {
            time_remaining: duration,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

impl_timed_modifier!(SmellyModifier);

/// Polymorph effect that transforms a unit into a sheep.
///
/// Stores the original unit state for restoration when the effect expires.
#[derive(Component)]
pub struct PolymorphedModifier {
    /// Time remaining before the unit reverts (in seconds).
    pub time_remaining: f32,
    /// Original current health to restore on revert.
    pub original_health_current: f32,
    /// Original max health to restore on revert.
    pub original_health_max: f32,
    /// Original material handle to restore on revert.
    pub original_material: Handle<StandardMaterial>,
    /// Original team to restore on revert.
    pub original_team: Team,
}

impl PolymorphedModifier {
    pub fn new(
        duration: f32,
        health_current: f32,
        health_max: f32,
        material: Handle<StandardMaterial>,
        team: Team,
    ) -> Self {
        Self {
            time_remaining: duration,
            original_health_current: health_current,
            original_health_max: health_max,
            original_material: material,
            original_team: team,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

/// Component indicating a unit is currently engaged in melee combat with a specific team.
///
/// A unit is considered in melee when there is an enemy within melee range.
/// This is used by archers to avoid friendly fire - they won't target units in melee
/// with someone on their own team.
#[derive(Component)]
pub struct InMelee(pub Team);

/// Targeting velocity toward target, set by the targeting system.
///
/// The targeting system calculates this based on the nearest enemy.
/// This is a normalized direction vector with distance information for weighting.
#[derive(Component, Default)]
pub struct TargetingVelocity {
    pub velocity: Vec3,
    pub distance_to_target: f32,
}

/// Per-unit multipliers for flocking forces.
///
/// Units without this component default to 1.0 for all forces.
/// Set individual fields to 0.0 to disable that force for a unit.
#[derive(Component)]
pub struct FlockingModifier {
    pub separation: f32,
    pub alignment: f32,
    pub cohesion: f32,
}

impl FlockingModifier {
    pub const fn new(separation: f32, alignment: f32, cohesion: f32) -> Self {
        Self {
            separation,
            alignment,
            cohesion,
        }
    }
}

/// King's Guard unit. Stores the slot index for positioning around the King.
#[derive(Component)]
pub struct KingsGuard(pub u32);

/// Flocking velocity from separation, alignment, and cohesion forces.
///
/// The flocking system calculates this based on nearby allies.
/// This is a normalized direction vector.
#[derive(Component, Default)]
pub struct FlockingVelocity {
    pub velocity: Vec3,
    /// Direct repulsion force from smelly units, applied as raw acceleration
    /// (bypasses the weighted flocking normalization).
    pub smelly_repulsion: Vec3,
}

/// Mind control effect — unit targets allies instead of enemies.
/// Used by both the Hag boss (Martina) and the player's Mind Control spell.
#[derive(Component)]
pub struct MindControlled {
    /// Time elapsed since mind control was applied.
    pub time_elapsed: f32,
    /// Duration before mind control wears off.
    pub wear_off_duration: f32,
    /// Original defender spawn position for restoring flow field on wear-off.
    pub original_spawn_pos: Option<Vec2>,
    /// Damage multiplier for controlled unit's attacks (Deep Domination talent).
    pub damage_multiplier: f32,
}

/// Marks a unit as wanting to retaliate against a specific entity.
/// Inserted when a mind-controlled unit attacks a same-team ally, causing
/// the victim to consider the attacker a valid target despite being on the same team.
#[derive(Component)]
pub struct RetaliationTarget(pub Entity);

/// Airborne state for units launched into the air (geyser, explosions, etc.).
/// Applies gravity, offsets Y visually, and deals velocity-based fall damage on landing.
/// The unit is rooted separately via `RootedModifier` during flight.
#[derive(Component)]
pub struct Airborne {
    /// Current vertical velocity (positive = upward).
    pub vertical_velocity: f32,
    /// Current vertical offset from ground.
    pub height: f32,
    /// The unit's original Y position before launch (restored on landing).
    pub base_y: f32,
    /// Gravity acceleration applied per second (units/s²).
    pub gravity: f32,
    /// Damage type to apply on landing.
    pub damage_type: DamageType,
}

impl Airborne {
    /// Creates a new airborne state with the given launch velocity, gravity, and base Y.
    pub fn new(launch_velocity: f32, gravity: f32, base_y: f32, damage_type: DamageType) -> Self {
        Self {
            vertical_velocity: launch_velocity,
            height: 0.0,
            base_y,
            gravity,
            damage_type,
        }
    }
}

/// Damage scale factor for fall damage: `damage = abs(impact_velocity) * FALL_DAMAGE_SCALE`.
/// Calibrated so a geyser launch (200 velocity, 120 gravity → ~200 impact velocity) deals 15 damage.
pub const FALL_DAMAGE_SCALE: f32 = 0.075;

/// Knockback effect that moves a unit outward over time with decay.
/// Applied by ogre melee attacks, hag leaps, and meteor aftershock.
/// Decays linearly for a "tumbling through dirt" feel.
#[derive(Component)]
pub struct Knockback {
    /// Direction of knockback (normalized XZ).
    pub direction_x: f32,
    pub direction_z: f32,
    /// Initial knockback speed (units/s at full strength).
    pub speed: f32,
    /// Total duration of the effect.
    pub duration: f32,
    /// Time remaining before the effect expires.
    pub remaining: f32,
}

impl Knockback {
    pub fn new(direction: Vec3, speed: f32, duration: f32) -> Self {
        let normalized = direction.normalize_or_zero();
        Self {
            direction_x: normalized.x,
            direction_z: normalized.z,
            speed,
            duration,
            remaining: duration,
        }
    }
}

// Re-export elite components
#[allow(unused_imports)]
pub use super::elite::{EliteAttackSpeedBonus, EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};

/// Persistent color glow for special unit types (dispeller, healer, shielder, commander, brute).
///
/// The visual system uses this to apply a pulsing color tint to the unit's material.
/// This is separate from elite glow and shield buff glow, allowing them to stack.
#[derive(Component, Clone)]
pub struct UnitTypeGlow {
    pub color: Color,
}
