use bevy::camera::ScalingMode;
use bevy::color::palettes::tailwind::*;
use bevy::image::ImageSamplerDescriptor;
use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
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
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, (check_collision_with_sky_and_ground))
        .add_observer(keep_player_on_screen)
        .run();
}

const PLAYER_SIZE: f32 = 32.;
const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
const FLOOR_SIZE: Vec2 = Vec2::new(480., 32.);
const SKY_SIZE: Vec2 = Vec2::new(480., 24.);
#[derive(Component)]
#[require(Gravity(1000.), Velocity)]
struct Player;

#[derive(Component)]
struct Gravity(f32);

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Sky;

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
            image: asset_server.load("mermaid3.png"),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
        Player,
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

fn gravity(mut transforms: Query<(&mut Velocity, &mut Transform, &Gravity)>, time: Res<Time>) {
    for (mut velocity, mut transform, gravity) in &mut transforms { //why do I need to say mut again here?
        velocity.0 -= gravity.0 * time.delta_secs(); //why do I need the .0??
        transform.translation.y += gravity.0 * time.delta_secs();
        //let boundaries = transform.translation.y.clamp(CANVAS_SIZE.y / 2., -CANVAS_SIZE.y / 2.);
    }
}
fn check_collision_with_sky_and_ground(
    mut commands: Commands,
    player: Single<(&Sprite, Entity), With<Player>>,
    sky: Single<(&Sprite, Entity), With<Sky>>,
    floor: Single<(&Sprite, Entity), With<Floor>>,
    mut gizmos: Gizmos,
    transform_helper: TransformHelper,
) -> Result<()> {
    //Pattern for the colliders:
    //Get the up-to-date global transform
    //Build the relevant collider struct
    //Draw the gizmo to show the collider

    let player_transform = transform_helper.compute_global_transform(player.1)?;
    let player_collider = BoundingCircle::new(player_transform.translation().xy(), PLAYER_SIZE / 2.);
    gizmos.circle_2d(player_transform.translation().xy(), PLAYER_SIZE / 2., RED_400);

    let sky_sprite = sky.0;
    let sky_transform = transform_helper.compute_global_transform(sky.1)?;
    let sky_collider = Aabb2d::new(
        sky_transform.translation().xy(),
        sky_sprite.custom_size.unwrap() / 2.,
    );
    gizmos.rect_2d(
        sky_transform.translation().xy(),
        sky_sprite.custom_size.unwrap() / 2.,
        RED_400,
    );

    if player_collider.intersects(&sky_collider)
    {
        info!("player on sky");
        commands.trigger(KeepPlayerOnScreen);
    }

    let floor_sprite = floor.0;
    let floor_transform = transform_helper.compute_global_transform(floor.1)?;
    let floor_collider = Aabb2d::new(
        floor_transform.translation().xy(),
        floor_sprite.custom_size.unwrap() / 2.,
    );
    gizmos.rect_2d(
        floor_transform.translation().xy(),
        floor_sprite.custom_size.unwrap() / 2.,
        RED_400,
    );

    if player_collider.intersects(&floor_collider)
    {
        info!("player on floor");
        commands.trigger(KeepPlayerOnScreen);
    }

    Ok(())
}

#[derive(Event)]
struct KeepPlayerOnScreen;

fn keep_player_on_screen(
    _: On<KeepPlayerOnScreen>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    transform_helper: TransformHelper,
)
{
    let player_transform = transform_helper.compute_global_transform(*player);
    let player_y = player_transform.unwrap().translation().y;
    if player_y > 0. {
        commands.entity(*player).insert((

            Transform::from_xyz(-CANVAS_SIZE.y / 4., CANVAS_SIZE.y / 2. - PLAYER_SIZE, 1.),
            Velocity(0.)
        ));
    }
    else {
        commands.entity(*player).insert((

            Transform::from_xyz(-CANVAS_SIZE.y / 4., - (CANVAS_SIZE.y / 2. - PLAYER_SIZE), 1.),
            Velocity(0.)
        ));

    }
    // commands.entity(*player).insert((
    //
    //     Transform::from_xyz(-CANVAS_SIZE.y / 4., CANVAS_SIZE.y / 2. - PLAYER_SIZE, 1.),
    //     Velocity(0.)
    //     ));
}
