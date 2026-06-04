use bevy::prelude::*;

/// Wet status — applied to units in ponds or during Storm.
/// Amplifies electric damage, enables shock spread between wet units.
/// Darkens unit visuals while active.
#[derive(Component)]
pub struct WetModifier {
    /// Current weather intensity (scales effect strength).
    pub intensity: f32,
    /// Time remaining before wet expires (refreshed while in pond or storm).
    pub time_remaining: f32,
}

/// Duration wet lasts after leaving a pond or storm ending (seconds).
pub const WET_DURATION: f32 = 10.0;

/// Cold status — applied to all units during Blizzard.
/// Frost slows stack multiplicatively. Frost + Cold = brief freeze (root).
#[derive(Component)]
pub struct ColdModifier {
    /// Current weather intensity (scales effect strength).
    pub intensity: f32,
}

/// Dry status — applied to all units during Drought.
/// Fire spells create burning ground patches. Healing reduced.
#[derive(Component)]
pub struct DryModifier {
    /// Current weather intensity (scales effect strength).
    pub intensity: f32,
}

/// Charged status — applied to all units during Storm.
/// Electric arc chance boosted, extra chain targets.
#[derive(Component)]
pub struct ChargedModifier {
    /// Current weather intensity (scales effect strength).
    pub intensity: f32,
}

/// A burning ground patch spawned by fire spells during Drought.
#[derive(Component)]
pub struct BurningPatch {
    /// Time remaining before the patch expires.
    pub lifetime: f32,
    /// Radius of the damage area.
    pub radius: f32,
    /// Damage dealt per tick.
    pub damage_per_tick: f32,
    /// Timer for the next damage tick.
    pub tick_timer: f32,
}

/// Visual marker for thunderstorm lightning strike flash.
#[derive(Component)]
pub struct LightningStrike {
    pub lifetime: f32,
}

/// Marker for a looping weather ambient sound entity, tagged with which weather
/// it plays so `update_weather_sfx` can reconcile (keep/add/remove) per weather
/// instead of restarting every sound on each change.
#[derive(Component)]
pub(crate) struct WeatherSfx(pub super::resources::WeatherType);

/// A single weather particle (rain drop, snowflake, etc.).
#[derive(Component)]
pub(crate) struct WeatherParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
}

/// Fullscreen UI overlay for weather color tinting.
#[derive(Component)]
pub(crate) struct WeatherOverlay;

/// Ground overlay mesh for weather ground effects (snow/drought browning).
#[derive(Component)]
pub(crate) struct WeatherGroundOverlay;
