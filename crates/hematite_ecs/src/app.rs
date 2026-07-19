use std::{
    collections::HashSet,
    error::Error,
    thread::sleep,
    time::{Duration, Instant},
};

use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};

use crate::{
    plugin::{Plugin, PluginGroup},
    schedules::{First, Last, PostTick, PreTick, Shutdown, Startup, TickUpdate},
};

pub mod prelude {
    pub use super::App;
    pub use super::ShutdownFlag;
}

pub struct App {
    world: World,
    started: bool,
    plugins: Vec<Box<dyn Plugin>>,
    plugin_names: HashSet<String>,
}

#[derive(Debug, Resource, Default)]
pub struct ShutdownFlag(bool);

impl ShutdownFlag {
    pub fn new() -> Self {
        Self(false)
    }

    pub fn is_set(&self) -> bool {
        self.0
    }
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();

        let startup = Schedule::new(Startup);
        let first = Schedule::new(First);
        let pre_tick = Schedule::new(PreTick);
        let tick = Schedule::new(TickUpdate);
        let post_tick = Schedule::new(PostTick);
        let last = Schedule::new(Last);
        let shutdown = Schedule::new(Shutdown);

        world.add_schedule(startup);
        world.add_schedule(first);
        world.add_schedule(pre_tick);
        world.add_schedule(tick);
        world.add_schedule(post_tick);
        world.add_schedule(last);
        world.add_schedule(shutdown);

        world.init_resource::<ShutdownFlag>();

        Self {
            world,
            started: false,
            plugins: Vec::new(),
            plugin_names: HashSet::new(),
        }
    }

    /// Adds one or more systems to the [`Schedule`] matching the provided [`ScheduleLabel`].
    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(schedule, systems);
        self
    }

    /// Inserts a new resource with the given `value`.
    ///
    /// Resources are "unique" data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data.
    pub fn insert_resource<R: Resource>(&mut self, value: R) -> &mut Self {
        self.world.insert_resource(value);
        self
    }

    /// Initializes a new resource and returns the [`ComponentId`] created for it.
    ///
    /// If the resource already exists, nothing happens.
    ///
    /// The value given by the [`FromWorld::from_world`] method will be used.
    /// Note that any resource with the [`Default`] trait automatically implements [`FromWorld`],
    /// and those default values will be here instead.
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if self.started {
            return Err("App::run() called multiple times".into());
        }

        self.started = true;
        self.world.try_run_schedule(Startup)?;

        let tick_length = Duration::from_millis(50);

        loop {
            let tick_start = Instant::now();

            self.world.try_run_schedule(First)?;
            self.world.try_run_schedule(PreTick)?;
            self.world.try_run_schedule(TickUpdate)?;
            self.world.try_run_schedule(PostTick)?;
            self.world.try_run_schedule(Last)?;

            if self.world.resource::<ShutdownFlag>().is_set() {
                break;
            }

            let elapsed_time = Instant::now() - tick_start;

            if elapsed_time < tick_length {
                let sleep_time = tick_length - elapsed_time;
                sleep(sleep_time);
            }
        }

        self.world.try_run_schedule(Shutdown)?;

        Ok(())
    }

    pub fn add_plugins(&mut self, plugins: impl PluginGroup) -> &mut Self {
        plugins.add_to_app(self);
        self
    }

    pub(crate) fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        if plugin.is_unique() && self.plugin_names.contains(plugin.name()) {
            panic!("Plugin {} can only be added once to the app", plugin.name())
        }

        self.plugin_names.insert(plugin.name().to_string());
        plugin.build(self);
        self.plugins.push(plugin);

        self
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
