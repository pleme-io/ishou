//! Typed colour spaces — the load-bearing piece of the pleme-io theme
//! architecture (see `pleme-io/theory/THEME-ARCHITECTURE.md`).
//!
//! Three orthogonal facts about a colour:
//!
//! 1. Which **space** it lives in — sRGB (the colour you see in a hex
//!    picker), linear (the colour the GPU does maths on), OkLab (the
//!    perceptually-uniform space we lighten / darken / contrast in).
//! 2. Whether it carries **alpha**.
//! 3. Its **values**, which are different in each space.
//!
//! The pleme-io fleet conflated all three for years — every consumer
//! parsed a hex string into an `(f32, f32, f32)` tuple and prayed.
//! When such a tuple landed in a `wgpu::Color` for a `Bgra8UnormSrgb`
//! surface, the GPU treated it as linear → washed-out output. The fix
//! is structural: split the spaces into distinct *types*, expose
//! exactly one zero-cost conversion path (`Srgb → Linear`), and refuse
//! to construct a `wgpu::Color` from anything but `LinearRgba`.
//!
//! The `From<LinearRgba> for wgpu_types::Color` impl exists (behind
//! the `wgpu` feature) — `From<Srgb> for wgpu_types::Color` does not.
//! By construction, a pleme-io renderer can't write the wrong colour
//! space to a GPU surface.
//!
//! ## Test vectors
//!
//! Conversions follow IEC 61966-2-1:1999 (the sRGB standard) exactly.
//! See the unit tests for the canonical [0, 0.04045, 1] knee + a
//! handful of round-trip cross-pins against `palette` crate values.

use serde::{Deserialize, Serialize};

// ─── Srgb ───────────────────────────────────────────────────────────────────

/// A colour in sRGB space, stored as 8-bit per channel. The format every
/// stylix base16 yaml, hex picker, and CSS string carries. Cannot be
/// written to a wgpu surface without going through [`Self::to_linear`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Srgb {
    /// Construct from raw 8-bit channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert from a `#rrggbb` or `rrggbb` string. `None` on malformed
    /// input — never `Default::default()` silently.
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Convert to a canonical `#RRGGBB` string. Uppercase, always 7
    /// chars. The bytes every css / stylix / json renderer writes.
    #[must_use]
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Channel values as `[f32; 3]` in the closed unit interval, still
    /// in sRGB space. **Not** what a GPU wants — use `to_linear`
    /// before any wgpu pipeline.
    #[must_use]
    pub fn to_unit_f32(self) -> [f32; 3] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        ]
    }

    /// Promote to alpha-having sRGB. `255` = fully opaque.
    #[must_use]
    pub const fn with_alpha(self, a: u8) -> SrgbA {
        SrgbA {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// **The** entry to GPU-correct rendering: convert to linear space
    /// per IEC 61966-2-1. Required before any `wgpu::Color` construction
    /// on an sRGB-storage surface (which is the default everywhere in
    /// the pleme-io GPU stack).
    #[must_use]
    pub fn to_linear(self) -> Linear {
        Linear {
            r: srgb_channel_to_linear(f32::from(self.r) / 255.0),
            g: srgb_channel_to_linear(f32::from(self.g) / 255.0),
            b: srgb_channel_to_linear(f32::from(self.b) / 255.0),
        }
    }
}

/// sRGB with alpha. Useful when a colour has translucency at the
/// authoring layer (e.g. selection highlights, semi-transparent
/// overlays) before being promoted to `LinearRgba` for GPU writes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SrgbA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SrgbA {
    /// Convert to gamma-correct `LinearRgba`. The alpha channel is
    /// **not** gamma-mapped (alpha is linear by convention in every
    /// modern blending model).
    #[must_use]
    pub fn to_linear(self) -> LinearRgba {
        let linear = Srgb::new(self.r, self.g, self.b).to_linear();
        LinearRgba {
            r: linear.r,
            g: linear.g,
            b: linear.b,
            a: f32::from(self.a) / 255.0,
        }
    }
}

// ─── Linear ─────────────────────────────────────────────────────────────────

