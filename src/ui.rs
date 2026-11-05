//! Title, HUD, pause veil and game-over presentation.

use bevy::prelude::*;

use crate::AppState;
use crate::art::Art;
use crate::game::{Game, Weapons};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudField {
    Score,
    Hi,
    Terrain,
    Speed,
    Gun,
    Ammo,
    Banner,
    Freeze,
    Van,
}

#[derive(Component)]
pub struct LifeIcon(pub u32);

#[derive(Component)]
pub struct PauseVeil;

#[derive(Component)]
pub struct TitleRoot;

#[derive(Component)]
pub struct TitlePrompt;

#[derive(Component)]
pub struct TitleHigh;

#[derive(Component)]
pub struct GameOverRoot;

fn fonted(art: &Art, content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content.into()),
        TextFont {
            font: art.font.clone().into(),
            font_size: FontSize::Px(size),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::new(2.0, 2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.9),
        },
    )
}

fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(gap),
        ..default()
    }
}

fn panel_style() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(10.0),
        ..default()
    }
}

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

pub fn spawn_title(mut commands: Commands, art: Res<Art>, game: Res<Game>) {
    commands
        .spawn((
            panel_style(),
            BackgroundColor(Color::srgba(0.01, 0.02, 0.045, 0.55)),
            ZIndex(90),
            TitleRoot,
            DespawnOnExit(AppState::Title),
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(art.logo.clone()),
                Node {
                    width: Val::Px(760.0),
                    height: Val::Px(247.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));
            root.spawn((
                fonted(
                    &art,
                    "PRESS ENTER OR CLICK TO DEPLOY",
                    30.0,
                    Color::srgb(1.0, 0.92, 0.65),
                ),
                TitlePrompt,
            ));
            root.spawn((
                fonted(
                    &art,
                    format!("HI-SCORE {:07}", game.hi),
                    22.0,
                    Color::srgb(0.5, 0.84, 0.92),
                ),
                TitleHigh,
            ));
            root.spawn((
                fonted(
                    &art,
                    "ARROWS / A-D STEER     W-S THROTTLE     SPACE FIRE",
                    19.0,
                    Color::srgb(0.86, 0.87, 0.9),
                ),
                Node {
                    margin: UiRect::top(Val::Px(18.0)),
                    ..default()
                },
            ));
            root.spawn((
                fonted(
                    &art,
                    "1 OIL SLICK   2 SMOKE   3 MISSILE   V SUMMON VAN   P PAUSE   M MUTE",
                    17.0,
                    Color::srgb(0.72, 0.76, 0.82),
                ),
                Node::default(),
            ));
            root.spawn((
                fonted(
                    &art,
                    "ORIGINAL PIXEL ART · ORIGINAL CHIPTUNE · RUST + BEVY 0.20",
                    15.0,
                    Color::srgb(0.80, 0.68, 0.42),
                ),
                Node {
                    margin: UiRect::top(Val::Px(22.0)),
                    ..default()
                },
            ));
        });
}

pub fn update_title(
    time: Res<Time>,
    game: Res<Game>,
    mut prompt: Query<&mut TextColor, With<TitlePrompt>>,
    mut high: Query<&mut Text, With<TitleHigh>>,
) {
    let t = time.elapsed_secs();
    if let Ok(mut color) = prompt.single_mut() {
        color.0 = Color::srgba(1.0, 0.92, 0.65, 0.45 + 0.55 * (t * 3.2).sin().abs());
    }
    if let Ok(mut text) = high.single_mut() {
        text.0 = format!("HI-SCORE {:07}", game.hi);
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

pub fn spawn_hud(mut commands: Commands, art: Res<Art>, game: Res<Game>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(66.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.92)),
            ZIndex(60),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn(row(12.0)).with_children(|r| {
                r.spawn(fonted(&art, "SCORE", 15.0, Color::srgb(0.90, 0.72, 0.38)));
                r.spawn((
                    fonted(&art, "0000000", 34.0, Color::srgb(1.0, 0.96, 0.84)),
                    HudField::Score,
                ));
                r.spawn((
                    fonted(&art, "HI 0000000", 15.0, Color::srgb(0.52, 0.80, 0.90)),
                    HudField::Hi,
                ));
            });

            bar.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    fonted(&art, "COAST HIGHWAY", 20.0, Color::srgb(0.52, 0.90, 0.84)),
                    HudField::Terrain,
                ));
                c.spawn((
                    fonted(&art, "000 KM/H", 14.0, Color::srgb(0.76, 0.80, 0.86)),
                    HudField::Speed,
                ));
            });

            bar.spawn(row(9.0)).with_children(|r| {
                r.spawn((
                    ImageNode::new(art.icon_gun.clone()),
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        ..default()
                    },
                ));
                r.spawn((
                    fonted(&art, "MG", 22.0, Color::srgb(1.0, 0.86, 0.40)),
                    HudField::Gun,
                ));
                for i in 0..6u32 {
                    r.spawn((
                        ImageNode::new(art.life_icon.clone()),
                        Node {
                            width: Val::Px(17.0),
                            height: Val::Px(26.0),
                            margin: UiRect::left(Val::Px(4.0)),
                            ..default()
                        },
                        LifeIcon(i),
                    ));
                }
            });
        });

    // Banner.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(84.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ZIndex(65),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn((
                fonted(&art, "", 30.0, Color::srgb(1.0, 0.86, 0.36)),
                HudField::Banner,
            ));
        });

    // Weapon / van status strip.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(34.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
            ZIndex(62),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|r| {
            for (icon, label) in [
                (art.icon_oil.clone(), "OIL"),
                (art.icon_smoke.clone(), "SMOKE"),
                (art.icon_missile.clone(), "MSL"),
                (art.icon_van.clone(), "VAN"),
            ] {
                r.spawn(row(6.0)).with_children(|item| {
                    item.spawn((
                        ImageNode::new(icon),
                        Node {
                            width: Val::Px(23.0),
                            height: Val::Px(23.0),
                            ..default()
                        },
                    ));
                    item.spawn(fonted(&art, label, 15.0, Color::srgb(0.76, 0.80, 0.86)));
                });
            }
            r.spawn((
                fonted(&art, "0/0/0", 18.0, Color::srgb(1.0, 0.86, 0.42)),
                HudField::Ammo,
            ));
            r.spawn((
                fonted(&art, "", 18.0, Color::srgb(1.0, 0.45, 0.32)),
                HudField::Freeze,
            ));
            r.spawn((
                fonted(&art, "", 18.0, Color::srgb(0.55, 0.92, 0.80)),
                HudField::Van,
            ));
        });

    // Bottom control hint.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.92)),
            ZIndex(60),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|bar| {
            bar.spawn(fonted(
                &art,
                "A/D STEER  W/S THROTTLE  SPACE FIRE  1 OIL  2 SMOKE  3 MISSILE  V VAN  P PAUSE  M MUTE",
                15.0,
                Color::srgb(0.60, 0.66, 0.74),
            ));
        });

    // Pause veil, toggled by `update_pause_veil`.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.72)),
            ZIndex(80),
            PauseVeil,
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|root| {
            root.spawn(fonted(&art, "PAUSED", 52.0, Color::srgb(1.0, 0.86, 0.42)));
            root.spawn(fonted(
                &art,
                "P TO RESUME",
                20.0,
                Color::srgb(0.80, 0.84, 0.9),
            ));
        });

    let _ = game;
}

