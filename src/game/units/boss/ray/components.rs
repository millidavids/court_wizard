use bevy::prelude::*;

#[derive(Component)]
pub struct Ray;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RayEyeType {
    Petrification,
    Disintegration,
    Fear,
    MindControl,
    Teleportation,
}

impl RayEyeType {
    pub const ALL: [RayEyeType; 5] = [
        RayEyeType::Petrification,
        RayEyeType::Disintegration,
        RayEyeType::Fear,
        RayEyeType::MindControl,
        RayEyeType::Teleportation,
    ];

    pub const COUNT: usize = 5;

    pub fn index(self) -> usize {
        match self {
            RayEyeType::Petrification => 0,
            RayEyeType::Disintegration => 1,
            RayEyeType::Fear => 2,
            RayEyeType::MindControl => 3,
            RayEyeType::Teleportation => 4,
        }
    }

}

#[derive(Component)]
pub struct RayEyeState {
    pub active: [bool; RayEyeType::COUNT],
}

impl RayEyeState {
    pub fn new() -> Self {
        Self {
            active: [true; RayEyeType::COUNT],
        }
    }

    pub fn is_disintegration_active(&self) -> bool {
        self.active[RayEyeType::Disintegration.index()]
    }
}

/// Marks an eye that has been killed and is playing its implode/despawn animation.
#[derive(Component)]
pub struct RayEyeDying {
    pub time_alive: f32,
    pub duration: f32,
    pub initial_scale: f32,
}

#[derive(Component)]
pub enum RayState {
    Approaching,
    Idle,
}

#[derive(Component)]
pub struct RayEye {
    pub eye_type: RayEyeType,
    pub heading: Vec2,
}

/// Particle traveling from Ray's body to an eye, with magic-missile-style wobble.
#[derive(Component)]
pub struct RayStalkParticle {
    pub eye_entity: Entity,
    pub velocity: Vec3,
    pub time_alive: f32,
    pub wobble_offset: f32,
}

#[derive(Component)]
pub struct RayBeamVisual {
    pub lifetime: f32,
}

#[derive(Component)]
pub struct RayDisintegrateBeam {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
    pub time_alive: f32,
    pub time_since_damage: f32,
}

impl RayDisintegrateBeam {
    pub fn new(origin: Vec3, direction: Vec3, length: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            time_alive: 0.0,
            time_since_damage: 0.0,
        }
    }
}

#[derive(Component)]
pub struct RayDisintegrateGlow {
    pub beam_entity: Entity,
}

/// Petrification beam — channels then fires, applying Petrified to hit units.
#[derive(Component)]
pub struct RayPetrificationBeam {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
    pub channel_progress: f32,
    pub has_fired: bool,
}

impl RayPetrificationBeam {
    pub fn new(origin: Vec3, direction: Vec3, length: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            channel_progress: 0.0,
            has_fired: false,
        }
    }
}

#[derive(Component)]
pub struct RayPetrificationGlow {
    pub beam_entity: Entity,
}

#[derive(Component)]
pub struct RayPetrificationSweep {
    pub beam_entity: Option<Entity>,
    pub glow_entity: Option<Entity>,
    pub cooldown: f32,
}

/// Charm beam — channels then fires, applying permanent MindControl to hit units.
#[derive(Component)]
pub struct RayMindControlBeam {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
    pub channel_progress: f32,
    pub has_fired: bool,
}

impl RayMindControlBeam {
    pub fn new(origin: Vec3, direction: Vec3, length: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            channel_progress: 0.0,
            has_fired: false,
        }
    }
}

#[derive(Component)]
pub struct RayMindControlGlow {
    pub beam_entity: Entity,
}

#[derive(Component)]
pub struct RayMindControlSweep {
    pub beam_entity: Option<Entity>,
    pub glow_entity: Option<Entity>,
    pub cooldown: f32,
}

/// The fear beam cone pointing straight down from the fear eye.
#[derive(Component)]
pub struct RayFearBeam {
    pub origin: Vec3,
    pub length: f32,
    pub time_alive: f32,
}

#[derive(Component)]
pub struct RayFearGlow {
    pub beam_entity: Entity,
}

#[derive(Component)]
pub struct RayFearSweep {
    pub beam_entity: Option<Entity>,
    pub glow_entity: Option<Entity>,
    pub fear_cooldown: f32,
}

#[derive(Component)]
pub struct RayTeleportSweep {
    pub cooldown: f32,
}

#[derive(Component)]
pub struct RayTeleportBubble {
    pub time_alive: f32,
}

#[derive(Component)]
pub struct RayDisintegrationSweep {
    pub beam_entity: Option<Entity>,
    pub glow_entity: Option<Entity>,
    pub sfx_entity: Option<Entity>,
    pub tip_position: Vec2,
    pub tip_velocity: Vec2,
    pub cooldown: f32,
}
