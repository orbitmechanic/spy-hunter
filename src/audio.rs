//! Original chiptune soundtrack and arcade sound effects.

use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;

#[derive(Message, Clone, Copy, Debug)]
pub enum Sfx {
    Shot,
    EnemyShot,
    Explosion,
    Missile,
    Pickup,
    Penalty,
    Death,
    Extra,
    Terrain,
    Ui,
}

#[derive(Resource)]
pub struct SoundAssets {
    pub shot: Handle<AudioSource>,
    pub enemy_shot: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
    pub missile: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub penalty: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub extra: Handle<AudioSource>,
    pub terrain: Handle<AudioSource>,
    pub ui: Handle<AudioSource>,
    pub music: Handle<AudioSource>,
}

impl SoundAssets {
    pub fn get(&self, sfx: Sfx) -> Handle<AudioSource> {
        match sfx {
            Sfx::Shot => self.shot.clone(),
            Sfx::EnemyShot => self.enemy_shot.clone(),
            Sfx::Explosion => self.explosion.clone(),
            Sfx::Missile => self.missile.clone(),
            Sfx::Pickup => self.pickup.clone(),
            Sfx::Penalty => self.penalty.clone(),
            Sfx::Death => self.death.clone(),
            Sfx::Extra => self.extra.clone(),
            Sfx::Terrain => self.terrain.clone(),
            Sfx::Ui => self.ui.clone(),
        }
    }
}

fn volume(sfx: Sfx) -> f32 {
    match sfx {
        Sfx::Shot => 0.20,
        Sfx::EnemyShot => 0.26,
        Sfx::Explosion => 0.58,
        Sfx::Missile => 0.42,
        Sfx::Pickup => 0.48,
        Sfx::Penalty => 0.46,
        Sfx::Death => 0.60,
        Sfx::Extra => 0.50,
        Sfx::Terrain => 0.38,
        Sfx::Ui => 0.30,
    }
}

pub fn load_sounds(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SoundAssets {
        shot: assets.load("audio/shot.wav"),
        enemy_shot: assets.load("audio/enemy_shot.wav"),
        explosion: assets.load("audio/explosion.wav"),
        missile: assets.load("audio/missile.wav"),
        pickup: assets.load("audio/pickup.wav"),
        penalty: assets.load("audio/penalty.wav"),
        death: assets.load("audio/death.wav"),
        extra: assets.load("audio/extra.wav"),
        terrain: assets.load("audio/terrain.wav"),
        ui: assets.load("audio/ui.wav"),
        music: assets.load("audio/music_interceptor.wav"),
    });
}

pub fn play_sfx(
    mut commands: Commands,
    mut messages: MessageReader<Sfx>,
    sounds: Res<SoundAssets>,
) {
    for sfx in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.get(*sfx)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume(*sfx))),
        ));
    }
}

#[derive(Component)]
pub struct MusicPlayer;

pub fn start_music(mut commands: Commands, sounds: Res<SoundAssets>) {
    commands.spawn((
        AudioPlayer::new(sounds.music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.22)),
        MusicPlayer,
    ));
}

pub fn apply_mute(muted: Res<Muted>, mut volume: ResMut<GlobalVolume>) {
    volume.volume = if muted.0 {
        Volume::Linear(0.0)
    } else {
        Volume::Linear(0.9)
    };
}

#[derive(Resource, Default)]
pub struct Muted(pub bool);
