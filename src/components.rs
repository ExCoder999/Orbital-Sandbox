use bevy::prelude::*;

// ── Markers ──────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Target;

#[derive(Component)]
pub struct PlacedBody;

#[derive(Component)]
pub struct TrailDot {
    pub lifetime: f32,
}

// ── Body ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum BodyKind {
    Planet,
    Star,
    BlackHole,
    Placed,
}

#[derive(Component)]
pub struct GravityBody {
    pub mass: f32,
    pub radius: f32,
    pub kind: BodyKind,
}

// ── Physics state ─────────────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

// ── UI markers ───────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct LaunchButton;

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct PlacedCountText;

#[derive(Component)]
pub struct ToolButton(pub PlaceTool);

// ── Global resources ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlaceTool {
    Planet,
    Star,
    BlackHole,
    Erase,
}

#[derive(Resource)]
pub struct GameState {
    pub simulating: bool,
    pub outcome: Outcome,
    pub current_level: usize,
    pub placed_count: usize,
    pub active_tool: PlaceTool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            simulating: false,
            outcome: Outcome::Playing,
            current_level: 0,
            placed_count: 0,
            active_tool: PlaceTool::Planet,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Outcome {
    Playing,
    Win,
    Fail,
}

// ── Level data ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BodyDef {
    pub pos: Vec2,
    pub mass: f32,
    pub radius: f32,
    pub kind: BodyKind,
}

#[derive(Resource, Clone)]
pub struct LevelConfig {
    pub index: usize,
    pub name: String,
    pub ship_pos: Vec2,
    pub ship_vel: Vec2,
    pub target_pos: Vec2,
    pub max_placed: usize,
    pub bodies: Vec<BodyDef>,
}

impl Default for LevelConfig {
    fn default() -> Self {
        crate::levels::LEVELS[0].clone()
    }
}
