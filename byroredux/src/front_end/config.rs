//! Installed Oblivion presentation settings, read without changing game configuration.
use anyhow::{bail, Context, Result};
use std::{collections::HashMap, path::{Path, PathBuf}};

pub(super) struct Config { pub data: PathBuf, pub startup: Vec<PathBuf>, pub title: PathBuf,
    pub map: PathBuf, pub credits: PathBuf, pub music: PathBuf }

fn read_settings(path: &Path) -> Result<HashMap<String,String>> {
    if std::fs::metadata(path)?.len() > 1024*1024 { bail!("INI exceeds 1 MiB"); }
    let text = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&text);
    Ok(text.lines().filter_map(|line| {
        let line = line.trim();
        if line.starts_with(';') || line.starts_with('#') { return None; }
        let (key,value) = line.split_once('=')?;
        Some((key.trim().to_ascii_lowercase(),value.trim().to_string()))
    }).collect())
}
fn local_path(root: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains([':', '\0']) || name.split(['/', '\\']).any(|s| s==".." || s.is_empty()) {
        bail!("invalid installed resource path");
    }
    let path = root.join(name.replace('\\', "/")).canonicalize().with_context(|| format!("missing original resource {name}"))?;
    if !path.starts_with(root.canonicalize()?) { bail!("resource escapes installation"); }
    Ok(path)
}
impl Config {
    pub fn load(data: &Path) -> Result<Self> {
        let data = data.canonicalize().with_context(|| format!("resolve original Oblivion Data directory {}", data.display()))?;
        let ini = data.parent().context("installation root missing")?.join("Oblivion_default.ini");
        let mut settings = read_settings(&ini).with_context(|| format!("read original settings {}", ini.display()))?;
        #[cfg(windows)]
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let path = PathBuf::from(profile).join("Documents/My Games/Oblivion/Oblivion.ini");
            if path.is_file() { settings.extend(read_settings(&path)?); }
        }
        let get = |key: &str| settings.get(key).context(format!("missing original INI key {key}"));
        let video = data.join("Video");
        let startup = get("sintrosequence")?.split(',').map(|name| local_path(&video,name.trim())).collect::<Result<Vec<_>>>()?;
        if startup.is_empty() || startup.len()>16 { bail!("invalid startup movie count"); }
        Ok(Self { title: local_path(&video,get("smainmenumovieintro")?)?,
            map: local_path(&video,get("smainmenumovie")?)?,
            credits: local_path(&video,get("screditsmenumovie")?)?,
            music: local_path(&data.join("Music"),get("smainmenumusic")?)?, data, startup })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_end_rejects_path_traversal_and_absolute_paths() {
        for name in ["../movie.bik", "C:\\movie.bik", "a\\..\\movie.bik", "", "/movie.bik"] {
            assert!(local_path(Path::new("."),name).is_err());
        }
    }
}
