
use bevy::{
    app::App,
    DefaultPlugins,
};
use bevy_flycam::PlayerPlugin;
use perf::ProfilerPlugin;
use world_generator::WorldGeneratorPlugin;
use world_worldmgr::WorldManagerPlugin;

pub mod registry;
pub mod world_generator;
pub mod world_observation;
pub mod world_worldmgr;
pub mod perf;
pub mod debug;

pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ProfilerPlugin::default());
    app.add_plugins(PlayerPlugin);
    app.add_plugins(debug::DebugPlugin::default());
    app.add_plugins(WorldGeneratorPlugin::default());
    app.add_plugins(WorldManagerPlugin::default());
    app.add_plugins(world_observation::WorldObservationPlugin::default());

    app
}