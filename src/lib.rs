//! SPY HUNTER: INTERCEPTOR — app wiring, states, and capture harness.

pub mod art;
pub mod audio;
pub mod game;
pub mod ui;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::audio::{GlobalVolume, Volume};
use bevy::camera::ScalingMode;
use bevy::image::ImagePlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::WindowResolution;

use art::*;
use audio::*;
use game::*;
use ui::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Title,
    InGame,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Paused(pub bool);

// Touch/UI input bits shared with the browser shell. The wasm module exposes
// `spy_input(mask)`; native builds always read 0 and use keyboard/mouse.
pub const BIT_LEFT: u32 = 1 << 0;
pub const BIT_RIGHT: u32 = 1 << 1;
pub const BIT_THROTTLE: u32 = 1 << 2;
pub const BIT_BRAKE: u32 = 1 << 3;
pub const BIT_FIRE: u32 = 1 << 4;
pub const BIT_OIL: u32 = 1 << 5;
pub const BIT_SMOKE: u32 = 1 << 6;
pub const BIT_MISSILE: u32 = 1 << 7;
pub const BIT_VAN: u32 = 1 << 8;
pub const BIT_START: u32 = 1 << 9;
pub const BIT_PAUSE: u32 = 1 << 10;
pub const BIT_MUTE: u32 = 1 << 11;

#[cfg(target_arch = "wasm32")]
static INPUT_MASK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn spy_input(mask: u32) {
    INPUT_MASK.store(mask, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(target_arch = "wasm32")]
pub fn touch_mask() -> u32 {
    INPUT_MASK.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn touch_mask() -> u32 {
    0
}

#[derive(Resource, Default)]
pub struct CaptureConfig {
    pub screenshot: Option<String>,
    pub frame: u32,
    pub start: bool,
    pub gameover: bool,
    pub terrain: Option<Terrain>,
}

pub fn run() {
    let capture = parse_args();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SPY HUNTER: INTERCEPTOR".into(),
                    resolution: WindowResolution::new(960, 720).with_scale_factor_override(1.0),
                    resizable: false,
                    canvas: Some("#game-canvas".into()),
                    fit_canvas_to_parent: false,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.008, 0.012, 0.02)))
    .insert_resource(GlobalVolume::new(Volume::Linear(0.9)))
    .insert_resource(capture)
    .init_resource::<Game>()
    .init_resource::<Weapons>()
    .init_resource::<Road>()
    .init_resource::<Fx>()
    .init_resource::<Rng>()
    .init_resource::<Actions>()
    .init_resource::<TouchPrev>()
    .init_resource::<Paused>()
    .init_resource::<Muted>()
    .init_state::<AppState>()
    .add_message::<Sfx>()
    .add_systems(PreStartup, load_art)
    .add_systems(
        Startup,
        (
            setup_camera,
            load_sounds,
            start_music,
            setup_world,
            spawn_title,
            apply_capture,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(AppState::InGame),
        (start_run, spawn_player, spawn_hud).chain(),
    )
    .add_systems(OnExit(AppState::InGame), cleanup_run)
    .add_systems(OnExit(AppState::Title), cleanup_title)
    .add_systems(OnEnter(AppState::GameOver), spawn_gameover)
    .add_systems(
        Update,
        (
            update_actions,
            state_input,
            apply_mute,
            play_sfx,
            scroll_world,
            move_scenery,
            sync_road,
            sync_terrain_art,
            tick_run
                .run_if(in_state(AppState::InGame))
                .run_if(not_paused),
            (
                player_control,
                spawn_enemies,
                summon_van,
                move_vans,
                move_enemies,
                player_fire,
                move_bullets,
                move_missiles,
                move_bombs,
                collisions,
            )
                .chain()
                .run_if(in_state(AppState::InGame))
                .run_if(not_paused),
            update_hazards.run_if(in_state(AppState::InGame)),
            sync_player_art.run_if(in_state(AppState::InGame)),
            update_hud.run_if(in_state(AppState::InGame)),
            update_title.run_if(in_state(AppState::Title)),
            update_explosions,
            update_particles,
            decay_fx,
            camera_shake,
            capture_system,
        )
            .chain(),
    )
    .run();
}

fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WORLD_H,
            },
            ..OrthographicProjection::default_2d()
        }),
        Msaa::Off,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn cleanup_title(mut commands: Commands, cars: Query<Entity, With<TitleCar>>) {
    for car in &cars {
        commands.entity(car).despawn();
    }
}

fn apply_capture(
    mut game: ResMut<Game>,
    capture: Res<CaptureConfig>,
    mut next: ResMut<NextState<AppState>>,
) {
    if let Some(terrain) = capture.terrain {
        game.terrain = terrain;
    }
    if capture.gameover {
        game.score = 12_340;
        game.hi = 54_320;
        game.lives = 0;
        next.set(AppState::GameOver);
    } else if capture.start {
        next.set(AppState::InGame);
    }
}

fn capture_system(
    capture: Res<CaptureConfig>,
    mut frame: Local<u32>,
    mut fired: Local<bool>,
    mut commands: Commands,
) {
    let Some(path) = capture.screenshot.clone() else {
        return;
    };
    if *fired {
        return;
    }
    *frame += 1;
    if *frame < capture.frame.max(1) {
        return;
    }
    *fired = true;
    info!("capturing {} at frame {}", path, *frame);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            move |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

fn parse_args() -> CaptureConfig {
    let mut cfg = CaptureConfig {
        frame: 240,
        ..default()
    };
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--screenshot" => {
                i += 1;
                if i < args.len() {
                    cfg.screenshot = Some(args[i].clone());
                }
            }
            "--frame" => {
                i += 1;
                if i < args.len() {
                    cfg.frame = args[i].parse().unwrap_or(240);
                }
            }
            "--start" => cfg.start = true,
            "--gameover" => {
                cfg.gameover = true;
                cfg.start = true;
            }
            "--terrain" => {
                i += 1;
                if i < args.len() {
                    cfg.terrain = match args[i].as_str() {
                        "snow" => Some(Terrain::Snow),
                        "water" => Some(Terrain::Water),
                        "night" => Some(Terrain::Night),
                        _ => Some(Terrain::Highway),
                    };
                    cfg.start = true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    cfg
}
