use bevy::{asset::LoadState, prelude::*, utils::HashMap, window::PrimaryWindow};
use core::time;
use rand::prelude::*;
use std::{f32::consts::PI, ops::Index, thread};

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct SpawnTimer {
    value: f32,
    cooldown: f32,
}

#[derive(Component)]
struct Target {
    direction: Vec3,
}

#[derive(Component)]
struct Velocity {
    value: Vec3,
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Weapon {
    name: String,
    rate_of_fire: f32,
    cooldown_timer: f32,
}

#[derive(Eq, Hash, PartialEq)]
enum ImageType {
    Player,
    Asteroid,
    Bullet,
    Background,
}

struct Sprite {
    image_handle: Handle<Image>,
    width: u32,
    height: u32,
}

impl Sprite {
    fn handle(&self) -> Handle<Image> {
        return self.image_handle.clone_weak();
    }
}

#[derive(Resource)]
struct ImageManager {
    images: HashMap<ImageType, Sprite>,
}

impl ImageManager {
    fn get(&self, key: ImageType) -> &Sprite {
        return &self.images[&key];
    }
}

impl Index<ImageType> for ImageManager {
    type Output = Sprite;
    fn index(&self, key: ImageType) -> &Sprite {
        self.get(key)
    }
}

#[derive(Component)]
struct Destroyed;

#[derive(Component)]
struct Circle {
    center: Vec2,
    radius: f32,
}

#[derive(Component)]
struct Asteroid;

#[derive(Event)]
struct CollisionEvent(Entity, Entity);
impl PartialEq for CollisionEvent {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
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

fn circle_collision_system(
    q_circle: Query<(Entity, &Circle)>,
    mut ev_collision: EventWriter<CollisionEvent>,
) {
    let mut collisions: Vec<CollisionEvent> = Vec::new();

    q_circle.for_each(|circle| {
        q_circle.for_each(|other| {
            if circle.0 == other.0 {
                return;
            }

            let distance = (circle.1.center - other.1.center).length();
            if distance < circle.1.radius + other.1.radius {
                let event = CollisionEvent(circle.0, other.0);
                if collisions.contains(&event) {
                    return;
                }

                collisions.push(event);
            }
        });
    });

    for collision in collisions {
        ev_collision.send(collision);
    }
}

fn apply_velocity_system(mut q_velocity: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    q_velocity.for_each_mut(|data| {
        let velocity = data.0;
        let mut transform = data.1;

        transform.translation += velocity.value * time.delta_seconds();
    })
}

fn player_wrap_system(
    mut q_player_transform: Query<&mut Transform, With<Player>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let result = q_player_transform.get_single_mut();
    let window = q_windows.single();

    match result {
        Ok(mut transform) => {
            let width = window.width();
            let height = window.height();

            let offset_x = -width / 2.0;
            let offset_y = -height / 2.0;

            let position = transform.translation - Vec3::new(offset_x, offset_y, 0.0);

            if position.x < 0.0 {
                transform.translation.x = width + offset_x;
            } else if position.x > width {
                transform.translation.x = 0.0 + offset_x;
            }

            if position.y < 0.0 {
                transform.translation.y = height + offset_y;
            } else if position.y > height {
                transform.translation.y = 0.0 + offset_y;
            }
        }
        Err(..) => return,
    }
}

fn player_shoot_system(
    mut commands: Commands,
    image_manager: Res<ImageManager>,
    mut q_player: Query<(&Transform, &Target, &mut Weapon), With<Player>>,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
) {
    let player = q_player.get_single_mut();
    let position;
    let direction: Vec3;

    let bullet_speed = 1000.0;
    let bullet_size = 1.0;
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
        let bullet_sprite = &image_manager[ImageType::Bullet];
        let bullet_sprite_width = bullet_sprite.width as f32;
        let bullet_sprite_height = bullet_sprite.height as f32;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: position,
                    scale: (Vec3::splat(bullet_size)),
                    ..default()
                },
                texture: bullet_sprite.handle(),
                ..default()
            },
            Asteroid,
            Velocity {
                value: direction.normalize() * bullet_speed,
            },
            Circle {
                center: Vec2::new(position.x, position.y),
                radius: (bullet_sprite_width + bullet_sprite_height) / 4.0,
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

    let strafe_movement =
        Quat::mul_vec3(Quat::from_rotation_z(PI / 2.0), movement).normalize() * player_strafe_speed;

    if keys.just_pressed(KeyCode::A) {
        player_velocity.value += strafe_movement;
    }

    if keys.just_pressed(KeyCode::D) {
        player_velocity.value -= strafe_movement;
    }
}

fn asteroid_spawner_system(
    mut commands: Commands,
    mut q_spawn_timer: Query<&mut SpawnTimer>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    image_manager: Res<ImageManager>,
) {
    let mut spawn_timer = q_spawn_timer.single_mut();
    let window = q_windows.single();

    let window_width = window.width();
    let window_height = window.height();

    let mut rng = rand::thread_rng();
    let pos_x = rng.gen::<f32>() * window_width - window_width / 2.0;
    let pos_y = rng.gen::<f32>() * window_height - window_height / 2.0;

    let asteroid_sprite = &image_manager[ImageType::Asteroid];
    let asteroid_sprite_width = asteroid_sprite.width as f32;
    let asteroid_sprite_height = asteroid_sprite.height as f32;

    if spawn_timer.value <= 0.0 {
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(pos_x, pos_y, 0.0),
                    scale: (Vec3::splat(1.0)),
                    ..default()
                },
                texture: asteroid_sprite.handle(),
                ..Default::default()
            },
            Asteroid,
            Circle {
                center: Vec2::new(pos_x, pos_y),
                radius: (asteroid_sprite_width + asteroid_sprite_height) / 4.0,
            },
        ));

        spawn_timer.value = spawn_timer.cooldown;
    }
}

