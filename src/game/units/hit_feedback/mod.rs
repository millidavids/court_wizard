//! Hit feedback: the element-colored flash and throttled sound that tell the
//! player a hit connected.

pub(in crate::game) mod flash;
pub(in crate::game) mod spell_hits;

pub(in crate::game) use flash::{HitFlash, HitFlashVfx, update_hit_flash_vfx, update_hit_flashes};
pub(in crate::game) use spell_hits::{HitSfxBudget, SpellHitCooldown, drive_spell_hit_feedback};
