//! Tutorial system for guiding new players.

pub(crate) mod components;
mod constants;
pub(crate) mod definitions;
pub(crate) mod lifecycle;
mod plugin;
pub(crate) mod resources;
pub(crate) mod systems;
pub(super) mod tagging;
mod text_glyphs;

pub(crate) use plugin::TutorialPlugin;
