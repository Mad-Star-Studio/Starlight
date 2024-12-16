use std::{hash::{DefaultHasher, Hash, Hasher}, ops::RangeInclusive};

use bevy::{
    app::{Startup, Update}, color::palettes::css::{GREEN_YELLOW, MAROON, WHEAT}, prelude::{BuildChildren, ChildBuild, Commands, Component, Entity, Query, ResMut, Text}, text::{Text2d, TextColor, TextFont, TextLayout, TextSpan}, time::Time, ui::{
        AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, PositionType, UiRect, Val,
    }, utils::default, window::Monitor
};
use bevy_egui::{EguiContext, EguiContexts};
use egui::{Color32, Style, Ui, Vec2};
use egui_dock::{DockArea, TabViewer};
use egui_plot::{AxisHints, Bar, BarChart, Corner, GridMark, Legend, Line, Plot, PlotPoint, PlotPoints};

use crate::{data::world, game::perf::{Profiler, ProfilerPoint}};

pub struct DebugPlugin;

impl bevy::prelude::Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_debug_menu);
        app.add_systems(Update, update_debug_menu);
    }
}

impl Default for DebugPlugin {
    fn default() -> Self {
        DebugPlugin {}
    }
}

#[derive(Component)]
pub struct DebugMenuComponent {
    pub show: bool,
}

pub trait DebugMenuTab {
    fn title(&mut self) -> String;
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct DebugMenu {
    pub tabs: Vec<usize>,
}

impl TabViewer for DebugMenu {
    type Tab = usize;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("Tab {tab}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of tab {tab}"));
    }

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
    mut profiler: ResMut<Profiler>,
    mut query: Query<(Entity, &DebugMenuComponent, &mut Text)>,
    camera: Query<(&bevy::prelude::Transform, &bevy::prelude::Camera3d)>,
    entities_query: Query<Entity>,
    mut egui_contexts: EguiContexts
) {
    let mut _recorder_point = ProfilerPoint::new();
    {
        let _recorder = _recorder_point.record();

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

        // Egui for showing profiler data
        let ctx = egui_contexts.ctx_mut();

        let profiler_window = egui::Window::new("Profiler").show(ctx, |ui| {
            // Assemble Legend
            let legend: Legend = Legend::default();
            
            // Axis hints
            let x_axis = vec![
                AxisHints::new_y()
                    .label("Ticks ago")
                    .placement(egui_plot::HPlacement::Left),
            ];
            let y_axis = vec![
                AxisHints::new_y()
                    .label("Time taken (ms)")
                    .placement(egui_plot::HPlacement::Left),
            ];

            let plot = Plot::new("Duration (s)")
                .legend(legend.clone())
                .allow_drag(false)
                .allow_zoom(false)
                .custom_x_axes(x_axis)
                .custom_y_axes(y_axis)
                .x_axis_formatter(
                |mark: GridMark, _range: &RangeInclusive<f64>| {
                        format!("{}t", mark.value)
                    }
                )
                .y_axis_formatter(
                    |mark: GridMark, _range: &RangeInclusive<f64>| {
                        format!("{:.2}ms", mark.value)
                    }
                )
                .min_size(
                    Vec2::new(profiler.max_ticks as f32, 16.)
                );
            plot.show(ui, |plot_fn: &mut egui_plot::PlotUi| {
                // Iterate over all the perf monitors
                let mut bar_charts: Vec<BarChart> = Vec::new();
                let mut avg_charts: Vec<Line> = Vec::new();
                let mut avg_y_accum: Vec<f32> = Vec::new();
                // fill with zeroes
                for _ in 0..profiler.max_ticks + 1 {
                    avg_y_accum.push(0.0);
                }
                for monitor in profiler.iter() {
                    // Determine color based on name string (hash)
                    let mut hasher = DefaultHasher::new();
                    for byte in monitor.name.as_bytes() {
                        hasher.write_u8(*byte);
                    }
                    let hash = hasher.finish();
                    let color = Color32::from_rgb(hash as u8, (hash >> 8) as u8, (hash >> 16) as u8);

                    let avg = monitor.average();
                    let text = format!("Avg. {:.4}ms", avg);

                    // TODO: Find a way to *accurately* calculate the width of the text.
                    // This is a hacky way to do it, but it works for now.
                    plot_fn.text(
                        egui_plot::Text::new(PlotPoint::new(-1. * text.len() as f32 - 1.1, avg + avg_y_accum[0]), text)
                        .color(color)
                        .name(monitor.name.clone())
                    );

                    let mut chart: Vec<Bar> = Vec::new();
                    let mut avg_chart: Vec<[f64; 2]> = Vec::new();
                    
                    // extend to label
                    avg_chart.push([-4.0, (avg + avg_y_accum[0]) as f64]);

                    let mut i = 0;
                    for (_, point) in monitor.points.iter().enumerate() {
                        let y = point.1.duration().as_secs_f64();
                    
                        let x= point.1.age as f64;

                        chart.push(Bar::new(x, y));
                        avg_chart.push([x, (point.0 + avg_y_accum[i]) as f64]);
                        // Accumulate
                        avg_y_accum[i] += point.0;
                        i += 1;
                    }

                    let chart = BarChart::new(chart)
                        .color(color)
                        .width(0.5)
                        .name(monitor.name.clone())
                        // &[&BarGraph]
                        .stack_on(&bar_charts.iter().map(|c| c).collect::<Vec<&BarChart>>());

                    bar_charts.push(chart);
                    avg_charts.push(
                        Line::new(avg_chart)
                            .color(color)
                            .name(monitor.name.clone())
                            .style(egui_plot::LineStyle::Dashed { length: (20.) }
                        )
                    );
                }

                // Draw the plot
                for chart in bar_charts {
                    plot_fn.bar_chart(chart);
                }

                for chart in avg_charts {
                    plot_fn.line(chart);
                }
            });
        });
        
        let mut ctx = egui_contexts.ctx_mut();

        let docking_window = egui::Window::new("Docking").show(&mut ctx, |ui| {
            
        });
    }

    profiler.record_manual("Debug::update_debug_menu", _recorder_point);
}
