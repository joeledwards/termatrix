use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use termatrix::settings::Settings;

pub fn default_path() -> io::Result<PathBuf> {
    directories::ProjectDirs::from("", "", "termatrix")
        .map(|d| d.config_dir().join("settings.toml"))
        .ok_or_else(|| io::Error::other("cannot locate configuration directory; use --config"))
}
pub fn load(path: &Path) -> io::Result<Settings> {
    match fs::read_to_string(path) {
        Ok(text) => {
            let mut table: toml::Table = toml::from_str(&text).map_err(io::Error::other)?;
            // Migration only: obsolete UI limits never override the compiled caps.
            table.remove("allowed_min_speed");
            table.remove("allowed_max_speed");
            let s: Settings = table.try_into().map_err(io::Error::other)?;
            s.validate().map_err(io::Error::other)?;
            Ok(s)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
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
