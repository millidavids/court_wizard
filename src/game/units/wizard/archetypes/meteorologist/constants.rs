use bevy::prelude::*;

/// Mana cost to activate a weather condition.
pub(crate) const WEATHER_MANA_COST: f32 = 25.0;

/// Cooldown between weather switches (seconds).
pub(crate) const WEATHER_SWITCH_COOLDOWN: f32 = 3.0;

/// Time to reach maximum intensity (seconds).
pub(super) const INTENSITY_RAMP_TIME: f32 = 30.0;

/// Maximum intensity multiplier.
pub(super) const INTENSITY_MAX: f32 = 1.5;

/// Starting intensity when weather is first activated.
pub(super) const INTENSITY_MIN: f32 = 1.0;

// === Storm (Wet + Charged) ===

/// Base radius for shock spread between wet units.
pub(super) const WET_SHOCK_SPREAD_RADIUS: f32 = 200.0;

/// Fire DoT duration multiplier while wet (halved).
pub(crate) const WET_FIRE_DOT_MULTIPLIER: f32 = 0.5;

// === Blizzard (Cold) ===

/// Multiplier applied to frost spell slow strength during blizzard.
pub(crate) const COLD_FROST_SLOW_MULTIPLIER: f32 = 1.5;

/// Duration of the freeze root when frost hits a cold unit (seconds).
pub(crate) const COLD_FREEZE_DURATION: f32 = 1.5;

// === Drought (Dry) ===

/// Healing reduction while dry (30%).
pub(crate) const DRY_HEALING_REDUCTION: f32 = 0.3;

/// Number of burning patches spawned per fire spell impact.
pub(crate) const DRY_BURNING_PATCH_COUNT: u32 = 3;

/// Radius around fire impact to scatter burning patches.
pub(crate) const DRY_BURNING_PATCH_SCATTER: f32 = 120.0;

/// Radius of each burning patch's damage area.
pub(crate) const BURNING_PATCH_RADIUS: f32 = 40.0;

/// Base lifetime of burning patches (seconds).
pub(crate) const BURNING_PATCH_LIFETIME: f32 = 6.0;

/// Damage per second dealt by burning patches.
pub(crate) const BURNING_PATCH_DPS: f32 = 5.0;

/// Tick interval for burning patch damage.
pub(crate) const BURNING_PATCH_TICK_INTERVAL: f32 = 0.5;

/// Color for burning patch visual.
pub(crate) const BURNING_PATCH_COLOR: Color = Color::srgba(1.0, 0.4, 0.1, 0.4);

// === Storm Lightning ===

/// Extra targets for electric arc chains during thunderstorm.
pub(crate) const CHARGED_EXTRA_ARC_TARGETS: usize = 2;

/// Interval between random lightning strikes (seconds).
pub(crate) const THUNDERSTORM_LIGHTNING_INTERVAL: f32 = 8.0;

/// Damage dealt by random storm lightning strikes.
pub(super) const THUNDERSTORM_LIGHTNING_DAMAGE: f32 = 3.0;

/// Radius of thunderstorm lightning strike AoE.
pub(super) const THUNDERSTORM_LIGHTNING_RADIUS: f32 = 80.0;

/// Color for thunderstorm lightning visual.
pub(super) const THUNDERSTORM_LIGHTNING_COLOR: Color = Color::srgba(0.8, 0.9, 1.0, 0.9);

// === Weather VFX ===

/// Number of rain particles to spawn per frame.
pub(super) const RAIN_PARTICLES_PER_FRAME: u32 = 8;

/// Rain particle fall speed.
pub(super) const RAIN_FALL_SPEED: f32 = 3000.0;

/// Rain particle horizontal drift (wind angle).
pub(super) const RAIN_WIND_SPEED: f32 = 600.0;

/// Rain particle lifetime (seconds).
pub(super) const RAIN_PARTICLE_LIFETIME: f32 = 1.2;

/// Rain particle color (light gray, translucent).
pub(super) const RAIN_PARTICLE_COLOR: Color = Color::srgba(0.7, 0.7, 0.8, 0.5);

/// Number of snow particles to spawn per frame.
pub(super) const SNOW_PARTICLES_PER_FRAME: u32 = 6;

/// Snow particle fall speed.
pub(super) const SNOW_FALL_SPEED: f32 = 800.0;

/// Snow particle horizontal drift (heavy wind).
pub(super) const SNOW_WIND_SPEED: f32 = 1200.0;

/// Snow particle lifetime (seconds).
pub(super) const SNOW_PARTICLE_LIFETIME: f32 = 2.5;

/// Snow particle color (white).
pub(super) const SNOW_PARTICLE_COLOR: Color = Color::srgba(0.95, 0.95, 1.0, 0.7);

/// Maximum sky darkening alpha for rain/thunderstorm.
pub(super) const RAIN_SKY_DARKEN_ALPHA: f32 = 0.25;

/// Maximum sky yellowish alpha for drought.
pub(super) const DROUGHT_TINT_ALPHA: f32 = 0.2;

/// Maximum ground whitening alpha for blizzard.
pub(super) const BLIZZARD_GROUND_ALPHA: f32 = 0.3;

/// Maximum ground browning alpha for drought.
pub(super) const DROUGHT_GROUND_ALPHA: f32 = 0.25;

/// Weather ambient SFX volume scale.
pub(super) const WEATHER_SFX_VOLUME: f32 = 0.4;

/// Spawn area half-size for weather particles (centered on battlefield).
pub(super) const PARTICLE_SPAWN_HALF_SIZE: f32 = 2500.0;

/// Height at which weather particles spawn.
pub(super) const PARTICLE_SPAWN_HEIGHT: f32 = 2000.0;

// === Weather Bar UI Colors ===

/// Storm weather color (blue-purple).
pub(crate) const STORM_COLOR: Color = Color::srgba(0.4, 0.4, 0.9, 0.8);

/// Blizzard weather color (light cyan).
pub(crate) const BLIZZARD_COLOR: Color = Color::srgba(0.7, 0.85, 1.0, 0.8);

/// Drought weather color (orange/amber).
pub(crate) const DROUGHT_COLOR: Color = Color::srgba(0.9, 0.6, 0.2, 0.8);

