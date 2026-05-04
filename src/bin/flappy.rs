/* [Creative Platformer Name]
Author: Trinity Adams and Rodrigo De Leon Bran
Course: CSC 372
Assignment: Final Project
Instructor: Lester McCann
TA: Muaz, Daniel
Due Date: 5/4/2026

Description: A 2d platformer game that ends when the player comes off the screen. Platforms are procedurally generated so game keeps going forever. 
The Bevy engine game engine was used for creating the game and the Rapier physics engine was used for the 2d physics. 

Language/Version: Rust (1.95.0), Bevy 0.18.0, bevy_rapier2d 0.32.0
Compilation: "cargo run --bin flappy"

Known Bugs / Missing Features:
1. Background image not generating because could not find good png, only jpg
2. PLAYER_VELOCIY_Y is a typo of PLAYER_VELOCITY_Y (kept to avoid breaking changes)
*/

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

// Adjust these to change the game window size
const WINDOW_WIDTH: f32 = 1024.0;  // Width of the game window in pixels
const WINDOW_HEIGHT: f32 = 720.0;  // Height of the game window in pixels

const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.0; // Y coordinate of the bottom edge of the window
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.0;    // X coordinate of the left edge of the window

const FLOOR_THICKNESS: f32 = 10.0;                         // Thickness of the floor in pixels, adjust to change floor height
const FLOOR_COLOR: Color = Color::srgb(0.45, 0.55, 0.66);  // Color used to render the floor

// Adjust these to change how fast the player moves
const PLAYER_VELOCITY_X: f32 = 400.0;  // Horizontal movement speed in pixels per second
const PLAYER_VELOCIY_Y: f32 = 850.0;   // Vertical movement speed in pixels per second, used for both jump and fall

const MAX_JUMP_HEIGHT: f32 = 230.0; // Maximum height in pixels the player can rise in a single jump, adjust to change jump height

// Adjust these to change camera scroll behavior
const CAMERA_INITIAL_SPEED: f32 = 150.0;   // Starting scroll speed of the camera in pixels per second
const CAMERA_SPEED_INCREMENT: f32 = 50.0;  // How much the camera speeds up each interval, adjust to change difficulty ramp
const CAMERA_SPEED_INTERVAL: f32 = 10.0;   // How many seconds between each speed increase, adjust to change how often it speeds up
const CAMERA_MAX_SPEED: f32 = 600.0;       // Maximum speed the camera can reach, adjust to cap difficulty

// Adjust these to change procedural generation behavior
const PLATFORM_SPAWN_AHEAD: f32 = 800.0;   // How far ahead of the camera to spawn new platforms in pixels
const PLATFORM_DESPAWN_BEHIND: f32 = 200.0; // How far behind the camera left edge to despawn old platforms in pixels
const PLATFORM_MIN_WIDTH: f32 = 60.0;      // Minimum platform width in pixels, adjust to change platform size range
const PLATFORM_MAX_WIDTH: f32 = 200.0;     // Maximum platform width in pixels, adjust to change platform size range
const PLATFORM_MIN_HEIGHT: f32 = 50.0;     // Minimum platform height in pixels
const PLATFORM_MAX_HEIGHT: f32 = 210.0;    // Maximum platform height in pixels, keep below window height
const PLATFORM_MIN_GAP: f32 = 150.0;       // Minimum horizontal gap between platforms in pixels, adjust to change difficulty
const PLATFORM_MAX_GAP: f32 = 300.0;       // Maximum horizontal gap between platforms in pixels, adjust to change difficulty

// CameraSpeed resource tracks the current scroll speed and the timer for speed increases
#[derive(Resource)]
struct CameraSpeed {
    speed: f32,         // Current camera scroll speed in pixels per second
    timer: f32,         // Accumulated time in seconds since last speed increase
}

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

#[derive(Component)]
struct Floor;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    StartScreen,
    Playing
}

#[derive(Component)]
struct StartScreen;

#[derive(Component)]
struct GameOverScreen; // marker for the game over UI entity so we can despawn it later if needed

#[derive(Resource, Default)]
struct GameOver(bool);

