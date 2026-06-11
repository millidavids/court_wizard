mod plugin;
mod states;
mod systems;

pub use plugin::StatePlugin;
pub use states::{
    AppState, InGameState, MenuState, MetaGameState, MultiplayerGameState, PauseMenuState,
    SplashState,
};
