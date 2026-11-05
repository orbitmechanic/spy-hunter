//! Core simulation: road, player, traffic, weapons, hazards and scoring.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::AppState;
use crate::art::{Art, sized};
use crate::audio::{Muted, Sfx};

pub const WORLD_W: f32 = 960.0;
pub const WORLD_H: f32 = 720.0;
pub const ROAD_W: f32 = 560.0;
pub const SEG_H: f32 = 40.0;
pub const SEG_COUNT: usize = 20;
pub const PLAYER_Y: f32 = -240.0;
pub const LANES: [f32; 4] = [-210.0, -70.0, 70.0, 210.0];

// ---------------------------------------------------------------------------
// Terrain
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Terrain {
    #[default]
    Highway,
    Snow,
    Water,
    Night,
}

impl Terrain {
    pub const ALL: [Terrain; 4] = [
        Terrain::Highway,
        Terrain::Snow,
        Terrain::Water,
        Terrain::Night,
    ];

    pub fn index(self) -> usize {
        match self {
            Terrain::Highway => 0,
            Terrain::Snow => 1,
            Terrain::Water => 2,
            Terrain::Night => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Terrain::Highway => "COAST HIGHWAY",
            Terrain::Snow => "ALPINE PASS",
            Terrain::Water => "DELTA WATERWAY",
            Terrain::Night => "NIGHT CITY",
        }
    }

    pub fn next(self) -> Terrain {
        match self {
            Terrain::Highway => Terrain::Snow,
            Terrain::Snow => Terrain::Water,
            Terrain::Water => Terrain::Night,
            Terrain::Night => Terrain::Highway,
        }
    }

    pub fn is_water(self) -> bool {
        matches!(self, Terrain::Water)
    }

    pub fn road_color(self) -> Color {
        match self {
            Terrain::Highway => Color::srgb(0.22, 0.23, 0.26),
            Terrain::Snow => Color::srgb(0.38, 0.42, 0.47),
            Terrain::Water => Color::srgb(0.10, 0.34, 0.51),
            Terrain::Night => Color::srgb(0.12, 0.13, 0.17),
        }
    }

    pub fn edge_color(self) -> Color {
        match self {
            Terrain::Highway => Color::srgb(0.90, 0.88, 0.82),
            Terrain::Snow => Color::srgb(0.96, 0.99, 1.0),
            Terrain::Water => Color::srgb(0.78, 0.91, 0.97),
            Terrain::Night => Color::srgb(0.86, 0.78, 0.45),
        }
    }

    pub fn lane_color(self) -> Color {
        match self {
            Terrain::Highway => Color::srgb(0.85, 0.77, 0.42),
            Terrain::Snow => Color::srgb(0.98, 0.86, 0.36),
            Terrain::Water => Color::srgb(0.52, 0.82, 0.93),
            Terrain::Night => Color::srgb(0.87, 0.80, 0.48),
        }
    }
}

