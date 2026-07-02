use std::collections::HashMap;
use std::time::Duration;

use glam::Vec3;
use kira::listener::ListenerHandle;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::PlaybackState;
use kira::track::TrackBuilder;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Panning, PlaybackRate, Tween};
use kira::{Decibels, track::TrackHandle};
use mint::{Quaternion, Vector3};
use rand::RngExt;

use crate::audio::api::{
    AudioError, LoopHandle, ONE_SHOT_POOL_SIZE, PitchRange, PlayParams, SoundData, SoundSource,
    VolumeLevels, DEFAULT_FADE,
};
use crate::audio::mixer::MixerState;
use crate::audio::presets::SoundPresetRegistry;
use crate::audio::spatial::{spatial_attenuation, ListenerState};
use crate::audio::volume::linear_to_decibels;
use crate::resources::assets::SOUNDS;
use crate::resources::manager::ResourceManager;
use crate::resources::SoundUid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MusicSlot {
    A,
    B,
}

impl MusicSlot {
    fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

struct OneShotSlot {
    handle: Option<StaticSoundHandle>,
}

struct LoopChannel {
    sound: Option<StaticSoundHandle>,
    clip: Option<SoundUid>,
    volume: f32,
    pitch_range: PitchRange,
    auto_random_pitch: bool,
    manual_pitch: Option<f32>,
    last_position: f64,
    spatial: Option<crate::audio::api::SpatialParams>,
}

struct MusicBus {
    handle: Option<StaticSoundHandle>,
    clip: Option<SoundUid>,
}

pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    sfx_track: TrackHandle,
    music_track: TrackHandle,
    #[allow(dead_code)]
    listener_handle: ListenerHandle,
    clips: HashMap<SoundUid, StaticSoundData>,
    presets: SoundPresetRegistry,
    one_shots: [OneShotSlot; ONE_SHOT_POOL_SIZE],
    loops: HashMap<LoopHandle, LoopChannel>,
    next_loop_handle: LoopHandle,
    music_a: MusicBus,
    music_b: MusicBus,
    active_music: MusicSlot,
    current_track: Option<SoundUid>,
    music_looping: bool,
    volumes: VolumeLevels,
    listener: ListenerState,
    mixer: MixerState,
}

