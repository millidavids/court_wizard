use bevy::prelude::*;

use super::components::{OnMenuBackground, ParallaxLayer};
use super::constants::*;
use super::resources::MenuBackgroundAssets;

pub(super) fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MenuBackgroundAssets {
        skybox: asset_server.load(SKYBOX_PATH),
        background: asset_server.load(BACKGROUND_PATH),
        foreground: asset_server.load(FOREGROUND_PATH),
    });
}

pub(super) fn setup(mut commands: Commands, assets: Res<MenuBackgroundAssets>) {
    spawn_layer(&mut commands, &assets.skybox, &SKYBOX);
    spawn_layer(&mut commands, &assets.background, &BACKGROUND);
    spawn_layer(&mut commands, &assets.foreground, &FOREGROUND);
}

fn spawn_layer(commands: &mut Commands, image: &Handle<Image>, config: &LayerConfig) {
    let flex_row = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for _ in 0..2 {
                row.spawn((
                    ImageNode::new(image.clone()),
                    Node {
                        width: Val::Px(config.width),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
        })
        .id();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            GlobalZIndex(config.z_index),
            OnMenuBackground,
            ParallaxLayer {
                speed: config.speed,
                image_width: config.width,
                offset: 0.0,
                flex_row,
            },
        ))
        .add_child(flex_row);
}

pub(super) fn scroll_parallax(
    time: Res<Time>,
    mut layer_query: Query<&mut ParallaxLayer>,
    mut node_query: Query<&mut Node>,
) {
    for mut layer in &mut layer_query {
        layer.offset += layer.speed * time.delta_secs();
        if layer.offset >= layer.image_width {
            layer.offset -= layer.image_width;
        }

        let new_left = Val::Px(-layer.offset);
        if let Ok(mut node) = node_query.get_mut(layer.flex_row)
            && node.left != new_left
        {
            node.left = new_left;
        }
    }
}
