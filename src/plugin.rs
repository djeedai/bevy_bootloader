use bevy::prelude::*;

use crate::{boot::update_boot, loader::LoaderPlugin};

/// Plugin to add systems related to [`Boot`] and [`Loader`].
///
/// This plugin is entirely optional. If you want more control, you can instead add manually
/// the relevant systems or plugins for the component you need:
///
/// - [`Boot`]: add the [`update_boot()`] system.
/// - [`Loader`]: add the [`LoaderPlugin`] plugin.
///
/// [`Boot`]: crate::boot::Boot
/// [`Loader`]: crate::loader::Loader
#[derive(Debug, Clone, Copy)]
pub struct BootloaderPlugin;

impl Plugin for BootloaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_boot).add_plugin(LoaderPlugin);
    }
}
