use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin, input::mouse::AccumulatedMouseScroll, prelude::*,
};
use iter_tools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

const BOUNDARY_SIZE: f32 = 1000.0;

#[derive(Component, Debug)]
pub struct Atom {
    radius: f32,
}

#[derive(Component, Debug)]
pub struct Velocity(Vec3);

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    let circle = meshes.add(Circle::new(10.0));
    let color = materials.add(Color::linear_rgb(1.0, 1.0, 1.0));

    for _ in 0..1000 {
        let x = rand::random::<f32>() * BOUNDARY_SIZE - BOUNDARY_SIZE / 2.0;
        let y = rand::random::<f32>() * BOUNDARY_SIZE - BOUNDARY_SIZE / 2.0;
        commands.spawn((
            Mesh2d(circle.clone()),
            MeshMaterial2d(color.clone()),
            Transform::from_xyz(x, y, 0.0),
            Velocity(Vec3::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
                0.0,
            )),
            Atom { radius: 10.0 },
        ));
    }

    let red = materials.add(Color::linear_rgb(1.0, 0.0, 0.0));

    commands.spawn((
        Mesh2d(circle.clone()),
        MeshMaterial2d(red),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

fn camera(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
    camera: Single<(&Camera2d, &mut Transform, &mut Projection)>,
) {
    let mut camera = camera.into_inner();
    let move_speed = match camera.2.into_inner() {
        Projection::Orthographic(orthographic) => {
            orthographic.scale =
                (orthographic.scale / (1.0 + mouse_wheel_input.delta.y * 0.1)).clamp(0.1, 1000.0);
            20.0 * orthographic.scale
        }
        _ => todo!(),
    };
    if keys.pressed(KeyCode::KeyW) {
        camera.1.translation += Vec3::new(0.0, move_speed, 0.0)
    }
    if keys.pressed(KeyCode::KeyS) {
        camera.1.translation += Vec3::new(0.0, -move_speed, 0.0)
    }
    if keys.pressed(KeyCode::KeyA) {
        camera.1.translation += Vec3::new(-move_speed, 0.0, 0.0)
    }
    if keys.pressed(KeyCode::KeyD) {
        camera.1.translation += Vec3::new(move_speed, 0.0, 0.0)
    }
}

fn gravity(mut query: Query<(&Atom, &mut Velocity, &Transform)>, time: Res<Time>) {
    let center = Vec3::new(0.0, 0.0, 0.0);
    for (_, mut velocity, position) in query.iter_mut() {
        let distance = position.translation.distance(center);
        if distance < 0.5 {
            continue;
        }
        let force_direction = (center - position.translation).normalize();
        let force_magnitude = 1000.0 / (distance);
        let force = force_direction * force_magnitude;
        velocity.0 += force * time.delta_secs();
    }
}

fn movement(mut query: Query<(&Atom, &Velocity, &mut Transform)>, time: Res<Time>) {
    for (_, velocity, mut position) in query.iter_mut() {
        position.translation += velocity.0 * time.delta_secs();
    }
}

const ELASTICITY: f32 = 0.2;

fn collision(mut query: Query<(Entity, &Atom, &mut Transform, &mut Velocity)>) {
    let deltas: Vec<(Entity, Vec3, Vec3)> = query
        .iter()
        .collect_vec()
        .into_iter()
        .tuple_combinations()
        .par_bridge()
        .filter_map(
            |(
                (entity_a, atom_a, transform_a, velocity_a),
                (entity_b, atom_b, transform_b, velocity_b),
            )| {
                if entity_a == entity_b {
                    return None;
                }

                let distance = transform_a.translation.distance(transform_b.translation);
                let min_distance = atom_a.radius + atom_b.radius;

                if distance > min_distance {
                    return None;
                }

                let normal = (transform_b.translation - transform_a.translation).normalize();
                if normal.is_nan() {
                    return None;
                }
                let relative_velocity = velocity_b.0 - velocity_a.0;
                let velocity_along_normal = relative_velocity.dot(normal);
                if velocity_along_normal > 0.0 {
                    return None;
                }
                let impulse = ELASTICITY * velocity_along_normal;
                let impulse_vector = normal * impulse;

                let delta_position = -(normal * (min_distance - distance) / 2.0);

                Some((entity_a, Vec3::new(0.0, 0.0, 0.0), impulse_vector))
            },
        )
        .collect();

    for (entity_a, delta_position_a, delta_velocity_a) in deltas {
        if let Ok((_, _, mut transform_a, mut velocity_a)) = query.get_mut(entity_a) {
            transform_a.translation += delta_position_a;
            velocity_a.0 += delta_velocity_a;
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FpsOverlayPlugin::default()))
        .add_systems(Startup, startup)
        .add_systems(Update, (camera, gravity, movement, collision))
        .run();
}