/// World-space road centre as a function of segment y and distance travelled.
pub fn road_center(y: f32, scroll: f32) -> f32 {
    (0.0036 * y + 0.0010 * scroll).sin() * 44.0 + (0.00042 * scroll + 0.7).sin() * 16.0
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RunPhase {
    #[default]
    Running,
    Dying,
}

#[derive(Resource)]
pub struct Game {
    pub score: u64,
    pub score_frac: f32,
    pub hi: u64,
    pub lives: u32,
    pub speed: f32,
    pub target_speed: f32,
    pub distance: f32,
    pub elapsed: f32,
    pub terrain: Terrain,
    pub terrain_timer: f32,
    pub spawn_timer: f32,
    pub heli_timer: f32,
    pub van_timer: f32,
    pub van_ready: bool,
    pub score_freeze: f32,
    pub offroad: f32,
    pub banner: String,
    pub banner_timer: f32,
    pub phase: RunPhase,
    pub death_timer: f32,
    pub next_bonus: u64,
    pub destroyed: u32,
    pub civilians_hit: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            score: 0,
            score_frac: 0.0,
            hi: 0,
            lives: 3,
            speed: 260.0,
            target_speed: 260.0,
            distance: 0.0,
            elapsed: 0.0,
            terrain: Terrain::Highway,
            terrain_timer: 48.0,
            spawn_timer: 0.8,
            heli_timer: 11.0,
            van_timer: 18.0,
            van_ready: false,
            score_freeze: 0.0,
            offroad: 0.0,
            banner: String::new(),
            banner_timer: 0.0,
            phase: RunPhase::Running,
            death_timer: 0.0,
            next_bonus: 15_000,
            destroyed: 0,
            civilians_hit: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Weapons {
    pub oil: u32,
    pub smoke: u32,
    pub missiles: u32,
    pub gun_timer: f32,
    pub van_ready: bool,
    pub van_timer: f32,
}

#[derive(Resource, Default)]
pub struct Road {
    pub scroll: f32,
}

#[derive(Resource, Default)]
pub struct Fx {
    pub shake: f32,
}

#[derive(Resource)]
pub struct Rng(pub u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0x9e37_79b9_7f4a_7c15)
    }
}

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 32) as u32
    }

    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    pub fn pick(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            self.next_u32() as usize % n
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct Player {
    pub invuln: f32,
}

#[derive(Component)]
pub struct TitleCar;

#[derive(Component)]
pub struct Backdrop;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoadPartKind {
    Asphalt,
    EdgeLeft,
    EdgeRight,
    Dash,
}

#[derive(Component)]
pub struct RoadPart {
    pub kind: RoadPartKind,
    pub index: usize,
    pub lane: f32,
}

#[derive(Component)]
pub struct Scenery {
    pub index: usize,
    pub side: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    TrafficRed,
    TrafficBlue,
    Switchblade,
    Civilian,
    Boat,
    Pursuer,
    Helicopter,
}

impl EnemyKind {
    pub fn score(self) -> u64 {
        match self {
            EnemyKind::TrafficRed | EnemyKind::TrafficBlue => 150,
            EnemyKind::Switchblade => 300,
            EnemyKind::Civilian => 0,
            EnemyKind::Boat | EnemyKind::Pursuer => 250,
            EnemyKind::Helicopter => 1_000,
        }
    }

    pub fn is_civilian(self) -> bool {
        matches!(self, EnemyKind::Civilian)
    }

    pub fn size(self) -> Vec2 {
        match self {
            EnemyKind::Helicopter => Vec2::new(84.0, 62.0),
            _ => Vec2::new(42.0, 66.0),
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub speed: f32,
    pub fire: f32,
    pub phase: f32,
    pub from_behind: bool,
    pub dead: bool,
    pub hover_y: f32,
}

#[derive(Component)]
pub struct Van;

#[derive(Component)]
pub struct Bullet {
    pub friendly: bool,
    pub vel: Vec2,
    pub life: f32,
}

#[derive(Component)]
pub struct Missile {
    pub target: Option<Entity>,
    pub life: f32,
}

#[derive(Component)]
pub struct Bomb {
    pub vel: Vec2,
    pub life: f32,
}

#[derive(Component)]
pub struct Crater {
    pub life: f32,
}

#[derive(Component)]
pub struct Slick {
    pub life: f32,
    pub flame: bool,
}

#[derive(Component)]
pub struct SmokeCloud {
    pub life: f32,
    pub max_life: f32,
}

#[derive(Component)]
pub struct Explosion {
    pub timer: f32,
    pub frame: usize,
}

#[derive(Component)]
pub struct Particle {
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub base: Color,
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn setup_world(mut commands: Commands, art: Res<Art>, rng: Res<Rng>) {
    commands.spawn((
        Sprite {
            image: art.terrain[0].clone(),
            custom_size: Some(Vec2::new(WORLD_W, WORLD_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -30.0),
        Backdrop,
    ));

    // Road segments: a 560 px carriageway with four lanes.
    for i in 0..SEG_COUNT {
        for kind in [
            RoadPartKind::Asphalt,
            RoadPartKind::EdgeLeft,
            RoadPartKind::EdgeRight,
        ] {
            let (w, h) = match kind {
                RoadPartKind::Asphalt => (ROAD_W, SEG_H),
                _ => (6.0, SEG_H),
            };
            commands.spawn((
                Sprite::from_color(Terrain::Highway.road_color(), Vec2::new(w, h)),
                Transform::from_xyz(0.0, 0.0, -10.0),
                RoadPart {
                    kind,
                    index: i,
                    lane: 0.0,
                },
            ));
        }
        if i % 2 == 0 {
            for lane in [-140.0, 0.0, 140.0] {
                commands.spawn((
                    Sprite::from_color(Terrain::Highway.lane_color(), Vec2::new(7.0, 22.0)),
                    Transform::from_xyz(0.0, 0.0, -9.0),
                    RoadPart {
                        kind: RoadPartKind::Dash,
                        index: i,
                        lane,
                    },
                ));
            }
        }
    }

    // Roadside scenery. Recycled vertically forever.
    let mut local_rng = Rng(rng.0 ^ 0x51ed_2701);
    for index in 0..18 {
        let side = if index % 2 == 0 { -1.0 } else { 1.0 };
        let y = local_rng.range(-380.0, 380.0);
        let x = road_center(y, 0.0) + side * local_rng.range(340.0, 520.0);
        commands.spawn((
            sized(art.tree.clone(), 48.0, 64.0),
            Transform::from_xyz(x, y, -20.0),
            Scenery {
                index,
                side,
                speed: local_rng.range(0.85, 1.15),
            },
        ));
    }

    // The car idling on the title screen.
    commands.spawn((
        sized(art.player_car.clone(), 48.0, 72.0),
        Transform::from_xyz(road_center(PLAYER_Y, 0.0), PLAYER_Y, 10.0),
        TitleCar,
    ));
}

pub fn start_run(
    mut game: ResMut<Game>,
    mut weapons: ResMut<Weapons>,
    mut road: ResMut<Road>,
    mut fx: ResMut<Fx>,
    capture: Res<crate::CaptureConfig>,
) {
    *game = Game::default();
    if let Some(terrain) = capture.terrain {
        game.terrain = terrain;
    }
    if capture.gameover {
        game.score = 12_340;
        game.hi = 54_320;
        game.lives = 0;
    }
    *weapons = Weapons::default();
    weapons.van_timer = game.van_timer;
    road.scroll = 0.0;
    fx.shake = 0.0;
}

pub fn spawn_player(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    road: Res<Road>,
    capture: Res<crate::CaptureConfig>,
) {
    if capture.gameover || game.lives == 0 {
        return;
    }
    let image = if game.terrain.is_water() {
        art.player_boat.clone()
    } else {
        art.player_car.clone()
    };
    commands.spawn((
        sized(image, 48.0, 72.0),
        Transform::from_xyz(road_center(PLAYER_Y, road.scroll), PLAYER_Y, 10.0),
        Player { invuln: 2.4 },
        GameEntity,
    ));
}

pub fn cleanup_run(mut commands: Commands, entities: Query<Entity, With<GameEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Input and state
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct Actions {
    pub left: bool,
    pub right: bool,
    pub throttle: bool,
    pub brake: bool,
    pub fire: bool,
    pub start: bool,
    pub pause: bool,
    pub mute: bool,
    pub oil: bool,
    pub smoke: bool,
    pub missile: bool,
    pub van: bool,
    pub restart: bool,
}

pub fn update_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut actions: ResMut<Actions>,
    mut muted: ResMut<Muted>,
) {
    actions.left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
    actions.right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);
    actions.throttle = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    actions.brake = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    actions.fire = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyJ);
    actions.start = keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left);
    actions.pause = keys.just_pressed(KeyCode::KeyP);
    actions.mute = keys.just_pressed(KeyCode::KeyM);
    actions.oil = keys.just_pressed(KeyCode::Digit1) || keys.just_pressed(KeyCode::KeyU);
    actions.smoke = keys.just_pressed(KeyCode::Digit2) || keys.just_pressed(KeyCode::KeyI);
    actions.missile = keys.just_pressed(KeyCode::Digit3) || keys.just_pressed(KeyCode::KeyO);
    actions.van = keys.just_pressed(KeyCode::KeyV);
    actions.restart = keys.just_pressed(KeyCode::KeyR);

    if actions.mute {
        muted.0 = !muted.0;
    }
}

pub fn state_input(
    actions: Res<Actions>,
    state: Res<State<AppState>>,
    mut paused: ResMut<crate::Paused>,
    mut next: ResMut<NextState<AppState>>,
    mut sfx: MessageWriter<Sfx>,
) {
    match state.get() {
        AppState::Title => {
            if actions.start {
                sfx.write(Sfx::Ui);
                next.set(AppState::InGame);
            }
        }
        AppState::InGame => {
            if actions.pause {
                paused.0 = !paused.0;
                sfx.write(Sfx::Ui);
            }
        }
        AppState::GameOver => {
            if actions.start || actions.restart {
                sfx.write(Sfx::Ui);
                next.set(AppState::InGame);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Road and scenery
// ---------------------------------------------------------------------------

pub fn scroll_world(
    time: Res<Time>,
    state: Res<State<AppState>>,
    paused: Res<crate::Paused>,
    game: Res<Game>,
    mut road: ResMut<Road>,
) {
    let dt = time.delta_secs().min(0.05);
    let speed = match state.get() {
        AppState::InGame if !paused.0 && game.phase == RunPhase::Running => game.speed,
        AppState::InGame => 0.0,
        _ => 150.0,
    };
    road.scroll += speed * dt;
}

pub fn sync_road(road: Res<Road>, mut parts: Query<(&RoadPart, &mut Transform, &mut Sprite)>) {
    let total = SEG_COUNT as f32 * SEG_H;
    for (part, mut tf, mut sprite) in &mut parts {
        let raw = (part.index as f32 * SEG_H - road.scroll).rem_euclid(total);
        let y = -400.0 + raw;
        let center = road_center(y, road.scroll);
        let offset = match part.kind {
            RoadPartKind::Asphalt => 0.0,
            RoadPartKind::EdgeLeft => -ROAD_W * 0.5,
            RoadPartKind::EdgeRight => ROAD_W * 0.5,
            RoadPartKind::Dash => part.lane,
        };
        tf.translation.x = center + offset;
        tf.translation.y = y;
        if part.kind == RoadPartKind::Asphalt {
            tf.translation.z = -10.0;
        } else {
            tf.translation.z = -9.0;
        }
        // Keep the lane dashes crisp even when the segment is recycled.
        if part.kind == RoadPartKind::Dash {
            sprite.custom_size = Some(Vec2::new(7.0, 22.0));
        }
    }
}

pub fn sync_terrain_art(
    mut last: Local<Option<Terrain>>,
    game: Res<Game>,
    art: Res<Art>,
    mut backdrop: Query<&mut Sprite, (With<Backdrop>, Without<RoadPart>, Without<Scenery>)>,
    mut parts: Query<&mut Sprite, (With<RoadPart>, Without<Backdrop>, Without<Scenery>)>,
    mut scenery: Query<(&Scenery, &mut Sprite), (Without<Backdrop>, Without<RoadPart>)>,
) {
    if *last == Some(game.terrain) {
        return;
    }
    *last = Some(game.terrain);
    if let Ok(mut sprite) = backdrop.single_mut() {
        sprite.image = art.terrain[game.terrain.index()].clone();
    }
    for mut sprite in &mut parts {
        // Recolouring is cheap and keeps water/snow/night readable.
        sprite.color = if sprite.custom_size == Some(Vec2::new(7.0, 22.0)) {
            game.terrain.lane_color()
        } else if sprite.custom_size.map(|s| s.x).unwrap_or(0.0) < 10.0 {
            game.terrain.edge_color()
        } else {
            game.terrain.road_color()
        };
    }
    for (item, mut sprite) in &mut scenery {
        let (image, w, h) = match game.terrain {
            Terrain::Highway => {
                if item.index % 3 == 0 {
                    (art.rock.clone(), 48.0, 32.0)
                } else {
                    (art.tree.clone(), 48.0, 64.0)
                }
            }
            Terrain::Snow => {
                if item.index % 4 == 0 {
                    (art.rock.clone(), 48.0, 32.0)
                } else {
                    (art.snow_pine.clone(), 48.0, 64.0)
                }
            }
            Terrain::Water => (art.buoy.clone(), 24.0, 32.0),
            Terrain::Night => (art.building.clone(), 64.0, 96.0),
        };
        sprite.image = image;
        sprite.custom_size = Some(Vec2::new(w, h));
    }
}

pub fn move_scenery(
    time: Res<Time>,
    state: Res<State<AppState>>,
    paused: Res<crate::Paused>,
    game: Res<Game>,
    road: Res<Road>,
    mut rng: ResMut<Rng>,
    mut scenery: Query<(&mut Scenery, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    let speed = match state.get() {
        AppState::InGame if !paused.0 && game.phase == RunPhase::Running => game.speed,
        AppState::InGame => 0.0,
        _ => 150.0,
    };
    for (mut item, mut tf) in &mut scenery {
        tf.translation.y += speed * item.speed * dt;
        if tf.translation.y > 420.0 {
            tf.translation.y = -420.0;
            let side = if rng.chance(0.5) { -1.0 } else { 1.0 };
            item.side = side;
            tf.translation.x =
                road_center(tf.translation.y, road.scroll) + side * rng.range(340.0, 520.0);
        }
        // Keep scenery beside the curving carriageway.
        let desired = road_center(tf.translation.y, road.scroll) + item.side * 410.0;
        tf.translation.x += (desired - tf.translation.x) * (dt * 2.5).min(1.0);
    }
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn player_control(
    time: Res<Time>,
    actions: Res<Actions>,
    mut game: ResMut<Game>,
    road: Res<Road>,
    mut player: Query<(&mut Player, &mut Transform, &mut Sprite)>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
) {
    let dt = time.delta_secs().min(0.05);
    let Ok((mut p, mut tf, mut sprite)) = player.single_mut() else {
        return;
    };
    p.invuln = (p.invuln - dt).max(0.0);

    // Terrain-appropriate chassis.
    // (The image swap is handled in `sync_player_art`; this keeps the transform.)
    let center = road_center(PLAYER_Y, road.scroll);
    let steer = (actions.right as i32 - actions.left as i32) as f32;
    tf.translation.x += steer * 430.0 * dt;
    tf.translation.x = tf.translation.x.clamp(center - 350.0, center + 350.0);
    tf.translation.y = PLAYER_Y + (tf.translation.y - PLAYER_Y) * 0.9;

    let offroad = (tf.translation.x - center).abs() > ROAD_W * 0.5 - 20.0;
    if offroad {
        game.offroad += dt;
        game.target_speed = 120.0;
        if game.offroad > 2.1 {
            game.offroad = 0.0;
            crate::game::damage_player_public(
                &mut commands,
                &mut game,
                &mut sfx,
                tf.translation.truncate(),
            );
        }
    } else {
        game.offroad = (game.offroad - dt * 1.8).max(0.0);
        game.target_speed = if actions.throttle {
            420.0
        } else if actions.brake {
            150.0
        } else {
            260.0
        };
    }
    game.speed += (game.target_speed - game.speed) * (dt * 2.4).min(1.0);
    game.speed = game.speed.clamp(80.0, 460.0);

    // A subtle lean while steering.
    let target_roll = steer * -0.10;
    tf.rotation = Quat::from_rotation_z(
        tf.rotation.to_euler(EulerRot::XYZ).2
            + (target_roll - tf.rotation.to_euler(EulerRot::XYZ).2) * (dt * 10.0).min(1.0),
    );
    if p.invuln > 0.0 {
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.55 + 0.45 * (p.invuln * 12.0).sin().abs());
    } else {
        sprite.color = Color::WHITE;
    }
    if offroad && p.invuln <= 0.0 && game.phase == RunPhase::Running {
        game.banner = "OFF ROAD — GET BACK ON THE PAVEMENT".into();
        game.banner_timer = game.banner_timer.max(0.2);
    }
}

pub fn sync_player_art(
    game: Res<Game>,
    art: Res<Art>,
    mut player: Query<&mut Sprite, With<Player>>,
) {
    let Ok(mut sprite) = player.single_mut() else {
        return;
    };
    let wanted = if game.terrain.is_water() {
        art.player_boat.clone()
    } else {
        art.player_car.clone()
    };
    if sprite.image != wanted {
        sprite.image = wanted;
    }
}

// ---------------------------------------------------------------------------
// Enemy spawning & movement
// ---------------------------------------------------------------------------

fn enemy_image(art: &Art, kind: EnemyKind) -> Handle<Image> {
    match kind {
        EnemyKind::TrafficRed => art.enemy_red.clone(),
        EnemyKind::TrafficBlue => art.enemy_blue.clone(),
        EnemyKind::Switchblade => art.enemy_black.clone(),
        EnemyKind::Civilian => art.civilian.clone(),
        EnemyKind::Boat | EnemyKind::Pursuer => art.enemy_boat.clone(),
        EnemyKind::Helicopter => art.helicopter.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_enemy_entity(
    commands: &mut Commands,
    art: &Art,
    road: &Road,
    rng: &mut Rng,
    kind: EnemyKind,
    ahead: bool,
) {
    let from_behind = matches!(kind, EnemyKind::Pursuer)
        || (ahead && kind == EnemyKind::Boat && rng.chance(0.18));
    let y = if from_behind { -430.0 } else { 400.0 };
    let lane = rng.pick(4);
    let x = road_center(y, road.scroll) + LANES[lane] + rng.range(-12.0, 12.0);
    let speed = match kind {
        EnemyKind::TrafficRed | EnemyKind::TrafficBlue => rng.range(120.0, 200.0),
        EnemyKind::Switchblade => rng.range(150.0, 215.0),
        EnemyKind::Civilian => rng.range(95.0, 160.0),
        EnemyKind::Boat => rng.range(130.0, 210.0),
        EnemyKind::Pursuer => rng.range(380.0, 470.0),
        EnemyKind::Helicopter => 0.0,
    };
    let image = enemy_image(art, kind);
    let size = kind.size();
    let hover_y = if kind == EnemyKind::Helicopter {
        rng.range(120.0, 210.0)
    } else {
        0.0
    };
    commands.spawn((
        sized(image, size.x, size.y),
        Transform::from_xyz(x, y, 12.0),
        Enemy {
            kind,
            speed,
            fire: rng.range(1.2, 2.6),
            phase: rng.range(0.0, 6.28),
            from_behind,
            dead: false,
            hover_y,
        },
        GameEntity,
    ));
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut weapons: ResMut<Weapons>,
    road: Res<Road>,
    art: Res<Art>,
    mut rng: ResMut<Rng>,
    enemies: Query<&Enemy>,
) {
    let dt = time.delta_secs().min(0.05);
    if game.phase != RunPhase::Running {
        return;
    }
    game.elapsed += dt;
    game.spawn_timer -= dt;
    game.heli_timer -= dt;
    weapons.van_timer -= dt;
    let alive = enemies.iter().filter(|e| !e.dead).count();

    // Difficulty ramps every 30 s, capped so the road stays navigable.
    let pressure = (game.elapsed / 30.0).min(4.0);
    if game.spawn_timer <= 0.0 && alive < 9 {
        game.spawn_timer = (rng.range(0.75, 1.35) - pressure * 0.08).max(0.30);
        let roll = rng.unit();
        let kind = if game.terrain.is_water() {
            if roll < 0.26 {
                EnemyKind::Pursuer
            } else {
                EnemyKind::Boat
            }
        } else if roll < 0.16 {
            EnemyKind::Civilian
        } else if roll < 0.40 {
            EnemyKind::Switchblade
        } else if roll < 0.70 {
            EnemyKind::TrafficRed
        } else {
            EnemyKind::TrafficBlue
        };
        spawn_enemy_entity(&mut commands, &art, &road, &mut rng, kind, true);
    }

    if game.heli_timer <= 0.0 && !game.terrain.is_water() {
        game.heli_timer = rng.range(13.0, 18.0);
        if !enemies
            .iter()
            .any(|e| e.kind == EnemyKind::Helicopter && !e.dead)
        {
            spawn_enemy_entity(
                &mut commands,
                &art,
                &road,
                &mut rng,
                EnemyKind::Helicopter,
                true,
            );
        }
    }

    if weapons.van_timer < 4.0 && weapons.van_timer > 0.0 {
        weapons.van_ready = true;
    }
    if weapons.van_timer <= 0.0 {
        weapons.van_timer = rng.range(20.0, 26.0);
        weapons.van_ready = false;
    }
    game.van_timer = weapons.van_timer;
    game.van_ready = weapons.van_ready;
}

pub fn summon_van(
    actions: Res<Actions>,
    mut weapons: ResMut<Weapons>,
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    road: Res<Road>,
    vans: Query<&Van>,
    mut sfx: MessageWriter<Sfx>,
) {
    if !actions.van || !weapons.van_ready || !vans.is_empty() {
        return;
    }
    weapons.van_ready = false;
    weapons.van_timer = 22.0;
    let x = road_center(390.0, road.scroll) + LANES[1];
    commands.spawn((
        sized(art.van.clone(), 64.0, 96.0),
        Transform::from_xyz(x, 390.0, 11.0),
        Van,
        GameEntity,
    ));
    sfx.write(Sfx::Ui);
    let _ = game;
}

#[allow(clippy::too_many_arguments)]
pub fn move_enemies(
    time: Res<Time>,
    game: Res<Game>,
    road: Res<Road>,
    art: Res<Art>,
    mut rng: ResMut<Rng>,
    mut commands: Commands,
    mut sfx: MessageWriter<Sfx>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform), Without<Player>>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_secs().min(0.05);
    let player_pos = player
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::new(0.0, PLAYER_Y));
    let mut despawn = Vec::new();
    for (entity, mut enemy, mut tf) in &mut enemies {
        if enemy.dead {
            continue;
        }
        if enemy.kind == EnemyKind::Helicopter {
            let target_y = enemy.hover_y;
            tf.translation.y += (target_y - tf.translation.y) * (dt * 0.9).min(1.0);
            tf.translation.x = road_center(tf.translation.y, road.scroll)
                + (game.elapsed * 0.8 + enemy.phase).sin() * 150.0;
            enemy.fire -= dt;
            if enemy.fire <= 0.0 {
                enemy.fire = rng.range(2.6, 4.2);
                let to_player = (player_pos.x - tf.translation.x) * 0.22;
                commands.spawn((
                    sized(art.bomb.clone(), 14.0, 22.0),
                    Transform::from_xyz(tf.translation.x, tf.translation.y - 30.0, 18.0),
                    Bomb {
                        vel: Vec2::new(to_player, -230.0),
                        life: 6.0,
                    },
                    GameEntity,
                ));
            }
            continue;
        }

        let weave = (game.elapsed * 1.7 + enemy.phase).sin() * 32.0;
        tf.translation.y += (enemy.speed - game.speed) * dt;
        tf.translation.x += weave * dt * 0.9;
        // Keep traffic near its lane; pursuers track the player.
        if enemy.from_behind {
            let toward = (player_pos.x - tf.translation.x).clamp(-90.0, 90.0);
            tf.translation.x += toward * dt * 0.9;
        } else {
            let center = road_center(tf.translation.y, road.scroll);
            let lane_anchor = LANES[((tf.translation.x - center + 280.0) / 140.0)
                .floor()
                .clamp(0.0, 3.0) as usize];
            tf.translation.x += ((center + lane_anchor) - tf.translation.x) * (dt * 0.7).min(1.0);
        }
        tf.translation.x = tf.translation.x.clamp(-460.0, 460.0);

        // Switchblades and boats fire at the Interceptor.
        if matches!(
            enemy.kind,
            EnemyKind::Switchblade | EnemyKind::Boat | EnemyKind::Pursuer
        ) {
            enemy.fire -= dt;
            if enemy.fire <= 0.0 && tf.translation.y > -300.0 && tf.translation.y < 340.0 {
                enemy.fire = rng.range(1.9, 3.2);
                let dir = (player_pos - tf.translation.truncate()).normalize_or_zero();
                commands.spawn((
                    sized(art.enemy_bullet.clone(), 8.0, 16.0),
                    Transform::from_xyz(tf.translation.x, tf.translation.y - 20.0, 19.0),
                    Bullet {
                        friendly: false,
                        vel: dir * 300.0,
                        life: 4.0,
                    },
                    GameEntity,
                ));
                sfx.write(Sfx::EnemyShot);
            }
        }

        if tf.translation.y < -470.0 || tf.translation.y > 470.0 || tf.translation.x.abs() > 520.0 {
            despawn.push(entity);
        }
    }
    for entity in despawn {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Weapons
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn player_fire(
    mut commands: Commands,
    time: Res<Time>,
    actions: Res<Actions>,
    game: Res<Game>,
    mut weapons: ResMut<Weapons>,
    art: Res<Art>,
    mut sfx: MessageWriter<Sfx>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<(Entity, &Enemy, &Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    weapons.gun_timer = (weapons.gun_timer - dt).max(0.0);
    let Ok(tf) = player.single() else {
        return;
    };
    let pos = tf.translation.truncate();

    if actions.fire && weapons.gun_timer <= 0.0 {
        weapons.gun_timer = 0.105;
        commands.spawn((
            sized(art.bullet.clone(), 6.0, 18.0),
            Transform::from_xyz(pos.x, pos.y + 38.0, 22.0),
            Bullet {
                friendly: true,
                vel: Vec2::new(0.0, 940.0),
                life: 1.4,
            },
            GameEntity,
        ));
        sfx.write(Sfx::Shot);
        spawn_burst(
            &mut commands,
            pos + Vec2::new(0.0, 34.0),
            Color::srgb(1.0, 0.82, 0.35),
            3,
            40.0,
        );
    }

    if actions.oil && weapons.oil > 0 {
        weapons.oil -= 1;
        let flame = game.terrain.is_water();
        commands.spawn((
            if flame {
                sized(art.explosion[1].clone(), 54.0, 54.0)
            } else {
                sized(art.oil.clone(), 56.0, 24.0)
            },
            Transform::from_xyz(pos.x, pos.y - 48.0, 6.0),
            Slick {
                life: if flame { 1.6 } else { 7.0 },
                flame,
            },
            GameEntity,
        ));
        sfx.write(Sfx::Ui);
    }

    if actions.smoke && weapons.smoke > 0 {
        weapons.smoke -= 1;
        commands.spawn((
            sized(art.smoke.clone(), 90.0, 90.0),
            Transform::from_xyz(pos.x, pos.y - 62.0, 7.0),
            SmokeCloud {
                life: 5.0,
                max_life: 5.0,
            },
            GameEntity,
        ));
        sfx.write(Sfx::Ui);
    }

    if actions.missile && weapons.missiles > 0 {
        weapons.missiles -= 1;
        let target = enemies
            .iter()
            .filter(|(_, e, t)| {
                !e.dead
                    && e.kind != EnemyKind::Civilian
                    && t.translation.y > pos.y - 30.0
                    && (t.translation.truncate() - pos).length() < 640.0
            })
            .min_by(|(_, _, a), (_, _, b)| {
                let da = (a.translation.truncate() - pos).length_squared();
                let db = (b.translation.truncate() - pos).length_squared();
                da.total_cmp(&db)
            })
            .map(|(e, _, _)| e);
        commands.spawn((
            sized(art.missile.clone(), 8.0, 24.0),
            Transform::from_xyz(pos.x, pos.y + 36.0, 23.0),
            Missile { target, life: 3.0 },
            GameEntity,
        ));
        sfx.write(Sfx::Missile);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn move_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut Bullet, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut bullet, mut tf) in &mut bullets {
        bullet.life -= dt;
        tf.translation.x += bullet.vel.x * dt;
        tf.translation.y += bullet.vel.y * dt;
        if bullet.life <= 0.0 || tf.translation.y < -420.0 || tf.translation.y > 420.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if bullet.friendly && tf.translation.y > 380.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn move_missiles(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut missiles: Query<(Entity, &mut Missile, &mut Transform), Without<Enemy>>,
    enemies: Query<(Entity, &Enemy, &Transform), Without<Missile>>,
    mut sfx: MessageWriter<Sfx>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut missile, mut tf) in &mut missiles {
        missile.life -= dt;
        let pos = tf.translation.truncate();
        let target_pos = if let Some(target) = missile.target {
            enemies
                .iter()
                .find(|(e, _, _)| *e == target)
                .map(|(_, _, t)| t.translation.truncate())
        } else {
            // Acquire the closest valid target ahead.
            enemies
                .iter()
                .filter(|(_, e, t)| {
                    !e.dead && e.kind != EnemyKind::Civilian && t.translation.y > pos.y - 20.0
                })
                .min_by(|(_, _, a), (_, _, b)| {
                    (a.translation.truncate() - pos)
                        .length_squared()
                        .total_cmp(&(b.translation.truncate() - pos).length_squared())
                })
                .map(|(e, _, t)| {
                    missile.target = Some(e);
                    t.translation.truncate()
                })
        };
        let mut heading = Vec2::new(0.0, 1.0);
        if let Some(tp) = target_pos {
            heading = (tp - pos).normalize_or_zero();
        }
        tf.translation.x += heading.x * 620.0 * dt;
        tf.translation.y += heading.y * 620.0 * dt;
        if let Some((e, _, t)) = enemies.iter().find(|(_, _, t)| {
            (t.translation.truncate() - tf.translation.truncate()).length() < 38.0
        }) {
            let _ = e;
            let at = t.translation.truncate();
            commands.entity(entity).despawn();
            explode_at(&mut commands, &art, at, 120.0, &mut sfx);
            continue;
        }
        if missile.life <= 0.0 || tf.translation.y > 420.0 {
            let at = tf.translation.truncate();
            commands.entity(entity).despawn();
            explode_at(&mut commands, &art, at, 90.0, &mut sfx);
        }
    }
}

pub fn move_bombs(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut bombs: Query<(Entity, &mut Bomb, &mut Transform), Without<Player>>,
    player: Query<&Transform, With<Player>>,
    mut sfx: MessageWriter<Sfx>,
) {
    let dt = time.delta_secs().min(0.05);
    let player_pos = player.single().ok().map(|t| t.translation.truncate());
    for (entity, mut bomb, mut tf) in &mut bombs {
        bomb.life -= dt;
        tf.translation.x += bomb.vel.x * dt;
        tf.translation.y += bomb.vel.y * dt;
        let hit_player = player_pos.is_some_and(|p| {
            (p - tf.translation.truncate())
                .abs()
                .cmple(Vec2::new(30.0, 42.0))
                .all()
        });
        if tf.translation.y < PLAYER_Y - 20.0 || bomb.life <= 0.0 || hit_player {
            let at = tf.translation.truncate();
            commands.entity(entity).despawn();
            commands.spawn((
                sized(art.crater.clone(), 72.0, 48.0),
                Transform::from_xyz(at.x, at.y.max(PLAYER_Y + 10.0), 1.0),
                Crater { life: 7.0 },
                GameEntity,
            ));
            explode_at(&mut commands, &art, at, 70.0, &mut sfx);
        }
    }
}

fn explode_at(
    commands: &mut Commands,
    art: &Art,
    at: Vec2,
    radius: f32,
    sfx: &mut MessageWriter<Sfx>,
) {
    spawn_explosion(commands, art, at, radius > 100.0);
    spawn_burst(commands, at, Color::srgb(1.0, 0.58, 0.18), 14, 260.0);
    sfx.write(Sfx::Explosion);
    let _ = radius;
}

// ---------------------------------------------------------------------------
// Collisions
// ---------------------------------------------------------------------------

fn overlaps(a: Vec2, a_half: Vec2, b: Vec2, b_half: Vec2) -> bool {
    (a - b).abs().cmple(a_half + b_half).all()
}

#[allow(clippy::too_many_arguments)]
pub fn collisions(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut weapons: ResMut<Weapons>,
    art: Res<Art>,
    mut fx: ResMut<Fx>,
    mut sfx: MessageWriter<Sfx>,
    player: Query<(Entity, &Player, &Transform)>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform)>,
    bullets: Query<(Entity, &Bullet, &Transform)>,
    vans: Query<(Entity, &Transform), With<Van>>,
    slicks: Query<(&Slick, &Transform)>,
    smoke: Query<(&SmokeCloud, &Transform)>,
    craters: Query<(&Crater, &Transform)>,
) {
    let player_data = player
        .single()
        .ok()
        .map(|(e, p, t)| (e, p.invuln, t.translation.truncate()));

    let mut dead_enemies: HashSet<Entity> = HashSet::new();
    let mut consumed_bullets: HashSet<Entity> = HashSet::new();
    let mut hit_player_now = false;

    // Player bullets vs traffic.
    for (bullet_entity, bullet, btf) in &bullets {
        if !bullet.friendly {
            continue;
        }
        let bpos = btf.translation.truncate();
        for (enemy_entity, mut enemy, etf) in &mut enemies {
            if enemy.dead || dead_enemies.contains(&enemy_entity) {
                continue;
            }
            if overlaps(
                bpos,
                Vec2::new(3.0, 9.0),
                etf.translation.truncate(),
                enemy.kind.size() * 0.47,
            ) {
                consumed_bullets.insert(bullet_entity);
                if enemy.kind.is_civilian() {
                    enemy.dead = true;
                    dead_enemies.insert(enemy_entity);
                    game.score_freeze = 3.0;
                    game.civilians_hit += 1;
                    game.banner = "CIVILIAN HIT — NO SCORE".into();
                    game.banner_timer = 2.2;
                    sfx.write(Sfx::Penalty);
                    spawn_explosion(&mut commands, &art, etf.translation.truncate(), false);
                    spawn_burst(
                        &mut commands,
                        etf.translation.truncate(),
                        Color::srgb(1.0, 0.3, 0.2),
                        10,
                        180.0,
                    );
                } else {
                    enemy.dead = true;
                    dead_enemies.insert(enemy_entity);
                    game.score += enemy.kind.score();
                    game.destroyed += 1;
                    explode_at(
                        &mut commands,
                        &art,
                        etf.translation.truncate(),
                        90.0,
                        &mut sfx,
                    );
                    fx.shake = (fx.shake + 3.0).min(12.0);
                }
                break;
            }
        }
    }

    // Enemy fire vs player.
    if let Some((_, invuln, ppos)) = player_data {
        for (bullet_entity, bullet, btf) in &bullets {
            if bullet.friendly || consumed_bullets.contains(&bullet_entity) {
                continue;
            }
            if overlaps(
                ppos,
                Vec2::new(19.0, 30.0),
                btf.translation.truncate(),
                Vec2::new(4.0, 8.0),
            ) && invuln <= 0.0
            {
                consumed_bullets.insert(bullet_entity);
                hit_player_now = true;
            }
        }
    }

    // Traffic / boats / helicopters vs the player.
    if let Some((_, invuln, ppos)) = player_data {
        for (enemy_entity, mut enemy, etf) in &mut enemies {
            if enemy.dead || dead_enemies.contains(&enemy_entity) {
                continue;
            }
            if overlaps(
                ppos,
                Vec2::new(20.0, 31.0),
                etf.translation.truncate(),
                enemy.kind.size() * 0.44,
            ) {
                if invuln <= 0.0 {
                    hit_player_now = true;
                }
                if !enemy.kind.is_civilian() {
                    enemy.dead = true;
                    dead_enemies.insert(enemy_entity);
                }
            }
        }
    }

    // Weapons van pickup.
    if let Some((_, _, ppos)) = player_data {
        for (van_entity, vtf) in &vans {
            if overlaps(
                ppos,
                Vec2::new(24.0, 38.0),
                vtf.translation.truncate(),
                Vec2::new(30.0, 46.0),
            ) {
                commands.entity(van_entity).despawn();
                weapons.oil = (weapons.oil + 3).min(9);
                weapons.smoke = (weapons.smoke + 3).min(9);
                weapons.missiles = (weapons.missiles + 2).min(9);
                weapons.van_ready = false;
                sfx.write(Sfx::Pickup);
                game.banner = "WEAPONS VAN — OIL / SMOKE / MISSILES ONLINE".into();
                game.banner_timer = 2.6;
            }
        }
    }

    // Oil slicks and smoke clouds wreck pursuing traffic.
    for (slick, stf) in &slicks {
        if slick.life <= 0.0 {
            continue;
        }
        for (enemy_entity, mut enemy, etf) in &mut enemies {
            if enemy.dead || dead_enemies.contains(&enemy_entity) {
                continue;
            }
            if overlaps(
                stf.translation.truncate(),
                Vec2::new(28.0, 14.0),
                etf.translation.truncate(),
                enemy.kind.size() * 0.45,
            ) && !enemy.kind.is_civilian()
            {
                enemy.dead = true;
                dead_enemies.insert(enemy_entity);
                game.score += enemy.kind.score();
                game.destroyed += 1;
                spawn_explosion(&mut commands, &art, etf.translation.truncate(), false);
                sfx.write(Sfx::Explosion);
            }
        }
    }
    for (cloud, ctf) in &smoke {
        if cloud.life <= 0.0 {
            continue;
        }
        for (enemy_entity, mut enemy, etf) in &mut enemies {
            if enemy.dead || dead_enemies.contains(&enemy_entity) {
                continue;
            }
            if overlaps(
                ctf.translation.truncate(),
                Vec2::new(52.0, 52.0),
                etf.translation.truncate(),
                enemy.kind.size() * 0.45,
            ) && !enemy.kind.is_civilian()
            {
                enemy.dead = true;
                dead_enemies.insert(enemy_entity);
                game.score += enemy.kind.score();
                game.destroyed += 1;
                spawn_explosion(&mut commands, &art, etf.translation.truncate(), false);
                sfx.write(Sfx::Explosion);
            }
        }
    }

    // Craters are hard hazards.
    if let Some((_, invuln, ppos)) = player_data {
        for (crater, ctf) in &craters {
            if crater.life > 0.0
                && invuln <= 0.0
                && overlaps(
                    ppos,
                    Vec2::new(18.0, 30.0),
                    ctf.translation.truncate(),
                    Vec2::new(34.0, 20.0),
                )
            {
                hit_player_now = true;
            }
        }
    }

    for bullet in consumed_bullets {
        commands.entity(bullet).despawn();
    }
    for enemy in dead_enemies {
        commands.entity(enemy).despawn();
    }

    if hit_player_now {
        if let Some((entity, _, ppos)) = player_data {
            commands.entity(entity).despawn();
            damage_player_public(&mut commands, &mut game, &mut sfx, ppos);
            fx.shake = 14.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Hazards, explosions, particles
// ---------------------------------------------------------------------------

pub fn update_hazards(
    mut commands: Commands,
    time: Res<Time>,
    mut items: Query<(Entity, &mut Slick)>,
    mut clouds: Query<(Entity, &mut SmokeCloud, &mut Sprite)>,
    mut craters: Query<(Entity, &mut Crater)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut slick) in &mut items {
        slick.life -= dt;
        if slick.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut cloud, mut sprite) in &mut clouds {
        cloud.life -= dt;
        let t = (cloud.life / cloud.max_life).clamp(0.0, 1.0);
        sprite.color = Color::srgba(0.8, 0.82, 0.86, 0.25 + 0.45 * t);
        if cloud.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut crater) in &mut craters {
        crater.life -= dt;
        if crater.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut explosions: Query<(Entity, &mut Explosion, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut ex, mut sprite) in &mut explosions {
        ex.timer += dt;
        let frame = (ex.timer / 0.075) as usize;
        if frame >= 4 {
            commands.entity(entity).despawn();
        } else if frame != ex.frame {
            ex.frame = frame;
            sprite.image = art.explosion[frame].clone();
        }
    }
}

pub fn spawn_explosion(commands: &mut Commands, art: &Art, at: Vec2, big: bool) {
    let size = if big { 144.0 } else { 96.0 };
    commands.spawn((
        sized(art.explosion[0].clone(), size, size),
        Transform::from_xyz(at.x, at.y, 30.0),
        Explosion {
            timer: 0.0,
            frame: 0,
        },
        GameEntity,
    ));
}

pub fn spawn_burst(commands: &mut Commands, at: Vec2, color: Color, count: usize, speed: f32) {
    for i in 0..count {
        let angle = i as f32 / count.max(1) as f32 * std::f32::consts::TAU;
        let jitter = 0.55 + (i % 4) as f32 * 0.18;
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(3.0)),
            Transform::from_xyz(at.x, at.y, 28.0),
            Particle {
                vel: Vec2::new(angle.cos(), angle.sin()) * speed * jitter,
                life: 0.55,
                max_life: 0.55,
                base: color,
            },
            GameEntity,
        ));
    }
}

pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut p, mut tf, mut sprite) in &mut particles {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        p.vel.y -= 160.0 * dt;
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        let t = (p.life / p.max_life).clamp(0.0, 1.0);
        sprite.color = p.base.with_alpha(t);
    }
}

pub fn decay_fx(time: Res<Time>, mut fx: ResMut<Fx>) {
    fx.shake = (fx.shake - time.delta_secs() * 20.0).max(0.0);
}

pub fn camera_shake(
    fx: Res<Fx>,
    mut rng: ResMut<Rng>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut tf) = cam.single_mut() else {
        return;
    };
    if fx.shake > 0.08 {
        tf.translation.x = rng.range(-fx.shake, fx.shake) * 0.45;
        tf.translation.y = rng.range(-fx.shake, fx.shake) * 0.45;
    } else {
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Run timers, scoring, death and respawn
// ---------------------------------------------------------------------------

pub fn damage_player_public(
    commands: &mut Commands,
    game: &mut Game,
    sfx: &mut MessageWriter<Sfx>,
    at: Vec2,
) {
    if game.phase != RunPhase::Running {
        return;
    }
    game.lives = game.lives.saturating_sub(1);
    game.phase = RunPhase::Dying;
    game.death_timer = 1.35;
    game.speed = 0.0;
    game.target_speed = 0.0;
    game.offroad = 0.0;
    game.banner = if game.lives == 0 {
        "INTERCEPTOR DESTROYED".into()
    } else {
        "INTERCEPTOR HIT — BACKUP INBOUND".into()
    };
    game.banner_timer = 2.0;
    sfx.write(Sfx::Death);
    spawn_burst(commands, at, Color::srgb(1.0, 0.55, 0.2), 18, 280.0);
}

#[allow(clippy::too_many_arguments)]
pub fn tick_run(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut weapons: ResMut<Weapons>,
    road: Res<Road>,
    art: Res<Art>,
    capture: Res<crate::CaptureConfig>,
    mut next: ResMut<NextState<AppState>>,
    mut sfx: MessageWriter<Sfx>,
) {
    let dt = time.delta_secs().min(0.05);
    if game.banner_timer > 0.0 {
        game.banner_timer = (game.banner_timer - dt).max(0.0);
    }
    if game.score_freeze > 0.0 {
        game.score_freeze = (game.score_freeze - dt).max(0.0);
    }
    if game.phase == RunPhase::Dying {
        game.death_timer -= dt;
        if game.death_timer <= 0.0 {
            if game.lives == 0 {
                game.hi = game.hi.max(game.score);
                next.set(AppState::GameOver);
            } else {
                let x = road_center(PLAYER_Y, road.scroll);
                let image = if game.terrain.is_water() {
                    art.player_boat.clone()
                } else {
                    art.player_car.clone()
                };
                commands.spawn((
                    sized(image, 48.0, 72.0),
                    Transform::from_xyz(x, PLAYER_Y, 10.0),
                    Player { invuln: 2.6 },
                    GameEntity,
                ));
                game.phase = RunPhase::Running;
                game.speed = 220.0;
                game.target_speed = 260.0;
                weapons.oil = 0;
                weapons.smoke = 0;
                weapons.missiles = 0;
            }
        }
        return;
    }

    game.terrain_timer -= dt;
    if game.terrain_timer <= 0.0 {
        game.terrain = game.terrain.next();
        game.terrain_timer = 48.0;
        game.banner = format!("ENTERING {}", game.terrain.name());
        game.banner_timer = 2.8;
        sfx.write(Sfx::Terrain);
    }

    // Score accrues with speed, but not off-road or after a civilian hit.
    if game.score_freeze <= 0.0 && game.offroad <= 0.0 {
        game.score_frac += game.speed * dt * 0.11;
        let whole = game.score_frac.floor();
        if whole > 0.0 {
            game.score += whole as u64;
            game.score_frac -= whole;
        }
    }
    game.distance += game.speed * dt;
    game.hi = game.hi.max(game.score);
    if game.score >= game.next_bonus {
        game.lives = (game.lives + 1).min(6);
        game.next_bonus += 30_000;
        game.banner = "EXTRA INTERCEPTOR".into();
        game.banner_timer = 2.0;
        sfx.write(Sfx::Extra);
    }

    // Capture hooks: a game-over still frame can be produced on demand.
    if capture.gameover && game.elapsed > 0.05 {
        game.hi = game.hi.max(game.score);
        next.set(AppState::GameOver);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn road_center_is_bounded() {
        for i in 0..10_000 {
            let y = (i as f32 * 0.37) % 720.0 - 360.0;
            let c = road_center(y, i as f32 * 13.0);
            assert!(c.abs() <= 60.01, "road centre {c} escaped the carriageway");
        }
    }

    #[test]
    fn lanes_stay_inside_the_road() {
        for lane in LANES {
            assert!(lane.abs() + 24.0 < ROAD_W * 0.5);
        }
    }

    #[test]
    fn overlapping_boxes_collide() {
        assert!(overlaps(
            Vec2::ZERO,
            Vec2::splat(10.0),
            Vec2::new(12.0, 0.0),
            Vec2::splat(5.0)
        ));
        assert!(!overlaps(
            Vec2::ZERO,
            Vec2::splat(10.0),
            Vec2::new(30.0, 0.0),
            Vec2::splat(5.0)
        ));
    }

    #[test]
    fn enemy_scores_are_ordered() {
        assert!(EnemyKind::Helicopter.score() > EnemyKind::Switchblade.score());
        assert!(EnemyKind::Switchblade.score() > EnemyKind::TrafficRed.score());
        assert_eq!(EnemyKind::Civilian.score(), 0);
    }

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng(42);
        let mut b = Rng(42);
        for _ in 0..100 {
            let x = a.range(-5.0, 5.0);
            let y = b.range(-5.0, 5.0);
            assert_eq!(x, y);
            assert!((-5.0..=5.0).contains(&x));
        }
    }

    #[test]
    fn terrain_cycles_through_every_region() {
        let mut t = Terrain::Highway;
        let mut seen = Vec::new();
        for _ in 0..4 {
            seen.push(t);
            t = t.next();
        }
        assert_eq!(t, Terrain::Highway);
        assert_eq!(seen.len(), 4);
    }
}
