//! Loads the active controller's official Steam Input glyph PNGs.
//!
//! Steam returns an absolute filesystem path to a PNG for each action origin, so
//! we read + decode those directly into `Image` assets (they live outside
//! `assets/`, so the `AssetServer` can't load them). The result is cached in
//! [`SteamGlyphs`] keyed by action; UI prompts use it when present and fall back
//! to the Kenney font glyph (the placeholder) otherwise.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy_steamworks::Client;

use super::handles::SteamInputHandles;
use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::focus::components::ModalOverlay;
use crate::ui::gamepad_glyphs::SteamGlyphs;

/// Re-gathers glyphs only when the active controller or action set changes
/// (decoding PNGs every frame would be wasteful). Old textures are dropped before
/// reloading so they don't accumulate in `Assets<Image>`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh_steam_glyphs(
    client: Res<Client>,
    handles: Res<SteamInputHandles>,
    action_state: Res<GamepadActionState>,
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    modals: Query<(), With<ModalOverlay>>,
    mut images: ResMut<Assets<Image>>,
    mut glyphs: ResMut<SteamGlyphs>,
    mut last_key: Local<Option<(u64, u64)>>,
) {
    if !handles.resolved {
        return;
    }

    let Some(handle) = action_state.active_steam_handle else {
        // No Steam pad: clear so prompts use the Kenney placeholder.
        if !glyphs.by_action.is_empty() {
            for (_, h) in glyphs.by_action.drain() {
                images.remove(&h);
            }
        }
        *last_key = None;
        return;
    };

    // Gather glyphs for the action set that's currently active (mirrors
    // `activate_steam_action_sets`), so prompts shown in this context resolve.
    let sp_running = sp_state.is_some_and(|s| *s.get() == InGameState::Running);
    let mp_running = mp_state.is_some_and(|s| *s.get() == MultiplayerGameState::Running);
    let gameplay = (sp_running || mp_running) && modals.is_empty();
    let set = if gameplay {
        handles.gameplay_set
    } else {
        handles.menu_set
    };

    let key = (handle, set);
    if *last_key == Some(key) {
        return;
    }
    *last_key = Some(key);

    for (_, h) in glyphs.by_action.drain() {
        images.remove(&h);
    }

    let input = client.input();
    let mut loaded: HashMap<GamepadAction, Handle<Image>> = HashMap::new();
    for action in GamepadAction::ALL {
        let Some(&action_handle) = handles.digital.get(&action) else {
            continue;
        };
        let origins = input.get_digital_action_origins(handle, set, action_handle);
        let Some(&origin) = origins.first() else {
            continue; // action not bound in this set
        };
        let path = input.get_glyph_for_action_origin(origin);
        if path.is_empty() {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        match Image::from_buffer(
            &bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        ) {
            Ok(image) => {
                loaded.insert(action, images.add(image));
            }
            Err(e) => warn!("[Steam Input] failed to decode glyph '{path}': {e:?}"),
        }
    }
    glyphs.by_action = loaded;
}
