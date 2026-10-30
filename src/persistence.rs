use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use termatrix::settings::Settings;

pub fn load_default() -> io::Result<(PathBuf, Settings)> {
    let dirs = directories::ProjectDirs::from("", "", "termatrix")
        .ok_or_else(|| io::Error::other("cannot locate configuration directory; use --config"))?;
    load_default_in(dirs.config_dir())
}
fn load_default_in(root: &Path) -> io::Result<(PathBuf, Settings)> {
    let path = root.join("profiles").join("default.toml");
    let settings = match load_optional(&path)? {
        Some(settings) => settings,
        None => load(&root.join("settings.toml"))?,
    };
    Ok((path, settings))
}
pub fn load(path: &Path) -> io::Result<Settings> {
    Ok(load_optional(path)?.unwrap_or_default())
}
fn load_optional(path: &Path) -> io::Result<Option<Settings>> {
    match fs::read_to_string(path) {
        Ok(text) => {
            let mut table: toml::Table = toml::from_str(&text).map_err(io::Error::other)?;
            // Migration only: obsolete UI limits never override the compiled caps.
            table.remove("allowed_min_speed");
            table.remove("allowed_max_speed");
            let s: Settings = table.try_into().map_err(io::Error::other)?;
            s.validate().map_err(io::Error::other)?;
            Ok(Some(s))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    settings.validate().map_err(io::Error::other)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(
        toml::to_string_pretty(settings)
            .map_err(io::Error::other)?
            .as_bytes(),
    )?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_profile_location_and_legacy_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let (path, fresh) = load_default_in(root).unwrap();
        assert_eq!(path, root.join("profiles/default.toml"));
        assert_eq!(fresh.glyph_set, "special");
        assert!(!path.exists());
        let legacy = root.join("settings.toml");
        fs::write(&legacy, "glyph_set = 'binary'").unwrap();
        let (target, migrated) = load_default_in(root).unwrap();
        assert_eq!(migrated.glyph_set, "binary");
        save(&target, &migrated).unwrap();
        assert_eq!(fs::read_to_string(&legacy).unwrap(), "glyph_set = 'binary'");
        fs::write(&legacy, "glyph_set = 'hex'").unwrap();
        assert_eq!(load_default_in(root).unwrap().1.glyph_set, "binary");
        fs::write(&target, "invalid toml [[").unwrap();
        assert!(load_default_in(root).is_err());
    }
    #[test]
    fn explicit_missing_file_does_not_inherit_legacy() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("settings.toml"), "glyph_set = 'binary'").unwrap();
        assert_eq!(
            load(&dir.path().join("custom.toml")).unwrap().glyph_set,
            "special"
        );
    }
    #[test]
    fn obsolete_limits_load_but_are_not_saved_or_enforced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        fs::write(&path,"allowed_min_speed = 10.0\nallowed_max_speed = 11.0\nmin_speed = 2.0\nmax_speed = 12.0\n").unwrap();
        let settings = load(&path).unwrap();
        assert_eq!(settings.min_speed, 2.);
        assert_eq!(settings.max_speed, 12.);
        save(&path, &settings).unwrap();
        assert!(!fs::read_to_string(path).unwrap().contains("allowed_"));
    }
    #[test]
    fn save_load_and_invalid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        save(&path, &Settings::default()).unwrap();
        assert_eq!(load(&path).unwrap().colors, "gG");
        fs::write(&path, "min_speed = -1").unwrap();
        assert!(load(&path).is_err());
    }
    #[test]
    fn failed_replace_cleans_temporary_and_preserves_destination() {
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("directory");
        fs::create_dir(&destination).unwrap();
        assert!(save(&destination, &Settings::default()).is_err());
        assert!(destination.is_dir());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn invalid_save_preserves_existing_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        save(&path, &Settings::default()).unwrap();
        let before = fs::read(&path).unwrap();
        let invalid = Settings {
            min_speed: -1.,
            ..Settings::default()
        };
        assert!(save(&path, &invalid).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
