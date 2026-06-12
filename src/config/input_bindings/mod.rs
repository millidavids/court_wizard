//! Configurable key bindings for all gameplay input.
//!
//! Each wizard archetype has its own binding context, and universal bindings
//! (action bar, activate) are shared across all archetypes.
//! All fields are `Option<KeyCode>` — `None` means the key is unbound.

mod bindings;
mod context;
mod display;
mod serde;

pub(crate) use bindings::InputBindings;
pub(crate) use context::{BindingAction, BindingContext};
pub(crate) use display::{is_bindable_key, key_display_name, key_name};
