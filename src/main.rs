use bevy::input::ButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

#[derive(Event)]
struct PlaceBlockEvent(Vec3);

#[derive(Event)]
struct DespawnBlockEvent(Vec3);

const PLAYER_SPEED: f32 = 5.0;
const JUMP_SPEED: f32 = 5.0;
const BLOCK_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Game".to_string(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, mouse_look))
        .add_systems(
            Update,
            (
                handle_place_block,
                handle_despawn_block,
                place_or_destroy_block,
                apply_gravity,
                player_jump,
            ),
        )
        .add_event::<PlaceBlockEvent>()
        .add_event::<DespawnBlockEvent>()
        .run();
}

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 1.0, 1.0),
        brightness: 0.5,
    });

    // Directional light (sun)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ground (flat 16x1x16 world)
    let ground_size = 16;
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.4, 0.8, 0.4),
        ..default()
    });

    for x in 0..ground_size {
        for z in 0..ground_size {
            commands.spawn(PbrBundle {
                mesh: cube_mesh.clone(),
                material: cube_material.clone(),
                transform: Transform::from_xyz(x as f32, 0.0, z as f32),
                ..default()
            });
        }
    }

    // Player (camera)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 5.0, 8.0)
                .looking_at(Vec3::new(8.0, 0.0, 7.0), Vec3::Y),
            ..default()
        },
        Player,
        CameraController {
            pitch: 0.0,
            yaw: 0.0,
        },
        Velocity {
            linvel: Vec3::ZERO,
            on_ground: true,
        },
    ));
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();
    let forward = transform.forward();
    let right = transform.right();
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *right;
    }

    direction.y = 0.0;
    direction = direction.normalize_or_zero();

    transform.translation += direction * PLAYER_SPEED * time.delta_seconds();

    // Basic jump (no gravity)
    if keys.just_pressed(KeyCode::Space) {
        transform.translation.y += JUMP_SPEED * time.delta_seconds();
    }

    // Ground clamp
    if transform.translation.y < 1.0 {
        transform.translation.y = 1.0;
    }
}

fn mouse_look(
    mut events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
    time: Res<Time>,
) {
    let sensitivity = 0.1;
    let (mut transform, mut controller) = query.single_mut();

    for event in events.read() {
        controller.yaw -= event.delta.x * sensitivity * time.delta_seconds();
        controller.pitch += event.delta.y * sensitivity * time.delta_seconds();

        // Clamp pitch to avoid flipping
        controller.pitch = controller.pitch.clamp(-1.54, 1.54); // ~±88°

        transform.rotation =
            Quat::from_rotation_y(controller.yaw) * Quat::from_rotation_x(-controller.pitch);
    }
}
#[derive(Component, Default)]
struct Velocity {
    linvel: Vec3,
    on_ground: bool,
}

fn apply_gravity(time: Res<Time>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    let gravity = -9.81;
    for (mut transform, mut velocity) in &mut query {
        if !velocity.on_ground {
            velocity.linvel.y += gravity * time.delta_seconds();
        }

        transform.translation += velocity.linvel * time.delta_seconds();

        // ground collision at y = 1.0 (top of ground cube)
        if transform.translation.y <= 1.0 {
            transform.translation.y = 1.0;
            velocity.linvel.y = 0.0;
            velocity.on_ground = true;
        }
    }
}

fn player_jump(keys: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let mut velocity = query.single_mut();
    if velocity.on_ground && keys.just_pressed(KeyCode::Space) {
        velocity.linvel.y = 5.0;
        velocity.on_ground = false;
    }
}

#[derive(Component)]
struct CameraController {
    pitch: f32,
    yaw: f32,
}

fn place_or_destroy_block(
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&Transform, With<Player>>,
    mut place_writer: EventWriter<PlaceBlockEvent>,
    mut despawn_writer: EventWriter<DespawnBlockEvent>,
) {
    let camera_transform = camera_query.single();
    let origin = camera_transform.translation;
    let direction = camera_transform.forward();

    for i in 1..10 {
        let check_pos = (origin + direction * i as f32).floor();
        let place_pos = (origin + direction * (i as f32 - 1.0)).floor();

        if buttons.just_pressed(MouseButton::Left) {
            despawn_writer.send(DespawnBlockEvent(check_pos));
            break;
        } else if buttons.just_pressed(MouseButton::Right) {
            place_writer.send(PlaceBlockEvent(place_pos));
            break;
        }
    }
}

fn handle_place_block(
    mut events: EventReader<PlaceBlockEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for PlaceBlockEvent(pos) in events.read() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
                1.0, 1.0, 1.0,
            ))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.4, 0.8, 0.4),
                ..default()
            }),
            transform: Transform::from_translation(*pos + Vec3::Y * 0.5),
            ..default()
        });
    }
}

fn handle_despawn_block(
    mut events: EventReader<DespawnBlockEvent>,
    mut commands: Commands,
    blocks: Query<(Entity, &Transform), Without<Player>>,
) {
    for DespawnBlockEvent(pos) in events.read() {
        for (entity, transform) in blocks.iter() {
            if transform.translation.floor() == *pos {
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}
