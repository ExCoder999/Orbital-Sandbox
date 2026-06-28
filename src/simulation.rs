use bevy::prelude::*;

use crate::components::*;
use crate::physics::gravitational_acceleration;

type Attractor = (Vec2, f32);

fn total_acceleration(pos: Vec2, attractors: &[Attractor]) -> Vec2 {
    attractors
        .iter()
        .map(|(attractor_pos, mass)| gravitational_acceleration(pos, *attractor_pos, *mass))
        .sum()
}

fn velocity_verlet_substep(
    pos: &mut Vec2,
    vel: &mut Vec2,
    acc_old: &mut Vec2,
    attractors: &[Attractor],
    dt: f32,
) {
    *pos += *vel * dt + 0.5 * *acc_old * dt * dt;
    let acc_new = total_acceleration(*pos, attractors);
    *vel += 0.5 * (*acc_old + acc_new) * dt;
    *acc_old = acc_new;
}

const TRAIL_SPAWN_INTERVAL: f32 = 0.05;
const TRAIL_LIFETIME: f32 = 3.0;
const TARGET_RADIUS: f32 = 20.0;
const MAX_TRAIL: usize = 200;

pub fn step_physics(
    mut ship_query: Query<(&mut Transform, &mut Velocity), With<Ship>>,
    bodies: Query<(&Transform, &GravityBody), Without<Ship>>,
    mut game: ResMut<GameState>,
    time: Res<Time>,
    mut trail_timer: Local<f32>,
    mut commands: Commands,
    trail_dots: Query<Entity, With<TrailDot>>,
) {
    let Ok((mut ship_tf, mut vel)) = ship_query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs().min(0.05); // cap at 50ms to avoid spiral on lag
    let steps = 4;
    let sub_dt = dt / steps as f32;

    let attractors: Vec<Attractor> = bodies
        .iter()
        .map(|(body_tf, body)| (body_tf.translation.truncate(), body.mass))
        .collect();

    let mut pos = ship_tf.translation.truncate();
    let mut acc_old = total_acceleration(pos, &attractors);

    for _ in 0..steps {
        velocity_verlet_substep(&mut pos, &mut vel.0, &mut acc_old, &attractors, sub_dt);
    }

    ship_tf.translation = pos.extend(0.0);

    // Spawn trail dot
    *trail_timer += dt;
    if *trail_timer >= TRAIL_SPAWN_INTERVAL {
        *trail_timer = 0.0;
        if trail_dots.iter().count() < MAX_TRAIL {
            commands.spawn((
                Transform::from_translation(ship_tf.translation),
                Visibility::default(),
                TrailDot { lifetime: TRAIL_LIFETIME },
            ));
        }
    }

    // Out of bounds → fail
    let pos = ship_tf.translation.truncate();
    if pos.x.abs() > 720.0 || pos.y.abs() > 450.0 {
        game.outcome = Outcome::Fail;
        game.simulating = false;
    }
}

pub fn check_win_fail(
    ship_query: Query<&Transform, With<Ship>>,
    bodies: Query<(&Transform, &GravityBody)>,
    target_query: Query<&Transform, With<Target>>,
    mut game: ResMut<GameState>,
) {
    let Ok(ship_tf) = ship_query.get_single() else { return };
    let ship_pos = ship_tf.translation.truncate();

    for (body_tf, body) in &bodies {
        if ship_pos.distance(body_tf.translation.truncate()) < body.radius + 4.0 {
            game.outcome = Outcome::Fail;
            game.simulating = false;
            return;
        }
    }

    let Ok(target_tf) = target_query.get_single() else { return };
    if ship_pos.distance(target_tf.translation.truncate()) < TARGET_RADIUS {
        game.outcome = Outcome::Win;
        game.simulating = false;
    }
}

