use bevy_ecs::schedule::ScheduleLabel;

/// Run once at startup of the server
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Startup;

/// First schedule that is run every tick
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct First;

/// Executed before the main [`Tick`]
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct PreTick;

/// Executed 20 times per second or every 50ms
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Tick;

/// Executed after the main [`Tick`]
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct PostTick;

/// First schedule that is run every tick
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Last;

/// Executed when [`ShutdownFlag`] is set to `true`
#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Shutdown;