impl AudioEngine {
    pub fn new(resources: &ResourceManager) -> Result<Self, AudioError> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|error| AudioError::Backend(error.to_string()))?;

        let sfx_track = manager
            .add_sub_track(TrackBuilder::new())
            .map_err(|error| AudioError::Backend(error.to_string()))?;
        let music_track = manager
            .add_sub_track(TrackBuilder::new())
            .map_err(|error| AudioError::Backend(error.to_string()))?;
        let listener_handle = manager
            .add_listener(
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Quaternion {
                    v: Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    s: 1.0,
                },
            )
            .map_err(|error| AudioError::Backend(error.to_string()))?;

        let mut clips = HashMap::new();
        for entry in SOUNDS {
            let path = resources.assets_root().join(entry.path);
            let data = StaticSoundData::from_file(&path)
                .map_err(|error| AudioError::Backend(error.to_string()))?;
            clips.insert(entry.uid, data);
        }

        let presets = SoundPresetRegistry::from_resources(resources)?;

        let mut engine = Self {
            manager,
            sfx_track,
            music_track,
            listener_handle,
            clips,
            presets,
            one_shots: std::array::from_fn(|_| OneShotSlot { handle: None }),
            loops: HashMap::new(),
            next_loop_handle: 1,
            music_a: MusicBus {
                handle: None,
                clip: None,
            },
            music_b: MusicBus {
                handle: None,
                clip: None,
            },
            active_music: MusicSlot::A,
            current_track: None,
            music_looping: false,
            volumes: VolumeLevels {
                master: 1.0,
                music: 1.0,
                sfx: 1.0,
            },
            listener: ListenerState::default(),
            mixer: MixerState::default(),
        };
        engine.refresh_bus_volumes();
        Ok(engine)
    }

    pub fn update(&mut self, _dt: f32) {
        self.recycle_one_shots();
        self.update_loop_channels();
    }

    pub fn mixer(&self) -> &MixerState {
        &self.mixer
    }

    pub fn mixer_mut(&mut self) -> &mut MixerState {
        &mut self.mixer
    }

    pub fn set_listener(&mut self, position: Vec3, forward: Vec3) {
        self.listener.position = position;
        self.listener.forward = forward;
        let _ = self.listener_handle.set_position(
            Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Tween::default(),
        );
        let orientation =
            glam::Quat::from_rotation_arc(Vec3::NEG_Z, forward.normalize_or_zero());
        let _ = self.listener_handle.set_orientation(
            Quaternion {
                v: Vector3 {
                    x: orientation.x,
                    y: orientation.y,
                    z: orientation.z,
                },
                s: orientation.w,
            },
            Tween::default(),
        );
    }

    // --- Volume ---

    pub fn set_master_volume(&mut self, volume: f32) {
        self.volumes.master = volume.clamp(0.0, 1.0);
        self.refresh_bus_volumes();
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.volumes.music = volume.clamp(0.0, 1.0);
        self.refresh_bus_volumes();
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.volumes.sfx = volume.clamp(0.0, 1.0);
        self.refresh_bus_volumes();
    }

    pub fn master_volume(&self) -> f32 {
        self.volumes.master
    }

    pub fn music_volume(&self) -> f32 {
        self.volumes.music
    }

    pub fn sfx_volume(&self) -> f32 {
        self.volumes.sfx
    }

    // --- One-shot SFX ---

    pub fn play(&mut self, source: SoundSource, params: PlayParams) {
        let sound = match self.presets.resolve(source) {
            Ok(sound) => sound,
            Err(error) => {
                eprintln!("[audio] {error}");
                return;
            }
        };

        if self.clips.get(&sound.clip).is_none() {
            eprintln!("[audio] unknown clip uid {}", sound.clip.index());
            return;
        }

        let pitch = random_pitch(params.pitch.unwrap_or(sound.pitch));
        let volume = params.volume.unwrap_or(1.0) * sound.volume;
        let Some(data) = self.build_sfx_data(sound.clip, volume, pitch, params.spatial, false) else {
            return;
        };

        let slot = self.one_shots.iter_mut().find(|slot| {
            slot.handle
                .as_ref()
                .map(|handle| handle.state() != PlaybackState::Playing)
                .unwrap_or(true)
        });

        let Some(slot) = slot else {
            eprintln!(
                "[audio] one-shot pool full ({ONE_SHOT_POOL_SIZE} slots), skipping sound"
            );
            return;
        };

        match self.sfx_track.play(data) {
            Ok(handle) => slot.handle = Some(handle),
            Err(error) => eprintln!("[audio] failed to play one-shot: {error}"),
        }
    }

    // --- Looping SFX ---

    pub fn create_loop_handle(&mut self) -> LoopHandle {
        let handle = self.next_loop_handle;
        self.next_loop_handle += 1;
        self.loops.insert(
            handle,
            LoopChannel {
                sound: None,
                clip: None,
                volume: 1.0,
                pitch_range: PitchRange::UNITY,
                auto_random_pitch: false,
                manual_pitch: None,
                last_position: 0.0,
                spatial: None,
            },
        );
        handle
    }

    pub fn release_loop_handle(&mut self, handle: LoopHandle, fade: Option<Duration>) {
        if let Some(mut channel) = self.loops.remove(&handle) {
            stop_loop_channel(&mut channel, fade.unwrap_or(DEFAULT_FADE));
        }
    }

    pub fn play_loop(
        &mut self,
        handle: LoopHandle,
        source: SoundSource,
        params: PlayParams,
        fade_in: Option<Duration>,
    ) {
        let sound = match self.presets.resolve(source) {
            Ok(sound) => sound,
            Err(error) => {
                eprintln!("[audio] {error}");
                return;
            }
        };

        let clip = sound.clip;
        let volume = params.volume.unwrap_or(1.0) * sound.volume;
        let pitch_range = params.pitch.unwrap_or(sound.pitch);
        let spatial = params.spatial;
        let fade = fade_in.unwrap_or(DEFAULT_FADE);

        let should_start = {
            let Some(channel) = self.loops.get_mut(&handle) else {
                eprintln!("[audio] invalid loop handle {handle}");
                return;
            };

            channel.volume = volume;
            channel.pitch_range = pitch_range;
            channel.spatial = spatial;
            channel.auto_random_pitch = channel.manual_pitch.is_none();

            let same_clip_playing = channel.clip == Some(clip)
                && channel.sound.as_ref().is_some_and(|handle| {
                    matches!(
                        handle.state(),
                        PlaybackState::Playing | PlaybackState::Paused
                    )
                });

            if same_clip_playing {
                if let Some(existing) = channel.sound.as_mut() {
                    apply_loop_params(
                        self.listener,
                        channel.volume,
                        channel.manual_pitch,
                        channel.pitch_range,
                        channel.spatial,
                        existing,
                    );
                }
                false
            } else {
                stop_loop_channel(channel, Duration::ZERO);
                channel.clip = Some(clip);
                true
            }
        };

        if !should_start {
            return;
        }

        let (manual_pitch, channel_volume, channel_spatial) = {
            let channel = self.loops.get(&handle).expect("loop handle exists");
            (channel.manual_pitch, channel.volume, channel.spatial)
        };
        let pitch = manual_pitch.unwrap_or_else(|| random_pitch(pitch_range));
        let Some(mut data) = self.build_sfx_data(clip, channel_volume, pitch, channel_spatial, true)
        else {
            return;
        };
        if !fade.is_zero() {
            data = data.fade_in_tween(Tween {
                duration: fade,
                ..Default::default()
            });
        }

        match self.sfx_track.play(data) {
            Ok(sound_handle) => {
                if let Some(channel) = self.loops.get_mut(&handle) {
                    channel.last_position = 0.0;
                    channel.sound = Some(sound_handle);
                }
            }
            Err(error) => eprintln!("[audio] failed to play loop: {error}"),
        }
    }

    pub fn update_loop(
        &mut self,
        handle: LoopHandle,
        should_play: bool,
        source: SoundSource,
        params: PlayParams,
    ) {
        let Some(channel) = self.loops.get_mut(&handle) else {
            eprintln!("[audio] invalid loop handle {handle}");
            return;
        };

        if should_play {
            self.play_loop(handle, source, params, Some(DEFAULT_FADE));
            return;
        }

        if channel.sound.is_some() {
            stop_loop_channel(channel, DEFAULT_FADE);
        }
    }

    pub fn stop_loop(&mut self, handle: LoopHandle, fade: Option<Duration>) {
        let Some(channel) = self.loops.get_mut(&handle) else {
            return;
        };
        stop_loop_channel(channel, fade.unwrap_or(DEFAULT_FADE));
    }

    pub fn stop_all_loops(&mut self, fade: Option<Duration>) {
        let fade = fade.unwrap_or(DEFAULT_FADE);
        for channel in self.loops.values_mut() {
            stop_loop_channel(channel, fade);
        }
    }

    pub fn is_loop_playing(&self, handle: LoopHandle) -> bool {
        self.loops
            .get(&handle)
            .and_then(|channel| channel.sound.as_ref())
            .map(|handle| handle.state() == PlaybackState::Playing)
            .unwrap_or(false)
    }

    pub fn set_loop_volume(&mut self, handle: LoopHandle, volume: f32, fade: Option<Duration>) {
        let Some(channel) = self.loops.get_mut(&handle) else {
            return;
        };
        channel.volume = volume.clamp(0.0, 1.0);
        if let Some(sound) = channel.sound.as_mut() {
            let tween = fade_tween(fade);
            let db = linear_to_decibels(channel.volume);
            sound.set_volume(db, tween);
        }
    }

    pub fn set_loop_pitch(&mut self, handle: LoopHandle, pitch: f32) {
        let Some(channel) = self.loops.get_mut(&handle) else {
            return;
        };
        channel.manual_pitch = Some(pitch);
        channel.auto_random_pitch = false;
        if let Some(sound) = channel.sound.as_mut() {
            sound.set_playback_rate(PlaybackRate(f64::from(pitch)), Tween::default());
        }
    }

    // --- Music ---

    pub fn play_music(
        &mut self,
        source: SoundSource,
        r#loop: bool,
        crossfade: Option<Duration>,
    ) {
        let sound = match self.presets.resolve(source) {
            Ok(sound) => sound,
            Err(error) => {
                eprintln!("[audio] {error}");
                return;
            }
        };

        let fade = crossfade.unwrap_or(Duration::ZERO);
        if fade.is_zero() {
            self.stop_music_immediate();
            self.start_music_on_slot(self.active_music, sound, r#loop, Duration::ZERO);
            self.current_track = Some(sound.clip);
            self.music_looping = r#loop;
            return;
        }

        let target = self.active_music.other();
        self.start_music_on_slot(target, sound, r#loop, fade);
        self.fade_out_slot(self.active_music, fade);
        self.active_music = target;
        self.current_track = Some(sound.clip);
        self.music_looping = r#loop;
    }

    pub fn crossfade_music(
        &mut self,
        source: SoundSource,
        duration: Duration,
        r#loop: bool,
    ) {
        self.play_music(source, r#loop, Some(duration));
    }

    pub fn stop_music(&mut self, fade: Option<Duration>) {
        let fade = fade.unwrap_or(DEFAULT_FADE);
        if fade.is_zero() {
            self.stop_music_immediate();
        } else {
            self.fade_out_slot(self.active_music, fade);
            self.current_track = None;
        }
    }

    pub fn set_music_loop(&mut self, on: bool) {
        self.music_looping = on;
        let slot = self.active_music;
        let clip = self.music_bus(slot).clip;
        if let Some(clip) = clip {
            let volume = 1.0;
            self.stop_music_immediate_on_slot(slot);
            let sound = SoundData {
                clip,
                volume,
                pitch: PitchRange::UNITY,
            };
            self.start_music_on_slot(slot, sound, on, Duration::ZERO);
        }
    }

    pub fn is_music_playing(&self) -> bool {
        self.music_bus(self.active_music)
            .handle
            .as_ref()
            .map(|handle| handle.state() == PlaybackState::Playing)
            .unwrap_or(false)
    }

    pub fn current_music_track(&self) -> Option<SoundUid> {
        self.current_track
    }

    // --- internals ---

    fn refresh_bus_volumes(&mut self) {
        self.manager
            .main_track()
            .set_volume(linear_to_decibels(self.volumes.master), Tween::default());
        self.sfx_track
            .set_volume(linear_to_decibels(self.volumes.sfx), Tween::default());
        self.music_track
            .set_volume(linear_to_decibels(self.volumes.music), Tween::default());
    }

    fn recycle_one_shots(&mut self) {
        for slot in &mut self.one_shots {
            if slot
                .handle
                .as_ref()
                .is_some_and(|handle| handle.state() == PlaybackState::Stopped)
            {
                slot.handle = None;
            }
        }
    }

    fn update_loop_channels(&mut self) {
        let listener = self.listener;
        for channel in self.loops.values_mut() {
            let Some(handle) = channel.sound.as_mut() else {
                continue;
            };

            if channel.auto_random_pitch {
                let position = handle.position();
                if position < channel.last_position - 0.05 {
                    let pitch = random_pitch(channel.pitch_range);
                    handle.set_playback_rate(PlaybackRate(f64::from(pitch)), Tween::default());
                }
                channel.last_position = position;
            }

            if channel.spatial.is_some() {
                apply_loop_params(
                    listener,
                    channel.volume,
                    channel.manual_pitch,
                    channel.pitch_range,
                    channel.spatial,
                    handle,
                );
            }
        }
    }

    fn build_sfx_data(
        &self,
        clip: SoundUid,
        volume: f32,
        pitch: f32,
        spatial: Option<crate::audio::api::SpatialParams>,
        looping: bool,
    ) -> Option<StaticSoundData> {
        let base = self.clips.get(&clip)?;
        let mut linear_volume = volume;
        let mut pan = 0.5f32;
        if let Some(spatial) = spatial {
            let att = spatial_attenuation(self.listener, spatial);
            linear_volume *= att.volume_linear;
            pan = att.pan;
        }

        let mut data = base
            .clone()
            .volume(Decibels::from(linear_to_decibels(linear_volume)))
            .playback_rate(f64::from(pitch))
            .panning(Panning(pan));

        if looping {
            data = data.loop_region(..);
        }

        Some(data)
    }

    fn music_bus(&self, slot: MusicSlot) -> &MusicBus {
        match slot {
            MusicSlot::A => &self.music_a,
            MusicSlot::B => &self.music_b,
        }
    }

    fn music_bus_mut(&mut self, slot: MusicSlot) -> &mut MusicBus {
        match slot {
            MusicSlot::A => &mut self.music_a,
            MusicSlot::B => &mut self.music_b,
        }
    }

    fn start_music_on_slot(
        &mut self,
        slot: MusicSlot,
        sound: SoundData,
        r#loop: bool,
        fade_in: Duration,
    ) {
        self.stop_music_immediate_on_slot(slot);
        let pitch = random_pitch(sound.pitch);
        let Some(base) = self.clips.get(&sound.clip) else {
            eprintln!("[audio] unknown music clip uid {}", sound.clip.index());
            return;
        };
        let mut data = base
            .clone()
            .volume(Decibels::from(linear_to_decibels(sound.volume)))
            .playback_rate(f64::from(pitch));
        if r#loop {
            data = data.loop_region(..);
        }
        if !fade_in.is_zero() {
            data = data.fade_in_tween(Tween {
                duration: fade_in,
                ..Default::default()
            });
        }

        match self.music_track.play(data) {
            Ok(handle) => {
                let bus = self.music_bus_mut(slot);
                bus.handle = Some(handle);
                bus.clip = Some(sound.clip);
            }
            Err(error) => eprintln!("[audio] failed to play music: {error}"),
        }
    }

    fn fade_out_slot(&mut self, slot: MusicSlot, fade: Duration) {
        if let Some(handle) = self.music_bus_mut(slot).handle.as_mut() {
            handle.set_volume(Decibels::from(linear_to_decibels(0.0)), fade_tween(Some(fade)));
            let _ = handle.stop(fade_tween(Some(fade)));
        }
        self.music_bus_mut(slot).handle = None;
        self.music_bus_mut(slot).clip = None;
    }

    fn stop_music_immediate_on_slot(&mut self, slot: MusicSlot) {
        if let Some(mut handle) = self.music_bus_mut(slot).handle.take() {
            let _ = handle.stop(Tween::default());
        }
        self.music_bus_mut(slot).clip = None;
    }

    fn stop_music_immediate(&mut self) {
        self.stop_music_immediate_on_slot(MusicSlot::A);
        self.stop_music_immediate_on_slot(MusicSlot::B);
        self.current_track = None;
    }
}

fn apply_loop_params(
    listener: ListenerState,
    volume: f32,
    manual_pitch: Option<f32>,
    pitch_range: PitchRange,
    spatial: Option<crate::audio::api::SpatialParams>,
    handle: &mut StaticSoundHandle,
) {
    let pitch = manual_pitch.unwrap_or_else(|| pitch_range.min);
    let db = linear_to_decibels(volume);
    handle.set_volume(db, Tween::default());
    handle.set_playback_rate(PlaybackRate(f64::from(pitch)), Tween::default());
    if let Some(spatial) = spatial {
        let att = spatial_attenuation(listener, spatial);
        handle.set_panning(Panning(att.pan), Tween::default());
        handle.set_volume(
            linear_to_decibels(volume * att.volume_linear),
            Tween::default(),
        );
    }
}

fn random_pitch(range: PitchRange) -> f32 {
    if range.min >= range.max {
        range.min
    } else {
        rand::rng().random_range(range.min..=range.max)
    }
}

fn stop_loop_channel(channel: &mut LoopChannel, fade: Duration) {
    if let Some(mut handle) = channel.sound.take() {
        let tween = fade_tween(Some(fade));
        let _ = handle.stop(tween);
    }
    channel.clip = None;
    channel.last_position = 0.0;
}

fn fade_tween(fade: Option<Duration>) -> Tween {
    fade.map(|duration| Tween {
        duration,
        ..Default::default()
    })
    .unwrap_or_default()
}
