//! Motor de áudio procedural DSP (`cpal` + comandos lock-free por atômicos).
//!
//! Executa síntese em tempo real sem buffers pesados em RAM e com fallback
//! silencioso caso o ambiente não possua dispositivo de saída de áudio.

pub mod dsp;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use estudy_domain::DspTrack;

pub use dsp::{DspSynthesizer, TARGET_RMS_MINUS_16_LUFS};

/// Fotografia imutável dos comandos atuais do canal de controle lock-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioCommandSnapshot {
    /// Paisagem sonora ativa.
    pub track: DspTrack,
    /// Volume em porcentagem (`0..=100`).
    pub volume_percent: u8,
    /// Mudo explícito.
    pub muted: bool,
    /// Reprodução habilitada pelo timer ou usuário.
    pub playing: bool,
}

/// Canal de controle sem locks (SPSC/atômicos) lido diretamente pelo callback de áudio.
#[derive(Debug)]
pub struct AudioCommandState {
    track_code: AtomicU8,
    volume_percent: AtomicU8,
    muted: AtomicBool,
    playing: AtomicBool,
}

impl Default for AudioCommandState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCommandState {
    /// Inicializa o controle lock-free em modo silencioso e desmutado (`70%`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            track_code: AtomicU8::new(track_to_u8(DspTrack::BrownRain)),
            volume_percent: AtomicU8::new(70),
            muted: AtomicBool::new(false),
            playing: AtomicBool::new(false),
        }
    }

    /// Seleciona a paisagem sonora sem alocar memória.
    pub fn set_track(&self, track: DspTrack) {
        self.track_code.store(track_to_u8(track), Ordering::Relaxed);
    }

    /// Ajusta o volume (`0..=100`).
    pub fn set_volume_percent(&self, percent: u8) {
        self.volume_percent
            .store(percent.min(100), Ordering::Relaxed);
    }

    /// Define estado de mudo.
    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    /// Liga ou pausa a saída sonora do motor DSP.
    pub fn set_playing(&self, playing: bool) {
        self.playing.store(playing, Ordering::Relaxed);
    }

    /// Lê de forma lock-free o estado atual de controle.
    #[must_use]
    pub fn snapshot(&self) -> AudioCommandSnapshot {
        AudioCommandSnapshot {
            track: u8_to_track(self.track_code.load(Ordering::Relaxed)),
            volume_percent: self.volume_percent.load(Ordering::Relaxed),
            muted: self.muted.load(Ordering::Relaxed),
            playing: self.playing.load(Ordering::Relaxed),
        }
    }
}

/// Controlador de alto nível do stream de áudio do eStudy.
pub struct AudioController {
    commands: Arc<AudioCommandState>,
    _stream: Option<Stream>,
    active_backend: &'static str,
}

impl std::fmt::Debug for AudioController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioController")
            .field("active_backend", &self.active_backend)
            .field("commands", &self.commands.snapshot())
            .finish_non_exhaustive()
    }
}

impl AudioController {
    /// Inicializa o dispositivo de áudio padrão via `cpal`, usando fallback silencioso se indisponível.
    #[must_use]
    pub fn try_start() -> Self {
        let commands = Arc::new(AudioCommandState::new());
        match build_cpal_stream(&commands) {
            Ok(stream) => {
                tracing::info!(
                    target: "estudy.audio",
                    backend = "cpal",
                    "motor de áudio procedural DSP inicializado"
                );
                Self {
                    commands,
                    _stream: Some(stream),
                    active_backend: "cpal",
                }
            }
            Err(reason) => {
                tracing::warn!(
                    target: "estudy.audio",
                    backend = "silent_fallback",
                    reason = %reason,
                    "dispositivo de áudio indisponível; operando em fallback silencioso"
                );
                Self {
                    commands,
                    _stream: None,
                    active_backend: "silent_fallback",
                }
            }
        }
    }

    /// Inicializa o controlador sem abrir dispositivo físico (ideal para testes e CI headless).
    #[must_use]
    pub fn start_silent_for_test() -> Self {
        Self {
            commands: Arc::new(AudioCommandState::new()),
            _stream: None,
            active_backend: "silent_test",
        }
    }