pub fn update_hud(
    game: Res<Game>,
    weapons: Res<Weapons>,
    paused: Res<crate::Paused>,
    mut fields: Query<(&HudField, &mut Text, &mut TextColor)>,
    mut lives: Query<(&LifeIcon, &mut Node), Without<PauseVeil>>,
    mut veil: Query<&mut Node, (With<PauseVeil>, Without<LifeIcon>)>,
) {
    for (field, mut text, mut color) in &mut fields {
        match field {
            HudField::Score => {
                text.0 = format!("{:07}", game.score);
                color.0 = if game.score_freeze > 0.0 {
                    Color::srgb(0.95, 0.38, 0.30)
                } else {
                    Color::srgb(1.0, 0.96, 0.84)
                };
            }
            HudField::Hi => text.0 = format!("HI {:07}", game.hi),
            HudField::Terrain => text.0 = game.terrain.name().into(),
            HudField::Speed => {
                text.0 = format!("{:03} KM/H", (game.speed * 0.72) as u32);
            }
            HudField::Gun => text.0 = "MG".into(),
            HudField::Ammo => {
                text.0 = format!(
                    "{:02}/{:02}/{:02}",
                    weapons.oil, weapons.smoke, weapons.missiles
                );
            }
            HudField::Banner => {
                text.0 = if game.banner_timer > 0.0 {
                    game.banner.clone()
                } else {
                    String::new()
                };
                let alpha = (game.banner_timer * 2.0).clamp(0.0, 1.0);
                color.0 = Color::srgba(1.0, 0.86, 0.36, alpha);
            }
            HudField::Freeze => {
                text.0 = if game.score_freeze > 0.0 {
                    "NO SCORE".into()
                } else {
                    String::new()
                };
            }
            HudField::Van => {
                text.0 = if weapons.van_ready {
                    "VAN READY — PRESS V".into()
                } else {
                    String::new()
                };
                color.0 = Color::srgba(
                    0.55,
                    0.92,
                    0.80,
                    0.55 + 0.45 * (game.elapsed * 7.0).sin().abs(),
                );
            }
        }
    }
    for (icon, mut node) in &mut lives {
        node.display = if icon.0 < game.lives {
            Display::Flex
        } else {
            Display::None
        };
    }
    let _ = paused;
    for mut node in &mut veil {
        node.display = if paused.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ---------------------------------------------------------------------------
// Game over
// ---------------------------------------------------------------------------

pub fn spawn_gameover(mut commands: Commands, art: Res<Art>, game: Res<Game>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.82)),
            ZIndex(95),
            GameOverRoot,
            DespawnOnExit(AppState::GameOver),
        ))
        .with_children(|root| {
            root.spawn(fonted(
                &art,
                "INTERCEPTOR DESTROYED",
                54.0,
                Color::srgb(0.96, 0.32, 0.28),
            ));
            root.spawn((
                fonted(
                    &art,
                    format!("FINAL SCORE  {:07}", game.score),
                    34.0,
                    Color::srgb(1.0, 0.92, 0.64),
                ),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
            root.spawn(fonted(
                &art,
                format!("HI-SCORE  {:07}", game.hi),
                22.0,
                Color::srgb(0.52, 0.84, 0.92),
            ));
            root.spawn(fonted(
                &art,
                format!(
                    "ENEMY VEHICLES DESTROYED  {}     CIVILIAN HITS  {}",
                    game.destroyed, game.civilians_hit
                ),
                18.0,
                Color::srgb(0.78, 0.80, 0.86),
            ));
            root.spawn((
                fonted(
                    &art,
                    "PRESS ENTER OR R TO RE-ENTER THE FIELD",
                    24.0,
                    Color::srgb(1.0, 0.86, 0.42),
                ),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
            root.spawn(fonted(
                &art,
                "SPY HUNTER: INTERCEPTOR · ORIGINAL ARCADE TRIBUTE · RUST + BEVY",
                14.0,
                Color::srgb(0.72, 0.62, 0.40),
            ));
        });
}