// main
// Purpose: Entry point of the application. Configures and launches the Bevy app
//          with all plugins, resources, and systems registered.
// Pre-condition:  None
// Post-condition: The Bevy app is running with physics, rendering, and input systems active.
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK)) // Set the window background color
        .insert_resource(CameraSpeed { speed: CAMERA_INITIAL_SPEED, timer: 0.0 }) // Initialize camera speed resource
        .insert_resource(LevelState { furthest_x: WINDOW_WIDTH / 2.0 }) // Start generating from the right edge of the screen
        .insert_resource(GameOver(false))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "FLAPPY BIRD CLONE".to_string(),   // Title displayed in the window bar
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32), // Window size
                resizable: false,                         // Prevent the user from resizing the window
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0)) // Physics plugin, 200px = 1 meter
        .add_plugins(RapierDebugRenderPlugin::default())  // Renders collider outlines for debugging
        .init_state::<GameState>()
        .add_systems(Startup, setup)// Run setup once at startup
        // .add_systems(Update, movement)// Handle left/right movement every frame
        // .add_systems(Update, jump)// Detect jump input every frame
        // .add_systems(Update, rise)// Apply upward movement while jumping
        // .add_systems(Update, fall)// Apply downward movement when not jumping
        // .add_systems(Update, scroll_camera)// Move the camera rightward every frame
        // .add_systems(Update, generate_platforms)// Spawn new platforms ahead of the camera
        // .add_systems(Update, despawn_platforms)// Remove platforms that have scrolled off screen
        // .add_systems(Update, check_player_death)// Check if the player has fallen behind the camera
        // .add_systems(Update, follow_camera_floor)
        .add_systems(Update,(movement, jump, rise, fall, scroll_camera, generate_platforms, despawn_platforms, check_player_death, follow_camera_floor).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::StartScreen), spawn_start_screen)
        .add_systems(Update, start_on_input.run_if(in_state(GameState::StartScreen)))
        .add_systems(OnExit(GameState::StartScreen), despawn_start_screen)
        .run();
}

// spawn_start_screen
// Purpose: Have a starting screen for the player to interact with before the game starts
// Pre-condition: A still state of the game, where wait for input from player
// Post-condition: Screen and Game enter play state where the game is running
// Parameters:
//      commands (in/out):  User input
fn spawn_start_screen(mut commands: Commands){
    commands.spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Default::default()
    },
    BackgroundColor(Color::BLACK),
    StartScreen,
    ))
    .with_children(|parent| {
        parent.spawn((Text::new("Press 'Space' to Start"),
        TextFont{
            font_size: 50.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}

// start_on_input
// Purpose: To have a start screen that will only play the game once the player is ready to engage
// Pre-condition: Waiting until player presses 'space' bar
// Post-condition: GameState will change to playing
// Parameters:
//      input (in): Keyboard input
//      next_state: A GameState condition
fn start_on_input(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

// despawn_start_screen
// Purpose: To remove the start screen after player has met the condition
// Pre-condition: GameState of Start screen
// Post-condition: GameState of the Playing screen
// Parameters:
//      commands (in/out): user input
//      query: all startscreen entities
fn despawn_start_screen(mut commands: Commands,  query: Query<Entity, With<StartScreen>>){
    for entity in query.iter(){
        commands.entity(entity).despawn();
    }
}

// rise
// Purpose: Moves the player upward while the Jump component is active. Removes
//          the Jump component once the player has reached MAX_JUMP_HEIGHT.
// Pre-condition:  The player entity has a KinematicCharacterController and Jump component.
// Post-condition: Player translation is updated upward; Jump is removed if max height is reached.
// Parameters:
//   commands (in/out): Used to remove the Jump component from the player entity
//   time (in):         Bevy time resource used to calculate frame-delta movement
//   query (in/out):    Query for the player entity, its controller, and jump state
fn rise(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut KinematicCharacterController, &mut Jump)>) {
    if query.is_empty() {
        return;
    }

    let Ok((entity, mut player, mut jump)) = query.single_mut() else { return; };

    let mut movement = time.delta().as_secs_f32() * PLAYER_VELOCIY_Y; // Upward distance to move this frame

    if movement + jump.0 >= MAX_JUMP_HEIGHT {
        movement = MAX_JUMP_HEIGHT - jump.0; // Clamp movement so player doesn't exceed max jump height
        commands.entity(entity).remove::<Jump>(); // Remove Jump component so fall system takes over
    }

    jump.0 += movement; // Accumulate total distance jumped so far

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)), // Preserve horizontal movement while rising
        None => player.translation = Some(Vec2::new(0.0, movement)),        // No horizontal input, only move vertically
    }
}

// fall
// Purpose: Moves the player downward every frame when the Jump component is not present.
// Pre-condition:  The player entity has a KinematicCharacterController and no Jump component.
// Post-condition: Player translation is updated downward each frame.
// Parameters:
//   time (in):      Bevy time resource used to calculate frame-delta movement
//   query (in/out): Query for the player controller, excluding entities with Jump
fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController, Without<Jump>>) {
    if query.is_empty() {
        return;
    }

    let Ok(mut player) = query.single_mut() else { return; };

    let movement = time.delta().as_secs_f32() * (PLAYER_VELOCIY_Y / 1.5) * -1.0; // Downward distance this frame, adjust divisor to change fall speed

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)), // Preserve horizontal movement while falling
        None => player.translation = Some(Vec2::new(0.0, movement)),        // No horizontal input, only move vertically
    }
}

