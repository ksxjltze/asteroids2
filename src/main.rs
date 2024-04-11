use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Target {
    direction: Vec3
}

fn get_player_direction(player_transform: &Transform, window: &Window) -> Option<Vec3> {
    if let Some(position) = window.cursor_position() {
        let world_position_x = position.x - window.width() / 2.0;
        let world_position_y = -(position.y - window.height() / 2.0);

        let mut direction = Vec3::ZERO;
        direction.x = world_position_x - player_transform.translation.x;
        direction.y = world_position_y - player_transform.translation.y;

        direction = direction.normalize();
        return Some(direction);
    }

    None
}

fn player_shoot_system() {

}

fn player_target_system(mut q_player: Query<(&mut Transform, &mut Target), With<Player>>, q_windows: Query<&Window, With<PrimaryWindow>>) {
    let window = q_windows.single();
    let mut player = q_player.single_mut();
    
    match get_player_direction(&player.0, window) {
        Some(direction) => player.1.direction = direction,
        None => ()
    }
}

fn player_move_system(
    mut query: Query<(&mut Transform, &mut Target), With<Player>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    let player = query.single_mut();
    let direction = player.1.direction;
    let mut player_transform = player.0;

    let player_speed = 100.0;
    let mut movement = Vec3::ZERO;

    player_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    movement = direction * player_speed * time.delta_seconds();

    if keys.just_pressed(KeyCode::Space) {}

    if keys.pressed(KeyCode::W) {
        player_transform.translation = player_transform.translation + movement;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: (Vec3::new(1.0, 1.0, 1.0)),
                ..default()
            },
            texture: asset_server.load("player.png"),
            ..default()
        },
        Player,
        Target {
            direction: Vec3::ZERO
        }
    ));
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_move_system, player_target_system))
        .run();
}
