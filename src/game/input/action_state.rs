//! Source-agnostic controller action state.
//!
//! Both controller readers — the Steam Input poll (`crate::steam::input`) and the
//! gilrs fallback adapter (`gamepad::systems::gilrs_adapter`) — write into the
//! single [`GamepadActionState`] resource each frame. Every downstream translation
//! system reads from this resource instead of touching raw `Gamepad` components,
//! so the game logic is identical whether a pad is captured by Steam Input or read
//! directly through gilrs.
//!
//! Digital inputs are modeled as [`GamepadAction`] and tracked with Bevy's own
//! [`ButtonInput`] (which gives `just_pressed`/`just_released` edge detection). The
//! producer for the frame `.clear()`s and re-`press`/`release`s every action from
//! the current button state, so edges are correct even on the frame the active
//! source switches. Analog inputs (sticks/triggers) are stored generically and
//! interpreted per-context by the consuming systems (aim cursor vs. avatar
//! movement vs. menu navigation), mirroring the original gilrs design.

use bevy::input::ButtonInput;
use bevy::prelude::*;

/// A semantic digital controller action. The Steam Input In-Game-Actions manifest
/// (`game_actions_<appid>.vdf`) defines one action per variant; the gilrs fallback
/// maps physical buttons to these with the default layout.
///
/// Direction actions (`Ability*`) are intentionally generic: in gameplay they are
/// the per-archetype D-pad abilities, and in menus they drive focus navigation —
/// the two are never consumed in the same context, exactly as with the original
/// raw D-pad reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GamepadAction {
    /// Primary "activate" button (default South / A / Cross): cast, rune activate,
    /// roulette fire.
    Activate,
    /// Primary cast trigger (default Right Trigger).
    PrimaryCast,
    /// Secondary cast / aim trigger (default Left Trigger).
    SecondaryCast,
    /// Open the spell book (default West / X / Square).
    OpenSpellBook,
    /// Open the cauldron menu (default North / Y / Triangle).
    OpenCauldron,
    /// Pause / escape (default Start).
    Pause,
    /// D-pad up — archetype ability in gameplay, navigate up in menus.
    AbilityUp,
    /// D-pad down — archetype ability in gameplay, navigate down in menus.
    AbilityDown,
    /// D-pad left — archetype ability in gameplay, navigate left in menus.
    AbilityLeft,
    /// D-pad right — archetype ability in gameplay, navigate right in menus.
    AbilityRight,
    /// Confirm in menus (default South / A / Cross).
    UIConfirm,
    /// Back / cancel in menus (default East / B / Circle).
    UIBack,
    /// Next tab (default Right Bumper).
    TabNext,
    /// Previous tab (default Left Bumper).
    TabPrev,
    /// Confirm in the wizard-tower study graph (default South / A / Cross).
    StudyConfirm,
}

impl GamepadAction {
    /// Every digital action, for handle resolution and per-frame polling.
    pub(crate) const ALL: [GamepadAction; 15] = [
        GamepadAction::Activate,
        GamepadAction::PrimaryCast,
        GamepadAction::SecondaryCast,
        GamepadAction::OpenSpellBook,
        GamepadAction::OpenCauldron,
        GamepadAction::Pause,
        GamepadAction::AbilityUp,
        GamepadAction::AbilityDown,
        GamepadAction::AbilityLeft,
        GamepadAction::AbilityRight,
        GamepadAction::UIConfirm,
        GamepadAction::UIBack,
        GamepadAction::TabNext,
        GamepadAction::TabPrev,
        GamepadAction::StudyConfirm,
    ];

    /// The action name as it appears in the Steam Input IGA manifest. MUST match
    /// `assets/controller_config/game_actions_4550880.vdf` exactly.
    pub(crate) fn manifest_name(self) -> &'static str {
        match self {
            GamepadAction::Activate => "Activate",
            GamepadAction::PrimaryCast => "PrimaryCast",
            GamepadAction::SecondaryCast => "SecondaryCast",
            GamepadAction::OpenSpellBook => "OpenSpellBook",
            GamepadAction::OpenCauldron => "OpenCauldron",
            GamepadAction::Pause => "Pause",
            GamepadAction::AbilityUp => "AbilityUp",
            GamepadAction::AbilityDown => "AbilityDown",
            GamepadAction::AbilityLeft => "AbilityLeft",
            GamepadAction::AbilityRight => "AbilityRight",
            GamepadAction::UIConfirm => "UIConfirm",
            GamepadAction::UIBack => "UIBack",
            GamepadAction::TabNext => "TabNext",
            GamepadAction::TabPrev => "TabPrev",
            GamepadAction::StudyConfirm => "StudyConfirm",
        }
    }
}

