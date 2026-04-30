//! Sprite animation components.

use bevy::math::Affine2;
use bevy::prelude::*;
use rand::Rng;

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

/// Optional per-entity multiplier on the global facing-axis angular buffer.
/// Used by units whose velocity oscillates from external forces (e.g. hag
/// separation) and would otherwise flicker between facing rows.
#[derive(Component, Clone, Copy)]
pub struct FacingHysteresisBoost(pub f32);

/// Minimum dwell time (seconds) before a facing direction can change again.
/// Inserted at spawn for jittery units; ticks down in `update_facing_direction`
/// and is reset to its configured duration whenever the facing actually flips.
#[derive(Component, Clone, Copy)]
pub struct FacingDwell {
    pub duration: f32,
    pub time_remaining: f32,
}

impl FacingDwell {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            time_remaining: 0.0,
        }
    }
}

/// Low-passed velocity used by `update_facing_direction` to pick the facing row.
/// Smoothing eliminates frame-to-frame velocity noise (e.g. flow-field cell
/// transitions, separation force, steering wobble) so the facing decision is
/// made against the unit's *trend* of motion, not its instantaneous velocity.
///
/// `time_constant` is the seconds for a step input to reach ~63% of its target;
/// larger = smoother but laggier. Hags use ~0.4s.
#[derive(Component, Clone, Copy)]
pub struct SmoothedFacingVelocity {
    pub velocity: bevy::math::Vec3,
    pub time_constant: f32,
}

impl SmoothedFacingVelocity {
    pub fn new(time_constant: f32) -> Self {
        Self {
            velocity: bevy::math::Vec3::ZERO,
            time_constant,
        }
    }
}

// Combined sprite sheet constants (shared by infantry and archer sheets).
pub const SPRITE_SHEET_IMAGE_WIDTH: f32 = 832.0;
pub const SPRITE_SHEET_IMAGE_HEIGHT: f32 = 256.0;
pub const SPRITE_FRAME_SIZE: f32 = 64.0;
pub const SPRITE_COLUMNS: usize = 9;
/// Sheet row index of the face-visible (forward-facing) sprite direction.
pub const SHEET_ROW_FORWARD_FACING: usize = 2;
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
            elapsed: 0.0,
            columns: SPRITE_COLUMNS,
            frame_uv: sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT),
            direction_rows: SPRITE_DIRECTION_ROWS,
        }
    }
}

impl WalkingAnimation {
    const FRAME_DURATION: f32 = 0.125;

    /// Creates a new WalkingAnimation with a random stagger offset.
    pub fn new_staggered(rng: &mut impl Rng) -> Self {
        Self {
            elapsed: rng.random::<f32>() * Self::FRAME_DURATION,
            ..Default::default()
        }
    }

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

/// Plays a unit's death sprite sheet in reverse to make a corpse visually
/// "stand up" when raised. While active, walking and combat animations are
/// suspended; on completion the material's texture is swapped back to the
/// walking sprite and the component is removed.
#[derive(Component)]
pub struct RisingAnimation {
    /// Frame index, played in reverse (starts at `DEATH_SPRITE_COLUMNS - 1` and
    /// decrements to -1 to indicate completion).
    pub elapsed: f32,
    pub current_frame: i32,
    pub frame_uv: Vec2,
    pub death_texture: Handle<Image>,
    pub walking_texture: Handle<Image>,
    pub started: bool,
}

impl RisingAnimation {
    const FRAME_DURATION: f32 = 0.15;

    pub fn new(death_texture: Handle<Image>, walking_texture: Handle<Image>) -> Self {
        Self {
            elapsed: 0.0,
            current_frame: (DEATH_SPRITE_COLUMNS as i32) - 1,
            frame_uv: sprite_frame_uv(DEATH_SHEET_IMAGE_HEIGHT),
            death_texture,
            walking_texture,
            started: false,
        }
    }

    /// Advances one tick. Returns true if the displayed frame changed.
    pub fn tick(&mut self, delta: f32) -> bool {
        if self.finished() {
            return false;
        }
        self.elapsed += delta;
        if self.elapsed >= Self::FRAME_DURATION {
            self.elapsed -= Self::FRAME_DURATION;
            self.current_frame -= 1;
            true
        } else {
            false
        }
    }

    pub fn finished(&self) -> bool {
        self.current_frame < 0
    }

    pub fn uv_offset(&self) -> Vec2 {
        let col = self.current_frame.max(0) as f32;
        Vec2::new(col * self.frame_uv.x, 0.0)
    }

    pub fn uv_transform(&self) -> Affine2 {
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, self.uv_offset())
    }
}

/// Marker that pauses `update_walking_animation` for an entity so a system
/// can hold its sprite on a chosen frame (used e.g. for Josephina's leap pose).
#[derive(Component)]
pub struct AnimationOverride;

/// Looping in-place sprite-sheet animation for entities that don't move
/// (e.g. eyes, idle props). Single row, no facing direction.
#[derive(Component)]
pub struct PulsingAnimation {
    pub current_frame: usize,
    pub elapsed: f32,
    pub columns: usize,
    pub frame_uv: Vec2,
    pub frame_duration: f32,
}

impl PulsingAnimation {
    pub fn new(columns: usize, frame_uv: Vec2, frame_duration: f32) -> Self {
        Self {
            current_frame: 0,
            elapsed: 0.0,
            columns,
            frame_uv,
            frame_duration,
        }
    }

    /// Creates a new pulsing animation with a random elapsed-time stagger so
    /// multiple instances don't pulse in lockstep.
    pub fn new_staggered(
        columns: usize,
        frame_uv: Vec2,
        frame_duration: f32,
        rng: &mut impl Rng,
    ) -> Self {
        Self {
            current_frame: rng.random_range(0..columns),
            elapsed: rng.random::<f32>() * frame_duration,
            columns,
            frame_uv,
            frame_duration,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        if self.elapsed >= self.frame_duration {
            self.elapsed -= self.frame_duration;
            let old = self.current_frame;
            self.current_frame = (self.current_frame + 1) % self.columns;
            old != self.current_frame
        } else {
            false
        }
    }

    pub fn uv_offset(&self) -> Vec2 {
        Vec2::new(self.current_frame as f32 * self.frame_uv.x, 0.0)
    }

    pub fn uv_transform(&self) -> Affine2 {
        Affine2::from_scale_angle_translation(self.frame_uv, 0.0, self.uv_offset())
    }
}
