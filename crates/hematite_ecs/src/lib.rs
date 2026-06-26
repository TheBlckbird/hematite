pub mod app;
pub mod plugin;
pub mod schedules;

pub mod prelude {
    pub use crate::app::prelude::*;
    pub use crate::plugin::prelude::*;
    pub use crate::schedules::prelude::*;
    pub use bevy_ecs::prelude::*;
}
