/* Flappy Knight
Author: Trinity Adams and Rodrigo De Leon Bran
Course: CSC 372
Assignment: Final Project
Instructor: Lester McCann
TA: Muaz, Daniel
Due Date: 5/4/2026

Adapted from: "Learning Game Dev - Building a platformer with Bevy"
The tutorial helped us get started with creating the platformer and showed us implementation of Bevy and Rapier 2d
physics to begin the development process. Showed how to add create the platforms, player, and basic movement mechanics.
After this was done, we created the physics mechanics for a flappy bird style game, space mechanic for flying , procedural generation, despawning of platforms for memory,
collision detection when a platform is hit by the player to end the game, and added visuals to complete the game.

Description: A 2d scroller game that resembles flappy bird in Rust, the game goes on forever until a user hits a platform, space is used
to fly in the air and when no jump is added the player falls from the sky. The platforms are walls split in half and the goal of the
player is to thread the needle.

Language/Version: Rust (1.95.0), Bevy 0.18.0, bevy_rapier2d 0.32.0
Compilation: "cargo run --bin flappy"

Known Bugs / Missing Features:
1. No scoring system implemented
2. No restart funtion was added so need to recompile after losing.
*/

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

// window dimensions
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.0;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.0;

// colors
const FLOOR_COLOR: Color = Color::srgb(0.20, 0.45, 0.60);
const FLOOR_THICKNESS: f32 = 10.0;


// platform generation
const PLATFORM_SPAWN_AHEAD: f32 = 800.0;    // how much ahead to spawn
const PLATFORM_DESPAWN_BEHIND: f32 = 200.0;     // how much behind to despawn
const PLATFORM_MIN_WIDTH: f32 = 60.0;   // minimum platform width
const PLATFORM_MAX_WIDTH: f32 = 200.0;  // max platform width
const PLATFORM_MIN_GAP: f32 = 150.0;    // min gap between platforms
const PLATFORM_MAX_GAP: f32 = 300.0;    // max gap between platforms
const NEEDLE_GAP_SIZE: f32 = 125.0;     // gap size in pixels the player threads through, adjust for difficulty
const NEEDLE_GAP_MIN_Y: f32 = 150.0;   // minimum height of the gap from the floor
const NEEDLE_GAP_MAX_Y: f32 = 400.0;   // maximum height of the gap from the floor

const GRAVITY: f32 = 1200.0;        // downward acceleration in pixels per second squared, increase to make game harder
const FLAP_STRENGTH: f32 = 400.0;   // upward velocity burst on jump, adjust to change how high player goes
const MAX_FALL_SPEED: f32 = -600.0; // terminal velocity cap, negative because downward
const PLAYER_FORWARD_SPEED: f32 = 300.0; // how fast player moves right automatically

// LevelState resource tracks how far the level has been generated so far
#[derive(Resource)]
struct LevelState {
    furthest_x: f32,   // The rightmost X position that has been generated up to
}

// Platform marker component used to identify platform entities for despawning
#[derive(Component)]
struct Platform;

// Player marker component used to identify the player entity for death detection
#[derive(Component)]
struct Player;

// Floor marker compenent to identify the floor
#[derive(Component)]
struct Floor;

#[derive(Component)]
struct GameOverScreen; // marker for the game over UI entity so we can despawn it later if needed

#[derive(Component)]
struct Background;


// PlayerVelocity tracks the player's current vertical speed
// positive = moving up, negative = moving down
#[derive(Component)]
struct PlayerVelocity {
    vy: f32, // vertical velocity in pixels per second
}

// GameStarted resource tracks whether the player has pressed space to begin
#[derive(Resource)]
struct GameStarted(bool);

// main
// Purpose: Entry point of the application. Configures and launches the Bevy app
// with all plugins, resources, and systems registered.
// Pre-condition:  None
// Post-condition: The Bevy app is running with physics, rendering, and input systems active.
fn main() {
    App::new()
        .insert_resource(LevelState { furthest_x: WINDOW_WIDTH / 2.0 }) // Start generating from the right edge of the screen
        .insert_resource(GameStarted(false)) // game waits for first space press before starting
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FLAPPY Knight".to_string(),   // Title displayed in the window bar
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32), // Window size
                resizable: false,                         // Prevent the user from resizing the window
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0)) // Physics plugin, 200px = 1 meter
        .add_plugins(RapierDebugRenderPlugin::default())  // Renders collider outlines for debugging
        .add_systems(Startup, setup)// Run setup once at startup
        .add_systems(Update, flap)
        .add_systems(Update, apply_gravity)
        .add_systems(Update, move_forward)// remove movement, jump, rise, fall
        .add_systems(Update, generate_platforms)// Spawn new platforms ahead of the camera
        .add_systems(Update, despawn_platforms)// Remove platforms that have scrolled off screen
        .add_systems(Update, keep_floor_in_frame)
        .add_systems(Update, camera_follows_player)
        .add_systems(Update, check_platform_collision)
        .add_systems(Update, camera_follows_background)
        .add_systems(Update, wait_for_start) // waits for first space press to begin the game
        .run();
}


