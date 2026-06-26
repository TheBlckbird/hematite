use std::{
    error::Error,
    thread::sleep,
    time::{Duration, Instant},
};

use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};

use crate::ecs::schedules::{First, Last, PostTick, PreTick, Shutdown, Startup, Tick};

pub struct App {
    world: World,
    started: bool,
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
        let tick = Schedule::new(Tick);
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
        }
    }

    pub fn add_systems<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(schedule, systems);

        self
    }

    pub fn insert_resource<R: Resource>(&mut self, value: R) -> &mut Self {
        self.world.insert_resource(value);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if self.started {
            return Err("App::run() called multiple times".into());
        }

        self.world.run_schedule(Startup);
        self.started = true;

        let tick_duration = Duration::from_millis(50);

        loop {
            let tick_start = Instant::now();

            self.world.try_run_schedule(First)?;
            self.world.try_run_schedule(PreTick)?;
            self.world.try_run_schedule(Tick)?;
            self.world.try_run_schedule(PostTick)?;
            self.world.try_run_schedule(Last)?;

            if self.world.resource::<ShutdownFlag>().is_set() {
                break;
            }

            let delta_time = Instant::now() - tick_start;

            if delta_time < tick_duration {
                let sleep_time = tick_duration - delta_time;
                sleep(sleep_time);
            }
        }

        self.world.try_run_schedule(Shutdown)?;

        Ok(())
    }
}
