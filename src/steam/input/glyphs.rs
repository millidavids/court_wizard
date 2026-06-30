//! Loads the active controller's official Steam Input glyph PNGs.
//!
//! Steam returns an absolute filesystem path to a PNG for each action origin, so
//! we read + decode those directly into `Image` assets (they live outside
//! `assets/`, so the `AssetServer` can't load them). The result is cached in
//! [`SteamGlyphs`] keyed by action; UI prompts use it when present and fall back
//! to the Kenney font glyph (the placeholder) otherwise.
//!
//! We use the newer `GetGlyphPNGForActionOrigin` (per-direction D-pad art, sized)
//! via raw FFI — the safe `Input` wrapper only exposes the *legacy* variant, whose
//! D-pad glyph is a single generic "+" for every direction.

use std::collections::HashMap;
use std::ffi::CStr;

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

    // Fetch the Steam Input interface once for the whole refresh.
    // SAFETY: only reached once handles resolved (i.e. after Steam Input init).
    let iface = unsafe { steamworks_sys::SteamAPI_SteamInput_v006() };
    if iface.is_null() {
        return;
    }

    let input = client.input();
    let mut loaded: HashMap<GamepadAction, Handle<Image>> = HashMap::new();
    for action in GamepadAction::ALL {
        let path = if let Some(xbox) = dpad_xbox_origin(action) {
            // Directional D-pad abilities: force the per-direction origin. The
            // actual binding usually resolves to the WHOLE d-pad (`DPad_Move`),
            // whose glyph can't show a direction. `GetActionOriginFromXboxOrigin`
            // maps the abstract direction to this controller's specific
            // per-direction origin (PS5 d-pad up, Xbox d-pad up, …).
            // SAFETY: `iface` is null-checked above; `handle` is the active
            // controller handle.
            let origin = unsafe {
                steamworks_sys::SteamAPI_ISteamInput_GetActionOriginFromXboxOrigin(
                    iface, handle, xbox,
                )
            };
            glyph_png_path(iface, origin)
        } else {
            let Some(&action_handle) = handles.digital.get(&action) else {
                continue;
            };
            // An action can have several origins (e.g. a keyboard fallback
            // alongside the binding); take the first that yields a glyph PNG.
            let origins = input.get_digital_action_origins(handle, set, action_handle);
            origins
                .iter()
                .find_map(|&origin| glyph_png_path(iface, origin))
        };
        let Some(path) = path else {
            continue; // action not bound to anything glyph-able in this set
        };
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            // Steam may not have finished downloading the glyph PNG yet; since we
            // don't commit `last_key` on an empty load, this retries next frame.
            Err(e) => {
                debug!("[Steam Input] glyph '{path}' not readable yet: {e}");
                continue;
            }
        };
        match Image::from_buffer(
            &bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true, // Steam glyph PNGs are sRGB display art.
            ImageSampler::Default,
            RenderAssetUsages::default(),
        ) {
            Ok(image) => {
                loaded.insert(action, images.add(image));
            }
            Err(e) => warn!("[Steam Input] failed to decode glyph '{path}': {e:?}"),
        }
    }

    // Only commit (swap in the new textures + cache the key) once at least one
    // glyph actually loaded — otherwise a not-yet-downloaded set would cache an
    // empty result for the session. On an empty load we keep the previous glyphs
    // visible and retry next frame.
    if loaded.is_empty() {
        return;
    }
    for (_, h) in glyphs.by_action.drain() {
        images.remove(&h);
    }
    glyphs.by_action = loaded;
    *last_key = Some(key);
}

/// The abstract Xbox D-pad direction for a directional ability action, or `None`
/// for non-directional actions. Used to force a per-direction glyph instead of
/// the whole-d-pad glyph the binding usually resolves to. Up=North, Down=South,
/// Left=West, Right=East — matching `GamepadAction::default_button`.
fn dpad_xbox_origin(action: GamepadAction) -> Option<steamworks_sys::EXboxOrigin> {
    match action {
        GamepadAction::AbilityUp => Some(steamworks_sys::EXboxOrigin::k_EXboxOrigin_DPad_North),
        GamepadAction::AbilityDown => Some(steamworks_sys::EXboxOrigin::k_EXboxOrigin_DPad_South),
        GamepadAction::AbilityLeft => Some(steamworks_sys::EXboxOrigin::k_EXboxOrigin_DPad_West),
        GamepadAction::AbilityRight => Some(steamworks_sys::EXboxOrigin::k_EXboxOrigin_DPad_East),
        _ => None,
    }
}

/// Absolute path to the official Steam Input glyph PNG for an action origin, via
/// the newer per-direction API (`GetGlyphPNGForActionOrigin`). The safe `Input`
/// wrapper only exposes the legacy generic-D-pad variant, so we go through
/// `steamworks-sys` (same pattern as the haptics FFI). Medium (128px) is ample
/// for our ≤40px UI slots and a quarter of Large's texture footprint.
fn glyph_png_path(
    iface: *mut steamworks_sys::ISteamInput,
    origin: steamworks_sys::EInputActionOrigin,
) -> Option<String> {
    // SAFETY: `iface` is the initialized Steam Input interface (null-checked by
    // the caller). The returned C string is owned by Steam; we copy it out
    // immediately.
    unsafe {
        let ptr = steamworks_sys::SteamAPI_ISteamInput_GetGlyphPNGForActionOrigin(
            iface,
            origin,
            steamworks_sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Medium,
            0,
        );
        if ptr.is_null() {
            return None;
        }
        let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        (!s.is_empty()).then_some(s)
    }
}
