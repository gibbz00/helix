use std::path::{Path, PathBuf};

pub fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().to_path_buf()
}

/// /VERSION
pub fn version() -> PathBuf {
    project_root().join("VERSION")
}

pub fn book_gen() -> PathBuf {
    project_root().join("book/src/generated/")
}

pub fn ts_queries() -> PathBuf {
    project_root().join("runtime/queries")
}

pub fn themes() -> PathBuf {
    project_root().join("runtime/themes")
}

pub fn default_lang_configs() -> PathBuf {
    project_root().join("helix-config/src/language_config/languages.toml")
}

pub fn default_theme() -> PathBuf {
    project_root().join("helix-config/src/theme_config/theme.toml")
}

pub fn default_base16_theme() -> PathBuf {
    project_root().join("helix-config/src/theme_config/base16_theme.toml")
}