// Jump component stores the total distance the player has risen during the current jump
#[derive(Component)]
struct Jump(f32); // f32 tracks cumulative upward distance traveled

// jump
// Purpose: Detects jump input and inserts the Jump component on the player if grounded.
// Pre-condition:  The player entity has a KinematicCharacterController and KinematicCharacterControllerOutput.
// Post-condition: Jump component is inserted on the player entity if ArrowUp is pressed and player is grounded.
// Parameters:
//   input (in):        Bevy input resource for reading keyboard state
//   commands (in/out): Used to insert the Jump component onto the player entity
//   query (in):        Query for player entities that are grounded and not already jumping
fn jump(input: Res<ButtonInput<KeyCode>>, mut commands: Commands, query: Query<(Entity, &KinematicCharacterControllerOutput), (With<KinematicCharacterController>, Without<Jump>)>) {
    if query.is_empty() {
        return;
    }

    let Ok((player, output)) = query.single() else { return; };
    if input.pressed(KeyCode::ArrowUp) && output.grounded { // Only jump if the player is on the ground
        commands.entity(player).insert(Jump(0.0)); // Insert Jump with 0 distance accumulated
    }
}

// PlatformBundle groups all components needed to spawn a static platform entity
#[derive(Bundle)]
struct PlatformBundle {
    sprite: Sprite,       // Visual appearance of the platform
    transform: Transform, // Position and scale of the platform in the world
    body: RigidBody,      // Rapier rigid body, set to Fixed so the platform does not move
    collider: Collider,   // Rapier collider shape used for physics collision detection
    marker: Platform,     // Marker component used to identify platforms for despawning
}

impl PlatformBundle {
    // new
    // Purpose: Constructs a PlatformBundle at a given horizontal position and scale.
    // Pre-condition:  x is a valid horizontal position, scale is a valid Vec3 size.
    // Post-condition: Returns a fully initialized PlatformBundle ready to be spawned.
    // Return: PlatformBundle
    // Parameters:
    //   x (in):     Horizontal position of the platform in world space, adjust to move platform left/right
    //   scale (in): Size of the platform, x = width, y = height, adjust to resize the platform
    fn new(x: f32, scale: Vec3, asset_server: &Res<AssetServer>) -> Self {
        Self {
            sprite: Sprite {
                image: asset_server.load("flappy_pillar.png"),
                custom_size: Some(Vec2::new(scale.x, scale.y)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(x, WINDOW_BOTTOM_Y + (scale.y / 2.0), 0.0), // Position platform so its base sits at the bottom, adjust x to reposition horizontally
                ..Default::default()
            },
            body: RigidBody::Fixed,           // Platform does not move under physics simulation
            collider: Collider::cuboid(0.5, 0.5), // Box collider matching the unit sprite size
            marker: Platform,                 // Tag this entity as a platform for despawn queries
        }
    }
}

// setup
// Purpose: Spawns all initial game entities including platforms, the floor, the player, and the camera.
// Pre-condition:  Called once at startup by Bevy.
// Post-condition: All game entities are present in the world and ready for simulation.
// Parameters:
//   commands (in/out):   Used to spawn entities into the Bevy world
//   meshes (in/out):     Asset store for meshes, used to create the player circle shape
//   materials (in/out):  Asset store for materials, used to apply color to the player mesh
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {

    // Spawn background
    commands.spawn((
        Sprite {
            image: asset_server.load("flappy_nightsky.jpg"),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, -100.0),
            ..Default::default()
        },
    ));

    // Spawn initial platforms, adjust x and Vec3 values to reposition or resize each platform
    commands.spawn(PlatformBundle::new(-100.0, Vec3::new(75.0, 200.0, 1.0), &asset_server));  // Left platform
    commands.spawn(PlatformBundle::new(100.0, Vec3::new(50.0, 350.0, 1.0), &asset_server));   // Center platform
    commands.spawn(PlatformBundle::new(350.0, Vec3::new(150.0, 250.0, 1.0), &asset_server)); // Right platform

