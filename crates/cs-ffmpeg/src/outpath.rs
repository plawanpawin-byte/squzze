//! Output file naming: templates, sanitising and overwrite policy.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Overwrite {
    /// Add " (1)", " (2)" … to the name.
    Rename,
    /// Replace the existing file (never the source itself).
    Replace,
    /// Fail the job if the file exists.
    Skip,
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
    "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Make a single path component safe on Windows, Linux and macOS.
pub fn sanitize_component(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_control() || "\\/:*?\"<>|".contains(c) { '_' } else { c })
        .collect();
    s = s.trim_matches(|c: char| c == ' ' || c == '.').to_string();
    if s.chars().count() > 120 {
        s = s.chars().take(120).collect();
    }
    if s.is_empty() {
        s = "output".into();
    }
    let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        s.insert(0, '_');
    }
    s
}

/// Expand `{name}` and `{preset}` in a template. Unknown `{...}` tokens are left as text.
pub fn expand_template(template: &str, source_stem: &str, preset_id: &str) -> String {
    let t = if template.trim().is_empty() { "{name}-{preset}" } else { template };
    sanitize_component(&t.replace("{name}", source_stem).replace("{preset}", preset_id))
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlanError {
    Exists(PathBuf),
    NoDir(String),
}

/// Decide the final output path. Never returns the source path.
pub fn plan_output(
    source: &Path,
    dest_dir: Option<&Path>,
    template: &str,
    preset_id: &str,
    ext: &str,
    policy: Overwrite,
) -> Result<PathBuf, PlanError> {
    let dir = match dest_dir {
        Some(d) if d.is_absolute() => d.to_path_buf(),
        Some(d) => return Err(PlanError::NoDir(format!("destination must be an absolute path: {}", d.display()))),
        None => source.parent().map(Path::to_path_buf).ok_or_else(|| PlanError::NoDir("source has no folder".into()))?,
    };
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "output".into());
    let base = expand_template(template, &stem, preset_id);
    let make = |n: u32| {
        let name = if n == 0 { format!("{base}.{ext}") } else { format!("{base} ({n}).{ext}") };
        dir.join(name)
    };
    let same_as_source = |p: &Path| p == source || std::fs::canonicalize(p).ok() == std::fs::canonicalize(source).ok();
    let first = make(0);
    if !first.exists() && !same_as_source(&first) {
        return Ok(first);
    }
    match policy {
        Overwrite::Skip => Err(PlanError::Exists(first)),
        Overwrite::Replace if !same_as_source(&first) => Ok(first),
        _ => {
            for n in 1..10_000 {
                let c = make(n);
                if !c.exists() {
                    return Ok(c);
                }
            }
            Err(PlanError::Exists(first))
        }
    }
}

/// Temporary name used while encoding; renamed to the final path on success.
pub fn temp_path(final_path: &Path) -> PathBuf {
    let ext = final_path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = final_path.file_stem().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    final_path.with_file_name(format!("{stem}.squeeze-part.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_hostile_names() {
        assert_eq!(sanitize_component("a/b\\c:d*e?f\"g<h>i|j"), "a_b_c_d_e_f_g_h_i_j");
        assert_eq!(sanitize_component("..."), "output");
        assert_eq!(sanitize_component("CON"), "_CON");
        assert_eq!(sanitize_component("nul.txt"), "_nul.txt");
        assert_eq!(sanitize_component("  name. "), "name");
        assert!(sanitize_component(&"x".repeat(500)).chars().count() <= 121);
    }

    #[test]
    fn traversal_is_neutralised() {
        let t = expand_template("../../evil{name}", "clip", "h264");
        assert!(!t.contains('/') && !t.contains('\\'));
    }

    #[test]
    fn thai_and_spaces_survive() {
        let t = expand_template("{name}-{preset}", "วิดีโอ ทดสอบ", "h264-1080");
        assert_eq!(t, "วิดีโอ ทดสอบ-h264-1080");
    }

    #[test]
    fn rename_replace_skip_policies() {
        let dir = std::env::temp_dir().join(format!("sq-out-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("clip.mov");
        std::fs::write(&src, b"x").unwrap();
        let first = dir.join("clip-p.mp4");
        std::fs::write(&first, b"x").unwrap();
        let r = plan_output(&src, Some(&dir), "", "p", "mp4", Overwrite::Rename).unwrap();
        assert_eq!(r.file_name().unwrap(), "clip-p (1).mp4");
        let r = plan_output(&src, Some(&dir), "", "p", "mp4", Overwrite::Replace).unwrap();
        assert_eq!(r, first);
        assert!(matches!(plan_output(&src, Some(&dir), "", "p", "mp4", Overwrite::Skip), Err(PlanError::Exists(_))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn never_overwrites_source() {
        let dir = std::env::temp_dir().join(format!("sq-src-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("a.mp4");
        std::fs::write(&src, b"x").unwrap();
        let r = plan_output(&src, Some(&dir), "a", "p", "mp4", Overwrite::Replace).unwrap();
        assert_ne!(r, src);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn relative_destination_rejected() {
        assert!(plan_output(Path::new("C:\\a.mp4"), Some(Path::new("rel")), "", "p", "mp4", Overwrite::Rename).is_err());
    }
}
