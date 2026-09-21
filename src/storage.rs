use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub const APP_NAME: &str = "tmrch";

#[derive(Debug)]
pub enum StorageError {
    Io { path: PathBuf, source: io::Error },
    InvalidConfig { path: PathBuf, message: String },
    ConfigDirectoryUnavailable,
}
impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::InvalidConfig { path, message } => write!(f, "{}: {message}", path.display()),
            Self::ConfigDirectoryUnavailable => {
                write!(f, "ユーザー設定ディレクトリを取得できません")
            }
        }
    }
}
impl std::error::Error for StorageError {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    pub editor: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    root: PathBuf,
}
impl AppPaths {
    pub fn from_config_root(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().join(APP_NAME),
        }
    }
    pub fn from_system() -> Result<Self, StorageError> {
        let base = if cfg!(target_os = "macos") {
            env::var_os("HOME").map(|v| PathBuf::from(v).join("Library/Application Support"))
        } else if cfg!(windows) {
            env::var_os("APPDATA").map(PathBuf::from)
        } else {
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| env::var_os("HOME").map(|v| PathBuf::from(v).join(".config")))
        }
        .ok_or(StorageError::ConfigDirectoryUnavailable)?;
        Ok(Self::from_config_root(base))
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates")
    }
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }
    pub fn template_file(&self, extension: &str) -> PathBuf {
        self.templates_dir().join(extension)
    }
    pub fn initialize(&self) -> Result<(), StorageError> {
        fs::create_dir_all(self.templates_dir()).map_err(|source| StorageError::Io {
            path: self.templates_dir(),
            source,
        })
    }
    pub fn read_template(&self, extension: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let path = self.template_file(extension);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(StorageError::Io { path, source }),
        }
    }
    pub fn write_template(&self, extension: &str, bytes: &[u8]) -> Result<(), StorageError> {
        self.initialize()?;
        let path = self.template_file(extension);
        fs::write(&path, bytes).map_err(|source| StorageError::Io { path, source })
    }
    pub fn load_config(&self) -> Result<Config, StorageError> {
        let path = self.config_file();
        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(source) => return Err(StorageError::Io { path, source }),
        };
        let mut config = Config::default();
        for line in contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let Some((key, value)) = line.split_once('=') else {
                return Err(StorageError::InvalidConfig {
                    path,
                    message: "TOML の形式が不正です".into(),
                });
            };
            if key.trim() == "editor" {
                let value = value.trim();
                if !(value.starts_with('"') && value.ends_with('"')) {
                    return Err(StorageError::InvalidConfig {
                        path,
                        message: "editor は文字列で指定してください".into(),
                    });
                }
                config.editor = Some(value[1..value.len() - 1].into());
            }
        }
        Ok(config)
    }
    pub fn save_config(&self, config: &Config) -> Result<(), StorageError> {
        self.initialize()?;
        let path = self.config_file();
        let contents = config
            .editor
            .as_ref()
            .map(|editor| format!("editor = \"{editor}\"\n"))
            .unwrap_or_default();
        fs::write(&path, contents).map_err(|source| StorageError::Io { path, source })
    }
}

#[cfg(test)]
mod tests {
    use super::{AppPaths, Config};
    use std::{fs, path::PathBuf};
    fn temp_root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("tmrch-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn storage_uses_tmrch_not_legacy_package_name() {
        assert_eq!(
            AppPaths::from_config_root("/tmp/config").root(),
            std::path::Path::new("/tmp/config/tmrch")
        );
    }
    #[test]
    fn stores_template_bytes_outside_config_toml() {
        let root = temp_root("bytes");
        let paths = AppPaths::from_config_root(&root);
        let bytes = b"hello\n";
        paths.write_template("rs", bytes).unwrap();
        paths
            .save_config(&Config {
                editor: Some("code --wait".into()),
            })
            .unwrap();
        assert_eq!(paths.read_template("rs").unwrap(), Some(bytes.to_vec()));
        assert!(
            !fs::read_to_string(paths.config_file())
                .unwrap()
                .contains("hello")
        );
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn injected_root_contains_all_created_files() {
        let root = temp_root("injected");
        let paths = AppPaths::from_config_root(&root);
        paths.initialize().unwrap();
        assert!(paths.templates_dir().is_dir());
        assert!(!root.join("tmtch").exists());
        let _ = fs::remove_dir_all(root);
    }
}
