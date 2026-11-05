//! Asset handles and sprite helpers.

use bevy::prelude::*;

#[derive(Resource)]
pub struct Art {
    pub font: Handle<Font>,
    pub logo: Handle<Image>,
    pub terrain: [Handle<Image>; 4],
    pub player_car: Handle<Image>,
    pub player_boat: Handle<Image>,
    pub enemy_red: Handle<Image>,
    pub enemy_blue: Handle<Image>,
    pub enemy_black: Handle<Image>,
    pub civilian: Handle<Image>,
    pub enemy_boat: Handle<Image>,
    pub van: Handle<Image>,
    pub helicopter: Handle<Image>,
    pub tree: Handle<Image>,
    pub snow_pine: Handle<Image>,
    pub rock: Handle<Image>,
    pub building: Handle<Image>,
    pub buoy: Handle<Image>,
    pub oil: Handle<Image>,
    pub smoke: Handle<Image>,
    pub crater: Handle<Image>,
    pub explosion: [Handle<Image>; 4],
    pub bullet: Handle<Image>,
    pub enemy_bullet: Handle<Image>,
    pub missile: Handle<Image>,
    pub bomb: Handle<Image>,
    pub spark: Handle<Image>,
    pub icon_gun: Handle<Image>,
    pub icon_oil: Handle<Image>,
    pub icon_smoke: Handle<Image>,
    pub icon_missile: Handle<Image>,
    pub icon_van: Handle<Image>,
    pub life_icon: Handle<Image>,
}

pub fn load_art(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Art {
        font: assets.load("fonts/Interceptor.ttf"),
        logo: assets.load("sprites/logo.png"),
        terrain: [
            assets.load("backgrounds/terrain_grass.png"),
            assets.load("backgrounds/terrain_snow.png"),
            assets.load("backgrounds/terrain_water.png"),
            assets.load("backgrounds/terrain_night.png"),
        ],
        player_car: assets.load("sprites/player_car.png"),
        player_boat: assets.load("sprites/player_boat.png"),
        enemy_red: assets.load("sprites/enemy_red.png"),
        enemy_blue: assets.load("sprites/enemy_blue.png"),
        enemy_black: assets.load("sprites/enemy_black.png"),
        civilian: assets.load("sprites/civilian.png"),
        enemy_boat: assets.load("sprites/enemy_boat.png"),
        van: assets.load("sprites/weapons_van.png"),
        helicopter: assets.load("sprites/helicopter.png"),
        tree: assets.load("sprites/tree.png"),
        snow_pine: assets.load("sprites/snow_pine.png"),
        rock: assets.load("sprites/rock.png"),
        building: assets.load("sprites/building.png"),
        buoy: assets.load("sprites/buoy.png"),
        oil: assets.load("sprites/oil_slick.png"),
        smoke: assets.load("sprites/smoke.png"),
        crater: assets.load("sprites/crater.png"),
        explosion: [
            assets.load("sprites/explosion_0.png"),
            assets.load("sprites/explosion_1.png"),
            assets.load("sprites/explosion_2.png"),
            assets.load("sprites/explosion_3.png"),
        ],
        bullet: assets.load("sprites/bullet.png"),
        enemy_bullet: assets.load("sprites/enemy_bullet.png"),
        missile: assets.load("sprites/missile.png"),
        bomb: assets.load("sprites/bomb.png"),
        spark: assets.load("sprites/spark.png"),
        icon_gun: assets.load("sprites/icon_gun.png"),
        icon_oil: assets.load("sprites/icon_oil.png"),
        icon_smoke: assets.load("sprites/icon_smoke.png"),
        icon_missile: assets.load("sprites/icon_missile.png"),
        icon_van: assets.load("sprites/icon_van.png"),
        life_icon: assets.load("sprites/life_icon.png"),
    });
}

pub fn sized(image: Handle<Image>, w: f32, h: f32) -> Sprite {
    Sprite {
        image,
        custom_size: Some(Vec2::new(w, h)),
        ..default()
    }
}
