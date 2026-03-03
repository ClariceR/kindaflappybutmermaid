use bevy::camera::ScalingMode;
use bevy::color::palettes::tailwind::*;
use bevy::image::ImageSamplerDescriptor;
use bevy::{input::common_conditions::input_toggle_active, prelude::*, time::common_conditions::on_timer};
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use std::time::Duration;
fn main() {
    App::new()
        .insert_resource(ClearColor(INDIGO_900.into()))
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
            EnhancedInputPlugin,
        ))
        .add_systems(Startup, (startup))
        .add_systems(FixedUpdate, (
            despawn_pipes,
            shift_pipes_to_the_left,
            spawn_hooks.run_if(on_timer(Duration::from_millis(1000))), //spawn a pipe every one second
        ))
        .add_systems(Update, (controls, calculate_physics))
        //.add_observer(respawn_on_endgame)
        .run();
}

const PLAYER_SIZE: f32 = 32.;
const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
const FLOOR_SIZE: Vec2 = Vec2::new(480., 32.);
const SKY_SIZE: Vec2 = Vec2::new(480., 24.);
const GRAVITY: f32 = 900.0;
const HOOK_SIZE: Vec2 =  Vec2::new(5., CANVAS_SIZE.y);
const GAP_SIZE: f32 = 100.0;
pub const HOOK_SPEED: f32 = 100.0;

//Player
#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct PlayerPhysics {
    velocity: Vec2,
    is_grounded: bool,
    is_afloat: bool,
}

#[derive(Debug, InputAction)]
#[action_output(f32)]
struct Movement;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Dive;

/*
// #[derive(Component)]
// #[require(Gravity(500.), Velocity)]
// struct Player;

// #[derive(Component)]
// struct Gravity(f32);
//
// #[derive(Component, Default)]
// struct Velocity(f32);
//
 */
#[derive(Component)]
struct Floor;
#[derive(Component)]
struct Sky;
#[derive(Component)]
struct Skull;
#[derive(Component)]
struct Coin;

//Obstacles
#[derive(Component)]
struct Hook;
#[derive(Component)]
struct HookTop; //Child of Hook
#[derive(Component)]
pub struct PointsGate; //Child of Hook

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: CANVAS_SIZE.x,
                max_height: CANVAS_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("mermaid-v2.png"),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
        Player,
        PlayerPhysics::default(),
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("Skull-Common.png"),
            //custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(70., 50., 1.),
        Skull,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("Skull-Pirate.png"),
            //custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(90., -50., 1.),
        Skull,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("Skull-Queen.png"),
            //custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(100., 0., 1.),
        Skull,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("Coin.png"),
            //custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(120., 25., 1.),
        Coin,
    ));

    commands.spawn((
        Sprite {
            image: asset_server.load("floor.png"),
            custom_size: Some(FLOOR_SIZE),
            ..default()
        },
        Transform::from_xyz(0., -(CANVAS_SIZE.y / 2. - FLOOR_SIZE.y / 2.), 0.),
        Floor,
    ));

    commands.spawn((
        Sprite {
            color: Color::Srgba(INDIGO_800),
            custom_size: Some(SKY_SIZE),
            ..default()
        },
        Transform::from_xyz(0., (CANVAS_SIZE.y / 2. - SKY_SIZE.y / 2.), 0.),
        Sky,
        ));
}

