use bevy::{
    app::{App, Startup, Update},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    ecs::schedule::ScheduleLabel,
    prelude::Schedule,
    text::TextFont,
    DefaultPlugins,
};
use bevy_flycam::PlayerPlugin;
use bevy_meshem::prelude::{generate_voxel_mesh, Face::Top};

use crate::game::{self, registry::BlockRegistry};
mod renderer;
mod systems;

pub struct Runtime {}

struct WorldData {
    // cube, 8x8x8
    pub blocks: [[[bool; 8]; 8]; 8],
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {}
    }

    pub fn run(&self) {
        let mut app = game::app();

        // add BlockRegistry resource
        let block_registry = BlockRegistry {
            block: generate_voxel_mesh(
                [1.0, 1.0, 1.0],
                [0, 0],
                [(Top, [0, 0]); 6],
                [0.5, 0.5, 0.5],
                0.05,
                Some(0.8),
                1.0,
            ),
        };
        app.insert_resource(block_registry);

        app.add_plugins(FpsOverlayPlugin::default());
        app.add_plugins(renderer::WorldRenderer::default());
        app.add_systems(Startup, systems::startup::setup_camera);
        app.add_systems(Startup, systems::test_scene::register);
        app.add_systems(Startup, systems::test_scene::setup);
        app.add_systems(Startup, systems::debug::setup_debug_menu);
        app.add_systems(Update, systems::test_scene::update);
        app.add_systems(Update, systems::debug::update_debug_menu);
        app.run();
    }
}
