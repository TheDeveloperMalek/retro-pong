use bevy::prelude::*;
use bevy::audio::Volume;
use crate::{game::components::PowerUpKind, persistence::Settings};

#[derive(Component)]
struct MusicTrack;

#[derive(Component)]
struct SfxVoice;

const MAX_CONCURRENT_SFX_VOICES: usize = 4;

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (refresh_music, limit_sfx_voices));
    }
}

fn refresh_music(
    mut commands: Commands, asset_server: Res<AssetServer>, settings: Res<Settings>,
    tracks: Query<Entity, With<MusicTrack>>,
) {
    
    
    if !settings.is_changed() { return; }
    for entity in &tracks { commands.entity(entity).despawn(); }
    if settings.music_enabled { spawn_music(&mut commands, &asset_server, settings.music_volume); }
}

fn spawn_music(commands: &mut Commands, asset_server: &AssetServer, volume: f32) {
    commands.spawn((
        MusicTrack,
        AudioPlayer::new(asset_server.load("audio/arcade_theme.wav")),
        PlaybackSettings { volume: Volume::Linear(volume.clamp(0.0, 1.0)), ..PlaybackSettings::LOOP },
    ));
}


fn limit_sfx_voices(mut commands: Commands, voices: Query<Entity, With<SfxVoice>>) {
    let active: Vec<Entity> = voices.iter().collect();
    let excess = active.len().saturating_sub(MAX_CONCURRENT_SFX_VOICES);
    
    for entity in active.into_iter().take(excess) {
        commands.entity(entity).despawn();
    }
}

#[derive(Clone, Copy)]
pub enum Sfx {
    Menu,
    Hit,
    Score,
    PowerUp,
    PowerUpRespawn,
    PowerUpCollect(PowerUpKind),
    Win,
    Lose,
}

pub fn play_sfx(commands: &mut Commands, asset_server: &AssetServer, settings: &Settings, sound: Sfx) {
    if !settings.sfx_enabled { return; }
    let file = match sound {
        Sfx::Menu => "audio/menu.wav",
        Sfx::Hit => "audio/hit.wav",
        Sfx::Score => "audio/score.wav",
        Sfx::PowerUp => "audio/powerup.wav",
        Sfx::PowerUpRespawn => "audio/powerup_respawn.wav",
        Sfx::PowerUpCollect(kind) => match kind {
            PowerUpKind::Speed => "audio/powerup_collect_speed.wav",
            PowerUpKind::Slow => "audio/powerup_collect_slow.wav",
            PowerUpKind::Enlarge => "audio/powerup_collect_enlarge.wav",
            PowerUpKind::DoubleBall => "audio/powerup_collect_double_ball.wav",
            PowerUpKind::Fireball => "audio/powerup_collect_fireball.wav",
            PowerUpKind::Snowball => "audio/powerup_collect_snowball.wav",
            PowerUpKind::ReversalBall => "audio/powerup_collect_reversal.wav",
            PowerUpKind::StickyBall => "audio/powerup_collect_sticky.wav",
            PowerUpKind::BallStorm => "audio/powerup_collect_ball_storm.wav",
            PowerUpKind::BoomerangBall => "audio/powerup_collect_boomerang.wav",
            PowerUpKind::MagneticField | PowerUpKind::TeleportBall | PowerUpKind::EnergyShield | PowerUpKind::Overdrive | PowerUpKind::PhantomBall => "audio/powerup.wav",
        },
        Sfx::Win => "audio/victory.wav",
        Sfx::Lose => "audio/defeat.wav",
    };
    commands.spawn((
        SfxVoice,
        AudioPlayer::new(asset_server.load(file)),
        PlaybackSettings { volume: Volume::Linear(settings.sfx_volume.clamp(0.0, 1.0)), ..PlaybackSettings::DESPAWN },
    ));
}

