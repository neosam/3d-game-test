// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Resource)]
pub struct GameAssets {
    pub player: Handle<Scene>,
    pub tree: Handle<Scene>,
}

#[derive(Component, Reflect)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    scene_bundle: SceneBundle,
    rigid_body: RigidBody,
    collider: Collider,
    player: Player,
    velocity: Velocity,
    character_controller: KinematicCharacterController,
    locked_axes: LockedAxes,
}
impl PlayerBundle {
    pub fn new(assets: &GameAssets) -> Self {
        PlayerBundle {
            scene_bundle: SceneBundle {
                scene: assets.player.clone(),
                ..Default::default()
            },
            rigid_body: RigidBody::Dynamic,
            collider: Collider::capsule_y(1.0, 0.5),
            player: Player,
            velocity: Velocity::default(),
            character_controller: KinematicCharacterController::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
        }
    }
}

#[derive(Component, Reflect)]
pub struct CameraController {
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub distance: f32,
    pub lock_entity: Entity,
}
impl CameraController {
    pub fn new(lock_entity: Entity) -> Self {
        CameraController {
            rotation_y: 0.0,
            rotation_x: 0.0,
            distance: 5.0,
            lock_entity,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                cursor_visible: false,
                cursor_grab_mode: bevy::window::CursorGrabMode::Locked,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(camera_movement)
        .add_system(keyboard_input)
        .add_system(apply_camera_position)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GameAssets {
        player: asset_server.load("human.glb#Scene0"),
        tree: asset_server.load("tree.glb#Scene0"),
    };
    let player = commands.spawn(PlayerBundle::new(&assets)).id();
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        CameraController::new(player),
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(1000.0, 0.1, 1000.0),
        Transform::from_xyz(0.0, -2.0, 0.0),
        GlobalTransform::default(),
    ));

    commands.spawn(SceneBundle {
        scene: asset_server.load("tree.glb#Scene0"),
        transform: Transform::from_xyz(2.0, 0.0, 0.0),
        ..Default::default()
    });
}

pub fn camera_movement(
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_controller_query: Query<&mut CameraController>,
) {
    if let Ok(mut camera_controller) = camera_controller_query.get_single_mut() {
        for mouse_event in mouse_motion_events.iter() {
            camera_controller.rotation_x += (mouse_event.delta.y as f32) * 0.01;
            camera_controller.rotation_x = camera_controller
                .rotation_x
                .min(std::f32::consts::PI / 2.0 * 0.9)
                .max(-std::f32::consts::PI / 2.0 * 0.9);
            camera_controller.rotation_y -= (mouse_event.delta.x as f32) * 0.01;
        }
        for scroll_event in scroll_events.iter() {
            camera_controller.distance += (scroll_event.y as f32) * 0.01;
            camera_controller.distance = camera_controller.distance.min(10.0).max(2.0);
        }
    }
}

pub fn apply_camera_position(
    mut camera_query: Query<(&mut Transform, &CameraController)>,
    entity_position_query: Query<&Transform, Without<CameraController>>,
) {
    if let Ok((mut camera_transform, camera_controller)) = camera_query.get_single_mut() {
        if let Ok(look_at_transform) = entity_position_query.get(camera_controller.lock_entity) {
            let distance = camera_controller.distance;
            let rot_y = camera_controller.rotation_y;
            let rot_x = camera_controller.rotation_x;
            *camera_transform = Transform::from_xyz(
                look_at_transform.translation.x + distance * (rot_y.sin() * rot_x.cos()),
                look_at_transform.translation.y + distance * rot_x.sin(),
                look_at_transform.translation.z + distance * (rot_y.cos() * rot_x.cos()),
            )
            .looking_at(look_at_transform.translation, Vec3::Y);
        }
    }
}

pub fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    camera_query: Query<&CameraController>,
    mut entity_position_query: Query<(&mut Transform, &mut Velocity), Without<CameraController>>,
) {
    if keys.pressed(KeyCode::W) {
        if let Ok(camera_controller) = camera_query.get_single() {
            if let Ok((mut look_at_transform, mut velocity)) =
                entity_position_query.get_mut(camera_controller.lock_entity)
            {
                let vector = Vec3::new(
                    camera_controller.rotation_y.sin(),
                    0.0,
                    camera_controller.rotation_y.cos(),
                );
                let direction = look_at_transform.translation + vector;
                look_at_transform.look_at(direction, Vec3::Y);
                velocity.linvel = -vector;
            }
        }
    }
    if keys.pressed(KeyCode::S) {
        if let Ok(camera_controller) = camera_query.get_single() {
            if let Ok((mut look_at_transform, mut velocity)) =
                entity_position_query.get_mut(camera_controller.lock_entity)
            {
                let vector = Vec3::new(
                    camera_controller.rotation_y.sin(),
                    0.0,
                    camera_controller.rotation_y.cos(),
                );
                let direction = look_at_transform.translation - vector;
                look_at_transform.look_at(direction, Vec3::Y);
                velocity.linvel = vector;
            }
        }
    }
}
