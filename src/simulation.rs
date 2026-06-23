use bevy::prelude::*;

use crate::components::*;
use crate::physics::gravitational_acceleration;

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

    for _ in 0..steps {
        let mut acc = Vec2::ZERO;
        for (body_tf, body) in &bodies {
            acc += gravitational_acceleration(
                ship_tf.translation.truncate(),
                body_tf.translation.truncate(),
                body.mass,
            );
        }
        vel.0 += acc * sub_dt;
        ship_tf.translation += (vel.0 * sub_dt).extend(0.0);
    }

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
