use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;

const BACKGROUND_COLOR: Color = Color::srgb(0.29, 0.31, 0.41);
const PLATFORM_COLOR: Color = Color::srgb(0.13, 0.13, 0.23);
const PLAYER_COLOR: Color = Color::srgb(0.60, 0.55, 0.60);

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;

const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.0;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.0;

const FLOOR_THICKNESS: f32 = 10.0;
const FLOOR_COLOR: Color = Color::srgb(0.45,0.55, 0.66);


fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FLAPPY BIRD CLONE".to_string(),
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .run(); 
}


fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {

    commands.spawn((
    Mesh2d(meshes.add(Circle::default())),
    MeshMaterial2d(materials.add(ColorMaterial::from(PLAYER_COLOR))),
    Transform {
        translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 30.0, 0.0),
        scale: Vec3::new(30.0, 30.0, 1.0),
        ..Default::default()
    },
));

    commands.spawn((
        Sprite {
            color: PLATFORM_COLOR,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(-100.0, 0.0, 0.0),
            scale: Vec3::new(75.0, 200.0, 1.0),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5),
    ));

    commands.spawn((
        Sprite {
            color: PLATFORM_COLOR,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(100.0, WINDOW_BOTTOM_Y + (350.0 / 2.0), 0.0),
            scale: Vec3::new(50.0, 350.0, 1.0),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5),
    ));

    commands.spawn((
        Sprite {
            color: PLATFORM_COLOR,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(350.0, WINDOW_BOTTOM_Y + (250.0 / 2.0), 0.0),
            scale: Vec3::new(150.0, 250.0, 1.0),
            ..Default::default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5),
    ));

    commands.spawn(Camera2d);
}