// setup
// Purpose: Spawns all initial game entities including the background, floor, player, and camera.
// Pre-condition:  Called once at startup by Bevy.
// Post-condition: All game entities are present in the world and ready for simulation.
// Parameters:
//   commands (in/out): Used to spawn entities into the Bevy world
//   asset_server (in): Used to load image assets for the background, floor, and player
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // spawn the camera
    commands.spawn(Camera2d);

    // spawn night sky
    commands.spawn((
    Sprite::from_image(asset_server.load("flappy_nightsky.png")),
    Transform {
        translation: Vec3::new(0.0, 0.0, -10.0), // Z = -10 puts it behind everything
        scale: Vec3::new(1.0, 1.0, 1.0),          // adjust scale to fit your image
        ..Default::default()
    },
    Background,
    ));

    // spawn the floor
    commands.spawn((
        Sprite {
            color: FLOOR_COLOR,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, WINDOW_BOTTOM_Y + (FLOOR_THICKNESS / 2.0), 0.0),
            scale: Vec3::new(WINDOW_WIDTH, FLOOR_THICKNESS, 1.0),
            ..Default::default()
        },
        RigidBody::Fixed,           // floor never moves
        Collider::cuboid(0.5, 0.5), // box collider matching unit sprite
        Floor,
    ));

    // spawn the player
    commands.spawn((
    Sprite::from_image(asset_server.load("flappy_bird.png")),
    Transform {
        translation: Vec3::new(WINDOW_LEFT_X + 100.0, 0.0, 0.0),
        scale: Vec3::new(0.05, 0.05, 1.0),
        ..Default::default()
    },
    RigidBody::KinematicPositionBased,
    Collider::ball(0.5),
    KinematicCharacterController::default(),
    ActiveEvents::COLLISION_EVENTS,
    PlayerVelocity { vy: 0.0 },
    Player,
    ));

}

// **************** PLATFORM CREATION **************************

// PlatformBundle groups all components needed to spawn a static platform entity
#[derive(Bundle)]
struct PlatformBundle {
    sprite: Sprite,       // visual appearance of the platform
    transform: Transform, // position and scale of the platform in the world
    body: RigidBody,      // rapier rigid body, set to Fixed so platform does not move
    collider: Collider,   // rapier collider shape used for physics collision detection
    marker: Platform,     // marker component used to identify platforms for despawning
}

impl PlatformBundle {
    // new_pillar
    // Purpose: Constructs a PlatformBundle at a specific vertical position, used for
    // needle obstacles where top and bottom pieces need exact placement.
    // Pre-condition:  x is a valid horizontal position, scale is a valid Vec3 size,
    // y_bottom is the distance from the window bottom to the base of the platform.
    // Post-condition: Returns a fully initialized PlatformBundle at the given height.
    // Return: PlatformBundle
    // Parameters:
    //   x (in): Horizontal position of the platform in world space
    //   scale (in): Size of the platform, x = width, y = height
    //   y_bottom (in): Distance from window bottom to the base of this platform in pixels
    fn new_pillar(x: f32, scale: Vec3, y_bottom: f32, asset_server: &Res<AssetServer>) -> Self {
        Self {
            sprite: Sprite {
            image: asset_server.load("flappy_pillar.png"),
            custom_size: Some(Vec2::new(scale.x, scale.y)), // ✅ controls the rendered size
            ..Default::default()
        },
            transform: Transform {
                translation: Vec3::new(
                    x,
                    WINDOW_BOTTOM_Y + y_bottom + (scale.y / 2.0), // offset by y_bottom instead of zero
                    0.0,
                ),
                scale: Vec3::ONE,
                ..Default::default()
            },
            body: RigidBody::Fixed,
            collider: Collider::cuboid(scale.x / 2.0, scale.y / 2.0),
            marker: Platform,
        }
    }
}

