//! PDQ-256 perceptual hash (registry plan, G1) — a faithful transcription of
//! Meta's reference implementation from the facebook/ThreatExchange
//! repository (fetched 2026-08-02): `pdq/cpp/hashing/pdqhashing.cpp`,
//! `pdq/cpp/downscaling/downscaling.cpp`, `pdq/cpp/hashing/torben.cpp`.
//! Known-answer tests (`tests/pdq.rs`) pin this module bit-for-bit against
//! the compiled reference (the `pdqhash` Python wrapper builds the same C++);
//! regenerate the expectations with `py scripts/gen_pdq_kats.py`.
//!
//! The algorithm, exactly as the reference does it:
//!
//! 1. Luminance: REC 601, `0.299 R + 0.587 G + 0.114 B`, f32 per pixel.
//!    Images under 5×5 are refused (reference returns a cleared hash;
//!    we return `None`).
//! 2. Downscale to 64×64 — unless already exactly 64×64, in which case the
//!    luma is copied straight through. Two passes ("Jarosz filter") of a
//!    sliding-window box blur along rows then columns, window size
//!    `(dim + 2·64 − 1) / (2·64)` per axis, then point decimation at cell
//!    centers `((out + 0.5) · in) / 64`, truncated.
//! 3. Quality: integer gradient metric over the 64×64 buffer
//!    (`Σ |trunc(Δ·100/255)| / 90`, capped at 100).
//! 4. DCT: `B = D·A·Dᵀ` with the 16×64 matrix
//!    `D[i][j] = √(2/64) · cos(π/128 · (i+1) · (2j+1))` — note the `i+1`:
//!    PDQ keeps AC frequencies 1..16 and never the DC row, f32 arithmetic
//!    in ascending-k accumulation order like the reference.
//! 5. Bits: coefficient > median of the 256 outputs. The reference's torben
//!    selector returns the 128th-smallest element for n=256 (lower-biased
//!    median); we sort and take the same order statistic.
//!
//! Bit/byte packing: CANONICAL PDQ order, verified bit-for-bit against the
//! reference — DCT bit `k = i·16 + j` lands at position `255 − k` of the
//! 256-bit string, i.e. `bits[31 − k/8] |= 1 << (k % 8)`, so our 32 bytes
//! hex-format identically to the reference `Hash256` (and to `pdqhash`'s
//! packed vector). Hamming distances are independent of the convention;
//! the G2 log format inherits the canonical one.
//!
//! This module is pure math over already-decoded pixels — no I/O, no image
//! codec, no network. The registry LAYER stays gated; nothing consumes this
//! hash in any verdict path until G3 lands with its own gates.

use crate::layers::watermark::DecodedImage;

/// A 256-bit PDQ hash plus the reference's 0–100 quality score (low quality
/// means a flat/featureless image whose hash carries little information).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdqHash {
    pub bits: [u8; 32],
    pub quality: u8,
}