fn asteroid_hit_system(mut ev_collision: EventReader<CollisionEvent>, mut commands: Commands) {
    for ev in ev_collision.read() {
        if let Some(mut entity) = commands.get_entity(ev.0) {
            entity.insert(Destroyed);
        }

        if let Some(mut entity) = commands.get_entity(ev.1) {
            entity.insert(Destroyed);
        }
    }
}

fn asteroid_destroy_system(
    mut q_destroyed: Query<(Entity, &mut Destroyed)>,
    mut commands: Commands,
) {
    q_destroyed.for_each_mut(|destroyed| {
        if let Some(mut entity) = commands.get_entity(destroyed.0) {
            entity.despawn();
        }
    })
}

fn circle_update_system(mut q_circle: Query<(&mut Circle, &Transform)>) {
    q_circle.for_each_mut(|mut bundle| {
        bundle.0.center.x = bundle.1.translation.x;
        bundle.0.center.y = bundle.1.translation.y;
    })
}

fn spawn_timer_update_system(mut q_spawn_timer: Query<&mut SpawnTimer>, time: Res<Time>) {
    let mut timer = q_spawn_timer.single_mut();
    timer.value -= time.delta_seconds();
}

fn setup_system(mut commands: Commands, image_manager: Res<ImageManager>) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: (Vec3::new(1.0, 1.0, 1.0)),
                ..default()
            },
            texture: image_manager[ImageType::Player].handle(),
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

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -1.0),
                scale: (Vec3::new(1.0, 1.0, 1.0)),
                ..default()
            },
            texture: image_manager[ImageType::Background].handle(),
            ..default()
        },
        Background,
    ));

    commands.spawn(SpawnTimer {
        value: 0.0,
        cooldown: 2.0,
    });
}

fn load_assets_system(mut image_manager: ResMut<ImageManager>, asset_server: Res<AssetServer>) {
    let player_sprite_asset: Handle<Image> = asset_server.load("player.png");
    let asteroid_sprite_asset: Handle<Image> = asset_server.load("asteroid.png");
    let bullet_sprite_asset: Handle<Image> = asset_server.load("bullet.png");
    let background_image_asset: Handle<Image> = asset_server.load("starfield.png");

    image_manager.images.insert(
        ImageType::Player,
        Sprite {
            image_handle: player_sprite_asset,
            width: 0,
            height: 0,
        },
    );
    image_manager.images.insert(
        ImageType::Asteroid,
        Sprite {
            image_handle: asteroid_sprite_asset,
            width: 0,
            height: 0,
        },
    );
    image_manager.images.insert(
        ImageType::Bullet,
        Sprite {
            image_handle: bullet_sprite_asset,
            width: 0,
            height: 0,
        },
    );
    image_manager.images.insert(
        ImageType::Background,
        Sprite {
            image_handle: background_image_asset,
            width: 0,
            height: 0,
        },
    );
}

fn process_image_descriptor_system(
    mut ev: EventReader<AssetEvent<Image>>,
    assets: ResMut<Assets<Image>>,
    mut image_manager: ResMut<ImageManager>,
) {
    for event in ev.read() {
        match event {
            AssetEvent::Added { id } => {
                for image in image_manager.images.values_mut() {
                    let image_id = image.image_handle.id();

                    if image_id == *id {
                        let texture = assets.get(image_id).unwrap();
                        image.width = texture.texture_descriptor.size.width;
                        image.height = texture.texture_descriptor.size.height;
                    }
                }
            }
            AssetEvent::Modified { id: _ } => (),
            AssetEvent::Removed { id: _ } => (),
            AssetEvent::LoadedWithDependencies { id: _ } => (),
        }
    }
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource::<ImageManager>(ImageManager {
            images: HashMap::new(),
        })
        .add_systems(PreStartup, load_assets_system)
        .add_systems(Startup, setup_system)
        .add_event::<CollisionEvent>()
        .add_systems(
            Update,
            (
                process_image_descriptor_system,
                player_move_system,
                player_target_system,
                player_shoot_system,
                player_wrap_system,
                asteroid_spawner_system,
                spawn_timer_update_system,
                apply_velocity_system,
                circle_update_system,
                circle_collision_system,
            ),
        )
        .add_systems(PostUpdate, asteroid_hit_system)
        .add_systems(Last, asteroid_destroy_system)
        .run();
}
