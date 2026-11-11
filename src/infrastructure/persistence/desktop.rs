use super::SaveData;
use std::{fs, path::PathBuf};

fn save_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let root = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));

    root.unwrap_or_else(|| PathBuf::from("."))
        .join("RetroPong")
        .join("save.json")
}

pub fn load() -> Result<SaveData, String> {
    let path = save_path();
    if !path.exists() { return Ok(SaveData::default()); }
    let raw = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&raw).map_err(|error| format!("save file could not be parsed ({}): {error}", path.display()))
}

pub fn save(data: &SaveData) -> Result<(), String> {
    let path = save_path();
    let parent = path.parent().ok_or_else(|| "save path has no parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;

    let json = serde_json::to_vec_pretty(data).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, json).map_err(|error| format!("{}: {error}", temporary.display()))?;

    #[cfg(windows)]
    {
        let backup = path.with_extension("json.bak");
        if path.exists() {
            if backup.exists() { fs::remove_file(&backup).map_err(|error| error.to_string())?; }
            fs::rename(&path, &backup).map_err(|error| format!("could not back up {}: {error}", path.display()))?;
        }
        if let Err(error) = fs::rename(&temporary, &path) {
            if backup.exists() { let _ = fs::rename(&backup, &path); }
            return Err(format!("could not replace {}: {error}", path.display()));
        }
        if backup.exists() { let _ = fs::remove_file(backup); }
    }
    #[cfg(not(windows))]
    fs::rename(&temporary, &path).map_err(|error| format!("could not replace {}: {error}", path.display()))?;

    Ok(())
}
