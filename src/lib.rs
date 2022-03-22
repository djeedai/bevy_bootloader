#![deny(
    warnings,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]

//! App startup and resource management plugin for the Bevy game engine
//!
//! 🚥 Bevy Bootloader is a utility crate for the Bevy game engine. It provides two main features:
//!
//! - A boot sequence helper, `Boot`, used to display a minimal user feedback like a progress bar while
//!   critical resources are loading and other features (like font asset / text rendering) are not available.
//! - A resource loading helper, `Loader`, used to easily wait for a group of resources to load without
//!   the verbosity of having to manage all those resources individually.
//!
//! Combined together, those two components allow loading at app startup a set of initial resources
//! before any other screen or menu is visible, while providing user feedback.
//!
//! # Example
//!
//! Add the `BootloaderPlugin` to your app:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_bootloader::*;
//!
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(BootloaderPlugin)
//!     .run();
//! ```
//!
//! Queue boot-time critical assets, and insert a [`BootBundle`]:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_bootloader::*;
//! # fn setup(mut commands: Commands) {
//! // Queue boot-time resources
//! let mut loader = Loader::new();
//! loader.enqueue("logo.png");
//! loader.enqueue("music.ogg");
//! loader.submit();
//!
//! // Insert a boot bundle
//! commands.spawn_bundle(BootBundle::new(loader));
//! # }
//! ```
//!
//! Check the boot state with either of [`Boot::progress()`], [`Boot::smoothed_progress()`], or
//! [`Loader::is_done()`]. For example, use [`Boot::smoothed_progress()`] to smoothly update a
//! progress bar made of a `Sprite`:
//! ```
//! # use bevy::prelude::*;
//! # use bevy_bootloader::*;
//! # const PROGRESS_BAR_SIZE: f32 = 200.;
//! # const PROGRESS_BAR_THICKNESS: f32 = 3.;
//! # #[derive(Component)]
//! # struct ProgressBar;
//! fn update_progress_bar(
//!     boot_query: Query<&Boot>,
//!     mut sprite_query: Query<(&mut Transform, &mut Sprite), With<ProgressBar>>,
//! ) {
//!     if let Ok(boot) = boot_query.get_single() {
//!         // Update the progress bar based on the fraction of assets already loaded, smoothed
//!         // with a snappy animation to be visually pleasant without too much artifically
//!         // delaying the boot sequence.
//!         let smoothed_progress = boot.smoothed_progress();
//!         let (mut transform, mut sprite) = sprite_query.single_mut();
//!         let size = PROGRESS_BAR_SIZE * smoothed_progress;
//!         // The sprite is a rect centered at the transform position, so move by half size to
//!         // keep aligned to the left while width grows.
//!         transform.translation.x = (size - PROGRESS_BAR_SIZE) / 2.;
//!         sprite.custom_size = Some(Vec2::new(size, PROGRESS_BAR_THICKNESS));
//!     }
//! }
//! ```
//!
//! See the `bootloader` example for the full code.
//!

mod boot;
mod loader;
mod plugin;

pub use boot::{update_boot, Boot, BootBundle};
pub use loader::{Loader, LoaderPlugin, LoaderStage};
pub use plugin::BootloaderPlugin;
