use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderRef;
use bevy::sprite::Material2d;
use bevy::sprite::Material2dPlugin;

pub const SHADER_PATH: &'static str = "shaders/board_material.wgsl";

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BoardMaterial>::default());
        app.add_systems(PostStartup, on_add_board);
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct BoardComponent {
    length: f32,
}

impl BoardComponent {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BoardMaterial {
    #[uniform(0)]
    light_color: LinearRgba,
    #[uniform(1)]
    dark_color: LinearRgba,
    #[uniform(2)]
    special_color: LinearRgba,
}

impl Material2d for BoardMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }
}

fn on_add_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BoardMaterial>>,
    q_board: Query<(Entity, &BoardComponent), Added<BoardComponent>>,
) {
    for (entity, board) in q_board.iter() {
        commands.entity(entity).insert((
            Mesh2d(meshes.add(Rectangle::new(board.length, board.length))),
            MeshMaterial2d(materials.add(BoardMaterial {
                light_color: Color::hsla(120., 0.7, 0.8, 1.0).to_linear(),
                dark_color: Color::hsla(120., 0.3, 0.3, 1.0).to_linear(),
                special_color: Color::hsla(30., 0.7, 0.8, 1.0).to_linear(),
            })),
        ));
    }
}
