//! Mono: true black and white with neutral grays, for seats that want no hue.
//!
//! No slot carries colour, including the accent slot (`base0D`) and the
//! semantic slots: a chosen item is white on black, and a warning is told by its
//! icon and text.

use ishou_tokens::Rgb;

const fn gray(v: u8) -> Rgb {
    Rgb::new(v, v, v)
}

/// The sixteen base16 slots.
#[must_use]
pub const fn base16_slots() -> [(&'static str, Rgb); 16] {
    [
        ("base00", gray(0x00)),
        ("base01", gray(0x0d)),
        ("base02", gray(0x1f)),
        ("base03", gray(0x6e)),
        ("base04", gray(0xa8)),
        ("base05", gray(0xe6)),
        ("base06", gray(0xf2)),
        ("base07", gray(0xff)),
        ("base08", gray(0xf2)),
        ("base09", gray(0xd9)),
        ("base0A", gray(0xc8)),
        ("base0B", gray(0xbf)),
        ("base0C", gray(0xb0)),
        ("base0D", gray(0xff)),
        ("base0E", gray(0xd0)),
        ("base0F", gray(0x8a)),
    ]
}

/// Mono as an already-parsed Nix attrset. See [`crate::base16_nix`].
#[must_use]
pub fn render_base16_nix() -> String {
    crate::base16_nix::render(
        &crate::base16_nix::SchemeMeta {
            name: "Mono (pleme-io / ishou)",
            author: "pleme-io ishou",
            variant: "dark",
            slug: "mono-dark",
        },
        &base16_slots(),
        "pleme-io/ishou/crates/ishou-render/src/mono.rs",
        "stylix.base16Scheme = import ./rendered/stylix-base16-mono-dark.nix;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_is_a_pure_gray() {
        for (slot, rgb) in base16_slots() {
            assert!(rgb.r == rgb.g && rgb.g == rgb.b, "{slot} carries a hue");
        }
    }

    #[test]
    fn the_ends_are_true_black_and_white_and_the_accent_is_white() {
        let get = |name: &str| {
            base16_slots()
                .into_iter()
                .find(|(s, _)| *s == name)
                .unwrap()
                .1
        };
        assert_eq!(get("base00").hex(), gray(0).hex());
        assert_eq!(get("base07").hex(), gray(0xff).hex());
        assert_eq!(get("base0D").hex(), gray(0xff).hex());
    }
}
