use bevy::{core::FixedTimestep, prelude::*};
use rand::random;

const HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const TAIL_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    height: f32,
    width: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component)]
struct Food;

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Left => Self::Right,
            Direction::Up => Self::Down,
            Direction::Right => Self::Left,
            Direction::Down => Self::Up,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn size_scaling(_windows: Res<Windows>, mut query: Query<(&Size, &mut Transform)>) {
    // let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * 500_f32 as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * 500_f32 as f32,
            1.0,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();

    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        )
    }
}

fn food_spawner(mut commands: Commands) {
    let rand_position = Position {
        x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
        y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
    };

    let bundle = SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(255.0, 0.0, 0.0),
            ..default()
        },
        ..default()
    };

    commands
        .spawn_bundle(bundle)
        .insert(Food)
        .insert(rand_position)
        .insert(Size::square(0.5));
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    let bundle = SpriteBundle {
        sprite: Sprite {
            color: HEAD_COLOR,
            ..default()
        },
        ..default()
    };

    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(bundle)
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.5))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    let bundle = SpriteBundle {
        sprite: Sprite {
            color: TAIL_COLOR,
            ..default()
        },
        ..default()
    };

    commands
        .spawn_bundle(bundle)
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.3))
        .id()
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segments_positions = segments
            .iter()
            .map(|entity| *positions.get_mut(*entity).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();

        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };

        segments_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
    }
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::D) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::A) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::W) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::S) {
            Direction::Down
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir
        }
    };
}

fn main() {
    let window_desc = WindowDescriptor {
        title: String::from("Snake"),
        width: 500.0,
        height: 500.0,
        ..default()
    };

    let clear_color = ClearColor(Color::rgb(0.04, 0.04, 0.04));

    App::new()
        .insert_resource(window_desc)
        .insert_resource(clear_color)
        .insert_resource(SnakeSegments::default())
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_system(snake_movement_input.before(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}
