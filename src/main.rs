//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{
    app::Startup,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{
        App, Assets, Camera3d, Circle, Color, Commands, Cuboid, DefaultPlugins, Mesh, Mesh3d,
        MeshMaterial3d, PointLight, Quat, ResMut, StandardMaterial, Transform, Vec3,
    },
    text::TextFont,
    utils::default,
};
use starlight_engine::client;

fn main() {
    client::Runtime::new().run();
}
