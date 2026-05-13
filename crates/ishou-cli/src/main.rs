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
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Render every supported target to a directory, one file per target.
    RenderAll {
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
    let tokens = TokenSet::pleme();

    match cli.cmd {
        Cmd::Render { target, out } => {
            let target = Target::from_str(&target)
                .ok_or_else(|| anyhow!("unknown target: {target}"))?;
            let content = target.render(&tokens);
            emit(content, out)?;
        }
        Cmd::RenderAll { out_dir } => {
            fs::create_dir_all(&out_dir).context("mkdir out_dir")?;
            for target in Target::all() {
                let path = out_dir.join(filename(target));
                let content = target.render(&tokens);
                fs::write(&path, content).with_context(|| format!("write {path:?}"))?;
                eprintln!("wrote {}", path.display());
            }
        }
        Cmd::Hash => {
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

fn filename(t: Target) -> &'static str {
    match t {
        Target::Css => "ishou.css",
        Target::Tailwind => "tailwind.config.js",
        Target::Scss => "_ishou.scss",
        Target::Rust => "ishou.rs",
        Target::Json => "ishou.tokens.json",
        Target::Glsl => "ishou.glsl",
        Target::Ghostty => "ishou.ghostty",
        Target::Tui => "ishou_tui.rs",
        Target::Svg => "mark.svg",
        Target::Stylix => "nord-dark.yaml",
        Target::Nix => "nord-palette.nix",
        Target::StylixFonts => "stylix-fonts.nix",
        Target::FleetFonts => "fleet-fonts.nix",
    }
}

fn target_name(t: Target) -> &'static str {
    match t {
        Target::Css => "css",
        Target::Tailwind => "tailwind",
        Target::Scss => "scss",
        Target::Rust => "rust",
        Target::Json => "json",
        Target::Glsl => "glsl",
        Target::Ghostty => "ghostty",
        Target::Tui => "tui",
        Target::Svg => "svg",
        Target::Stylix => "stylix",
        Target::Nix => "nix",
        Target::StylixFonts => "stylix-fonts",
        Target::FleetFonts => "fleet-fonts",
    }
}
