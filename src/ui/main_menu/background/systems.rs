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
            left: Val::Percent(0.0),
            top: Val::Px(0.0),
            // A definite width, not `Auto`: the strips below are sized in
            // percent, and a percentage child cannot resolve against a
            // shrink-to-fit parent. The strips deliberately overflow this row
            // and are clipped by the container.
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for _ in 0..2 {
                row.spawn((
                    ImageNode::new(image.clone()),
                    Node {
                        width: Val::Percent(config.width_percent),
                        height: Val::Percent(100.0),
                        // Without this the strips shrink to fit the row.
                        flex_shrink: 0.0,
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
                width_percent: config.width_percent,
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
        if layer.offset >= layer.width_percent {
            layer.offset -= layer.width_percent;
        }

        let new_left = Val::Percent(-layer.offset);
        if let Ok(mut node) = node_query.get_mut(layer.flex_row)
            && node.left != new_left
        {
            node.left = new_left;
        }
    }
}
