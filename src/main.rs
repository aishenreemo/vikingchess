use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use board::BoardMarker;
use board::BoardPlugin;
use cursor::CursorPositionPlugin;
use magics::MagicTableAsset;
use magics::MagicTableLoader;

mod board;
mod cursor;
mod magics;

fn main() {
    App::new()
        .add_plugins(default_plugins())
        .add_plugins(CursorPositionPlugin)
        .add_plugins(BoardPlugin)
        .add_systems(Startup, setup)
        .init_asset::<MagicTableAsset>()
        .init_asset_loader::<MagicTableLoader>()
        .run();
}

fn default_plugins() -> PluginGroupBuilder {
    DefaultPlugins.set(window_plugin()).set(ImagePlugin::default_nearest())
}

fn window_plugin() -> WindowPlugin {
    let window = Window {
        title: "Viking Chess bevy_dev".to_owned(),
        name: Some("Bevy".to_owned()),
        resolution: (800., 600.).into(),
        resizable: false,
        position: WindowPosition::Centered(MonitorSelection::Primary),
        ..default()
    };

    WindowPlugin {
        primary_window: Some(window),
        ..default()
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((BoardMarker::new(550.), Transform::from_xyz(100., 0., 0.)));
}
