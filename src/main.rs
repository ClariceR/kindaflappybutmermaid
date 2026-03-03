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
            check_collisions,
            despawn_moving_objects,
            shift_pipes_to_the_left,
            shift_collectibles_to_the_left,
            spawn_hooks.run_if(on_timer(Duration::from_millis(1000))), //spawn a pipe every one second
            spawn_coins.run_if(on_timer(Duration::from_millis(900))),
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
const COLLECTIBLES_SPEED: f32 = 50.0;

const COIN_SIZE: f32 = 6.0;

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

fn spawn_coins(mut commands: Commands, asset_server: Res<AssetServer>, time: ResMut<Time>)
{
    let image = asset_server.load("Coin.png");
    let gap_y_position = (time.elapsed_secs() * 4.2309875).sin() * CANVAS_SIZE.y / 4.;
    let transform = Transform::from_xyz(CANVAS_SIZE.x / 2., 0., 1.);

    commands.spawn((
        transform,
        Visibility::Visible,
        Coin,
        children![
(            Sprite {
                image,
                ..default()
            },
            Transform::from_xyz(0.0, gap_y_position, 1.0),
            Coin,)
        ]
        ));
}
fn shift_pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Hook>>, //We want to act on the position of the pipes, so we query the Transform components that are tagged as Pipe (as Pipe is the parent)
    time: Res<Time>,
) {
    for mut pipe in &mut pipes {
        pipe.translation.x -= HOOK_SPEED * time.delta_secs(); //move according to how fast we want the pipes to move
    }
}

fn shift_collectibles_to_the_left(
    mut collectibles: Query<&mut Transform, With<Coin>>,
    time: Res<Time>,
) {
    for mut collectible in &mut collectibles {
        collectible.translation.x -= COLLECTIBLES_SPEED * time.delta_secs();
    }
}

fn despawn_moving_objects(
    mut commands: Commands,
    objects: Query<
        (Entity, &Transform),
        Or<(With<Hook>, With<Coin>)>, //will match either hook or coin, so we're grouping them together here
    >,
) {
    for (entity, transform) in objects {
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

#[derive(Event)]
struct EndGame;

fn check_collisions(
    mut commands: Commands,
    player: Single<(&Sprite, Entity), With<Player>>,
    hook_segments: Query<
        (&Sprite, Entity), With<HookTop>, //will match either top or bottom, so we're grouping them together here
    >,
    coins: Query<
        (&Sprite, Entity), With<Coin>, //will match either top or bottom, so we're grouping them together here
    >,
    hook_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    mut gizmos: Gizmos, //visualize the colliders, draw the collider to the screen
    transform_helper: TransformHelper, //force the global transforms to be up to date when we need them to be
) -> Result<()> {
    //Pattern for the colliders:
    //Get the up-to-date global transform
    //Build the relevant collider struct
    //Draw the gizmo to show the collider

    //Get the up-to-date global transform
    let player_transform = transform_helper.compute_global_transform(player.1)?;

    //Build the relevant collider struct. This is how a collider looks like!
    let player_collider =
        BoundingCircle::new(player_transform.translation().xy(), PLAYER_SIZE / 2.);

    //Draw the gizmo to show the collider
    gizmos.circle_2d(
        player_transform.translation().xy(),
        PLAYER_SIZE / 2.,
        RED_400,
    );

    for (sprite, entity) in &hook_segments {
        //Get the up-to-date global transform
        let hook_transform = transform_helper.compute_global_transform(entity)?;

        //Build the relevant collider struct. This is how a collider looks like!
        let hook_collider = Aabb2d::new(
            hook_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        //Draw the gizmo to show the collider
        gizmos.rect_2d(
            hook_transform.translation().xy(),
            sprite.custom_size.unwrap(),
            RED_400,
        );
        if player_collider.intersects(&hook_collider) {
            info!("Collision detected!")
            //commands.trigger(EndGame);
        }
    }

    for (sprite, entity) in &coins {
        //Get the up-to-date global transform
        let coin_transform = transform_helper.compute_global_transform(entity)?;

        //Build the relevant collider struct. This is how a collider looks like!
        let coin_collider = BoundingCircle::new(coin_transform.translation().xy(), COIN_SIZE / 2.);

        //Draw the gizmo to show the collider
        gizmos.circle_2d(
            coin_transform.translation().xy(),
            COIN_SIZE / 2.,
            RED_400,
        );
        if player_collider.intersects(&coin_collider) {
            info!("Point!")
            //commands.trigger(EndGame);
        }
    }

    /*
    // for (sprite, entity) in &hook_gaps {
    //     //Get the up-to-date global transform
    //     let gap_transform = transform_helper.compute_global_transform(entity)?;
    //
    //     //Build the relevant collider struct
    //     let gap_collider = Aabb2d::new(
    //         gap_transform.translation().xy(),
    //         sprite.custom_size.unwrap() / 2.,
    //     );
    //     //Draw the gizmo to show the collider
    //     gizmos.rect_2d(
    //         gap_transform.translation().xy(),
    //         sprite.custom_size.unwrap().xy(),
    //         RED_400,
    //     );
    //
    //     if player_collider.intersects(&gap_collider) {
    //         commands.trigger(ScorePoint);
    //         commands.entity(entity).despawn();
    //     }
    // }
    */

    Ok(())
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
 */
