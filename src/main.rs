use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use cursor::CursorPositionPlugin;

mod board;
mod cursor;

fn main() {
    App::new()
        .add_plugins(default_plugins())
        .add_plugins(CursorPositionPlugin)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, board::spawn_board_system)
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
        decorations: false,
        ..default()
    };

    WindowPlugin {
        primary_window: Some(window),
        ..default()
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn(board::Board::new(550., 11));
}
