mod support;

use ghosted::engine::{Controls, Engine, Settings, Voices};

const SR: f32 = 48_000.0;

fn riff() -> Vec<f32> {
    let (mut input, sr) = support::read_wav(&support::path("di", "riff.wav"));
    assert_eq!(sr, SR as u32);
    input.resize(input.len() + SR as usize / 2, 0.0);
    input
}

/// Runs mono input through the engine in host-sized blocks, as a guitar DI
/// on both channels.
fn render(engine: &mut Engine, input: &[f32], settings: Settings) -> (Vec<f32>, Vec<f32>) {
    engine.set(settings);
    let mut left = input.to_vec();
    let mut right = input.to_vec();
    for (l, r) in left.chunks_mut(512).zip(right.chunks_mut(512)) {
        engine.process(l, r);
    }
    (left, right)
}

fn render_new(sr: f32, input: &[f32], settings: Settings) -> (Vec<f32>, Vec<f32>) {
    render(&mut Engine::new(sr), input, settings)
}

#[test]
fn riff_double_tracked_matches_golden() {
    let (l, r) = render_new(SR, &riff(), Controls::default().settings());
    support::check_golden_stereo("riff-adt", &l, &r, SR as u32);
}

#[test]
fn riff_slapback_matches_golden() {
    let controls = Controls { distance_ms: 110.0, drift: 0.2, spook: 0.3, voices: Voices::Two, ..Controls::default() };
    let (l, r) = render_new(SR, &riff(), controls.settings());
    support::check_golden_stereo("riff-slapback", &l, &r, SR as u32);
}

#[test]
fn default_controls_match_ghost_takes_defaults() {
    let ours = Controls::default().settings();
    let theirs = Settings::default();
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
    assert!(close(ours.wet, theirs.wet));
    assert!(close(ours.base_ms, theirs.base_ms));
    assert!(close(ours.drift_ms, theirs.drift_ms));
    assert!(close(ours.drift_hz, theirs.drift_hz));
    assert!(close(ours.looseness, theirs.looseness));
    assert!(close(ours.jitter_ms, theirs.jitter_ms));
    assert!(close(ours.jitter_dur_ms, theirs.jitter_dur_ms));
    assert!(close(ours.sensitivity, theirs.sensitivity));
    assert_eq!(ours.voices, theirs.voices);
}

#[test]
fn no_haunt_passes_input_through() {
    let input = riff();
    let (l, r) = render_new(SR, &input, Controls { haunt: 0.0, ..Controls::default() }.settings());
    assert_eq!(l, input);
    assert_eq!(r, input);
}

#[test]
fn two_voices_are_hard_left_and_right() {
    let input = riff();
    let full_wet = Controls { haunt: 1.0, voices: Voices::Two, ..Controls::default() }.settings();
    let (l, r) = render_new(SR, &input, full_wet);
    let rms = |x: &[f32]| (x.iter().map(|s| (s * s) as f64).sum::<f64>() / x.len() as f64).sqrt();
    let correlation = l.iter().zip(&r).map(|(a, b)| (a * b) as f64).sum::<f64>() / l.len() as f64 / (rms(&l) * rms(&r));
    assert!(rms(&l) > 0.1 * rms(&input) && rms(&r) > 0.1 * rms(&input));
    assert!(correlation < 0.9, "the two ghosts should differ, correlation {correlation}");
}

#[test]
fn reset_repeats_the_same_render() {
    let input = riff();
    let mut engine = Engine::new(SR);
    let first = render(&mut engine, &input, Settings::default());
    engine.reset();
    let second = render(&mut engine, &input, Settings::default());
    assert_eq!(first, second);
}

#[test]
fn turning_distance_glides_instead_of_jumping() {
    let input = support::tone(220.0, 1.0, SR);
    let mut engine = Engine::new(SR);
    let full_wet = Controls { haunt: 1.0, ..Controls::default() };
    let (mut l, mut r) = (input.clone(), input.clone());
    engine.set(full_wet.settings());
    engine.process(&mut l[..24_000], &mut r[..24_000]);
    engine.set(Controls { distance_ms: 150.0, ..full_wet }.settings());
    engine.process(&mut l[24_000..], &mut r[24_000..]);
    let worst_step = |x: &[f32]| x.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f32::max);
    let before = worst_step(&l[12_000..24_000]);
    let after = worst_step(&l[24_000..25_000]);
    assert!(after < before * 1.5, "jumped by {after} after the change, {before} before");
}

#[test]
fn extreme_settings_stay_finite_at_any_sample_rate() {
    for sr in [44_100.0, 96_000.0, 192_000.0] {
        let input = support::tone(110.0, 1.0, sr);
        let wild = Settings {
            wet: 1.0,
            base_ms: 150.0,
            drift_ms: 100.0,
            drift_hz: 10.0,
            looseness: 1.0,
            jitter_ms: 50.0,
            jitter_dur_ms: 400.0,
            sensitivity: 3.0,
            voices: Voices::Four,
        };
        let (l, r) = render_new(sr, &input, wild);
        assert!(l.iter().chain(&r).all(|s| s.is_finite() && s.abs() < 2.0), "{sr} Hz");
    }
}

#[test]
fn silence_stays_silent() {
    let (l, r) = render_new(SR, &vec![0.0; 48_000], Controls::default().settings());
    assert!(l.iter().chain(&r).all(|&s| s == 0.0));
}
