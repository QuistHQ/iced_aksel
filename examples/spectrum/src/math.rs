use std::f32::consts::PI;

use crate::{MAX_DB, MAX_FREQ, MIN_DB, MIN_FREQ, SMOOTHING_BAND_SHAPE};

pub fn fractional_width(freq: f64) -> f64 {
    let freq = freq.clamp(MIN_FREQ, MAX_FREQ);
    let log_freq = freq.log10();

    if freq <= SMOOTHING_BAND_SHAPE[0].0 {
        return SMOOTHING_BAND_SHAPE[0].1;
    }

    for window in SMOOTHING_BAND_SHAPE.windows(2) {
        let (f0, w0) = window[0];
        let (f1, w1) = window[1];

        if freq <= f1 {
            if (f1 - f0).abs() < f64::EPSILON {
                return w1;
            }

            let t = (log_freq - f0.log10()) / (f1.log10() - f0.log10());
            return (w1 - w0).mul_add(t.clamp(0.0, 1.0), w0);
        }
    }

    SMOOTHING_BAND_SHAPE.last().map(|(_, w)| *w).unwrap_or(0.15)
}

pub fn sample_fractional_octave(
    magnitudes: &[f32],
    freq: f64,
    sample_rate: f64,
    width_octaves: f64,
    tilt: f64,
) -> f64 {
    if magnitudes.len() < 2 || sample_rate <= 0.0 {
        return MIN_DB;
    }

    let nyquist = sample_rate * 0.5;
    let center = freq.clamp(MIN_FREQ, nyquist);
    let half_span = (width_octaves * 0.5).max(0.05);

    let sample_count = ((width_octaves * 96.0).round() as usize).clamp(12, 150);
    let mut weighted = 0.0;
    let mut weight_sum = 0.0;

    for i in 0..sample_count {
        let t = (i as f64 + 0.5) / sample_count as f64;
        let offset = width_octaves.mul_add(t, -half_span);
        let sample_freq = center * offset.exp2();
        if !(MIN_FREQ..=nyquist).contains(&sample_freq) {
            continue;
        }

        let dist = offset / half_span;
        let weight = (-0.5 * dist * dist).exp();
        let amplitude = interpolate_fft_bin(magnitudes, sample_freq, sample_rate);

        weighted += amplitude * weight;
        weight_sum += weight;
    }

    let amplitude = if weight_sum > 0.0 {
        weighted / weight_sum
    } else {
        0.0
    };

    let tilt_db = (freq.log2() - 20f64.log2()) * tilt;
    (amplitude_to_db(amplitude) + tilt_db).clamp(MIN_DB, MAX_DB)
}

pub fn amplitude_to_db(value: f64) -> f64 {
    20.0 * value.max(1e-9).log10()
}

pub fn interpolate_fft_bin(magnitudes: &[f32], freq: f64, sample_rate: f64) -> f64 {
    if magnitudes.len() < 2 {
        return 0.0;
    }

    let fft_size = (magnitudes.len() - 1) * 2;
    if fft_size == 0 || sample_rate == 0.0 {
        return 0.0;
    }

    let nyquist = sample_rate * 0.5;
    let freq = freq.clamp(MIN_FREQ, nyquist);
    let bin_hz = sample_rate / fft_size as f64;
    let index = freq / bin_hz;

    let base = index.floor() as usize;
    if base >= magnitudes.len() - 1 {
        return magnitudes[magnitudes.len() - 1] as f64;
    }

    let next = base + 1;
    let frac = index - base as f64;
    let lower = magnitudes[base] as f64;
    let upper = magnitudes[next] as f64;

    (upper - lower).mul_add(frac, lower)
}

pub fn apply_blackman_harris(block: &mut [f32]) {
    let n = block.len().saturating_sub(1) as f32;
    if n <= 0.0 {
        return;
    }

    for (i, sample) in block.iter_mut().enumerate() {
        let k = i as f32 / n;
        let window = 0.01168f32.mul_add(
            -(6.0 * PI * k).cos(),
            0.14128f32.mul_add(
                (4.0 * PI * k).cos(),
                0.48829f32.mul_add(-(2.0 * PI * k).cos(), 0.35875),
            ),
        );
        *sample *= window;
    }
}