// generate_platforms
// Purpose: Procedurally spawns needle obstacles ahead of the player as they move right.
// Each obstacle is a top and bottom platform with a gap the player must fly through.
// Pre-condition:  A Camera2d entity exists; LevelState resource tracks generation progress.
// Post-condition: New needle obstacles are spawned up to PLATFORM_SPAWN_AHEAD pixels ahead.
// Parameters:
//   commands (in/out): Used to spawn new platform entities
//   camera_query (in): Query to read the current camera X position
//   level_state (in/out): Resource tracking the furthest generated X position
fn generate_platforms(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut level_state: ResMut<LevelState>, asset_server: Res<AssetServer>,
    started: Res<GameStarted>
) {
    if !started.0 { return; } // do nothing until first space press
    let Ok(camera) = camera_query.single() else { return; };

    let camera_right = camera.translation.x + WINDOW_WIDTH / 2.0;
    let spawn_target = camera_right + PLATFORM_SPAWN_AHEAD;

    let mut rng = rand::rng();

    while level_state.furthest_x < spawn_target {
        let gap = rng.random_range(PLATFORM_MIN_GAP..PLATFORM_MAX_GAP);
        let width = rng.random_range(PLATFORM_MIN_WIDTH..PLATFORM_MAX_WIDTH);
        let x = level_state.furthest_x + gap + width / 2.0;

        // random height for the gap center so every needle is different
        let gap_center_y = rng.random_range(NEEDLE_GAP_MIN_Y..NEEDLE_GAP_MAX_Y);

        // bottom piece: sits on floor and goes up to gap
        commands.spawn(PlatformBundle::new_pillar(x, Vec3::new(width, gap_center_y, 1.0),0.0, &asset_server));

        // top piece: starts above gap and goes to top of window
        let top_y = gap_center_y + NEEDLE_GAP_SIZE;
        let top_height = WINDOW_HEIGHT - top_y;
        commands.spawn(PlatformBundle::new_pillar(x,Vec3::new(width, top_height, 1.0),top_y, &asset_server));

        level_state.furthest_x = x + width / 2.0;
    }
}

// despawn_platforms
// Purpose: Removes platform entities that have scrolled behind the camera left edge
// to prevent unbounded memory growth.
// Pre-condition:  A Camera2d entity exists; platform entities have a Transform and Platform marker.
// Post-condition: Platforms more than PLATFORM_DESPAWN_BEHIND pixels behind the camera are despawned.
// Parameters:
//   commands (in/out): Used to despawn platform entities
//   camera_query (in): Query to read the current camera X position
//   platform_query (in): Query for all platform entities and their transforms
fn despawn_platforms(mut commands: Commands, camera_query: Query<&Transform, With<Camera2d>>, platform_query: Query<(Entity, &Transform), With<Platform>>) {
    let Ok(camera) = camera_query.single() else { return; };

    let camera_left = camera.translation.x - WINDOW_WIDTH / 2.0; // X position of the left edge of the camera view
    let despawn_threshold = camera_left - PLATFORM_DESPAWN_BEHIND; // Platforms behind this X will be removed

    for (entity, transform) in platform_query.iter() {
        if transform.translation.x < despawn_threshold { // Platform has scrolled off the left edge of the screen
            commands.entity(entity).despawn(); // Remove the platform entity from the world
        }
    }
}

// ***************** CAMERA *********************

// camera_follows_player
// Purpose: Keeps the camera centered on the player's X position so the view
// scrolls with the player as they move forward automatically.
// Pre-condition: A Camera2d and Player entity both exist in the world.
// Post-condition: Camera X position matches the player X position every frame.
// Parameters:
//   player_query (in):  Query to read the current player position
//   camera_query (in/out): Query for the camera transform
fn camera_follows_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player) = player_query.single() else { return; };
    let Ok(mut camera) = camera_query.single_mut() else { return; };

    camera.translation.x = player.translation.x; // keep camera centered on player x
}



// keep_floor_in_frame
// Purpose: Keeps the floor centered on the camera X position so it always fills the screen.
// Pre-condition:  A Camera2d entity and a Floor entity both exist in the world.
// Post-condition: Floor X position matches the camera X position every frame.
// Parameters:
//   camera_query (in): Query to read the current camera X position
//   floor_query (in/out): Query for the floor transform
fn keep_floor_in_frame(camera_query: Query<&Transform, With<Camera2d>>, mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>) {
    let Ok(camera) = camera_query.single() else { return; };
    let Ok(mut floor) = floor_query.single_mut() else { return; };

    floor.translation.x = camera.translation.x; // Keep floor centered on camera so it always fills the screen
}


// camera_follows_background
// Purpose: Keeps the background centered on the camera X position so it always fills the screen.
// Pre-condition:  A Camera2d entity and a Background entity both exist in the world.
// Post-condition: Background X position matches the camera X position every frame.
// Parameters:
//   camera_query (in): Query to read the current camera X position
//   bg_query (in/out): Query for the background transform
fn camera_follows_background(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut bg_query: Query<&mut Transform, (With<Background>, Without<Camera2d>)>,
) {
    let Ok(camera) = camera_query.single() else { return; };
    let Ok(mut bg) = bg_query.single_mut() else { return; };

    bg.translation.x = camera.translation.x;
}