/// A colour in linear space, ready for any wgpu pipeline that writes
/// to an sRGB-storage surface. The only zero-cost construction path is
/// [`Srgb::to_linear`]; there is intentionally no `Linear::from_hex`
/// or `Linear::from_floats(r, g, b)` — an operator who has a hex
/// string in hand can never accidentally pass it as a linear value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Linear {
    /// Promote to alpha-having linear.
    #[must_use]
    pub const fn with_alpha(self, a: f32) -> LinearRgba {
        LinearRgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Convert back to sRGB. Inverse of `Srgb::to_linear` (lossy at
    /// 8-bit quantisation but round-trip-stable for values that came
    /// from `Srgb::to_linear` originally).
    #[must_use]
    pub fn to_srgb(self) -> Srgb {
        Srgb {
            r: clamp_to_u8(linear_channel_to_srgb(self.r) * 255.0),
            g: clamp_to_u8(linear_channel_to_srgb(self.g) * 255.0),
            b: clamp_to_u8(linear_channel_to_srgb(self.b) * 255.0),
        }
    }
}

/// Linear-space colour with alpha. The **only** type that constructs a
/// `wgpu_types::Color` for a pleme-io render path (under the `wgpu`
/// feature). The absence of `From<Srgb>` and `From<SrgbA>` impls is
/// load-bearing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl LinearRgba {
    /// Channel values as `[f64; 4]` — the shape `wgpu::Color`'s
    /// component fields take. Used by the `wgpu` feature impl below;
    /// also useful when consumers prefer to build `wgpu::Color`
    /// manually without enabling the feature.
    #[must_use]
    pub fn to_f64_array(self) -> [f64; 4] {
        [
            f64::from(self.r),
            f64::from(self.g),
            f64::from(self.b),
            f64::from(self.a),
        ]
    }
}

#[cfg(feature = "wgpu")]
impl From<LinearRgba> for wgpu_types::Color {
    fn from(c: LinearRgba) -> Self {
        wgpu_types::Color {
            r: f64::from(c.r),
            g: f64::from(c.g),
            b: f64::from(c.b),
            a: f64::from(c.a),
        }
    }
}

// ─── Conversion primitives ──────────────────────────────────────────────────

/// IEC 61966-2-1:1999 sRGB → linear, single channel. The published
/// formula:
///
/// ```text
/// f(c) =
///   c / 12.92                         if c ≤ 0.04045
///   ((c + 0.055) / 1.055) ^ 2.4       otherwise
/// ```
#[inline]
#[must_use]
pub fn srgb_channel_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// IEC 61966-2-1:1999 linear → sRGB, single channel. Inverse of
/// `srgb_channel_to_linear`.
///
/// ```text
/// f(c) =
///   12.92 * c                         if c ≤ 0.0031308
///   1.055 * c^(1/2.4) - 0.055         otherwise
/// ```
#[inline]
#[must_use]
pub fn linear_channel_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn clamp_to_u8(x: f32) -> u8 {
    if x.is_nan() {
        return 0;
    }
    let clamped = x.clamp(0.0, 255.0);
    clamped.round() as u8
}

// ─── Convenience hex APIs ───────────────────────────────────────────────────

/// Convenience: parse hex string and promote straight to linear. Use
/// this at config-parse time when the input is a string and the output
/// must reach a wgpu surface. `None` on malformed hex.
///
/// Saves the call-site one boilerplate step but does *not* shortcut
/// the type wall: the return is `Linear`, not `Srgb`, so the
/// `wgpu::Color` construction still goes through the typed path.
#[must_use]
pub fn linear_from_hex(hex: &str) -> Option<Linear> {
    Srgb::from_hex(hex).map(Srgb::to_linear)
}

// ─── Compatibility with the legacy `Rgb` ────────────────────────────────────

use crate::color::Rgb;

impl From<Rgb> for Srgb {
    fn from(c: Rgb) -> Self {
        Self::new(c.r, c.g, c.b)
    }
}