fn spawn_hooks(mut commands: Commands, asset_server: Res<AssetServer>, time: ResMut<Time>)
{
    let image = asset_server.load("Hook.png");
    let image_mode = SpriteImageMode::Sliced(TextureSlicer {
        border: BorderRect::axes(1., 10.),
        center_scale_mode: SliceScaleMode::Stretch, //We are stretching only the center part of the pipe, preserving the top border part unchanged
        ..default()
    });

    let transform = Transform::from_xyz(CANVAS_SIZE.x / 2., 0., 1.);
    let gap_y_position = (time.elapsed_secs() * 4.2309875).sin() * CANVAS_SIZE.y / 4.; //sin function returns a value between -1 and 1.
    let pipe_offset = HOOK_SIZE.y / 2.0 + GAP_SIZE / 2.; //the positing is measured from the center, if we need to offset from the edge, we need to divide by 2

    commands.spawn((
        transform,           //position of the parent
        Visibility::Visible, //we need to add visibility as the parent it's just a tag and won't be visible
        Hook,                //parent tag
        children![
            (
                Sprite {
                    image,
                    custom_size: Some(HOOK_SIZE),
                    image_mode: image_mode.clone(), //same here, cloning as we need it again for bottom
                    ..default()
                },
                Transform::from_xyz(0.0, pipe_offset + gap_y_position, 1.0,),
                HookTop
            ),
            // (
            //     //Visibility::Hidden,
            //     Sprite {
            //         color: Color::WHITE,
            //         custom_size: Some(Vec2::new(5.0, HOOK_SIZE.y - gap_y_position,)),
            //         image_mode,
            //         ..default()
            //     },
            //     Transform::from_xyz(0.0, gap_y_position, 1.0,),
            //     PointsGate
            // ),
        ],
    ));

    // commands.spawn((
    //     Sprite {
    //         image: asset_server.load("Hook.png"),
    //         //custom_size: Some(FLOOR_SIZE),
    //         ..default()
    //     },
    //     Transform::from_xyz(0., 100., 1.),
    //     Hook,
    // ));
}

fn shift_pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Hook>>, //We want to act on the position of the pipes, so we query the Transform components that are tagged as Pipe (as Pipe is the parent)
    time: Res<Time>,
) {
    for mut pipe in &mut pipes {
        pipe.translation.x -= HOOK_SPEED * time.delta_secs(); //move according to how fast we want the pipes to move
    }
}

fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Hook>>,
    //we are query by entity as we want to destroy it, but only when it's out of the screen, so we need to query the transform as well to get the position of the pipes
) {
    for (entity, transform) in pipes {
        if transform.translation.x < -(CANVAS_SIZE.x / 2.0 + HOOK_SIZE.x) {
            commands.entity(entity).despawn(); //.entity gets access to the entity and .despawn despawns the entity and remove its components
            //this happens recursively, and will include all children and their components as well
        }
    }
}

//helper function to visualise if pipes are despawning
fn count_pipes(query: Query<&Hook>) {
    info!("{} pipes exist", query.iter().len());
}

fn calculate_physics(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerPhysics)>
) {
    for (mut transform, mut physics) in query.iter_mut() {
        physics.velocity.y -= GRAVITY * time.delta_secs();
        transform.translation.y += physics.velocity.y * time.delta_secs();

        //Prevent moving off-screen
        const MAX_Y: f32 = CANVAS_SIZE.y / 2.0 - PLAYER_SIZE / 2.0;
        transform.translation.y = transform.translation.y.clamp(-MAX_Y, MAX_Y)
    }
}

fn controls(
    mut velocity: Query<&mut PlayerPhysics, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    for mut velocity in velocity.iter_mut() {
        if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
            //Mouse button pressed will add 400 to the velocity that was previously 0, making the player position go up in a sudden movement.
            //Gravity will then make the position go down over time
            velocity.velocity.y = 300.;
        }
    }

}

