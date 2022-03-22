use bevy::{
    asset::{AssetIo, AssetIoError, AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    tasks::IoTaskPool,
    utils::HashMap,
};
use bevy_bootloader::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

// A dummy asset type which contains nothing but takes time to load.
// This helps demonstrate the bootloader even on faster hardware, by
// artificially slowing down the asset loading process.
#[derive(Debug, TypeUuid, Reflect)]
#[uuid = "a5d77fcd-09a0-47bc-9bb7-726a31bc28cc"]
pub struct DummyAsset {
    pub delay: Duration,
}

// The asset loader for the dummy asset, which artificially slows down
// loading by sleeping for a time equal to the asset's delay.
#[derive(Default)]
struct DummyAssetLoader;

impl AssetLoader for DummyAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let s = std::str::from_utf8(bytes)?;
            let delay = f32::from_str(s)?;
            let delay = Duration::from_secs_f32(delay);
            load_context.set_default_asset(LoadedAsset::new(DummyAsset { delay }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dummy"]
    }
}

// In-memory asset source.
#[derive(Default, Debug)]
struct MemoryAssetIo {
    pub assets: HashMap<String, DummyAsset>,
}

impl MemoryAssetIo {
    pub fn add(&mut self, s: &str, delay: f32) {
        self.assets.insert(
            s.to_owned(),
            DummyAsset {
                delay: Duration::from_secs_f32(delay),
            },
        );
    }
}

impl AssetIo for MemoryAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            let path_str = path
                .to_str()
                .ok_or(AssetIoError::NotFound(path.to_path_buf()))?;
            let asset = self
                .assets
                .get(path_str)
                .ok_or(AssetIoError::NotFound(path.to_path_buf()))?;
            std::thread::sleep(asset.delay.min(Duration::from_secs_f32(10.)));
            let s = format!("{}", asset.delay.as_secs_f32());
            Ok(s.as_bytes().to_vec())
        })
    }

    fn read_directory(
        &self,
        _path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        let asset_paths: Vec<PathBuf> =
            self.assets.iter().map(|(k, _v)| PathBuf::from(k)).collect();
        Ok(Box::new(asset_paths.into_iter()))
    }

    fn is_directory(&self, _path: &Path) -> bool {
        false
    }

    fn watch_path_for_changes(&self, _path: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}

struct DummyPlugin;

impl Plugin for DummyPlugin {
    fn build(&self, app: &mut App) {
        let task_pool = app
            .world
            .get_resource::<IoTaskPool>()
            .expect("`IoTaskPool` resource not found.")
            .0
            .clone();
        let mut source = MemoryAssetIo::default();
        source.add("file1.dummy", 0.2);
        source.add("file2.dummy", 7.5);
        source.add("file3.dummy", 3.1);
        let asset_server = AssetServer::with_boxed_io(Box::new(source), task_pool);
        //app.world.remove_resource::<AssetServer>();
        app.world.insert_resource(asset_server);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            filter: "wgpu=error,naga=error,bevy_render=info,bevy_bootloader=trace".to_string(),
        })
        .insert_resource(WindowDescriptor {
            title: "Bootloader".to_string(),
            width: 1200.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins_with(DefaultPlugins, |group| {
            // the custom asset io plugin must be inserted in-between the
            // `CorePlugin' and `AssetPlugin`. It needs to be after the
            // CorePlugin, so that the IO task pool has already been constructed.
            // And it must be before the `AssetPlugin` so that the asset plugin
            // doesn't create another instance of an asset server. In general,
            // the AssetPlugin should still run so that other aspects of the
            // asset system are initialized correctly.
            group.add_before::<bevy::asset::AssetPlugin, _>(DummyPlugin)
        })
        .add_asset::<DummyAsset>()
        .init_asset_loader::<DummyAssetLoader>()
        .add_plugin(BootloaderPlugin)
        .add_startup_system(setup_boot_screen)
        .add_system(update_progress_bar)
        .add_plugin(WorldInspectorPlugin::new())
        .run();

    Ok(())
}

#[derive(Component)]
struct ProgressBar;

/// Size of the progress bar, in pixels.
const PROGRESS_BAR_SIZE: f32 = 200.0;

/// Thickness of the progress bar, in pixels.
const PROGRESS_BAR_THICKNESS: f32 = 3.0;

fn setup_boot_screen(mut commands: Commands, mut clear_color: ResMut<ClearColor>) {
    // Set clear color to background color
    clear_color.0 = Color::rgba(0.1, 0.1, 0.1, 0.0);

    // Queue boot-time resources
    let mut loader = Loader::new();
    loader.enqueue("file1.dummy");
    loader.enqueue("file2.dummy");
    loader.enqueue("file3.dummy");
    loader.submit();

    // Insert the boot bundle
    commands.spawn_bundle(BootBundle::new(loader));

    // Spawn a camera to render the progress bar
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Spawn a progress bar formed of a background fixed sprite and a foreground sprite
    // whose size will be animated to form the progress bar.
    let color = Color::rgba(0.3, 0.4, 0.3, 1.0);
    let background_color = Color::rgba(0.2, 0.3, 0.2, 1.0);
    let size = Vec2::new(PROGRESS_BAR_SIZE, PROGRESS_BAR_THICKNESS);
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: background_color,
            custom_size: Some(size),
            ..Default::default()
        },
        ..Default::default()
    });
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::ZERO), // invisible initially
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ProgressBar);
}

fn update_progress_bar(
    boot_query: Query<&Boot>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite), With<ProgressBar>>,
) {
    if let Ok(boot) = boot_query.get_single() {
        // Update the progress bar based on the fraction of assets already loaded, smoothed with
        // a snappy animation to be visually pleasant without too much artifically delaying the
        // boot sequence.
        let smoothed_progress = boot.smoothed_progress();
        let (mut transform, mut sprite) = sprite_query.single_mut();
        let size = PROGRESS_BAR_SIZE * smoothed_progress;
        // The sprite is a rect centered at the transform position, so move by half size to keep
        // aligned to the left while width grows.
        transform.translation.x = (size - PROGRESS_BAR_SIZE) / 2.;
        sprite.custom_size = Some(Vec2::new(size, PROGRESS_BAR_THICKNESS));
    }
}