pub fn draw_trail(
    mut gizmos: Gizmos,
    mut dots: Query<(Entity, &Transform, &mut TrailDot)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, transform, mut dot) in &mut dots {
        dot.lifetime -= time.delta_secs();
        if dot.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (dot.lifetime / TRAIL_LIFETIME).clamp(0.0, 1.0) * 0.65;
        gizmos.circle_2d(
            transform.translation.truncate(),
            2.5,
            Color::srgb(1.0, 0.5, 0.2).with_alpha(alpha),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::G;

    fn euler_substep(
        pos: &mut Vec2,
        vel: &mut Vec2,
        attractors: &[Attractor],
        dt: f32,
    ) {
        let acc = total_acceleration(*pos, attractors);
        *vel += acc * dt;
        *pos += *vel * dt;
    }

    fn circular_orbit_state(radius: f32, central_mass: f32) -> (Vec2, Vec2) {
        let speed = (G * central_mass / radius).sqrt();
        (Vec2::new(radius, 0.0), Vec2::new(0.0, speed))
    }

    fn specific_energy(pos: Vec2, vel: Vec2, central_mass: f32) -> f32 {
        let r = pos.length().max(1.0);
        0.5 * vel.length_squared() - G * central_mass / r
    }

    #[test]
    fn verlet_circular_orbit_energy_more_stable_than_euler() {
        let attractors = [(Vec2::ZERO, 1_000.0)];
        let radius = 200.0;
        let steps = 2_000;
        let dt = 0.01;

        let (start_pos, start_vel) = circular_orbit_state(radius, attractors[0].1);
        let initial_energy = specific_energy(start_pos, start_vel, attractors[0].1);

        let mut verlet_pos = start_pos;
        let mut verlet_vel = start_vel;
        let mut acc_old = total_acceleration(verlet_pos, &attractors);
        for _ in 0..steps {
            velocity_verlet_substep(
                &mut verlet_pos,
                &mut verlet_vel,
                &mut acc_old,
                &attractors,
                dt,
            );
        }
        let verlet_drift =
            (specific_energy(verlet_pos, verlet_vel, attractors[0].1) - initial_energy).abs();

        let mut euler_pos = start_pos;
        let mut euler_vel = start_vel;
        for _ in 0..steps {
            euler_substep(&mut euler_pos, &mut euler_vel, &attractors, dt);
        }
        let euler_drift =
            (specific_energy(euler_pos, euler_vel, attractors[0].1) - initial_energy).abs();

        assert!(
            verlet_drift < euler_drift / 10.0,
            "expected Verlet energy drift ({verlet_drift}) to be much smaller than Euler ({euler_drift})"
        );
    }

    #[test]
    fn verlet_substep_updates_position_before_velocity() {
        let attractors = [(Vec2::new(100.0, 0.0), 500.0)];
        let dt = 0.1;
        let mut pos = Vec2::new(-50.0, 0.0);
        let mut vel = Vec2::new(0.0, 30.0);
        let mut acc_old = total_acceleration(pos, &attractors);

        let pos_before = pos;
        let vel_before = vel;
        let acc_before = acc_old;
        let expected_pos = pos_before + vel_before * dt + 0.5 * acc_before * dt * dt;
        let acc_new = total_acceleration(expected_pos, &attractors);
        let expected_vel = vel_before + 0.5 * (acc_before + acc_new) * dt;

        velocity_verlet_substep(&mut pos, &mut vel, &mut acc_old, &attractors, dt);

        assert!(
            pos.distance(expected_pos) < 1e-4,
            "position must use pre-step velocity and acc_old"
        );
        assert!(
            vel.distance(expected_vel) < 1e-4,
            "velocity must use averaged old and new accelerations"
        );
    }

    #[test]
    fn verlet_orbit_radius_stays_bounded() {
        let attractors = [(Vec2::ZERO, 800.0)];
        let radius = 150.0;
        let steps = 5_000;
        let dt = 0.008;

        let (start_pos, start_vel) = circular_orbit_state(radius, attractors[0].1);
        let mut pos = start_pos;
        let mut vel = start_vel;
        let mut acc_old = total_acceleration(pos, &attractors);

        for _ in 0..steps {
            velocity_verlet_substep(&mut pos, &mut vel, &mut acc_old, &attractors, dt);
            let r = pos.length();
            assert!(
                (r - radius).abs() < radius * 0.15,
                "radius drifted to {r}, expected to stay near {radius}"
            );
        }
    }
}