    /// Retorna referência compartilhada ao canal atômico de comandos.
    #[must_use]
    pub fn commands(&self) -> &AudioCommandState {
        &self.commands
    }

    /// Atualiza todo o estado de áudio de forma lock-free em uma única chamada.
    pub fn apply(&self, track: DspTrack, volume_percent: u8, muted: bool, playing: bool) {
        self.commands.set_track(track);
        self.commands.set_volume_percent(volume_percent);
        self.commands.set_muted(muted);
        self.commands.set_playing(playing);
    }

    /// Nome do backend ativo (`"cpal"`, `"silent_fallback"` ou `"silent_test"`).
    #[must_use]
    pub const fn active_backend(&self) -> &'static str {
        self.active_backend
    }
}

fn build_cpal_stream(commands: &Arc<AudioCommandState>) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "nenhum dispositivo de saída padrão".to_string())?;
    let supported = device
        .default_output_config()
        .map_err(|err| format!("erro ao consultar configuração de áudio: {err}"))?;

    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let sample_rate = config.sample_rate;
    let channels = usize::from(config.channels.max(1));

    let stream = match sample_format {
        SampleFormat::F32 => build_typed_stream::<f32>(
            &device,
            &config,
            Arc::clone(commands),
            sample_rate,
            channels,
            |sample| sample,
        )?,
        SampleFormat::I16 => build_typed_stream::<i16>(
            &device,
            &config,
            Arc::clone(commands),
            sample_rate,
            channels,
            f32_to_i16,
        )?,
        SampleFormat::U16 => build_typed_stream::<u16>(
            &device,
            &config,
            Arc::clone(commands),
            sample_rate,
            channels,
            f32_to_u16,
        )?,
        other => {
            return Err(format!("formato de amostra não suportado: {other:?}"));
        }
    };

    stream
        .play()
        .map_err(|err| format!("falha ao iniciar playback cpal: {err}"))?;
    Ok(stream)
}

fn build_typed_stream<S>(
    device: &Device,
    config: &StreamConfig,
    commands: Arc<AudioCommandState>,
    sample_rate: u32,
    channels: usize,
    convert: fn(f32) -> S,
) -> Result<Stream, String>
where
    S: cpal::SizedSample + 'static,
{
    let mut synth = DspSynthesizer::new(sample_rate);
    device
        .build_output_stream(
            *config,
            move |data: &mut [S], _: &cpal::OutputCallbackInfo| {
                let snap = commands.snapshot();
                let active_track = if snap.playing && !snap.muted {
                    snap.track
                } else {
                    DspTrack::Silence
                };
                let volume = f32::from(snap.volume_percent) / 100.0;

                for frame in data.chunks_mut(channels) {
                    let (left, right) = synth.next_stereo_frame(active_track, volume);
                    for (ch_idx, out) in frame.iter_mut().enumerate() {
                        let sample = if ch_idx % 2 == 0 { left } else { right };
                        *out = convert(sample);
                    }
                }
            },
            |err| {
                tracing::error!(target: "estudy.audio", error = %err, "erro no stream cpal");
            },
            None,
        )
        .map_err(|err| format!("falha ao construir stream cpal: {err}"))
}

#[allow(clippy::cast_possible_truncation)]
fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn f32_to_u16(sample: f32) -> u16 {
    ((sample.clamp(-1.0, 1.0) * 0.5 + 0.5) * f32::from(u16::MAX)) as u16
}

const fn track_to_u8(track: DspTrack) -> u8 {
    match track {
        DspTrack::Silence => 0,
        DspTrack::BrownRain => 1,
        DspTrack::PinkWind => 2,
        DspTrack::WarmDrone => 3,
        DspTrack::BinauralAlpha => 4,
    }
}

const fn u8_to_track(code: u8) -> DspTrack {
    match code {
        1 => DspTrack::BrownRain,
        2 => DspTrack::PinkWind,
        3 => DspTrack::WarmDrone,
        4 => DspTrack::BinauralAlpha,
        _ => DspTrack::Silence,
    }
}
