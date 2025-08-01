use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource, Debug)]
pub struct CursorPosition(pub Vec2);

pub struct CursorPositionPlugin;
impl Plugin for CursorPositionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition(Vec2::default()));
        app.add_observer(cursor_position_system);
    }
}

pub fn cursor_position_system(
    _trigger: Trigger<Pointer<Move>>,
    mut res_cursor_position: ResMut<CursorPosition>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Result {
    let window = q_windows.single()?;
    let (camera, camera_transform) = q_camera.single()?;

    let Some(cursor_position) = window.cursor_position() else {
        return Ok(());
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return Err("Error while converting viewport vector to 2d world.".into());
    };

    info!("{world_position:?}");
    res_cursor_position.0 = world_position;
    Ok(())
}
