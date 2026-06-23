use bevy::prelude::*;
use std::sync::LazyLock;

use crate::components::*;

pub static LEVELS: LazyLock<Vec<LevelConfig>> = LazyLock::new(build_levels);

fn build_levels() -> Vec<LevelConfig> {
    vec![
        // Level 1 — straight shot, one planet deflects
        LevelConfig {
            index: 0,
            name: "Gravity Assist".into(),
            ship_pos: Vec2::new(-480.0, 0.0),
            ship_vel: Vec2::new(180.0, 0.0),
            target_pos: Vec2::new(480.0, 0.0),
            max_placed: 2,
            bodies: vec![BodyDef {
                pos: Vec2::new(0.0, -200.0),
                mass: 3000.0,
                radius: 28.0,
                kind: BodyKind::Planet,
            }],
        },
        // Level 2 — orbit around a star
        LevelConfig {
            index: 1,
            name: "Solar Slingshot".into(),
            ship_pos: Vec2::new(-400.0, 200.0),
            ship_vel: Vec2::new(120.0, -80.0),
            target_pos: Vec2::new(400.0, -200.0),
            max_placed: 2,
            bodies: vec![BodyDef {
                pos: Vec2::ZERO,
                mass: 8000.0,
                radius: 42.0,
                kind: BodyKind::Star,
            }],
        },
        // Level 3 — binary planets
        LevelConfig {
            index: 2,
            name: "Binary Dance".into(),
            ship_pos: Vec2::new(-500.0, 0.0),
            ship_vel: Vec2::new(200.0, 30.0),
            target_pos: Vec2::new(500.0, 0.0),
            max_placed: 3,
            bodies: vec![
                BodyDef {
                    pos: Vec2::new(-150.0, 160.0),
                    mass: 4000.0,
                    radius: 32.0,
                    kind: BodyKind::Planet,
                },
                BodyDef {
                    pos: Vec2::new(150.0, -160.0),
                    mass: 4000.0,
                    radius: 32.0,
                    kind: BodyKind::Planet,
                },
            ],
        },
        // Level 4 — black hole funnel
        LevelConfig {
            index: 3,
            name: "Event Horizon".into(),
            ship_pos: Vec2::new(-500.0, 180.0),
            ship_vel: Vec2::new(200.0, -60.0),
            target_pos: Vec2::new(480.0, 180.0),
            max_placed: 3,
            bodies: vec![BodyDef {
                pos: Vec2::new(0.0, -80.0),
                mass: 12000.0,
                radius: 22.0,
                kind: BodyKind::BlackHole,
            }],
        },
        // Level 5 — maze of gravity wells
        LevelConfig {
            index: 4,
            name: "Asteroid Field".into(),
            ship_pos: Vec2::new(-520.0, 0.0),
            ship_vel: Vec2::new(220.0, 0.0),
            target_pos: Vec2::new(520.0, 0.0),
            max_placed: 4,
            bodies: vec![
                BodyDef {
                    pos: Vec2::new(-200.0, 180.0),
                    mass: 2500.0,
                    radius: 22.0,
                    kind: BodyKind::Planet,
                },
                BodyDef {
                    pos: Vec2::new(0.0, -180.0),
                    mass: 2500.0,
                    radius: 22.0,
                    kind: BodyKind::Planet,
                },
                BodyDef {
                    pos: Vec2::new(200.0, 180.0),
                    mass: 2500.0,
                    radius: 22.0,
                    kind: BodyKind::Planet,
                },
            ],
        },
    ]
}

pub fn spawn_level_entities(
    commands: &mut Commands,
    level: &LevelConfig,
    _game: &GameState,
) {
    // Ship
    commands.spawn((
        Transform::from_translation(level.ship_pos.extend(0.0)),
        Visibility::default(),
        Ship,
        Velocity(level.ship_vel),
    ));

    // Target zone
    commands.spawn((
        Transform::from_translation(level.target_pos.extend(0.0)),
        Visibility::default(),
        Target,
    ));

    // Fixed gravity bodies
    for body in &level.bodies {
        commands.spawn((
            Transform::from_translation(body.pos.extend(0.0)),
            Visibility::default(),
            GravityBody {
                mass: body.mass,
                radius: body.radius,
                kind: body.kind,
            },
        ));
    }
}
