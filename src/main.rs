use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseMotion;
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

const PLAYER_SPEED: f32 = 5.0;
const JUMP_SPEED: f32 = 5.0;
const BLOCK_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, mouse_look))
        .run();
}

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Ground (flat 16x1x16 world)
    let ground_size = 16;
    let cube_mesh = meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    // let cube_material = materials.add(Color::rgb(0.2, 0.7, 0.2).into());

    for x in 0..ground_size {
        for z in 0..ground_size {
            commands.spawn(PbrBundle {
                mesh: cube_mesh.clone(),
                // material: cube_material.clone(),
                transform: Transform::from_xyz(x as f32, 0.0, z as f32),
                ..default()
            });
        }
    }

    // Player (camera)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 4.0, 8.0)
                .looking_at(Vec3::new(8.0, 1.0, 7.0), Vec3::Y),
            ..default()
        },
        Player,
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
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut transform = query.single_mut();
    let sensitivity = 0.1;

    for event in events.read() {
        let yaw = Quat::from_rotation_y(-event.delta.x * sensitivity * time.delta_seconds());
        let pitch = Quat::from_rotation_x(-event.delta.y * sensitivity * time.delta_seconds());
        transform.rotation = yaw * transform.rotation;
        transform.rotation = transform.rotation * pitch;
    }
}
