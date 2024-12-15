use bevy::{
    color::palettes::css::{GREEN_YELLOW, MAROON, WHEAT},
    prelude::{BuildChildren, ChildBuild, Commands, Component},
    text::{Text2d, TextColor, TextFont, TextLayout, TextSpan},
    ui::{
        AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, PositionType, UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub struct DebugMenuComponent {
    pub show: bool,
}

pub fn setup_debug_menu(mut commands: Commands) {
    commands.spawn((
        TextSpan::new("Debug Menu"),
        TextFont { ..default() },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
    ));
}

pub fn update_debug_menu() {}
