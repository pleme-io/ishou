//! SVG renderer — stamps the brand mark (a swerve inside the viewbox) as a
//! standalone SVG document. Variants: `mark`, `mark-mono-black`, `mark-mono-white`.

use ishou_tokens::TokenSet;

pub fn render(t: &TokenSet) -> String {
    let b = &t.brand;
    let vb = b.swerve.viewbox;
    let stroke_cap = match b.swerve.stroke_cap {
        ishou_tokens::brand::StrokeCap::Butt => "butt",
        ishou_tokens::brand::StrokeCap::Round => "round",
        ishou_tokens::brand::StrokeCap::Square => "square",
    };
    let ink = t.color.ink.hex();
    let paper = t.color.paper.hex();
    format!(
        r#"<!-- ishou mark — pleme-io swerve (generated) -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {vb} {vb}" width="{vb}" height="{vb}">
  <rect width="{vb}" height="{vb}" rx="{rx}" fill="{paper}"/>
  <path d="{d}"
        fill="none"
        stroke="{ink}"
        stroke-width="{sw}"
        stroke-linecap="{stroke_cap}"
        stroke-linejoin="round"/>
</svg>
"#,
        vb = vb,
        rx = vb / 10,
        paper = paper,
        ink = ink,
        d = b.swerve.path_d,
        sw = b.swerve.stroke_width,
        stroke_cap = stroke_cap,
    )
}
