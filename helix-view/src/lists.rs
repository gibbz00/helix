use crate::{UITree, document::SCRATCH_BUFFER_NAME};

trait List {
    pub fn get<T>(ui_tree: &UITree) -> Vec<T>;
}

pub enum List {
    Buffers(List),
    Themes(List),
    WorkspaceSymbols(List),
    WorkspaceFiles(List),
    LSPWorkspaceCommands(List),
    ConfigOptions(List),
    Languages(List),
    Commands(List),
    // Found Directories directly under current working directory
    DirectoryPaths(List),
    // Found Files directly under current working directory
    FilePaths(List)
}


impl List for List::Buffers {
    fn get() {
        ui_tree.documents.values().map(|document| {
            let name = document.relative_path()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| String::from(SCRATCH_BUFFER_NAME));
            ((0..), Cow::from(name))
        }).collect()
    }
}

impl List for List::Themes {
    fn get() {
        use crate::theme::Loader::read_names;
        let mut theme_names: List = read_names(&helix_loader::runtime_themes());
        theme_names.extend(read_names(&helix_loader::user_themes_dir()));
        theme_names.extend(read_names(&helix_loader::repo_paths::default_theme()));
        theme_names.extend(read_names(&helix_loader::repo_paths::default_base16_theme()));
        theme_names.sort();
        theme_names.dedup();
        theme_names
    }
}

impl List for List::WorkspaceSymbols {
    fn get() {
        vec!["workspace_symbol_stub".to_string()]
    }
}

impl List for List::WorkspaceFiles {
    fn get() {
        vec!["file_name_stub".to_string()]
    }
}

impl List for List::LSPWorkspaceCommands {
    fn get() {
        let (_, focused_document) = current_ref!(ui_tree);

        if Some(language_server) = focused_document.language_server() {
            if Some(lsp_workspace_commands) = language_server.capabilities().execute_command_provider.commands {
                return lsp_workspace_commands
            }
        }
        return Vec::new()
    }
}

impl List for List::Directories {
    fn get() {
        vec!["directory_name_stub".to_string()]
    }
}

impl List for List::Filepaths {
    fn get() {
        vec!["file-paths-stub".to_string()]
    }
}

impl List for List::ConfigOptions {
    fn get() {
         static KEYS: Lazy<Vec<String>> = Lazy::new(|| {
                let mut config_options = Vec::new();
                let json_config = serde_json::json!(crate::config::Config::default());
                traverse_config_tree(&json_config, &mut config_options, None);
                config_options
            });       
    }

    fn traverse_config_tree(json_config: &serde_json::Value, config_options: &mut Vec<String>, scope: Option<&str>) {
        if let Some(tree) = json_config.as_object() {
            for (key, sub_tree) in tree.iter() {
                let config_option = match scope {
                    Some(scope) => format!("{scope}.{key}"),
                    None => key.clone(),
                };
                traverse_config_tree(sub_tree, config_options, Some(&config_option));
                if !sub_tree.is_object() {
                    config_options.push(config_option);
                }
            }
        }
    }
}

impl List for List::Languages {
    fn get() {
        use helix_core::syntax::LanguageConfigurations;
        let language_ids = LanguageConfigurations::merged()
            .unwrap_or_default()
            .map(|language_config| &language_config.language_id)
            .push("text".to_string());
    }
}

impl List for List::Commands {
    fn get() {
        COMMAND_LIST.iter().collect();
    }
}