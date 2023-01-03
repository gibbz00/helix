mod repo_paths;

use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use std::path::PathBuf;

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");
pub static RUNTIME_DIR: once_cell::sync::Lazy<PathBuf> = once_cell::sync::Lazy::new(runtime_dir);

static CONFIG_FILE: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();
static LOG_FILE: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();

pub fn config_file() -> PathBuf {
    match CONFIG_FILE.get() {
        Some(config_path) => config_path.to_path_buf(),
        None => {
            initialize_config_file(None);
            config_file()
        }
    }
}
pub fn initialize_config_file(specified_file: Option<PathBuf>) {
    let config_file = specified_file.unwrap_or_else(|| {
        let config_dir = config_dir();
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).ok();
        }
        config_dir.join("config.toml")
    });
    CONFIG_FILE.set(config_file).ok();
}
pub fn config_dir() -> PathBuf {
    if let Ok(env_config_dir) = std::env::var("HELIX_CONFIG_DIR") {
        return env_config_dir.into();
    }

    let strategy = choose_base_strategy().expect("Unable to determine system base directory specification!");
    let mut path = strategy.config_dir();
    path.push("helix");
    path
}
pub fn log_file() -> PathBuf {
    match LOG_FILE.get() {
        Some(log_path) => log_path.to_path_buf(),
        None => {
            initialize_log_file(None);
            log_file()
        }
    }
}
pub fn initialize_log_file(specified_file: Option<PathBuf>) {
    let log_file = specified_file.unwrap_or_else(|| {
        let log_dir = cache_dir();
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir).ok();
        }
        log_dir.join("helix.log")
    });
    LOG_FILE.set(log_file).ok();
}
pub fn cache_dir() -> PathBuf {
    if let Ok(env_config_dir) = std::env::var("HELIX_CACHE_DIR") {
        return env_config_dir.into();
    }
    let strategy = choose_base_strategy().expect("Unable to determine system base directory specification!");
    let mut path = strategy.cache_dir();
    path.push("helix");
    path
}

// TODO: shouldn't it also look for XDG_RUNTIME_DIR? 
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("HELIX_RUNTIME") {
        return dir.into();
    }

    const RT_DIR: &str = "runtime";
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        // this is the directory of the crate being run by cargo, we need the workspace path so we take the parent
        let path = std::path::PathBuf::from(dir).parent().unwrap().join(RT_DIR);
        log::debug!("runtime dir: {}", path.to_string_lossy());
        return path;
    }

    let conf_dir = config_dir().join(RT_DIR);
    if conf_dir.exists() {
        return conf_dir;
    }

    // fallback to location of the executable being run
    // canonicalize the path in case the executable is symlinked
    std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::canonicalize(path).ok())
        .and_then(|path| path.parent().map(|path| path.to_path_buf().join(RT_DIR)))
        .unwrap()
}

