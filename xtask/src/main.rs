//! Project automation, invoked as `cargo xtask <command>`.
//!
//! For now it only knows how to package a built plugin as a CLAP bundle in the
//! host's plugin directory. Add more subcommands here as the need shows up.

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Parser, Subcommand};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Rift project automation",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Build a plugin and install it as a CLAP bundle hosts can scan.
    Install {
        /// Plugin package to build, e.g. `minimal_gain`.
        package: String,

        /// Build the release profile instead of debug.
        #[arg(short, long)]
        release: bool,

        /// Install into <DIR> instead of the host's default CLAP directory.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Task::Install {
            package,
            release,
            out,
        } => install(&package, release, out),
    }
}

/// Builds `package` and copies its cdylib into a CLAP bundle so a host can scan
/// it. On macOS the bundle is `<PACKAGE>.clap/Contents/MacOS/<PACKAGE>`, which
/// is the layout FL Studio (and other CLAP hosts) expect.
fn install(package: &str, release: bool, out: Option<PathBuf>) -> Result<()> {
    let root = workspace_root()?;
    build(&root, package, release)?;

    let profile = if release { "release" } else { "debug" };
    let file = format!("{LIB_PREFIX}{}{LIB_SUFFIX}", package.replace('-', "_"));
    let library = root.join("target").join(profile).join(file);
    if !library.is_file() {
        return Err(format!("no build artifact at `{}`", library.display()).into());
    }

    let out = out.unwrap_or_else(default_plugin_dir);
    let bundle = bundle_path(&out, package);
    if let Some(parent) = bundle.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&library, &bundle)?;

    println!("installed `{package}` -> {}", bundle.display());
    Ok(())
}

fn build(root: &Path, package: &str, release: bool) -> Result<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let mut command = Command::new(cargo);
    command.current_dir(root).args(["build", "-p", package]);
    if release {
        command.arg("--release");
    }

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo build -p {package}` failed ({status})").into())
    }
}

/// `xtask/` sits next to the workspace root, so its parent is the root.
fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or("`xtask` is not inside the workspace".into())
}

// ---------------------------------------------------------------------------
// Platform specifics
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
const LIB_PREFIX: &str = "lib";
#[cfg(target_os = "windows")]
const LIB_PREFIX: &str = "";

#[cfg(target_os = "macos")]
const LIB_SUFFIX: &str = ".dylib";
#[cfg(target_os = "windows")]
const LIB_SUFFIX: &str = ".dll";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const LIB_SUFFIX: &str = ".so";

/// macOS CLAP plugins are bundles; everywhere else the `.clap` file *is* the
/// library.
#[cfg(target_os = "macos")]
fn bundle_path(out: &Path, package: &str) -> PathBuf {
    out.join(format!("{package}.clap"))
        .join("Contents")
        .join("MacOS")
        .join(package)
}

#[cfg(not(target_os = "macos"))]
fn bundle_path(out: &Path, package: &str) -> PathBuf {
    out.join(format!("{package}.clap"))
}

#[cfg(target_os = "macos")]
fn default_plugin_dir() -> PathBuf {
    home_dir().join("Library/Audio/Plug-Ins/CLAP")
}

#[cfg(target_os = "linux")]
fn default_plugin_dir() -> PathBuf {
    home_dir().join(".clap")
}

#[cfg(target_os = "windows")]
fn default_plugin_dir() -> PathBuf {
    env::var_os("COMMONPROGRAMFILES")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:/Program Files/Common Files"))
        .join("CLAP")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn default_plugin_dir() -> PathBuf {
    PathBuf::from(".")
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
