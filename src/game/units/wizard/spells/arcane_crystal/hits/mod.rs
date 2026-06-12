//! Hit detection for spells absorbed by crystals.

mod beam;
mod chain_lightning;
mod fireball;
mod magic_missile;
mod meteor;

pub(super) use beam::detect_beam_hits;
pub(super) use chain_lightning::detect_chain_lightning_hits;
pub(super) use fireball::detect_fireball_hits;
pub(super) use magic_missile::detect_magic_missile_hits;
pub(super) use meteor::detect_meteor_hits;
