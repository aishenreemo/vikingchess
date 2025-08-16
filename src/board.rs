use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderRef;
use bevy::sprite::Material2d;
use bevy::sprite::Material2dPlugin;
use engine::prelude::*;

use crate::cursor::CursorPosition;
use crate::magics::MagicTableAsset;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BoardMaterial>::default());
        app.add_systems(PostStartup, on_add_board.before(on_add_piece));
        app.add_systems(PostStartup, on_add_piece);
        app.add_observer(on_cursor_move);
    }
}

#[derive(Component)]
#[require(Transform, BoardState, CursorState)]
pub struct BoardMarker {
    length: f32,
}

impl BoardMarker {
    pub const SHADER_PATH: &'static str = "shaders/board_material.wgsl";
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
        BoardMarker::SHADER_PATH.into()
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct BoardState(Board);

#[allow(unused)]
#[derive(Component)]
pub struct MagicTableMarker(pub Handle<MagicTableAsset>);

fn on_add_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BoardMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    q_board: Query<(Entity, &BoardMarker, &BoardState), Added<BoardMarker>>,
) -> Result {
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(64), 3, 1, None, None));

    for (entity, board, state) in q_board.iter() {
        commands.entity(entity).insert((
            MagicTableMarker(asset_server.load("magics.ron")),
            Mesh2d(meshes.add(Rectangle::new(board.length, board.length))),
            MeshMaterial2d(materials.add(BoardMaterial {
                light_color: Color::hsla(90., 0.7, 0.8, 1.0).to_linear(),
                dark_color: Color::hsla(60., 0.3, 0.4, 1.0).to_linear(),
                special_color: Color::hsla(30., 0.7, 0.8, 1.0).to_linear(),
            })),
        ));

        for (piece, square) in state.iter_bitboard() {
            commands.spawn((PieceMarker::new(piece, square, layout.clone()), ChildOf(entity)));
        }
    }

    Ok(())
}

#[derive(Component, Debug)]
#[require(Pickable)]
pub struct PieceMarker {
    variant: Piece,
    square: Square,
    layout: Handle<TextureAtlasLayout>,
}

impl PieceMarker {
    pub fn new(piece: Piece, square: Square, layout: Handle<TextureAtlasLayout>) -> Self {
        Self {
            variant: piece,
            square,
            layout,
        }
    }
}

fn on_add_piece(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_piece: Query<(Entity, &PieceMarker, &ChildOf), Added<PieceMarker>>,
    q_board: Query<&BoardMarker, With<BoardMarker>>,
) -> Result {
    let texture: Handle<Image> = asset_server.load("pieces.png");

    for (entity, marker, childof) in q_piece.iter() {
        let board = q_board.get(childof.parent())?;
        let (x, y) = square_to_xy(marker.square, board.length);
        let sprite_index = match marker.variant {
            Piece::Defender => 0,
            Piece::Attacker => 1,
            Piece::King => 2,
            _ => return Err(format!("Unhandled piece {marker:?}.").into()),
        };

        let texture_atlas = TextureAtlas {
            layout: marker.layout.clone(),
            index: sprite_index,
        };

        let sprite = Sprite {
            image: texture.clone(),
            texture_atlas: Some(texture_atlas),
            custom_size: Some(Vec2::splat(board.length / Bitboard::BOARD_LENGTH as f32 * 0.9)),
            ..default()
        };

        commands
            .entity(entity)
            .insert((Transform::from_xyz(x, y, 0.), sprite))
            .observe(on_drag)
            .observe(on_drag_end);
    }

    Ok(())
}

#[derive(Default, Component, Debug)]
struct CursorState {
    hovered_square: Option<Square>,
}

fn on_cursor_move(
    _trigger: Trigger<Pointer<Move>>,
    res_curpos: Res<CursorPosition>,
    mut q_board: Query<(&mut CursorState, &Transform, &BoardMarker), With<BoardMarker>>,
) {
    if !res_curpos.is_changed() {
        return;
    }

    for (mut cursor_data, transform, board) in q_board.iter_mut() {
        let half_length = board.length / 2.;
        let rel_curpos = res_curpos.0 - transform.translation.xy();
        let rel_curpos = rel_curpos - Vec2::new(-half_length, half_length);
        let rel_curpos_normal = rel_curpos / Vec2::new(board.length, -board.length);
        let square_pos = (rel_curpos_normal * Vec2::splat(Bitboard::BOARD_LENGTH as f32)).floor();

        cursor_data.hovered_square = Square::try_from((square_pos.x, square_pos.y)).ok();
    }
}

fn on_drag(
    trigger: Trigger<Pointer<Drag>>,
    res_curpos: Res<CursorPosition>,
    mut q_piece: Query<(&mut Transform, &ChildOf), With<PieceMarker>>,
    q_board: Query<&Transform, (With<BoardMarker>, Without<PieceMarker>)>,
) -> Result {
    let (mut transform, childof) = q_piece.get_mut(trigger.target())?;
    let board_transform = q_board.get(childof.parent())?;
    transform.translation = res_curpos.0.extend(1.) - board_transform.translation;
    transform.scale = Vec3::splat(1.1);
    Ok(())
}

fn on_drag_end(
    trigger: Trigger<Pointer<DragEnd>>,
    mut q_piece: Query<(&mut Transform, &mut PieceMarker, &ChildOf), With<PieceMarker>>,
    mut q_board: Query<(&BoardMarker, &CursorState, &MagicTableMarker, &mut BoardState), With<BoardMarker>>,
    tables: Res<Assets<MagicTableAsset>>,
) -> Result {
    let (mut transform, mut piece, childof) = q_piece.get_mut(trigger.target())?;
    let (board, cursor_state, magic_table, mut state) = q_board.get_mut(childof.parent())?;
    let magic_table = tables.get(&magic_table.0).map(|v| &v.0);

    if let Some(square) = cursor_state.hovered_square {
        let result = state.move_piece(piece.variant, piece.square, square, magic_table);
        if result.is_ok() {
            piece.square = square;
        }
    }

    let (x, y) = square_to_xy(piece.square, board.length);
    transform.translation = Vec3::new(x, y, 0.);
    transform.scale = Vec3::splat(1.);

    Ok(())
}

fn square_to_xy(square: Square, board_size: f32) -> (f32, f32) {
    let square_size = board_size / Bitboard::BOARD_LENGTH as f32;
    let x = square.col as f32 * square_size - board_size / 2. + square_size / 2.;
    let y = board_size / 2. - square.row as f32 * square_size - square_size / 2.;

    (x, y)
}