    // Spawn the floor as a fixed static entity spanning the window width
    commands.spawn((
        Sprite {
            color: FLOOR_COLOR, // Use the global floor color constant
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(0.0, WINDOW_BOTTOM_Y + (FLOOR_THICKNESS / 2.0), 0.0), // Center floor at the bottom of the window
            scale: Vec3::new(WINDOW_WIDTH, FLOOR_THICKNESS, 1.0), // Stretch floor across the full window width
            ..Default::default()
        },
        RigidBody::Fixed,           // Floor does not move
        Collider::cuboid(0.5, 0.5), // Box collider for the floor
        Floor,
    ));

    // Spawn the player as a kinematic circle with physics control
    commands.spawn((
        Sprite::from_image(asset_server.load("flappy_bird.png")), // Apply player color material
        Transform {
            translation: Vec3::new(WINDOW_LEFT_X + 100.0, WINDOW_BOTTOM_Y + 30.0, 0.0), // Starting position, adjust to move player spawn point
            scale: Vec3::new(0.05, 0.05, 1.0), // Player size in pixels, adjust to resize the player
            ..Default::default()
        },
        RigidBody::KinematicPositionBased,       // Player is kinematic, movement is controlled manually
        Collider::ball(0.5),                     // Circle collider matching the unit circle mesh
        KinematicCharacterController::default(),  // Rapier character controller for collision response
        Player,                                  // Tag this entity as the player for death detection
    ));

    commands.spawn(Camera2d); // Spawn the 2D camera to render the scene
}

// follow_camera_floor
// Purpose: Keeps the floor centered on the camera X position so it always fills the screen.
// Pre-condition:  A Camera2d entity and a Floor entity both exist in the world.
// Post-condition: Floor X position matches the camera X position every frame.
// Parameters:
//   camera_query (in):  Query to read the current camera X position
//   floor_query (in/out): Query for the floor transform
fn follow_camera_floor(camera_query: Query<&Transform, With<Camera2d>>, mut floor_query: Query<&mut Transform, (With<Floor>, Without<Camera2d>)>) {
    let Ok(camera) = camera_query.single() else { return; };
    let Ok(mut floor) = floor_query.single_mut() else { return; };

    floor.translation.x = camera.translation.x; // Keep floor centered on camera so it always fills the screen
}


// scroll_camera
// Purpose: Moves the camera rightward at the current speed every frame. Increases
//          the speed by CAMERA_SPEED_INCREMENT every CAMERA_SPEED_INTERVAL seconds.
// Pre-condition:  A Camera2d entity exists in the world.
// Post-condition: Camera X position is updated; camera speed may have increased.
// Parameters:
//   time (in):         Bevy time resource used to calculate frame-delta movement
//   camera_speed (in/out): Resource tracking current speed and time since last increase
//   query (in/out):    Query for the camera transform
fn scroll_camera(time: Res<Time>, mut camera_speed: ResMut<CameraSpeed>, mut query: Query<&mut Transform, With<Camera2d>>, game_over: Res<GameOver>) {
    let Ok(mut camera) = query.single_mut() else { return; };

    let delta = time.delta_secs(); // Time elapsed since last frame in seconds

    camera_speed.timer += delta; // Accumulate time toward next speed increase

    if game_over.0 {
        return;
    }

    if camera_speed.timer >= CAMERA_SPEED_INTERVAL {
        camera_speed.timer = 0.0; // Reset the interval timer
        camera_speed.speed = (camera_speed.speed + CAMERA_SPEED_INCREMENT).min(CAMERA_MAX_SPEED); // Increase speed, capped at max
    }

    camera.translation.x += camera_speed.speed * delta; // Move camera rightward by speed scaled to frame time
}

// generate_platforms
// Purpose: Procedurally spawns new platforms ahead of the camera as it scrolls right.
//          Platforms are randomly sized and spaced within configured bounds.
// Pre-condition:  A Camera2d entity exists; LevelState resource tracks generation progress.
// Post-condition: New platforms are spawned up to PLATFORM_SPAWN_AHEAD pixels ahead of the camera.
// Parameters:
//   commands (in/out):   Used to spawn new platform entities
//   camera_query (in):   Query to read the current camera X position
//   level_state (in/out): Resource tracking the furthest generated X position
fn generate_platforms(mut commands: Commands, camera_query: Query<&Transform, With<Camera2d>>, mut level_state: ResMut<LevelState>, game_over: Res<GameOver>, asset_server: Res<AssetServer>) {
    let Ok(camera) = camera_query.single() else { return; };

    let camera_right = camera.translation.x + WINDOW_WIDTH / 2.0; // X position of the right edge of the camera view
    let spawn_target = camera_right + PLATFORM_SPAWN_AHEAD;        // How far ahead we want platforms to exist

    if game_over.0 {
        return;
    }

    let mut rng = rand::rng(); // Random number generator for platform sizing and spacing

    while level_state.furthest_x < spawn_target {
        let gap = rng.random_range(PLATFORM_MIN_GAP..PLATFORM_MAX_GAP);       // Random horizontal gap before next platform, adjust range to change spacing
        let width = rng.random_range(PLATFORM_MIN_WIDTH..PLATFORM_MAX_WIDTH);  // Random platform width, adjust range to change platform sizes
        let height = rng.random_range(PLATFORM_MIN_HEIGHT..PLATFORM_MAX_HEIGHT); // Random platform height, adjust range to change platform heights

        let x = level_state.furthest_x + gap + width / 2.0; // Center X of the new platform

        commands.spawn(PlatformBundle::new(x, Vec3::new(width, height, 1.0), &asset_server)); // Spawn the platform at the calculated position

        level_state.furthest_x = x + width / 2.0; // Advance the generation cursor to the right edge of the new platform
    }
}

