use bevy::{prelude::*, window::PrimaryWindow};

use crate::components::*;
use crate::levels::LEVELS;

const SIDEBAR_W: f32 = 180.0;
const BTN_H: f32 = 40.0;
const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.18, 0.92);
const BTN_NORMAL: Color = Color::srgb(0.15, 0.18, 0.32);
const BTN_ACTIVE: Color = Color::srgb(0.1, 0.5, 0.8);

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(SIDEBAR_W),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(PANEL_BG),
            ))
            .with_children(|sidebar| {
                sidebar.spawn((
                    Text::new("ORBITAL\nSANDBOX"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.85, 1.0)),
                ));

                sidebar.spawn((
                    Text::new("Level 1: Gravity Assist"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.7, 0.9)),
                    LevelText,
                ));

                separator(sidebar);

                sidebar.spawn((
                    Text::new("PLACE BODIES"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.6, 0.8)),
                ));

                tool_button(sidebar, "Planet", PlaceTool::Planet, BTN_ACTIVE);
                tool_button(sidebar, "Star", PlaceTool::Star, BTN_NORMAL);
                tool_button(sidebar, "Black Hole", PlaceTool::BlackHole, BTN_NORMAL);
                tool_button(sidebar, "Erase", PlaceTool::Erase, BTN_NORMAL);

                separator(sidebar);

                sidebar.spawn((
                    Text::new("Placed: 0 / ?"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.8, 0.6)),
                    PlacedCountText,
                ));

                separator(sidebar);

                action_button(sidebar, "Launch", LaunchButton, BTN_NORMAL);
                action_button(sidebar, "Reset", ResetButton, BTN_NORMAL);

                separator(sidebar);

                sidebar.spawn((
                    Text::new("Place bodies\nthen Launch"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::WHITE),
                    StatusText,
                ));

                sidebar
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(BTN_H),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.6, 0.2)),
                        NextLevelButton,
                    ))
                    .with_child((
                        Text::new("Next Level"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
            });
        });
}

fn separator(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.4, 0.4, 0.6, 0.3)),
    ));
}

fn tool_button(parent: &mut ChildBuilder, label: &str, tool: PlaceTool, bg: Color) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            ToolButton(tool),
        ))
        .with_child((
            Text::new(label.to_string()),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::WHITE),
        ));
}

fn action_button<M: Component>(parent: &mut ChildBuilder, label: &str, marker: M, bg: Color) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            marker,
        ))
        .with_child((
            Text::new(label.to_string()),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
}

// ── Tool selection & body placement ──────────────────────────────────────────

pub fn handle_planet_placement(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut game: ResMut<GameState>,
    level: Res<LevelConfig>,
    placed_bodies: Query<(Entity, &Transform), With<PlacedBody>>,
    tool_buttons: Query<(&Interaction, &ToolButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, tool_btn) in &tool_buttons {
        if *interaction == Interaction::Pressed {
            game.active_tool = tool_btn.0;
        }
    }

    if game.simulating {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_tf)) = camera_q.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_tf, cursor_pos) else { return };

    if cursor_pos.x < SIDEBAR_W {
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        match game.active_tool {
            PlaceTool::Erase => {
                for (entity, tf) in &placed_bodies {
                    if tf.translation.truncate().distance(world_pos) < 36.0 {
                        commands.entity(entity).despawn();
                        game.placed_count = game.placed_count.saturating_sub(1);
                        break;
                    }
                }
            }
            tool => {
                if game.placed_count >= level.max_placed {
                    return;
                }
                let (mass, radius, kind) = tool_stats(tool);
                commands.spawn((
                    Transform::from_translation(world_pos.extend(0.0)),
                    Visibility::default(),
                    GravityBody { mass, radius, kind },
                    PlacedBody,
                ));
                game.placed_count += 1;
            }
        }
    }
}

fn tool_stats(tool: PlaceTool) -> (f32, f32, BodyKind) {
    match tool {
        PlaceTool::Planet => (3000.0, 24.0, BodyKind::Placed),
        PlaceTool::Star => (8000.0, 36.0, BodyKind::Star),
        PlaceTool::BlackHole => (14000.0, 18.0, BodyKind::BlackHole),
        PlaceTool::Erase => unreachable!(),
    }
}

// ── Launch / Reset / Next Level ───────────────────────────────────────────────

