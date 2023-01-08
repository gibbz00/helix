use crate::{helpers, DynError};
use helix_view::commands::COMMAND_LIST;
use helix_treesitter::TsFeature;
use helix_loader::path;
use std::fs;

pub const COMMANDS_MD_OUTPUT: &str = "commands.md";
pub const LANG_SUPPORT_MD_OUTPUT: &str = "lang-support.md";

fn md_table_heading(cols: &[String]) -> String {
    let mut header = String::new();
    header += &md_table_row(cols);
    header += &md_table_row(&vec!["---".to_string(); cols.len()]);
    header
}

fn md_table_row(cols: &[String]) -> String {
    format!("| {} |\n", cols.join(" | "))
}

fn md_mono(s: &str) -> String {
    format!("`{}`", s)
}

/// Returns an unsorted list of language names that
/// are configured to use a Tree-sitter grammar.
fn default_langs_with_treesitter_support() -> Vec<&str> {
    use helix_core::config::LanguageConfigurations;
    let mut langs_with_ts_support: HashSet<&str> = HashSet::new();
    for lang in LanguageConfigurations::default().language_configurations {
        if let Some(_) = lang.grammar {
            langs_with_ts_support.push(&lang.language_id)
        }
    }
}

pub fn typable_commands() -> Result<String, DynError> {
    let mut md = String::new();
    md.push_str(&md_table_heading(&[
        "Name".to_owned(),
        "Description".to_owned(),
    ]));

    for commmand in COMMAND_LIST {
        let names = std::iter::once(&commmand.name)
            .chain(commmand.aliases.iter())
            .collect::<Vec<_>>()
            .join(", ");

        let description = commmand.description.replace('\n', "<br>");

        md.push_str(&md_table_row(&[names.to_owned(), description.to_owned()]));
    }
    Ok(md)
}

pub fn lang_features() -> Result<String, DynError> {
    let mut md = String::new();
    let ts_features = TsFeature::all();

    let mut cols = vec!["Language".to_owned()];
    cols.append(
        &mut ts_features
            .iter()
            .map(|t| t.long_title().to_string())
            .collect::<Vec<_>>(),
    );
    cols.push("Default LSP".to_owned());

    md.push_str(&md_table_heading(&cols));
    let config = helpers::lang_config();

    let mut langs = config
        .language_configurations
        .iter()
        .map(|l| l.language_id.clone())
        .collect::<Vec<_>>();
    langs.sort_unstable();

    let mut ts_features_to_langs = Vec::new();
    for &feat in ts_features {
        ts_features_to_langs.push((feat, helpers::default_langs_with_treesitter_support(feat)));
    }

    let mut row = Vec::new();
    for lang in langs {
        let lc = config
            .language_configurations
            .iter()
            .find(|l| l.language_id == lang)
            .unwrap(); // lang comes from config
        row.push(lc.language_id.clone());

        for (_feat, support_list) in &ts_features_to_langs {
            row.push(
                if support_list.contains(&lang) {
                    "✓"
                } else {
                    ""
                }
                .to_owned(),
            );
        }
        row.push(
            lc.language_server
                .as_ref()
                .map(|s| s.command.clone())
                .map(|c| md_mono(&c))
                .unwrap_or_default(),
        );

        md.push_str(&md_table_row(&row));
        row.clear();
    }

    Ok(md)
}

pub fn write(filename: &str, data: &str) {
    let error = format!("Could not write to {}", filename);
    let path = path::book_gen().join(filename);
    fs::write(path, data).expect(&error);
}
