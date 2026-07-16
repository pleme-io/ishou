//! ishou — CLI for rendering pleme-io design tokens.
//!
//! ```text
//! ishou render --target css --out tokens.css
//! ishou render --target tailwind --out tailwind.config.js
//! ishou render-all --out-dir generated/
//! ishou hash                   # print BLAKE3-esque content hash
//! ishou targets                # list supported targets
//! ```

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use ishou_render::Target;
use ishou_tokens::TokenSet;

#[derive(Parser, Debug)]
#[command(name = "ishou", version, about = "pleme-io design system renderer")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Render the token set to a single target.
    Render {
        #[arg(long)]
        target: String,
        /// Theme to render (`pleme` = the default Nord set, `steel` = the cool
        /// brushed-metal set). Defaults to `pleme`.
        #[arg(long, default_value = "pleme")]
        theme: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Render every supported target to a directory, one file per target.
    RenderAll {
        /// Theme to render (`pleme` | `steel`). Defaults to `pleme`.
        #[arg(long, default_value = "pleme")]
        theme: String,
        #[arg(long, default_value = "generated")]
        out_dir: PathBuf,
    },
    /// Print the content hash of the current token set.
    Hash,
    /// List every supported render target.
    Targets,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Render { target, theme, out } => {
            let tokens = token_set(&theme)?;
            let target = Target::from_str(&target)
                .ok_or_else(|| anyhow!("unknown target: {target}"))?;
            let content = target.render(&tokens);
            emit(content, out)?;
        }
        Cmd::RenderAll { theme, out_dir } => {
            let tokens = token_set(&theme)?;
            fs::create_dir_all(&out_dir).context("mkdir out_dir")?;
            for target in Target::all() {
                let path = out_dir.join(filename(target));
                let content = target.render(&tokens);
                fs::write(&path, content).with_context(|| format!("write {path:?}"))?;
                eprintln!("wrote {}", path.display());
            }
        }
        Cmd::Hash => {
            let tokens = TokenSet::pleme();
            let h = tokens.content_hash();
            for b in h {
                print!("{b:02x}");
            }
            println!();
        }
        Cmd::Targets => {
            for t in Target::all() {
                println!("{}", target_name(t));
            }
        }
    }
    Ok(())
}

fn emit(content: String, out: Option<PathBuf>) -> Result<()> {
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, content).with_context(|| format!("write {path:?}"))?;
            eprintln!("wrote {}", path.display());
        }
        None => {
            print!("{content}");
        }
    }
    Ok(())
}

/// Select the token set for a theme name. Unknown themes are a hard error
/// (no silent fallback) so a typo never renders the wrong theme.
fn token_set(theme: &str) -> Result<TokenSet> {
    Ok(match theme {
        "pleme" | "default" | "nord" => TokenSet::pleme(),
        "steel" => TokenSet::steel(),
        other => return Err(anyhow!("unknown theme: {other} (try: pleme, steel)")),
    })
}

fn filename(t: Target) -> &'static str {
    match t {
        Target::Css => "ishou.css",
        Target::Md3 => "md-sys-color.css",
        Target::Tailwind => "tailwind.config.js",
        Target::Scss => "_ishou.scss",
        Target::Rust => "ishou.rs",
        Target::Json => "ishou.tokens.json",
        Target::Glsl => "ishou.glsl",
        Target::Ghostty => "ishou.ghostty",
        Target::Tui => "ishou_tui.rs",
        Target::Svg => "mark.svg",
        Target::Stylix => "nord-dark.yaml",
        Target::Bat => "nord.tmTheme",
        Target::Nix => "nord-palette.nix",
        Target::StylixFonts => "stylix-fonts.nix",
        Target::FleetFonts => "fleet-fonts.nix",
        Target::StylixVellum => "vellum.yaml",
        Target::StylixVellumBase24 => "vellum-base24.yaml",
        Target::SvgVellumPalette => "vellum-palette.svg",
        Target::SkimVellum => "vellum.skim",
        Target::EscribaVellum => "vellum.lisp",
    }
}

fn target_name(t: Target) -> &'static str {
    match t {
        Target::Css => "css",
        Target::Md3 => "md3",
        Target::Tailwind => "tailwind",
        Target::Scss => "scss",
        Target::Rust => "rust",
        Target::Json => "json",
        Target::Glsl => "glsl",
        Target::Ghostty => "ghostty",
        Target::Tui => "tui",
        Target::Svg => "svg",
        Target::Stylix => "stylix",
        Target::Bat => "bat",
        Target::Nix => "nix",
        Target::StylixFonts => "stylix-fonts",
        Target::FleetFonts => "fleet-fonts",
        Target::StylixVellum => "stylix-vellum",
        Target::StylixVellumBase24 => "stylix-vellum-base24",
        Target::SvgVellumPalette => "svg-vellum-palette",
        Target::SkimVellum => "skim-vellum",
        Target::EscribaVellum => "escriba-vellum",
    }
}