/// Hamming distance between two PDQ hashes (0..=256).
pub fn hamming(a: &PdqHash, b: &PdqHash) -> u32 {
    a.bits
        .iter()
        .zip(b.bits.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

/// The sixteen 16-bit words of the packed hash (word w = bytes 2w..2w+2,
/// big-endian) — the multi-index nomination unit the registry design uses.
/// Each word covers 16 consecutive DCT bits (in canonical packing order),
/// so exact-word equality nominates exactly as the plan describes.
pub fn words(hash: &PdqHash) -> [u16; 16] {
    let mut out = [0u16; 16];
    for (w, chunk) in hash.bits.chunks_exact(2).enumerate() {
        out[w] = u16::from_be_bytes([chunk[0], chunk[1]]);
    }
    out
}

pub fn pdq_hash(image: &DecodedImage) -> Option<PdqHash> {
    let (rows, cols) = (image.height, image.width);
    // Reference guard: pdqHash256FromFloatLuma clears the hash under 5×5.
    if rows < 5 || cols < 5 {
        return None;
    }

    // 1. REC 601 luminance.
    let mut buffer1 = vec![0f32; rows * cols];
    for (idx, luma) in buffer1.iter_mut().enumerate() {
        let p = idx * 3;
        *luma = 0.299 * image.rgb[p] as f32
            + 0.587 * image.rgb[p + 1] as f32
            + 0.114 * image.rgb[p + 2] as f32;
    }

    // 2. Two-pass Jarosz box filter + center decimation to 64×64.
    let mut b64 = vec![0f32; 64 * 64];
    if rows == 64 && cols == 64 {
        b64.copy_from_slice(&buffer1);
    } else {
        let window_along_rows = jarosz_window_size(cols);
        let window_along_cols = jarosz_window_size(rows);
        let mut buffer2 = vec![0f32; rows * cols];
        for _ in 0..2 {
            box_along_rows(&buffer1, &mut buffer2, rows, cols, window_along_rows);
            box_along_cols(&buffer2, &mut buffer1, rows, cols, window_along_cols);
        }
        for out_i in 0..64 {
            let in_i = ((out_i as f64 + 0.5) * rows as f64 / 64.0) as usize;
            for out_j in 0..64 {
                let in_j = ((out_j as f64 + 0.5) * cols as f64 / 64.0) as usize;
                b64[out_i * 64 + out_j] = buffer1[in_i * cols + in_j];
            }
        }
    }

    // 3. Quality (reference integer semantics: truncation toward zero).
    let mut gradient_sum: i32 = 0;
    for i in 0..63 {
        for j in 0..64 {
            let d = ((b64[i * 64 + j] - b64[(i + 1) * 64 + j]) * 100.0 / 255.0) as i32;
            gradient_sum += d.abs();
        }
    }
    for i in 0..64 {
        for j in 0..63 {
            let d = ((b64[i * 64 + j] - b64[i * 64 + j + 1]) * 100.0 / 255.0) as i32;
            gradient_sum += d.abs();
        }
    }
    let quality = (gradient_sum / 90).min(100) as u8;

    // 4. B = D·A·Dᵀ, f32, ascending accumulation like the reference.
    let dct = dct_matrix_16x64();
    let mut t = [[0f32; 64]; 16];
    for i in 0..16 {
        for j in 0..64 {
            let mut sum = 0f32;
            for (k, row) in dct[i].iter().enumerate() {
                sum += row * b64[k * 64 + j];
            }
            t[i][j] = sum;
        }
    }
    let mut b16 = [0f32; 256];
    for i in 0..16 {
        for j in 0..16 {
            let mut sum = 0f32;
            for k in 0..64 {
                sum += t[i][k] * dct[j][k];
            }
            b16[i * 16 + j] = sum;
        }
    }

    // 5. Lower-biased median (torben's 128th-smallest for n=256), then bits.
    let mut sorted = b16;
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("DCT outputs are finite"));
    let median = sorted[127];
    let mut bits = [0u8; 32];
    for (k, coeff) in b16.iter().enumerate() {
        if *coeff > median {
            // Canonical PDQ bit order (see module docs): position 255 - k.
            bits[31 - k / 8] |= 1 << (k % 8);
        }
    }
    Some(PdqHash { bits, quality })
}

/// `computeJaroszFilterWindowSize(oldDimension, 64)`, verbatim.
fn jarosz_window_size(old_dimension: usize) -> usize {
    old_dimension.div_ceil(2 * 64)
}

fn box_along_rows(input: &[f32], output: &mut [f32], rows: usize, cols: usize, window: usize) {
    for i in 0..rows {
        box_1d(&input[i * cols..], &mut output[i * cols..], cols, 1, window);
    }
}

fn box_along_cols(input: &[f32], output: &mut [f32], rows: usize, cols: usize, window: usize) {
    for j in 0..cols {
        box_1d(&input[j..], &mut output[j..], rows, cols, window);
    }
}

/// `box1DFloat`, verbatim: a sliding-window mean with the reference's four
/// phases (grow, small-window writes, full-window writes, shrink).
fn box_1d(invec: &[f32], outvec: &mut [f32], vector_length: usize, stride: usize, window: usize) {
    let half_window = (window + 2) / 2;
    let phase_1 = half_window - 1;
    let phase_2 = window - half_window + 1;
    let phase_3 = vector_length - window;
    let phase_4 = half_window - 1;

    let (mut li, mut ri, mut oi) = (0usize, 0usize, 0usize);
    let mut sum = 0f32;
    let mut current_window = 0usize;

    for _ in 0..phase_1 {
        sum += invec[ri];
        current_window += 1;
        ri += stride;
    }
    for _ in 0..phase_2 {
        sum += invec[ri];
        current_window += 1;
        outvec[oi] = sum / current_window as f32;
        ri += stride;
        oi += stride;
    }
    for _ in 0..phase_3 {
        sum += invec[ri];
        sum -= invec[li];
        outvec[oi] = sum / current_window as f32;
        li += stride;
        ri += stride;
        oi += stride;
    }
    for _ in 0..phase_4 {
        sum -= invec[li];
        current_window -= 1;
        outvec[oi] = sum / current_window as f32;
        li += stride;
        oi += stride;
    }
}

/// `dct_matrix_64`, verbatim: `√(2/64) · cos(π/128 · (i+1) · (2j+1))`,
/// f64 trig truncated to f32 on store like the C++.
fn dct_matrix_16x64() -> [[f32; 64]; 16] {
    let scale = (2.0f64 / 64.0).sqrt() as f32;
    let mut matrix = [[0f32; 64]; 16];
    for (i, row) in matrix.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            let angle = (std::f64::consts::PI / 128.0) * (i as f64 + 1.0) * (2.0 * j as f64 + 1.0);
            *cell = (scale as f64 * angle.cos()) as f32;
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_image(width: usize, height: usize, value: u8) -> DecodedImage {
        DecodedImage {
            rgb: vec![value; width * height * 3],
            width,
            height,
        }
    }

    #[test]
    fn tiny_images_are_refused_like_the_reference() {
        assert!(pdq_hash(&flat_image(4, 400, 128)).is_none());
        assert!(pdq_hash(&flat_image(400, 4, 128)).is_none());
    }

    #[test]
    fn identical_images_hash_identically_and_flat_images_score_zero_quality() {
        let a = pdq_hash(&flat_image(100, 80, 200)).unwrap();
        let b = pdq_hash(&flat_image(100, 80, 200)).unwrap();
        assert_eq!(a, b);
        assert_eq!(hamming(&a, &b), 0);
        assert_eq!(a.quality, 0, "a featureless image has no gradients");
    }

    #[test]
    fn words_split_matches_the_packing() {
        let mut bits = [0u8; 32];
        bits[0] = 0xAB;
        bits[1] = 0xCD;
        bits[30] = 0x01;
        bits[31] = 0x02;
        let hash = PdqHash { bits, quality: 50 };
        let words = words(&hash);
        assert_eq!(words[0], 0xABCD);
        assert_eq!(words[15], 0x0102);
    }
}
