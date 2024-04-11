use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Target {
    direction: Vec3,
}

#[derive(Component)]
struct Velocity {
    value: Vec3,
}

#[derive(Component)]
struct Weapon {
    name: String,
    rate_of_fire: f32,
    cooldown_timer: f32,
}

#[derive(Component)]
struct Asteroid;

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

fn apply_velocity_system(mut q_velocity: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    q_velocity.for_each_mut(|data| {
        let velocity = data.0;
        let mut transform = data.1;

        transform.translation += velocity.value * time.delta_seconds();
    })
}

fn player_shoot_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_player: Query<(&Transform, &Target, &mut Weapon), With<Player>>,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
) {
    let player = q_player.get_single_mut();
    let position;
    let direction: Vec3;

    let bullet_speed = 1000.0;
    let mut weapon;

    match player {
        Ok(x) => {
            position = x.0.translation;
            direction = x.1.direction;

            weapon = x.2;
        }
        Err(..) => {
            return;
        }
    }

    weapon.cooldown_timer -= time.delta_seconds();

    if mouse.pressed(MouseButton::Left) && weapon.cooldown_timer <= 0.0 {
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: position,
                    scale: (Vec3::splat(1.0)),
                    ..default()
                },
                texture: asset_server.load("bullet.png"),
                ..default()
            },
            Asteroid,
            Velocity {
                value: direction.normalize() * bullet_speed,
            },
        ));

        weapon.cooldown_timer = 1.0 / weapon.rate_of_fire;
    }
}

fn player_target_system(
    mut q_player: Query<(&mut Transform, &mut Target), With<Player>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_windows.single();
    let mut player = q_player.single_mut();

    match get_player_direction(&player.0, window) {
        Some(direction) => player.1.direction = direction,
        None => (),
    }
}

fn player_move_system(
    mut query: Query<(&mut Transform, &Target, &mut Velocity), With<Player>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    let player = query.single_mut();
    let direction = player.1.direction;

    let mut player_transform = player.0;
    let mut player_velocity = player.2;

    let player_acceleration = 100.0;
    let player_strafe_speed = 100.0;
    
    let movement = direction * player_acceleration * time.delta_seconds();

    player_transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    if keys.just_pressed(KeyCode::Space) {}

    if keys.pressed(KeyCode::W) {
        player_velocity.value += movement;
    }

    if keys.pressed(KeyCode::S) {
        player_velocity.value += movement;
    }

    let strafe_movement = Quat::mul_vec3(Quat::from_rotation_z(PI / 2.0), movement).normalize()
        * player_strafe_speed;

    if keys.just_pressed(KeyCode::A) {
        player_velocity.value += strafe_movement;
    }

    if keys.just_pressed(KeyCode::D) {
        player_velocity.value -= strafe_movement;
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
        Weapon {
            name: "Railgun".to_string(),
            rate_of_fire: 1.0,
            cooldown_timer: 0.0,
        },
        Target {
            direction: Vec3::ZERO,
        },
        Velocity { value: Vec3::ZERO },
    ));
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_move_system,
                player_target_system,
                player_shoot_system,
                apply_velocity_system
            ),
        )
        .run();
}
