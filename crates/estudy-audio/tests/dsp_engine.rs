//! Testes de contrato do sintetizador procedural DSP e controle lock-free.

#![allow(clippy::expect_used)]

use estudy_audio::{AudioCommandState, AudioController, DspSynthesizer, TARGET_RMS_MINUS_16_LUFS};
use estudy_domain::DspTrack;

#[test]
fn should_render_silence_and_bounded_minus_16_lufs_procedural_tracks() {
    let mut buffer = vec![0.0_f32; 48_000 * 2];

    let mut synth = DspSynthesizer::new(48_000);
    synth.render_interleaved(DspTrack::Silence, 1.0, &mut buffer);
    assert!(buffer.iter().all(|&sample| sample == 0.0));

    for track in [
        DspTrack::BrownRain,
        DspTrack::PinkWind,
        DspTrack::WarmDrone,
        DspTrack::BinauralAlpha,
    ] {
        buffer.fill(0.0);
        synth.render_interleaved(track, 1.0, &mut buffer);
        let peak = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        #[allow(clippy::cast_precision_loss)]
        let rms = (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt();

        assert!(
            buffer.iter().all(|s| s.is_finite()),
            "amostras inválidas na faixa {track:?}"
        );
        assert!(
            peak > 0.01 && peak <= 0.90,
            "pico fora da faixa {track:?}: {peak}"
        );
        assert!(
            (rms - TARGET_RMS_MINUS_16_LUFS).abs() <= 0.10,
            "RMS fora da meta -16 LUFS em {track:?}: {rms}"
        );
    }

    buffer.fill(0.0);
    synth.render_interleaved(DspTrack::BinauralAlpha, 1.0, &mut buffer);
    let differs_lr = buffer
        .chunks_exact(2)
        .any(|pair| (pair[0] - pair[1]).abs() > 0.005);
    assert!(
        differs_lr,
        "BinauralAlpha deve gerar batimento L/R distinto"
    );
}

#[test]
fn should_update_lock_free_audio_commands_and_start_gracefully() {
    let state = AudioCommandState::new();
    state.set_track(DspTrack::BrownRain);
    state.set_volume_percent(65);
    state.set_muted(false);
    state.set_playing(true);

    let snap = state.snapshot();
    assert_eq!(snap.track, DspTrack::BrownRain);
    assert_eq!(snap.volume_percent, 65);
    assert!(!snap.muted);
    assert!(snap.playing);

    let controller = AudioController::start_silent_for_test();
    controller.apply(DspTrack::PinkWind, 80, false, true);
    assert_eq!(controller.commands().snapshot().track, DspTrack::PinkWind);
}