pub fn handle_launch(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LaunchButton>)>,
    mut game: ResMut<GameState>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed && !game.simulating && game.outcome == Outcome::Playing {
            game.simulating = true;
        }
    }
}

pub fn handle_reset(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ResetButton>)>,
    mut game: ResMut<GameState>,
    mut commands: Commands,
    ship_q: Query<Entity, With<Ship>>,
    target_q: Query<Entity, With<Target>>,
    bodies_q: Query<Entity, With<GravityBody>>,
    trail_q: Query<Entity, With<TrailDot>>,
    level: Res<LevelConfig>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            do_reset(&mut commands, &mut game, &ship_q, &target_q, &bodies_q, &trail_q, &level);
        }
    }
}

pub fn handle_next_level(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<NextLevelButton>)>,
    mut game: ResMut<GameState>,
    mut level: ResMut<LevelConfig>,
    mut commands: Commands,
    ship_q: Query<Entity, With<Ship>>,
    target_q: Query<Entity, With<Target>>,
    bodies_q: Query<Entity, With<GravityBody>>,
    trail_q: Query<Entity, With<TrailDot>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            let next = game.current_level + 1;
            if next < LEVELS.len() {
                *level = LEVELS[next].clone();
                game.current_level = next;
                let level_snapshot = level.clone();
                do_reset(&mut commands, &mut game, &ship_q, &target_q, &bodies_q, &trail_q, &level_snapshot);
            }
        }
    }
}

fn do_reset(
    commands: &mut Commands,
    game: &mut GameState,
    ship_q: &Query<Entity, With<Ship>>,
    target_q: &Query<Entity, With<Target>>,
    bodies_q: &Query<Entity, With<GravityBody>>,
    trail_q: &Query<Entity, With<TrailDot>>,
    level: &LevelConfig,
) {
    for e in ship_q { commands.entity(e).despawn(); }
    for e in target_q { commands.entity(e).despawn(); }
    for e in bodies_q { commands.entity(e).despawn(); }
    for e in trail_q { commands.entity(e).despawn(); }

    game.simulating = false;
    game.outcome = Outcome::Playing;
    game.placed_count = 0;

    crate::levels::spawn_level_entities(commands, level, game);
}

// ── UI state sync ─────────────────────────────────────────────────────────────

pub fn update_ui_state(
    game: Res<GameState>,
    level: Res<LevelConfig>,
    mut status_q: Query<
        &mut Text,
        (With<StatusText>, Without<LevelText>, Without<PlacedCountText>),
    >,
    mut level_q: Query<
        &mut Text,
        (With<LevelText>, Without<StatusText>, Without<PlacedCountText>),
    >,
    mut count_q: Query<
        &mut Text,
        (With<PlacedCountText>, Without<StatusText>, Without<LevelText>),
    >,
    mut next_btn_q: Query<&mut Node, With<NextLevelButton>>,
    mut tool_btns: Query<(&mut BackgroundColor, &ToolButton)>,
    mut launch_q: Query<&mut BackgroundColor, (With<LaunchButton>, Without<ToolButton>)>,
) {
    if let Ok(mut text) = status_q.get_single_mut() {
        text.0 = match game.outcome {
            Outcome::Playing => {
                if game.simulating {
                    "Simulating...".into()
                } else {
                    "Place bodies\nthen Launch".into()
                }
            }
            Outcome::Win => "SUCCESS!\nReached target!".into(),
            Outcome::Fail => "FAILED!\nReset & retry.".into(),
        };
    }

    if let Ok(mut text) = level_q.get_single_mut() {
        text.0 = format!("Level {}: {}", level.index + 1, level.name);
    }

    if let Ok(mut text) = count_q.get_single_mut() {
        text.0 = format!("Placed: {} / {}", game.placed_count, level.max_placed);
    }

    if let Ok(mut node) = next_btn_q.get_single_mut() {
        node.display = if game.outcome == Outcome::Win && game.current_level + 1 < LEVELS.len() {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (mut bg, tool_btn) in &mut tool_btns {
        bg.0 = if tool_btn.0 == game.active_tool {
            BTN_ACTIVE
        } else {
            BTN_NORMAL
        };
    }

    if let Ok(mut bg) = launch_q.get_single_mut() {
        bg.0 = if game.simulating {
            Color::srgb(0.2, 0.2, 0.2)
        } else {
            BTN_NORMAL
        };
    }
}