// *********************** MOVEMENT MECHANICS ******************************

// apply_gravity
// Purpose: Pulls the player downward every frame by reducing vertical velocity,
// simulating gravity. Player falls faster over time.
// Pre-condition:  Player entity has a PlayerVelocity and KinematicCharacterController.
// Post-condition: Player velocity is reduced by gravity and translation is updated downward.
// Parameters:
//   time (in): Bevy time resource for frame delta
//   query (in/out): Query for player velocity and controller
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut PlayerVelocity, &mut KinematicCharacterController), With<Player>>,
    started: Res<GameStarted>
) {
     if !started.0 { return; } // do nothing until first space press
    let Ok((mut velocity, mut player)) = query.single_mut() else { return; };

    let delta = time.delta_secs();

    velocity.vy -= GRAVITY * delta; // pull velocity downward each frame
    velocity.vy = velocity.vy.max(MAX_FALL_SPEED); // cap fall speed so player doesn't fall too fast

    let movement = velocity.vy * delta; // actual distance to move this frame

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)), // preserve horizontal
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

// flap
// Purpose: Detects jump input and gives the player an upward velocity burst.
// Unlike the old jump system, this works anytime not just when grounded.
// Pre-condition:  Player entity has a PlayerVelocity component.
// Post-condition: Player vertical velocity is set to FLAP_STRENGTH on input.
// Parameters:
//   input (in): Bevy input resource for reading keyboard state
//   query (in/out): Query for the player velocity component
fn flap(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerVelocity, With<Player>>,
) {
    let Ok(mut velocity) = query.single_mut() else { return; };

    if input.just_pressed(KeyCode::Space) { // just_pressed so holding doesnt count
        velocity.vy = FLAP_STRENGTH; // set velocity upward, overrides any downward momentum
    }
}

// move_forward
// Purpose: Moves the player rightward automatically every frame at a constant speed,
// since in flappy bird style the player always moves forward.
// Pre-condition:  Player entity has a KinematicCharacterController.
// Post-condition: Player translation is updated with rightward movement each frame.
// Parameters:
//   time (in): Bevy time resource for frame delta
//   query (in/out): Query for the player controller
fn move_forward(
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController, With<Player>>,
    started: Res<GameStarted>
) {
    if !started.0 { return; } // do nothing until first space press
    let Ok(mut player) = query.single_mut() else { return; };

    let forward = time.delta_secs() * PLAYER_FORWARD_SPEED;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(forward, vec.y)), // preserve vertical
        None => player.translation = Some(Vec2::new(forward, 0.0)),
    }
}

// wait_for_start
// Purpose: Waits for the first space key press and marks the game as started.
// Until this happens all movement and generation systems are paused.
// Pre-condition:  GameStarted resource exists and is false.
// Post-condition: GameStarted is set to true on the first space press.
// Parameters:
//   input (in): Bevy input resource for reading keyboard state
//   started (in/out): Resource tracking whether the game has begun
fn wait_for_start(
    input: Res<ButtonInput<KeyCode>>,
    mut started: ResMut<GameStarted>,
) {
    if input.just_pressed(KeyCode::Space) {
        started.0 = true;
    }
}


// ************** COLLISION DETECTION *******************

// check_platform_collision
// Purpose: Detects when the player collides with a platform and triggers the game over screen.
// Despawns the player and spawns a UI overlay with a game over message.
// Pre-condition:  Player entity has a KinematicCharacterControllerOutput component.
//  GameOverScreen does not already exist in the world.
// Post-condition: Player is despawned and GameOverScreen UI is spawned on any collision.
// Parameters:
//   commands (in/out):       Used to despawn the player and spawn the game over UI
//   player_query (in):       Query for the player entity and its collision output
//   game_over_query (in):    Query to check if a game over screen already exists
fn check_platform_collision(
    mut commands: Commands,
    player_query: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
    game_over_query: Query<&GameOverScreen>,
) {
    if !game_over_query.is_empty() {
        return;
    }

    let Ok((player_entity, output)) = player_query.single() else { return; };

    // output.collisions contains all collisions this frame
    if !output.collisions.is_empty() {
        commands.entity(player_entity).despawn();

        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GameOverScreen,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.0, 0.0)),
            ));
            parent.spawn((
                Text::new("You hit a platform!"),
                TextFont {
                    font_size: 30.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
    }
}
