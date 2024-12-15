use bevy::{
    app::{App, Startup, Update},
    prelude::Event,
    transform::systems,
    DefaultPlugins,
};
use bevy_flycam::PlayerPlugin;
use world_generator::{GenerateWorldSignal, WorldGeneratorPlugin};

pub mod lua;
pub mod mesher;
pub mod mods;
pub mod registry;
pub mod world_generator;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(WorldGeneratorPlugin::default());

    app
}