/// A generic analog controller input. Sticks/triggers are interpreted per-context
/// by the consuming systems rather than being bound to a single semantic action,
/// because the same physical stick means different things per archetype/screen
/// (virtual cursor, avatar movement, menu navigation, study reticle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AnalogAction {
    /// Left stick (IGA `AimStick`).
    AimStick,
    /// Right stick (IGA `Camera`).
    Camera,
    /// Left trigger as an analog value (IGA `ZoomOut`; study-tab zoom).
    ZoomOut,
    /// Right trigger as an analog value (IGA `ZoomIn`; study-tab zoom).
    ZoomIn,
}

impl AnalogAction {
    pub(crate) const ALL: [AnalogAction; 4] = [
        AnalogAction::AimStick,
        AnalogAction::Camera,
        AnalogAction::ZoomOut,
        AnalogAction::ZoomIn,
    ];

    pub(crate) fn manifest_name(self) -> &'static str {
        match self {
            AnalogAction::AimStick => "AimStick",
            AnalogAction::Camera => "Camera",
            AnalogAction::ZoomOut => "ZoomOut",
            AnalogAction::ZoomIn => "ZoomIn",
        }
    }
}

/// Which kind of controller is currently active — drives glyph selection. Mapped
/// from Steam Input's `GetInputTypeForHandle` on the Steam path, or from USB
/// vendor id on the gilrs fallback path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ControllerKind {
    #[default]
    Xbox,
    PlayStation,
    Switch,
    SteamDeck,
}

/// The per-frame snapshot every controller-reading system consumes.
///
/// Filled by exactly one producer per frame (Steam Input poll OR gilrs adapter).
/// Analog values are raw (deadzone/response-curve via `GamepadAimSettings` is
/// applied by the consumer, as before).
#[derive(Resource, Debug, Default)]
pub(crate) struct GamepadActionState {
    /// Digital action edge/press state. Owned by the active producer, which
    /// clears and re-presses it each frame.
    pub buttons: ButtonInput<GamepadAction>,
    /// Left stick, range roughly [-1, 1] per axis (Y up-positive).
    pub left_stick: Vec2,
    /// Right stick, range roughly [-1, 1] per axis (Y up-positive).
    pub right_stick: Vec2,
    /// Left trigger pull, [0, 1].
    pub left_trigger: f32,
    /// Right trigger pull, [0, 1].
    pub right_trigger: f32,
    /// Steam Input handle of the active controller, if the Steam path is driving.
    pub active_steam_handle: Option<u64>,
    /// gilrs `Gamepad` entity of the active controller, if the fallback is driving.
    pub active_gilrs_entity: Option<Entity>,
    /// Active controller kind, for glyph selection.
    pub controller_kind: ControllerKind,
}

impl GamepadActionState {
    #[inline]
    pub(crate) fn pressed(&self, action: GamepadAction) -> bool {
        self.buttons.pressed(action)
    }

    #[inline]
    pub(crate) fn just_pressed(&self, action: GamepadAction) -> bool {
        self.buttons.just_pressed(action)
    }

    #[inline]
    pub(crate) fn just_released(&self, action: GamepadAction) -> bool {
        self.buttons.just_released(action)
    }

    /// Releases all digital actions and zeros analog inputs. Used when no
    /// controller is the active device (mouse/keyboard) so stale gamepad state
    /// never leaks to consumers. A still-held action gets exactly one
    /// `just_released` on the frame this is first called, then stays released.
    pub(crate) fn clear_inputs(&mut self) {
        self.buttons.clear();
        for action in GamepadAction::ALL {
            self.buttons.release(action);
        }
        self.left_stick = Vec2::ZERO;
        self.right_stick = Vec2::ZERO;
        self.left_trigger = 0.0;
        self.right_trigger = 0.0;
        self.active_gilrs_entity = None;
        self.active_steam_handle = None;
    }
}
