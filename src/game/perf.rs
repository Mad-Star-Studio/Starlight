use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}, time::Instant};

use bevy::{app::{App, Last, Plugin, PostUpdate, Startup}, prelude::{Commands, ResMut, Resource}};

#[derive(Debug, Clone)]
pub struct ProfilerPoint {
    pub age: u32,
    pub begin: Instant,
    pub end: Instant
}

pub struct ProfilerPointRecordGuard<'a> {
    point: &'a mut ProfilerPoint,
}

impl Drop for ProfilerPointRecordGuard<'_> {
    fn drop(&mut self) {
        self.point.end = Instant::now();
    }
}

impl ProfilerPoint {
    pub fn new() -> ProfilerPoint {
        ProfilerPoint {
            age: 0,
            begin: Instant::now(),
            end: Instant::now()
        }
    }

    pub fn duration(&self) -> std::time::Duration {
        self.end - self.begin
    }

    pub fn record(&mut self) -> ProfilerPointRecordGuard {
        self.begin = Instant::now();
        ProfilerPointRecordGuard {
            point: self
        }
    }
}

#[derive(Debug)]
pub struct ProfilerMonitor {
    pub name: String,
    pub points: Vec<(f32, ProfilerPoint)>
}

impl ProfilerMonitor {
    pub fn average(&self) -> f32 {
        let sum: f32 = self.points.iter().map(|(_, point)| {
            point.duration().as_secs_f32()
        }).sum();
        if (self.points.len() as f32) == 0.0 {
            return 0.0;
        }
        sum / self.points.len() as f32
    }
}

#[derive(Debug, Resource)]
pub struct Profiler {
    monitors: HashMap<String, ProfilerMonitor>,
    pub max_ticks: u32
}

pub struct ProfilerRecordGuard<'a> {
    monitor: String,
    profiler: &'a mut Profiler,
    point: ProfilerPoint
}

impl Drop for ProfilerRecordGuard<'_> {
    fn drop(&mut self) {
        self.point.end = Instant::now();
        let monitor = self.profiler.monitors.get_mut(&self.monitor).unwrap();
        monitor.points.push((monitor.average(), self.point.clone()));
    }
}

impl Profiler {
    fn ensure_monitor(&mut self, name: &str) {
        if !self.monitors.contains_key(name) {
            self.monitors.insert(name.to_string(), ProfilerMonitor {
                name: name.to_string(),
                points: Vec::new()
            });
        }
    }

    pub fn record(&mut self, name: &str) -> ProfilerRecordGuard {
        self.ensure_monitor(name);
        let monitor = self.monitors.get_mut(name).unwrap();

        ProfilerRecordGuard {
            monitor: name.to_string(),
            profiler: self,
            point: ProfilerPoint {
                age: 0,
                begin: Instant::now(),
                end: Instant::now()
            }
        }
    }

    pub fn record_manual(&mut self, name: &str, point: ProfilerPoint) {
        self.ensure_monitor(name);
        let monitor = self.monitors.get_mut(name).unwrap();
        monitor.points.push((monitor.average(), point));
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProfilerMonitor> {
        // return references to the second element of the hashmap
        self.monitors.values()
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Plugin                                   */
/* -------------------------------------------------------------------------- */

pub struct ProfilerPlugin {

}

impl Plugin for ProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Profiler {
            monitors: HashMap::new(),
            max_ticks: 300
        });
        app.add_systems(Startup, sys_setup);
        app.add_systems(PostUpdate, sys_update);
    }
}

impl Default for ProfilerPlugin {
    fn default() -> Self {
        ProfilerPlugin {}
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Systems                                  */
/* -------------------------------------------------------------------------- */

fn sys_setup(mut commands: Commands) {

}

fn sys_update(profiler: ResMut<Profiler>) {
    let mut record: ProfilerPoint = ProfilerPoint::new();

    let profiler = Mutex::new(profiler);
    {
        let _recorder = record.record();

        // monitor index, point index
        let remove_indexes: Vec<(usize, usize)>;
        {
            let profiler = profiler.lock().unwrap();
            remove_indexes = profiler.monitors.iter().enumerate().flat_map(|(i, monitor)| {
                monitor.1.points.iter().enumerate().filter_map(|(j, point)| {
                    if point.1.age >= profiler.max_ticks {
                        Some((i, j))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            }).collect();
        }

        let mut profiler = profiler.lock().unwrap();
        for (i, j) in remove_indexes {
            // Remove element i from the hashmap
            let id = profiler.monitors.keys().nth(i).unwrap().clone();
            profiler.monitors.get_mut(&id).unwrap().points.remove(j);
        }

        // Update the age of all the points
        for monitor in profiler.monitors.values_mut() {
            for point in monitor.points.iter_mut() {
                point.1.age += 1;
            }
        }
    }
    profiler.lock().unwrap().record_manual("Profiler::sys_update", record);
}
