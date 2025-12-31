use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
use std::fs;

mod config;
use config::{ConfigFile, ConfigPlugin};

fn main() {
    // Pre-load config for initial window setup
    let config = load_initial_config();

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Game".into(),
                    resolution: WindowResolution::new(config.window.width, config.window.height)
                        .with_scale_factor_override(
                            config.window.scale_factor.unwrap_or(1.0) as f32
                        ),
                    present_mode: match config.window.vsync.as_str() {
                        "off" => PresentMode::AutoNoVsync,
                        _ => PresentMode::AutoVsync,
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(ConfigPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

/// Load config before App initialization for initial window setup
fn load_initial_config() -> ConfigFile {
    let config_path = "config.toml";
    if std::path::Path::new(config_path).exists()
        && let Ok(contents) = fs::read_to_string(config_path)
        && let Ok(config) = toml::from_str::<ConfigFile>(&contents)
    {
        return config;
    }
    ConfigFile::default()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
