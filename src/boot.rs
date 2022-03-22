use bevy::prelude::*;

use crate::loader::Loader;

/// Component for the boot sequence entity holding the [`Loader`] which handles
/// the critical boot assets.
///
/// This component in itself simply manages some smoother progress value for updating
/// some minimal UI like a progress bar. It relies on an associated [`Loader`] to
/// report the progress of loading a batch of assets.
///
/// If using the default update system, [`update_boot()`], then this component must be
/// added to an entity with a [`Loader`] component. This can be done easily by adding
/// a [`BootBundle`].
#[derive(Debug, Component)]
pub struct Boot {
    /// Actual realtime asset loading progress, based on number of loaded assets.
    progress: f32,
    /// Smoother progress, based on [`progress`] and smoothed for a nice animated effect.
    ///
    /// [`progress`]: Boot::progress
    smoothed_progress: f32,
    /// Maximum progress speed, in percent per second. This is the maximum speed at which
    /// [`smoothed_progress`] tries to catch up to [`progress`].
    ///
    /// [`progress`]: Boot::progress
    /// [`smoothed_progress`]: Boot::smoothed_progress
    speed: f32,
    /// Collection of entities of the boot screen, to delete once boot is done.
    entities: Vec<Entity>,
}

impl Default for Boot {
    fn default() -> Self {
        Boot {
            progress: 0.0,
            smoothed_progress: 0.0,
            speed: 1.0, // percent per second; 1.0 = 100% in 1 second
            entities: vec![],
        }
    }
}

impl Boot {
    /// Create a default object.
    pub fn new() -> Self {
        Boot::default()
    }

    /// Update the boot progress based on the actual `progress` in \[0:1\] and the current
    /// frame delta time in seconds (for progress smoothing).
    ///
    /// This is called automatically by the default update system, [`update_boot()`], based on
    /// the progress reported by the associated [`Loader`].
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_bootloader::*;
    /// # fn calc_progress() -> f32 { 0.5 }
    /// fn update(time: Res<Time>, mut query: Query<&mut Boot>) {
    ///   let progress = calc_progress();
    ///   let mut boot = query.single_mut();
    ///   boot.set_progress(progress, time.delta_seconds());
    /// }
    /// ```
    pub fn set_progress(&mut self, progress: f32, dt: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        let delta_p = (self.progress - self.smoothed_progress) / self.speed;
        let smoothed_progress = self.smoothed_progress + dt * delta_p;
        self.smoothed_progress = smoothed_progress.min(self.progress);
    }

    /// Get the actual loading progress, in \[0:1\].
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_bootloader::*;
    /// # let boot = Boot::new();
    /// println!("Progress: {}%", boot.progress() * 100.0);
    /// ```
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Get the smoothed progress, in \[0:1\], which is always less than or equal to the actual [`progress()`].
    ///
    /// # Example
    ///
    /// The smoothed progress value is typically used to animate some kind of minimal UI like a progress bar:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_bootloader::*;
    /// # const PROGRESS_BAR_SIZE: f32 = 200.;
    /// # const PROGRESS_BAR_THICKNESS: f32 = 3.;
    /// # #[derive(Component)]
    /// # struct ProgressBar;
    /// fn update_progress_bar(
    ///     boot_query: Query<&Boot>,
    ///     mut sprite_query: Query<(&mut Transform, &mut Sprite), With<ProgressBar>>,
    /// ) {
    ///     if let Ok(boot) = boot_query.get_single() {
    ///         // Update the progress bar based on the fraction of assets already loaded, smoothed
    ///         // with a snappy animation to be visually pleasant without too much artifically
    ///         // delaying the boot sequence.
    ///         let smoothed_progress = boot.smoothed_progress();
    ///         let (mut transform, mut sprite) = sprite_query.single_mut();
    ///         let size = PROGRESS_BAR_SIZE * smoothed_progress;
    ///         // The sprite is a rect centered at the transform position, so move by half size to
    ///         // keep aligned to the left while width grows.
    ///         transform.translation.x = (size - PROGRESS_BAR_SIZE) / 2.;
    ///         sprite.custom_size = Some(Vec2::new(size, PROGRESS_BAR_THICKNESS));
    ///     }
    /// }
    /// ```
    ///
    /// [`progress()`]: Boot::progress()
    pub fn smoothed_progress(&self) -> f32 {
        self.smoothed_progress
    }
}

/// Bundle with a [`Boot`] helper and its associated [`Loader`].
#[derive(Debug, Default, Bundle)]
pub struct BootBundle {
    /// The boot component managing the loading progress, based on the data reported by the [`Loader`].
    pub boot: Boot,
    /// The loader component monitoring the assets loading.
    pub loader: Loader,
}

impl BootBundle {
    /// Create a new bundle from the given loader.
    pub fn new(loader: Loader) -> Self {
        BootBundle {
            boot: Boot::new(),
            loader,
        }
    }
}

/// Update the [`Boot`] progress based on its [`Loader`] completion state, and despawn
/// the entity holding them once done.
///
/// The [`Boot`] and [`Loader`] components must be on the same entity, and there must
/// be only one such entity. The simplest way is to use a [`BootBundle`].
///
/// This system is automatically added to the app when adding the [`BootloaderPlugin`] plugin.
///
/// # Panics
///
/// This system panics if there is more than one entity with both a [`Boot`] and a [`Loader`]
/// components.
///
/// [`BootloaderPlugin`]: crate::BootloaderPlugin
pub fn update_boot(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Loader, &mut Boot)>,
) {
    if let Ok((id, loader, mut boot)) = query.get_single_mut() {
        if loader.is_done() {
            // Mark the Boot entity for later destruction (at the end of the stage)
            commands.entity(id).despawn();

            // Also delete all related entities for the boot screen
            for id in &boot.entities {
                commands.entity(*id).despawn();
            }

            // TODO -- use resources?

            // Change app state to transition to the main menu
            //assert!(*state.current() == AppState::Boot);
            //state.set(AppState::MainMenu).unwrap();
        } else {
            // Calculate the upper progress ratio. Traditionally one would calculate the current ratio of
            // completed work, that is the number of assets loaded over the total number that needs to be
            // loaded. This ratio would only reach 1.0 (100%) once all assets are loaded, and therefore
            // once the boot sequence is done and likely the boot screen disappears. This means the progress
            // bar would never reach 100%. Instead, calculate the upper bound of the ratio, which is the
            // ratio of completed items plus one, accounting for the fact one item is currently being loaded.
            // This means the progress bar will reach (N-1)/N once the last asset remains, and will smoothly
            // get close to 1.0 (100%) from there. In theory this ratio would go over 1.0 once the last
            // asset is loaded, but at this point we transition to another screen so we don't care.
            let total = loader.total_count();
            let remain = loader.pending_count();
            let upper_ratio = if total > 0 && remain < total {
                (total - remain + 1) as f32 / total as f32
            } else {
                1.0
            };
            // Update the progress bar based on the fraction of assets already loaded, smoothed with
            // a snappy animation to be visually pleasant without too much artifically delaying the
            // boot sequence.
            boot.set_progress(upper_ratio, time.delta_seconds());
        }
    }
}
