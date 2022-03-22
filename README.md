# 🚥 Bevy Bootloader

[![License: MIT/Apache](https://img.shields.io/badge/License-MIT%20or%20Apache2-blue.svg)](https://opensource.org/licenses/MIT)
[![Doc](https://docs.rs/bevy_bootloader/badge.svg)](https://docs.rs/bevy_bootloader)
[![Crate](https://img.shields.io/crates/v/bevy_bootloader.svg)](https://crates.io/crates/bevy_bootloader)
[![Build Status](https://github.com/djeedai/bevy_bootloader/actions/workflows/ci.yaml/badge.svg)](https://github.com/djeedai/bevy_bootloader/actions/workflows/ci.yaml)
[![Coverage Status](https://coveralls.io/repos/github/djeedai/bevy_bootloader/badge.svg?branch=main&kill_cache=1)](https://coveralls.io/github/djeedai/bevy_bootloader?branch=main)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-v0.6-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

App startup and resource management plugin for the Bevy game engine.

## Usage

### Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
bevy_bootloader = "0.1"
```

### System setup

Add the `BootloaderPlugin` to your app:

```rust
App::default()
    .add_plugins(DefaultPlugins)
    .add_plugin(BootloaderPlugin)
    .run();
```

Queue boot-time critical assets, and insert a `BootBundle`:

```rust
// Queue boot-time resources
let mut loader = Loader::new();
loader.enqueue("logo.png");
loader.enqueue("music.ogg");
loader.submit();

// Insert a boot bundle
commands.spawn_bundle(BootBundle::new(loader));
```

Check the boot state with either of `Boot::progress()`, `Boot::smoothed_progress()`, or
`Loader::is_done()`. For example, use `Boot::smoothed_progress()` to smoothly update a
progress bar made of a `Sprite`:

```rust
fn update_progress_bar(
    boot_query: Query<&Boot>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite), With<ProgressBar>>,
) {
    if let Ok(boot) = boot_query.get_single() {
        // Update the progress bar based on the fraction of assets already loaded, smoothed
        // with a snappy animation to be visually pleasant without too much artifically
        // delaying the boot sequence.
        let smoothed_progress = boot.smoothed_progress();
        let (mut transform, mut sprite) = sprite_query.single_mut();
        let size = PROGRESS_BAR_SIZE * smoothed_progress;
        // The sprite is a rect centered at the transform position, so move by half size to
        // keep aligned to the left while width grows.
        transform.translation.x = (size - PROGRESS_BAR_SIZE) / 2.;
        sprite.custom_size = Some(Vec2::new(size, PROGRESS_BAR_THICKNESS));
    }
}
```

 See [the `bootloader` example](./examples/bootloader.rs) for the full code.

## Compatible Bevy versions

The `main` branch is compatible with the latest Bevy release.

Compatibility of `bevy_bootloader` versions:

| `bevy_bootloader` | `bevy` |
| :--               | :--    |
| `0.1`             | `0.6`  |

Due to the fast-moving nature of Bevy and frequent breaking changes, and the limited resources to maintan 🚥 Bevy Bootloader, the `main` (unreleased) Bevy branch is not supported.
