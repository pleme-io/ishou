//! Shader uniform tokens — typed knobs for the 13 blackmatter-ghostty shaders.
//!
//! Each field is a uniform the shader expects. Renderers can emit either:
//!   - a GLSL `#define` header (`ishou-render::glsl`)
//!   - a Ghostty config block (`ishou-render::ghostty`)
//!
//! Names mirror the shader filenames in
//! `blackmatter-ghostty/module/shaders/*.glsl` for 1:1 traceability.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Shaders {
    pub background_pulse: BackgroundPulse,
    pub bloom: Bloom,
    pub chromatic_aberration: ChromaticAberration,
    pub cursor_glow: CursorGlow,
    pub cursor_trail: CursorTrail,
    pub film_grain: FilmGrain,
    pub frost_haze: FrostHaze,
    pub prompt_saber: PromptSaber,
    pub screen_curvature: ScreenCurvature,
    pub sonic_boom: SonicBoom,
    pub spotlight: Spotlight,
    pub stardust: Stardust,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundPulse {
    pub intensity: f32,
    pub speed_hz: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct Bloom {
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChromaticAberration {
    pub magnitude: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct CursorGlow {
    pub radius_px: f32,
    pub intensity: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct CursorTrail {
    pub length_px: f32,
    pub decay: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct FilmGrain {
    pub intensity: f32,
    pub size_px: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct FrostHaze {
    pub intensity: f32,
    pub blur_px: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct PromptSaber {
    pub thickness_px: f32,
    pub swoop_ms: u16,
}
#[derive(Debug, Clone, Serialize)]
pub struct ScreenCurvature {
    pub curvature: f32,
    pub vignette: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct SonicBoom {
    pub intensity: f32,
    pub attack_ms: u16,
    pub settle_ms: u16,
}
#[derive(Debug, Clone, Serialize)]
pub struct Spotlight {
    pub radius_px: f32,
    pub softness: f32,
    pub dim: f32,
}
#[derive(Debug, Clone, Serialize)]
pub struct Stardust {
    pub particle_count: u32,
    pub twinkle_speed: f32,
    pub spread: f32,
}

impl Default for Shaders {
    fn default() -> Self {
        Self {
            background_pulse: BackgroundPulse {
                intensity: 0.08,
                speed_hz: 0.25,
            },
            bloom: Bloom {
                threshold: 0.80,
                intensity: 0.50,
                radius: 2.0,
            },
            chromatic_aberration: ChromaticAberration { magnitude: 0.002 },
            cursor_glow: CursorGlow {
                radius_px: 12.0,
                intensity: 0.6,
            },
            cursor_trail: CursorTrail {
                length_px: 48.0,
                decay: 0.92,
            },
            film_grain: FilmGrain {
                intensity: 0.04,
                size_px: 1.0,
            },
            frost_haze: FrostHaze {
                intensity: 0.20,
                blur_px: 6.0,
            },
            prompt_saber: PromptSaber {
                thickness_px: 2.0,
                swoop_ms: 250,
            },
            screen_curvature: ScreenCurvature {
                curvature: 0.04,
                vignette: 0.25,
            },
            sonic_boom: SonicBoom {
                intensity: 0.6,
                attack_ms: 80,
                settle_ms: 600,
            },
            spotlight: Spotlight {
                radius_px: 320.0,
                softness: 0.35,
                dim: 0.45,
            },
            stardust: Stardust {
                particle_count: 120,
                twinkle_speed: 0.5,
                spread: 0.8,
            },
        }
    }
}
