mod components;
mod levels;
mod physics;
mod simulation;
mod ui;

use bevy::prelude::*;
use components::*;
use simulation::*;
use ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Orbital Sandbox".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.12)))
        .insert_resource(GameState::default())
        .insert_resource(LevelConfig::default())
        .add_systems(Startup, (setup_camera, setup_ui, spawn_level))
        .add_systems(
            Update,
            (
                step_physics.run_if(is_simulating),
                check_win_fail.run_if(is_simulating),
                draw_trail,
                handle_planet_placement,
                handle_launch,
                handle_reset,
                update_ui_state,
                handle_next_level,
                draw_bodies,
                draw_target,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn is_simulating(state: Res<GameState>) -> bool {
    state.simulating
}

pub fn spawn_level(
    mut commands: Commands,
    level: Res<LevelConfig>,
    game: Res<GameState>,
) {
    levels::spawn_level_entities(&mut commands, &level, &game);
}

fn draw_bodies(
    mut gizmos: Gizmos,
    bodies: Query<(&Transform, &GravityBody)>,
    ship: Query<&Transform, With<Ship>>,
) {
    for (transform, body) in &bodies {
        let pos = transform.translation.truncate();
        let color = match body.kind {
            BodyKind::Planet => Color::srgb(0.3, 0.6, 1.0),
            BodyKind::Star => Color::srgb(1.0, 0.85, 0.2),
            BodyKind::BlackHole => Color::srgb(0.6, 0.0, 0.9),
            BodyKind::Placed => Color::srgb(0.2, 0.9, 0.4),
        };
        gizmos.circle_2d(pos, body.radius, color);
        if body.radius > 8.0 {
            gizmos.circle_2d(pos, body.radius * 0.6, color.with_alpha(0.4));
        }
    }

    for transform in &ship {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 6.0, Color::srgb(1.0, 0.4, 0.2));
        gizmos.circle_2d(pos, 3.0, Color::WHITE);
    }
}

fn draw_target(mut gizmos: Gizmos, targets: Query<&Transform, With<Target>>) {
    for transform in &targets {
        let pos = transform.translation.truncate();
        gizmos.circle_2d(pos, 20.0, Color::srgb(0.2, 1.0, 0.5));
        gizmos.circle_2d(pos, 13.0, Color::srgb(0.2, 1.0, 0.5).with_alpha(0.5));
        gizmos.circle_2d(pos, 5.0, Color::srgb(0.2, 1.0, 0.5));
    }
}
