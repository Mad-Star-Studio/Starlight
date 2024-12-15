use bevy::{
    color::palettes::css::{GREEN_YELLOW, MAROON, WHEAT},
    prelude::{BuildChildren, ChildBuild, Commands, Component, Entity, Query, Text},
    text::{Text2d, TextColor, TextFont, TextLayout, TextSpan},
    time::Time,
    ui::{
        AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, PositionType, UiRect, Val,
    },
    utils::default,
};
use bevy_egui::{EguiContext, EguiContexts};
use egui::Color32;

use crate::data::world;

#[derive(Component)]
pub struct DebugMenuComponent {
    pub show: bool,
}

pub fn setup_debug_menu(mut commands: Commands) {
    commands.spawn((
        Text("Debug Menu".to_string()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        DebugMenuComponent { show: true },
    ));
}

pub fn update_debug_menu(
    time: bevy::prelude::Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &DebugMenuComponent, &mut Text)>,
    camera: Query<(&bevy::prelude::Transform, &bevy::prelude::Camera3d)>,
    entities_query: Query<Entity>,
    mut egui_contexts: EguiContexts
) {
    let camera_transform = camera.iter().next().unwrap().0;
    for (entity, menu, text) in query.iter_mut() {
        if menu.show {
            // Features:
            // - Entity count
            // - FPS Counter
            // - Current coordinates
            // - Direction facing (N, S, E, W)

            // Get data first
            let mut entities = 0;
            let mut fps = 60.0;
            let mut coords = (0, 0, 0);
            let mut dir = "North";

            // Get entity count
            {
                let entities_count = entities_query.iter().count();
                entities = entities_count;
            }
            // Calulate FPS via delta time
            {
                let delta = time.delta_secs_f64();
                fps = 1.0 / delta;
            }
            // Get current coordinates
            {
                let player = camera_transform;
                let x = player.translation.x as i32;
                let y = player.translation.y as i32;
                let z = player.translation.z as i32;

                coords = (x, y, z);
            }
            // Get direction facing
            {
                let rotation = camera_transform.rotation;
                // -180 to 180, wrapped
                let facing = rotation.y.to_degrees();
                dir = match facing {
                    -180.0..=-135.0 => "North",
                    -135.0..=-45.0 => "East",
                    -45.0..=45.0 => "South",
                    45.0..=135.0 => "West",
                    135.0..=180.0 => "North",
                    _ => "North",
                };
            }

            // Apply data
            commands.entity(entity).insert(Text(format!(
                "Entities: {}\nFPS: {:.2}\nCoords: {:?}\nFacing: {}",
                entities, fps, coords, dir
            )));
        }
    }
}
