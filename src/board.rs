use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::cursor::CursorPosition;

#[derive(Component)]
#[require(Transform, BoardState)]
pub struct Board {
    board_size: f32,
    square_length: usize,
}

impl Board {
    pub fn new(board_size: f32, square_length: usize) -> Self {
        Self {
            board_size,
            square_length,
        }
    }
}

#[derive(Component)]
pub struct BoardState {
    state: Vec<Option<Piece>>,
    turn: Turn,
}

pub enum Piece {
    King,
    Defender,
    Attacker,
}

#[derive(Default)]
pub enum Turn {
    #[default]
    Black,
    White,
}

#[derive(Component)]
pub struct PieceMarker;

#[derive(Component)]
pub struct PieceDraggingMarker;

impl BoardState {
    pub const STARTING_VCS_NOTATION_11: &'static str = "4AAA4/5A5/92/5D5/A4D4A/AA1DDKDD1AA/A4D4A/5D5/92/5A5/4AAA4 B";

    pub fn from_notation(notation: &'static str) -> Self {
        let parts: Vec<&str> = notation.split(' ').collect();
        let board_str = parts[0];
        let turn_str = parts[1];

        let mut board_vec: Vec<Option<Piece>> = Vec::new();

        for (_, ch) in board_str.chars().enumerate() {
            BoardState::consume_char(&mut board_vec, ch);
        }

        if board_vec.len() != 11 * 11 {
            panic!(
                "Malformed notation: Board has {} squares, expected {}",
                board_vec.len(),
                11 * 11
            );
        }

        BoardState {
            state: board_vec,
            turn: match turn_str {
                "B" => Turn::Black,
                "W" => Turn::White,
                _ => panic!("Invalid turn color, expected W or B, got {turn_str}."),
            },
        }
    }

    pub fn consume_char(board_vec: &mut Vec<Option<Piece>>, ch: char) {
        match ch {
            'A' => board_vec.push(Some(Piece::Attacker)),
            'D' => board_vec.push(Some(Piece::Defender)),
            'K' => board_vec.push(Some(Piece::King)),
            '1'..='9' => {
                let num_empty_squares = ch.to_digit(10).expect("Invalid digit in notation") as usize;
                for _ in 0..num_empty_squares {
                    board_vec.push(None);
                }
            }
            '/' => {}
            _ => panic!("Invalid character in notation: {}", ch),
        }
    }
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState::from_notation(BoardState::STARTING_VCS_NOTATION_11)
    }
}

pub fn spawn_board_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    q_board: Single<(Entity, &Board, &BoardState), Added<Board>>,
) -> Result {
    let (id, board, board_state) = q_board.into_inner();
    let texture: Handle<Image> = asset_server.load("pieces.png");
    let layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(64), 3, 1, None, None));

    let board_size = board.board_size;
    let square_length = board.square_length;
    let square_last = square_length - 1;
    let square_size = board_size / square_length as f32;

    let board_mesh = meshes.add(Rectangle::new(board_size * 1.01, board_size * 1.01));
    let square_mesh = meshes.add(Rectangle::new(square_size, square_size));

    let foreground_material = materials.add(Color::hsl(120., 0.3, 0.9));
    let background_material = materials.add(Color::hsl(120., 0.3, 0.2));
    let special_material = materials.add(Color::hsl(50., 0.7, 0.8));

    commands
        .entity(id)
        .insert((Mesh2d(board_mesh.clone()), MeshMaterial2d(foreground_material.clone())));

    for (i, square) in board_state.state.iter().enumerate() {
        let col = i % square_length;
        let row = i / square_length;
        let x = col as f32 * square_size - board_size / 2. + square_size / 2.;
        let y = board_size / 2. - row as f32 * square_size - square_size / 2.;
        let color = match (col, row) {
            (0, 0) => special_material.clone(),
            (c, r) if (c == 0 || c == square_last) && (r == 0 || r == square_last) => special_material.clone(),
            (c, r) if c == square_last / 2 && r == square_last / 2 => special_material.clone(),
            (c, r) if ((c + r) % 2) == 0 => foreground_material.clone(),
            _ => background_material.clone(),
        };

        commands.spawn((
            Mesh2d(square_mesh.clone()),
            MeshMaterial2d(color),
            Transform::from_xyz(x, y, 0.),
            ChildOf(id),
        ));

        let Some(piece) = square else {
            continue;
        };

        let texture_atlas = TextureAtlas {
            layout: layout.clone(),
            index: match piece {
                Piece::Defender => 0,
                Piece::Attacker => 1,
                Piece::King => 2,
            },
        };

        commands
            .spawn((
                Pickable::default(),
                Transform::from_xyz(x, y, 0.),
                Sprite {
                    image: texture.clone(),
                    texture_atlas: Some(texture_atlas),
                    custom_size: Some(Vec2::splat(square_size * 0.9)),
                    ..default()
                },
            ))
            .observe(on_piece_drag);
    }

    Ok(())
}

fn on_piece_drag(trigger: Trigger<Pointer<Drag>>, cursor_pos: Res<CursorPosition>, mut commands: Commands) {
    let new_translation = cursor_pos.0.extend(0.);

    commands
        .entity(trigger.target())
        .entry::<Transform>()
        .and_modify(move |mut t| t.translation = new_translation);
}
