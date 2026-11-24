//! Síntese procedural de paisagens sonoras em `-16 LUFS` sem buffers gravados em RAM.

use std::f64::consts::TAU;

use estudy_domain::DspTrack;

/// Amplitude RMS alvo correspondente a aproximadamente `-16 LUFS` (`10^(-16/20)`).
pub const TARGET_RMS_MINUS_16_LUFS: f32 = 0.1585;

/// Estado puro de síntese DSP (sem alocações por amostra).
#[derive(Debug, Clone)]
pub struct DspSynthesizer {
    sample_rate: f64,
    rng_state: u64,
    brown_left: f32,
    brown_right: f32,
    pink_rows: [f32; 8],
    pink_running_sum: f32,
    pink_index: u32,
    lfo_phase: f64,
    drone_phase: f64,
    binaural_phase_left: f64,
    binaural_phase_right: f64,
    rain_drop_env: f32,
    smooth_gain: f32,
}

impl DspSynthesizer {
    /// Inicializa o sintetizador para a taxa de amostragem informada (`Hz`).
    #[must_use]
    pub fn new(sample_rate_hz: u32) -> Self {
        let rate = f64::from(sample_rate_hz.max(8_000));
        Self {
            sample_rate: rate,
            rng_state: 0x5EED_CAFE_BABE_1337,
            brown_left: 0.0,
            brown_right: 0.0,
            pink_rows: [0.0; 8],
            pink_running_sum: 0.0,
            pink_index: 0,
            lfo_phase: 0.0,
            drone_phase: 0.0,
            binaural_phase_left: 0.0,
            binaural_phase_right: 0.0,
            rain_drop_env: 0.0,
            smooth_gain: 0.0,
        }
    }

    /// Preenche um buffer estéreo intercalado (`L, R, L, R...`) sem alocações no heap.
    pub fn render_interleaved(&mut self, track: DspTrack, volume: f32, output: &mut [f32]) {
        let clamped_vol = volume.clamp(0.0, 1.0);
        if track == DspTrack::Silence || clamped_vol <= f32::EPSILON {
            self.smooth_gain = 0.0;
            output.fill(0.0);
            return;
        }

        for pair in output.chunks_exact_mut(2) {
            let (left, right) = self.next_stereo_frame(track, clamped_vol);
            pair[0] = left;
            pair[1] = right;
        }
    }

    /// Gera o próximo par de amostras estéreo `(L, R)` com suavização anti-clique.
    #[allow(clippy::cast_possible_truncation)]
    pub fn next_stereo_frame(&mut self, track: DspTrack, volume: f32) -> (f32, f32) {
        let target_gain = if track == DspTrack::Silence {
            0.0
        } else {
            volume.clamp(0.0, 1.0)
        };
        self.smooth_gain += (target_gain - self.smooth_gain) * 0.002;
        if self.smooth_gain <= 1e-5 && target_gain == 0.0 {
            return (0.0, 0.0);
        }

        let (raw_left, raw_right) = match track {
            DspTrack::Silence => (0.0, 0.0),
            DspTrack::BrownRain => self.sample_brown_rain(),
            DspTrack::PinkWind => self.sample_pink_wind(),
            DspTrack::WarmDrone => self.sample_warm_drone(),
            DspTrack::BinauralAlpha => self.sample_binaural_alpha(),
        };

        let left = (raw_left * self.smooth_gain).tanh().clamp(-0.90, 0.90);
        let right = (raw_right * self.smooth_gain).tanh().clamp(-0.90, 0.90);
        (left, right)
    }

    fn next_white_bipolar(&mut self) -> f32 {
        // Xorshift64 determinístico e extremamente leve para áudio em tempo real.
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        #[allow(clippy::cast_precision_loss)]
        let unit = ((x >> 40) as f32) / 16_777_215.0;
        unit * 2.0 - 1.0
    }

    fn sample_brown_rain(&mut self) -> (f32, f32) {
        let w_left = self.next_white_bipolar();
        let w_right = self.next_white_bipolar();

        self.brown_left = (self.brown_left * 0.985 + w_left * 0.06).clamp(-1.0, 1.0);
        self.brown_right = (self.brown_right * 0.985 + w_right * 0.06).clamp(-1.0, 1.0);

        if self.rng_state.trailing_zeros() >= 10 {
            self.rain_drop_env = 0.08;
        }
        self.rain_drop_env *= 0.996;
        let droplet = w_left * self.rain_drop_env;

        let gain = TARGET_RMS_MINUS_16_LUFS / 0.14;
        (
            (self.brown_left + droplet) * gain,
            (self.brown_right - droplet * 0.5) * gain,
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sample_pink_wind(&mut self) -> (f32, f32) {
        self.pink_index = self.pink_index.wrapping_add(1);
        let bit = (self.pink_index.trailing_zeros() as usize).min(self.pink_rows.len() - 1);
        let prev = self.pink_rows[bit];
        let next = self.next_white_bipolar();
        self.pink_rows[bit] = next;
        self.pink_running_sum += next - prev;

        let white = self.next_white_bipolar();
        let pink = (self.pink_running_sum + white) * 0.11;

        self.lfo_phase = (self.lfo_phase + 0.18 / self.sample_rate) % 1.0;
        let breeze = 0.72 + 0.28 * (self.lfo_phase * TAU).sin() as f32;
        let pan_r = 0.78 + 0.22 * (self.lfo_phase * TAU).cos() as f32;

        let scale = TARGET_RMS_MINUS_16_LUFS / 0.16;
        (pink * breeze * scale, pink * pan_r * scale)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sample_warm_drone(&mut self) -> (f32, f32) {
        self.drone_phase = (self.drone_phase + 110.0 / self.sample_rate) % 1.0;
        self.lfo_phase = (self.lfo_phase + 0.12 / self.sample_rate) % 1.0;

        let fundamental = (self.drone_phase * TAU).sin() as f32;
        let fifth = ((self.drone_phase * 1.5) * TAU).sin() as f32 * 0.55;
        let octave = ((self.drone_phase * 2.0) * TAU).sin() as f32 * 0.25;
        let breath = 0.82 + 0.18 * (self.lfo_phase * TAU).sin() as f32;
        let air = self.next_white_bipolar() * 0.015;

        let mono = (fundamental + fifth + octave) * 0.22 * breath + air;
        let scale = TARGET_RMS_MINUS_16_LUFS / 0.16;
        (mono * scale, mono * scale)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sample_binaural_alpha(&mut self) -> (f32, f32) {
        // 200 Hz no canal esquerdo e 210 Hz no direito produzem batimento alfa suave de 10 Hz.
        self.binaural_phase_left = (self.binaural_phase_left + 200.0 / self.sample_rate) % 1.0;
        self.binaural_phase_right = (self.binaural_phase_right + 210.0 / self.sample_rate) % 1.0;

        let tone_l = (self.binaural_phase_left * TAU).sin() as f32;
        let tone_r = (self.binaural_phase_right * TAU).sin() as f32;
        let warm_noise = self.next_white_bipolar() * 0.025;

        let scale = TARGET_RMS_MINUS_16_LUFS / 0.20;
        (
            (tone_l * 0.26 + warm_noise) * scale,
            (tone_r * 0.26 + warm_noise) * scale,
        )
    }
}