pub fn lang_config_file() -> PathBuf {
    config_dir().join("languages.toml")
}
pub fn default_lang_config() -> toml::Value {
    toml::from_slice(&std::fs::read(repo_paths::default_lang_configs()).unwrap())
        .expect("Could not parse built-in languages.toml to valid toml")
}
pub fn merged_lang_config() -> Result<toml::Value, toml::de::Error> {
    let config = local_lang_config_dirs()
        .into_iter()
        .chain([config_dir()].into_iter())
        .map(|path| path.join("languages.toml"))
        .filter_map(|file| {
            std::fs::read(&file)
                .map(|config| toml::from_slice(&config))
                .ok()
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .chain([default_lang_config()].into_iter())
        .fold(toml::Value::Table(toml::value::Table::default()), |a, b| {
            merge_toml_values(b, a, 3)
        });

    Ok(config)
}

pub fn local_lang_config_dirs() -> Vec<PathBuf> {
    let current_dir = std::env::current_dir().expect("Unable to determine current directory.");
    let mut directories = Vec::new();
    for ancestor in current_dir.ancestors() {
        if ancestor.join(".git").exists() {
            directories.push(ancestor.to_path_buf().join(".helix"));
            // Don't go higher than repo if we're in one
            break;
        } else if ancestor.join(".helix").is_dir() {
            directories.push(ancestor.to_path_buf().join(".helix"));
        }
    }
    log::debug!("Located langauge configuration folders: {:?}", directories);
    directories
}

/// Merge two TOML documents, merging values from `right` onto `left`
///
/// When an array exists in both `left` and `right`, `right`'s array is
/// used. When a table exists in both `left` and `right`, the merged table
/// consists of all keys in `left`'s table unioned with all keys in `right`
/// with the values of `right` being merged recursively onto values of
/// `left`.
///
/// `merge_toplevel_arrays` controls whether a top-level array in the TOML
/// document is merged instead of overridden. This is useful for TOML
/// documents that use a top-level array of values like the `languages.toml`,
/// where one usually wants to override or add to the array instead of
/// replacing it altogether.
///
/// For example:
///
/// left:
///   [[language]]
///   name = "toml"
///   language-server = { command = "taplo", args = ["lsp", "stdio"] }
///
/// right:
///   [[language]]
///   language-server = { command = "/usr/bin/taplo" }
///
/// result:
///   [[language]]
///   name = "toml"
///   language-server = { command = "/usr/bin/taplo" }
pub fn merge_toml_values(left: toml::Value, right: toml::Value, merge_depth: usize) -> toml::Value {
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    match (left, right) {
        (Value::Array(mut left_items), Value::Array(right_items)) => {
            // The top-level arrays should be merged but nested arrays should
            // act as overrides. For the `languages.toml` config, this means
            // that you can specify a sub-set of languages in an overriding
            // `languages.toml` but that nested arrays like Language Server
            // arguments are replaced instead of merged.
            if merge_depth > 0 {
                left_items.reserve(right_items.len());
                for rvalue in right_items {
                    let lvalue = get_name(&rvalue)
                        .and_then(|rname| {
                            left_items.iter().position(|v| get_name(v) == Some(rname))
                        })
                        .map(|lpos| left_items.remove(lpos));
                    let mvalue = match lvalue {
                        Some(lvalue) => merge_toml_values(lvalue, rvalue, merge_depth - 1),
                        None => rvalue,
                    };
                    left_items.push(mvalue);
                }
                Value::Array(left_items)
            } else {
                Value::Array(right_items)
            }
        }
        (Value::Table(mut left_map), Value::Table(right_map)) => {
            if merge_depth > 0 {
                for (rname, rvalue) in right_map {
                    match left_map.remove(&rname) {
                        Some(lvalue) => {
                            let merged_value = merge_toml_values(lvalue, rvalue, merge_depth - 1);
                            left_map.insert(rname, merged_value);
                        }
                        None => {
                            left_map.insert(rname, rvalue);
                        }
                    }
                }
                Value::Table(left_map)
            } else {
                Value::Table(right_map)
            }
        }
        // Catch everything else we didn't handle, and use the right value
        (_, value) => value,
    }
}

#[cfg(test)]
mod merge_toml_tests {
    use super::merge_toml_values;
    use toml::Value;

    #[test]
    fn language_toml_map_merges() {
        const USER: &str = r#"
        [[language]]
        name = "nix"
        test = "bbb"
        indent = { tab-width = 4, unit = "    ", test = "aaa" }
        "#;

        let base: Value = toml::from_slice(include_bytes!("../../languages.toml"))
            .expect("Couldn't parse built-in languages config");
        let user: Value = toml::from_str(USER).unwrap();

        let merged = merge_toml_values(base, user, 3);
        let languages = merged.get("language").unwrap().as_array().unwrap();
        let nix = languages
            .iter()
            .find(|v| v.get("name").unwrap().as_str().unwrap() == "nix")
            .unwrap();
        let nix_indent = nix.get("indent").unwrap();

        // We changed tab-width and unit in indent so check them if they are the new values
        assert_eq!(
            nix_indent.get("tab-width").unwrap().as_integer().unwrap(),
            4
        );
        assert_eq!(nix_indent.get("unit").unwrap().as_str().unwrap(), "    ");
        // We added a new keys, so check them
        assert_eq!(nix.get("test").unwrap().as_str().unwrap(), "bbb");
        assert_eq!(nix_indent.get("test").unwrap().as_str().unwrap(), "aaa");
        // We didn't change comment-token so it should be same
        assert_eq!(nix.get("comment-token").unwrap().as_str().unwrap(), "#");
    }

    #[test]
    fn language_toml_nested_array_merges() {
        const USER: &str = r#"
        [[language]]
        name = "typescript"
        language-server = { command = "deno", args = ["lsp"] }
        "#;

        let base: Value = toml::from_slice(include_bytes!("../../languages.toml"))
            .expect("Couldn't parse built-in languages config");
        let user: Value = toml::from_str(USER).unwrap();

        let merged = merge_toml_values(base, user, 3);
        let languages = merged.get("language").unwrap().as_array().unwrap();
        let ts = languages
            .iter()
            .find(|v| v.get("name").unwrap().as_str().unwrap() == "typescript")
            .unwrap();
        assert_eq!(
            ts.get("language-server")
                .unwrap()
                .get("args")
                .unwrap()
                .as_array()
                .unwrap(),
            &vec![Value::String("lsp".into())]
        )
    }
}