/*
// fn gravity(mut transforms: Query<(&mut Velocity, &mut Transform, &Gravity)>, time: Res<Time>) {
//     for (mut velocity, mut transform, gravity) in &mut transforms { //why do I need to say mut again here?
//         velocity.0 -= gravity.0 * time.delta_secs();
//         transform.translation.y -= velocity.0 * time.delta_secs();
//         //let boundaries = transform.translation.y.clamp(CANVAS_SIZE.y / 2., -CANVAS_SIZE.y / 2.);
//     }
// }
//


// #[derive(Event)]
// struct ResetPlayerPosition;

// fn check_in_bounds(player: Single<&Transform, With<Player>>, mut commands: Commands) {
//     if player.translation.y < -CANVAS_SIZE.y / 2.0 - PLAYER_SIZE
//         || player.translation.y > CANVAS_SIZE.y / 2.0 + PLAYER_SIZE
//     {
//         commands.trigger(ResetPlayerPosition);
//     }
// }
//
// //Observers are like systems, but the first argument has to be the event the system will listen for
// fn respawn_on_endgame(
//     _: On<ResetPlayerPosition>, //we don't care about the value, so we prefix it with an _
//     mut commands: Commands,
//     player: Single<Entity, With<Player>>, //As we want to act on the position of the player if the game ends, we query the player
// ) {
//     //Re-inserting the transform and velocity components will move the player to the initial position with initial velocity
//     commands.entity(*player).insert((
//         //commands.entity lets us take the actions on the entity directly
//         Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
//         Velocity(0.),
//     ));
// }

// fn check_collision_with_sky_and_ground(
//     mut commands: Commands,
//     player: Single<(&Sprite, Entity), With<Player>>,
//     sky: Single<(&Sprite, Entity), With<Sky>>,
//     floor: Single<(&Sprite, Entity), With<Floor>>,
//     mut gizmos: Gizmos,
//     transform_helper: TransformHelper,
// ) -> Result<()> {
//     //Pattern for the colliders:
//     //Get the up-to-date global transform
//     //Build the relevant collider struct
//     //Draw the gizmo to show the collider
//
//     let player_transform = transform_helper.compute_global_transform(player.1)?;
//     let player_collider = BoundingCircle::new(player_transform.translation().xy(), PLAYER_SIZE / 2.);
//     gizmos.circle_2d(player_transform.translation().xy(), PLAYER_SIZE / 2., RED_400);
//
//     let sky_sprite = sky.0;
//     let sky_transform = transform_helper.compute_global_transform(sky.1)?;
//     let sky_collider = Aabb2d::new(
//         sky_transform.translation().xy(),
//         sky_sprite.custom_size.unwrap() / 2.,
//     );
//     gizmos.rect_2d(
//         sky_transform.translation().xy(),
//         sky_sprite.custom_size.unwrap() / 2.,
//         RED_400,
//     );
//
//     if player_collider.intersects(&sky_collider)
//     {
//         info!("player on sky");
//         commands.trigger(KeepPlayerOnScreen);
//     }
//
//     let floor_sprite = floor.0;
//     let floor_transform = transform_helper.compute_global_transform(floor.1)?;
//     let floor_collider = Aabb2d::new(
//         floor_transform.translation().xy(),
//         floor_sprite.custom_size.unwrap() / 2.,
//     );
//     gizmos.rect_2d(
//         floor_transform.translation().xy(),
//         floor_sprite.custom_size.unwrap() / 2.,
//         RED_400,
//     );
//
//     if player_collider.intersects(&floor_collider)
//     {
//         info!("player on floor");
//         commands.trigger(KeepPlayerOnScreen);
//     }
//
//     Ok(())
// }

// #[derive(Event)]
// struct KeepPlayerOnScreen;

// fn keep_player_on_screen(
//     _: On<KeepPlayerOnScreen>,
//     mut commands: Commands,
//     player: Single<Entity, With<Player>>,
//     transform_helper: TransformHelper,
// )
// {
//     let player_transform = transform_helper.compute_global_transform(*player);
//     let player_y = player_transform.unwrap().translation().y;
//     if player_y > 0. {
//         commands.entity(*player).insert((
//
//             Transform::from_xyz(-CANVAS_SIZE.y / 4., CANVAS_SIZE.y / 2. - PLAYER_SIZE, 1.),
//             Velocity(0.)
//         ));
//     }
//     else {
//         commands.entity(*player).insert((
//
//             Transform::from_xyz(-CANVAS_SIZE.y / 4., - (CANVAS_SIZE.y / 2. - PLAYER_SIZE), 1.),
//             Velocity(0.)
//         ));
//
//     }
//     // commands.entity(*player).insert((
//     //
//     //     Transform::from_xyz(-CANVAS_SIZE.y / 4., CANVAS_SIZE.y / 2. - PLAYER_SIZE, 1.),
//     //     Velocity(0.)
//     //     ));
// }

 */
