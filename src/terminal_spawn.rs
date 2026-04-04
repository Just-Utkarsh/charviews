use anyhow::{anyhow, Context, Result};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

const FALLBACKS: &[&str] = &["foot", "kitty", "alacritty", "ghostty", "wezterm"];

/// Window title for the viewer (optional Hyprland matching).
pub const VIEWER_WINDOW_TITLE: &str = "Charvision Viewer";

/// Default spawned viewer grid (no TUI control); Omarchy Hypr `TUI.float` + Ghostty use this for `--window-*`.
pub const DEFAULT_VIEWER_GEOMETRY: ViewerGeometry = ViewerGeometry {
    cols: 96,
    rows: 28,
};

/// Omarchy `apps/system.conf` floats this class alongside `org.omarchy.btop`, etc.
pub const DEFAULT_OMARCHY_FLOAT_CLASS: &str = "TUI.float";

fn float_window_class() -> String {
    std::env::var("CHARVISION_FLOAT_CLASS").unwrap_or_else(|_| DEFAULT_OMARCHY_FLOAT_CLASS.to_string())
}

#[derive(Clone, Copy, Debug)]
pub struct ViewerGeometry {
    pub cols: u16,
    pub rows: u16,
}

impl ViewerGeometry {
    pub fn clamp(self) -> Self {
        Self {
            cols: self.cols.clamp(40, 400),
            rows: self.rows.clamp(8, 200),
        }
    }
}

/// Spawn a new terminal running `exe` with `viewer …` after emulator-specific options.
pub fn spawn_viewer(
    exe: &Path,
    device_arg: Option<&str>,
    fps: u32,
    geom: ViewerGeometry,
) -> Result<()> {
    let geom = geom.clamp();

    let mut viewer_args: Vec<OsString> = vec![
        exe.as_os_str().to_owned(),
        OsString::from("viewer"),
    ];
    if let Some(d) = device_arg {
        viewer_args.push(OsString::from("--device"));
        viewer_args.push(OsString::from(d));
    }
    viewer_args.push(OsString::from("--fps"));
    viewer_args.push(OsString::from(fps.to_string()));

    let term = terminal_binary()?;
    let base = Path::new(&term)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(term.as_str())
        .to_lowercase();

    let mut cmd = Command::new(&term);
    match base.as_str() {
        "foot" => {
            cmd.arg("-T").arg(VIEWER_WINDOW_TITLE);
            cmd.arg(format!("-W{}x{}", geom.cols, geom.rows));
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
        "alacritty" => {
            cmd.arg("-T").arg(VIEWER_WINDOW_TITLE);
            cmd.arg("-o")
                .arg(format!("window.dimensions.columns={}", geom.cols));
            cmd.arg("-o")
                .arg(format!("window.dimensions.lines={}", geom.rows));
            cmd.arg("-e");
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
        "kitty" => {
            cmd.arg("-T").arg(VIEWER_WINDOW_TITLE);
            cmd.arg("-o")
                .arg(format!("initial_window_width={}c", geom.cols));
            cmd.arg("-o")
                .arg(format!("initial_window_height={}c", geom.rows));
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
        "ghostty" => {
            cmd.arg(format!("--class={}", float_window_class()));
            cmd.arg("--title").arg(VIEWER_WINDOW_TITLE);
            cmd.arg(format!("--window-width={}", geom.cols));
            cmd.arg(format!("--window-height={}", geom.rows));
            cmd.arg("-e");
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
        "wezterm" => {
            // Match floating rules by app class; set cols/rows in wezterm config if needed.
            cmd.args(["start", "--class", "charvision-viewer", "--"]);
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
        _ => {
            for a in &viewer_args {
                cmd.arg(a);
            }
        }
    }

    cmd.spawn()
        .with_context(|| format!("failed to spawn terminal emulator: {term}"))?;
    Ok(())
}

fn terminal_binary() -> Result<String> {
    if let Ok(t) = std::env::var("TERMINAL") {
        let t = t.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    for name in FALLBACKS {
        if which_simple(name) {
            return Ok((*name).to_string());
        }
    }
    Err(anyhow!(
        "no terminal found. Set TERMINAL to your emulator (e.g. foot) or install one of: {}",
        FALLBACKS.join(", ")
    ))
}

fn which_simple(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
