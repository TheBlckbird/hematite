use std::any::type_name;

use variadics_please::all_tuples;

use crate::app::App;

pub mod prelude {
    pub use super::Plugin;
}

pub trait Plugin {
    fn build(&self, app: &mut App);

    /// Whether the plugin can only occur once in the app.
    ///
    /// `true` by default
    fn is_unique(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        type_name::<Self>()
    }
}

pub trait Plugins {
    fn add_to_app(self, app: &mut App);
}

impl<P: Plugin + 'static> Plugins for P {
    fn add_to_app(self, app: &mut App) {
        app.add_boxed_plugin(Box::new(self));
    }
}

macro_rules! impl_plugins_tuple {
    ($($name:ident),*) => {
        impl<$($name: Plugin + 'static),*> Plugins for ($($name,)*) {
            #[allow(unused)]
            fn add_to_app(self, app: &mut App) {
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(app.add_boxed_plugin(Box::new($name));)*
            }
        }
    };
}

all_tuples!(impl_plugins_tuple, 0, 15, P);
