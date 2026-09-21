use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    pub fn from_config_root(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().join("tmrch"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::AppPaths;
    use std::path::Path;

    #[test]
    fn storage_uses_tmrch_not_legacy_package_name() {
        let paths = AppPaths::from_config_root("/tmp/config");
        assert_eq!(paths.root(), Path::new("/tmp/config/tmrch"));
    }
}
