use std::f32::consts::PI;

use bevy::prelude::*;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct Weapon;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct ListTimer(Timer);

#[derive(Component)]
struct GameCamera;

#[derive(Component)]
struct Player;

pub struct WeaponsPlugin;
impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ListTimer(Timer::from_seconds(2.0, TimerMode::Once)))
            .add_systems(Startup, add_weapons)
            .add_systems(Update, list_weapons);
    }
}

fn add_weapons(mut commands: Commands) {
    commands.spawn((Weapon, Name("Railgun".to_string())));
}

fn list_weapons(time: Res<Time>, mut timer: ResMut<ListTimer>, query: Query<&Name, With<Weapon>>) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("Weapons:");
        for name in &query {
            println!("- {}", name.0);
        }
    }
}

fn spin(mut query: Query<&mut Transform, With<Player>>, time: Res<Time>){
    let mut player_transform = query.single_mut();

    player_transform.rotate_axis(Vec3::new(0.0, 0.0, 1.0), PI / 180.0 * 30.0 * time.delta_seconds());
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: (Vec3::new(100.0, 100.0, 1.0)),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 0.7),
                ..default()
            },
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
        .add_plugins((DefaultPlugins, WeaponsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}
