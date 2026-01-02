#[cfg(test)]
mod tests {
    use crate::config::resources::{AudioConfig, WindowConfig};
    use crate::config::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert_eq!(config.mode, WindowMode::Windowed);
        assert_eq!(config.vsync, VsyncMode::On);
        assert_eq!(config.scale_factor, Some(1.0));
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.master_volume, 1.0);
        assert_eq!(config.music_volume, 0.8);
        assert_eq!(config.sfx_volume, 0.8);
    }

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.difficulty, Difficulty::Normal);
    }

    #[test]
    fn test_config_file_default() {
        let config = ConfigFile::default();
        assert_eq!(config.window.width, 1280);
        assert_eq!(config.audio.master_volume, 1.0);
        assert_eq!(config.game.difficulty, Difficulty::Normal);
    }

    #[test]
    fn test_difficulty_enum_variants() {
        assert_eq!(Difficulty::Easy, Difficulty::Easy);
        assert_eq!(Difficulty::Normal, Difficulty::Normal);
        assert_eq!(Difficulty::Hard, Difficulty::Hard);
        assert_ne!(Difficulty::Easy, Difficulty::Hard);
    }

    #[test]
    fn test_window_mode_enum_variants() {
        assert_eq!(WindowMode::Windowed, WindowMode::Windowed);
        assert_eq!(WindowMode::Borderless, WindowMode::Borderless);
        assert_eq!(WindowMode::Fullscreen, WindowMode::Fullscreen);
        assert_ne!(WindowMode::Windowed, WindowMode::Fullscreen);
    }

    #[test]
    fn test_vsync_mode_enum_variants() {
        assert_eq!(VsyncMode::On, VsyncMode::On);
        assert_eq!(VsyncMode::Off, VsyncMode::Off);
        assert_eq!(VsyncMode::Adaptive, VsyncMode::Adaptive);
        assert_ne!(VsyncMode::On, VsyncMode::Off);
    }

    #[test]
    fn test_config_file_serialization() {
        let config = ConfigFile::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");

        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[audio]"));
        assert!(toml_str.contains("[game]"));
        assert!(toml_str.contains("width = 1280"));
        assert!(toml_str.contains("difficulty = \"Normal\""));
    }

    #[test]
    fn test_config_file_deserialization() {
        let toml_str = r#"
            [window]
            width = 1920
            height = 1080
            mode = "Fullscreen"
            vsync = "Off"
            scale_factor = 2.0

            [audio]
            master_volume = 0.5
            music_volume = 0.6
            sfx_volume = 0.7

            [game]
            difficulty = "Hard"
        "#;

        let config: ConfigFile = toml::from_str(toml_str).expect("Failed to deserialize");

        assert_eq!(config.window.width, 1920);
        assert_eq!(config.window.height, 1080);
        assert_eq!(config.window.mode, WindowMode::Fullscreen);
        assert_eq!(config.window.vsync, VsyncMode::Off);
        assert_eq!(config.window.scale_factor, Some(2.0));

        assert_eq!(config.audio.master_volume, 0.5);
        assert_eq!(config.audio.music_volume, 0.6);
        assert_eq!(config.audio.sfx_volume, 0.7);

        assert_eq!(config.game.difficulty, Difficulty::Hard);
    }

    #[test]
    fn test_config_file_round_trip() {
        let original = ConfigFile::default();
        let toml_str = toml::to_string(&original).expect("Failed to serialize");
        let deserialized: ConfigFile = toml::from_str(&toml_str).expect("Failed to deserialize");

        assert_eq!(original.window.width, deserialized.window.width);
        assert_eq!(original.window.height, deserialized.window.height);
        assert_eq!(original.window.mode, deserialized.window.mode);
        assert_eq!(original.window.vsync, deserialized.window.vsync);
        assert_eq!(original.game.difficulty, deserialized.game.difficulty);
    }

    #[test]
    fn test_game_config_clone() {
        let config1 = GameConfig {
            difficulty: Difficulty::Hard,
        };
        let config2 = config1.clone();

        assert_eq!(config1.difficulty, config2.difficulty);
    }

    #[test]
    fn test_difficulty_default() {
        assert_eq!(Difficulty::default(), Difficulty::Normal);
    }

    #[test]
    fn test_window_mode_default() {
        assert_eq!(WindowMode::default(), WindowMode::Windowed);
    }

    #[test]
    fn test_vsync_mode_default() {
        assert_eq!(VsyncMode::default(), VsyncMode::On);
    }
}
