//! Greys: a surgical palette for a Grey's Anatomy seat. Hospital-night slate,
//! ceil-blue scrubs as the accent, monitor green and a heart red for the
//! semantic slots.

use ishou_tokens::Rgb;

/// The sixteen base16 slots.
#[must_use]
pub const fn base16_slots() -> [(&'static str, Rgb); 16] {
    [
        ("base00", Rgb::new(0x10, 0x18, 0x20)),
        ("base01", Rgb::new(0x1a, 0x25, 0x30)),
        ("base02", Rgb::new(0x2a, 0x3a, 0x47)),
        ("base03", Rgb::new(0x5f, 0x73, 0x82)),
        ("base04", Rgb::new(0xa9, 0xba, 0xc6)),
        ("base05", Rgb::new(0xdf, 0xe7, 0xec)),
        ("base06", Rgb::new(0xee, 0xf3, 0xf6)),
        ("base07", Rgb::new(0xff, 0xff, 0xff)),
        ("base08", Rgb::new(0xc8, 0x43, 0x4f)),
        ("base09", Rgb::new(0xd9, 0x8a, 0x5b)),
        ("base0A", Rgb::new(0xe3, 0xc1, 0x6f)),
        ("base0B", Rgb::new(0x6f, 0xbf, 0x9f)),
        ("base0C", Rgb::new(0x4f, 0xb3, 0xbf)),
        ("base0D", Rgb::new(0x5f, 0xa8, 0xd3)),
        ("base0E", Rgb::new(0x9d, 0x8e, 0xc7)),
        ("base0F", Rgb::new(0x8a, 0x5a, 0x5a)),
    ]
}

/// Greys as an already-parsed Nix attrset. See [`crate::base16_nix`].
#[must_use]
pub fn render_base16_nix() -> String {
    crate::base16_nix::render(
        &crate::base16_nix::SchemeMeta {
            name: "Greys (pleme-io / ishou)",
            author: "pleme-io ishou",
            variant: "dark",
            slug: "greys-dark",
        },
        &base16_slots(),
        "pleme-io/ishou/crates/ishou-render/src/greys.rs",
        "stylix.base16Scheme = import ./rendered/stylix-base16-greys-dark.nix;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(rgb: Rgb) -> f64 {
        let lin = |c: u8| {
            let c = f64::from(c) / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(rgb.r) + 0.7152 * lin(rgb.g) + 0.0722 * lin(rgb.b)
    }

    fn get(name: &str) -> Rgb {
        base16_slots()
            .into_iter()
            .find(|(s, _)| *s == name)
            .unwrap()
            .1
    }

    #[test]
    fn body_text_and_the_accent_read_on_the_background() {
        let bg = luminance(get("base00"));
        for slot in ["base05", "base0D"] {
            let ratio = (luminance(get(slot)) + 0.05) / (bg + 0.05);
            assert!(ratio >= 4.5, "{slot} contrast is {ratio}");
        }
    }
}
