use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Player;

fn player_controller(
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

    let player_speed = 100.0;
    let window = q_windows.single();

    if let Some(position) = window.cursor_position() {
        let world_position_x = position.x - window.width() / 2.0;
        let world_position_y = -(position.y - window.height() / 2.0);

        direction.x = world_position_x - player_transform.translation.x;
        direction.y = world_position_y - player_transform.translation.y;

        direction = direction.normalize();
        
        player_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    }

    if keys.just_pressed(KeyCode::Space) {}

    if keys.pressed(KeyCode::W) {
        player_transform.translation =
            player_transform.translation + (direction * player_speed * time.delta_seconds());
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
    ));
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, player_controller)
        .run();
}