// despawn_platforms
// Purpose: Removes platform entities that have scrolled behind the camera left edge
//          to prevent unbounded memory growth.
// Pre-condition:  A Camera2d entity exists; platform entities have a Transform and Platform marker.
// Post-condition: Platforms more than PLATFORM_DESPAWN_BEHIND pixels behind the camera are despawned.
// Parameters:
//   commands (in/out):    Used to despawn platform entities
//   camera_query (in):    Query to read the current camera X position
//   platform_query (in):  Query for all platform entities and their transforms
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

// check_player_death
// Purpose: Checks if the player has fallen behind the left edge of the camera view.
//          If so, despawns the player and displays a Game Over screen.
// Pre-condition:  A Camera2d entity and a Player entity both exist in the world.
// Post-condition: Player entity is despawned and Game Over UI is spawned if player falls behind camera.
// Parameters:
//   commands (in/out):   Used to despawn the player and spawn the game over UI
//   camera_query (in):   Query to read the current camera X position
//   player_query (in):   Query to read the player transform
//   game_over_query(in): Query to check if game over screen is already showing
fn check_player_death(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera2d>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    game_over_query: Query<&GameOverScreen>,
    mut game_over: ResMut<GameOver>,
) {
    if !game_over_query.is_empty() {
        return; // Game over screen already showing, do nothing
    }

    let Ok(camera) = camera_query.single() else { return; };
    let Ok((player_entity, player_transform)) = player_query.single() else { return; };

    let camera_left = camera.translation.x - WINDOW_WIDTH / 2.0; // X position of the left edge of the camera view

    if player_transform.translation.x < camera_left {
        commands.entity(player_entity).despawn(); // Remove the player from the world
        game_over.0 = true;

        // Spawn a full screen dark overlay
        commands.spawn((
            Node {
                width: Val::Percent(100.0),   // Cover full screen width
                height: Val::Percent(100.0),  // Cover full screen height
                justify_content: JustifyContent::Center,  // Center children horizontally
                align_items: AlignItems::Center,          // Center children vertically
                flex_direction: FlexDirection::Column,    // Stack children vertically
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent dark overlay
            GameOverScreen,
        )).with_children(|parent| {
            // Game Over text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0, // Adjust to change the size of the game over text
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.3, 0.3)), // Red color for game over text
            ));

            // Subtitle prompt
            parent.spawn((
                Text::new("You were left behind!"),
                TextFont {
                    font_size: 30.0, // Adjust to change the size of the subtitle text
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)), // White subtitle text
            ));
        });
    }
}

// movement
// Purpose: Reads arrow key input and applies horizontal movement to the player each frame.
// Pre-condition:  The player entity has a KinematicCharacterController component.
// Post-condition: Player translation is updated with horizontal movement based on input.
// Parameters:
//   input (in):     Bevy input resource for reading keyboard state
//   time (in):      Bevy time resource used to calculate frame-delta movement
//   query (in/out): Query for the player's character controller
fn movement(input: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut query: Query<&mut KinematicCharacterController>) {
    let Ok(mut player) = query.single_mut() else { return; };

    let mut movement = Vec2::new(0.0, 0.0); // Stores the movement vector for this frame

    if input.pressed(KeyCode::ArrowRight) {
        movement.x += time.delta_secs() * PLAYER_VELOCITY_X; // Move right by velocity scaled to frame time
    }

    if input.pressed(KeyCode::ArrowLeft) {
        movement.x += time.delta_secs() * PLAYER_VELOCITY_X * -1.0; // Move left by negating the velocity
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement.x, vec.y)), // Apply horizontal input while preserving vertical movement
        None => player.translation = Some(Vec2::new(movement.x, 0.0)),        // No existing translation, apply horizontal only
    }
}
