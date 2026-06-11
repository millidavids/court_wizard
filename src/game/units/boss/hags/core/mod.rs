//! Hag spawn, animations, movement, combat.

mod animation;
mod combat;
mod death;
mod eye_transfer;
mod movement;
mod spawn;

// Public systems (used by plugin.rs and systems.rs via `pub use super::core::*`)
pub use combat::{hag_combat, update_hag_targeting};
pub use death::{apply_enrage_to_last_hag, intercept_blind_hag_death, resurrect_eyed_hags};
pub use eye_transfer::{tick_eye_transfer, update_eye_flight};
pub use movement::{hag_movement, hag_separation, justina_kite_distance};
pub use spawn::spawn_hags;

// pub(super) items — visible to the hags module (abilities, systems, plugin)
pub use animation::{hag_casting_animation, restore_hag_walking_pose, set_hag_attack_pose_frame};