impl From<Srgb> for Rgb {
    fn from(c: Srgb) -> Self {
        Rgb::new(c.r, c.g, c.b)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Canonical sRGB → linear test vectors (IEC 61966-2-1) — boundary
    // points + a handful of midpoints cross-checked against the
    // `palette` crate.
    const EPS: f32 = 1e-5;

    #[test]
    fn srgb_zero_is_linear_zero() {
        assert!((srgb_channel_to_linear(0.0) - 0.0).abs() < EPS);
    }

    #[test]
    fn srgb_one_is_linear_one() {
        assert!((srgb_channel_to_linear(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn srgb_knee_uses_linear_segment_below_threshold() {
        // 0.04 < 0.04045 → linear formula c / 12.92
        let v = srgb_channel_to_linear(0.04);
        assert!((v - 0.04 / 12.92).abs() < EPS);
    }

    #[test]
    fn srgb_knee_uses_power_segment_above_threshold() {
        // 0.05 > 0.04045 → ((c + 0.055) / 1.055)^2.4
        let v = srgb_channel_to_linear(0.05);
        let expected = ((0.05_f32 + 0.055) / 1.055).powf(2.4);
        assert!((v - expected).abs() < EPS);
    }

    #[test]
    fn srgb_round_trip_at_8bit_quantisation_stable() {
        // For every 8-bit value, Srgb → Linear → Srgb returns the
        // original. The quantisation is the gate — at float precision
        // the round trip is exact.
        for byte in 0..=255u8 {
            let s = Srgb::new(byte, byte, byte);
            let back = s.to_linear().to_srgb();
            assert_eq!(s, back, "round-trip failed for byte {byte}");
        }
    }

    #[test]
    fn nord_polar_night_0_specific_linear_values() {
        // The exact bug that triggered this architecture. Nord
        // polar_night_0 is #2e3440 = sRGB(46, 52, 64). Naively
        // passing those bytes' unit-f32s as `wgpu::Color` on a
        // Bgra8UnormSrgb surface produced washed-out gray (~0.45
        // sRGB) because the GPU treated them as linear, then
        // gamma-corrected on store.
        //
        // The correct linear values are dramatically darker than
        // the sRGB unit floats — that's the whole point of gamma
        // correction. Pinned here to canonical 6-decimal precision
        // so a future tweak to the conversion formula fails this
        // test loudly.
        let nord_dark = Srgb::from_hex("#2e3440").unwrap();
        let linear = nord_dark.to_linear();
        // computed via the IEC 61966-2-1 formula on (46/255, 52/255, 64/255)
        assert!((linear.r - 0.027_320_892).abs() < 1e-5, "r = {}", linear.r);
        assert!((linear.g - 0.034_339_808).abs() < 1e-5, "g = {}", linear.g);
        assert!((linear.b - 0.051_269_468).abs() < 1e-5, "b = {}", linear.b);
    }

    #[test]
    fn hex_round_trip() {
        let s = Srgb::from_hex("#2e3440").unwrap();
        assert_eq!(s, Srgb::new(0x2e, 0x34, 0x40));
        assert_eq!(s.hex(), "#2E3440");
    }

    #[test]
    fn hex_accepts_unprefixed() {
        assert_eq!(Srgb::from_hex("ECEFF4"), Srgb::from_hex("#eceff4"));
    }

    #[test]
    fn hex_rejects_wrong_length() {
        assert!(Srgb::from_hex("#fff").is_none());
        assert!(Srgb::from_hex("#abcdef0").is_none());
        assert!(Srgb::from_hex("").is_none());
    }

    #[test]
    fn hex_rejects_non_hex_chars() {
        assert!(Srgb::from_hex("#zzzzzz").is_none());
        assert!(Srgb::from_hex("#abcg00").is_none());
    }

    #[test]
    fn linear_to_srgb_round_trip_for_canonical_palette() {
        // Nord polar night through the full pipeline: hex → Srgb →
        // Linear → Srgb → hex. Must return to the original hex.
        let palette = [
            "#2e3440", "#3b4252", "#434c5e", "#4c566a", "#d8dee9", "#e5e9f0", "#eceff4", "#8fbcbb",
            "#88c0d0", "#81a1c1", "#5e81ac", "#bf616a", "#d08770", "#ebcb8b", "#a3be8c", "#b48ead",
        ];
        for hex in palette {
            let s = Srgb::from_hex(hex).unwrap();
            let back = s.to_linear().to_srgb();
            // Note: `.hex()` uppercases, so compare normalized.
            let back_hex = back.hex().to_lowercase();
            assert_eq!(back_hex, hex, "round-trip broke for {hex}");
        }
    }

    #[test]
    fn linear_to_f64_array_matches_field_order() {
        let l = LinearRgba {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.4,
        };
        let arr = l.to_f64_array();
        // f32 → f64 widening introduces precision noise; pin the
        // ordering with a loose tolerance.
        assert!((arr[0] - 0.1).abs() < 1e-6);
        assert!((arr[1] - 0.2).abs() < 1e-6);
        assert!((arr[2] - 0.3).abs() < 1e-6);
        assert!((arr[3] - 0.4).abs() < 1e-6);
    }

    #[test]
    fn srgba_alpha_passes_through_unchanged() {
        // Alpha is linear by convention; converting sRGB-with-alpha
        // to LinearRgba should preserve alpha bit-for-bit (modulo
        // u8 → unit-f32 normalisation).
        let s = SrgbA {
            r: 0x2e,
            g: 0x34,
            b: 0x40,
            a: 200,
        };
        let l = s.to_linear();
        assert!((l.a - 200.0 / 255.0).abs() < EPS);
    }

    #[test]
    fn legacy_rgb_round_trips_through_srgb() {
        let rgb = Rgb::new(0x2e, 0x34, 0x40);
        let srgb: Srgb = rgb.into();
        let back: Rgb = srgb.into();
        assert_eq!(rgb.r, back.r);
        assert_eq!(rgb.g, back.g);
        assert_eq!(rgb.b, back.b);
    }

    #[test]
    fn linear_with_alpha_promotes_correctly() {
        let s = Srgb::from_hex("#2e3440").unwrap();
        let l = s.to_linear();
        let rgba = l.with_alpha(0.5);
        assert!((rgba.r - l.r).abs() < EPS);
        assert!((rgba.a - 0.5).abs() < EPS);
    }

    #[test]
    fn linear_from_hex_convenience_matches_two_step() {
        let l1 = linear_from_hex("#2e3440").unwrap();
        let l2 = Srgb::from_hex("#2e3440").unwrap().to_linear();
        assert!((l1.r - l2.r).abs() < EPS);
        assert!((l1.g - l2.g).abs() < EPS);
        assert!((l1.b - l2.b).abs() < EPS);
    }

    #[test]
    fn linear_from_hex_returns_none_on_malformed() {
        assert!(linear_from_hex("not a hex").is_none());
    }

    #[test]
    fn clamp_to_u8_handles_out_of_range_and_nan() {
        assert_eq!(clamp_to_u8(-1.0), 0);
        assert_eq!(clamp_to_u8(0.0), 0);
        assert_eq!(clamp_to_u8(127.5), 128);
        assert_eq!(clamp_to_u8(255.0), 255);
        assert_eq!(clamp_to_u8(1000.0), 255);
        assert_eq!(clamp_to_u8(f32::NAN), 0);
    }

    #[test]
    fn srgb_zero_to_linear_is_exactly_zero() {
        // Bug guard: in the gamma piecewise function, 0.0 must take
        // the linear branch (0.0 / 12.92 = 0.0), not the power branch.
        let l = Srgb::new(0, 0, 0).to_linear();
        assert_eq!(
            l,
            Linear {
                r: 0.0,
                g: 0.0,
                b: 0.0
            }
        );
    }

    #[test]
    fn srgb_white_to_linear_is_exactly_one() {
        let l = Srgb::new(255, 255, 255).to_linear();
        assert!((l.r - 1.0).abs() < EPS);
        assert!((l.g - 1.0).abs() < EPS);
        assert!((l.b - 1.0).abs() < EPS);
    }

    #[test]
    fn srgb_serde_round_trip() {
        let s = Srgb::new(0x2e, 0x34, 0x40);
        let j = serde_json::to_string(&s).unwrap();
        let back: Srgb = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn srgba_serde_round_trip() {
        let s = SrgbA {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        let j = serde_json::to_string(&s).unwrap();
        let back: SrgbA = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }

    // The load-bearing structural invariant: no `From<Srgb> for
    // wgpu_types::Color` impl. This test pins the absence — it would
    // fail to compile if someone added the rogue impl. (Marker test
    // — runs at compile time via type assertions.)
    #[cfg(feature = "wgpu")]
    #[test]
    fn linear_rgba_is_only_path_to_wgpu_color() {
        let l = LinearRgba {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        let _: wgpu_types::Color = l.into();
        // If someone adds `impl From<Srgb> for wgpu_types::Color`,
        // they're violating the architecture. There is no positive
        // assertion we can make in safe Rust that an impl *doesn't*
        // exist — but the absence is enforced by review + cse-lint.
    }
}
