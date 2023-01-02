use crossterm::{style::{Color, Print, Stylize}, tty::IsTty};
use helix_core::config::LanguageConfigurations;
use helix_loader::grammar::load_runtime_file;
use helix_view::clipboard::get_clipboard_provider;
use std::io::Write;
/// Display general diagnostics.
pub fn general() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let config_file = helix_loader::config_file();
    let lang_file = helix_loader::lang_config_file();
    let log_file = helix_loader::log_file();
    let rt_dir = helix_loader::runtime_dir();
    let clipboard_provider = get_clipboard_provider();

    if config_file.exists() {
        writeln!(stdout, "Config file: {}", config_file.display())?;
    } else {
        writeln!(stdout, "Config file: default")?;
    }
    if lang_file.exists() {
        writeln!(stdout, "Language file: {}", lang_file.display())?;
    } else {
        writeln!(stdout, "Language file: default")?;
    }
    writeln!(stdout, "Log file: {}", log_file.display())?;
    writeln!(stdout, "Runtime directory: {}", rt_dir.display())?;

    if let Ok(path) = std::fs::read_link(&rt_dir) {
        let msg = format!("Runtime directory is symlinked to {}", path.display());
        writeln!(stdout, "{}", msg.yellow())?;
    }
    if !rt_dir.exists() {
        writeln!(stdout, "{}", "Runtime directory does not exist.".red())?;
    }
    if rt_dir.read_dir().ok().map(|it| it.count()) == Some(0) {
        writeln!(stdout, "{}", "Runtime directory is empty.".red())?;
    }
    writeln!(stdout, "Clipboard provider: {}", clipboard_provider.name())?;

    Ok(())
}

pub fn clipboard() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let board = get_clipboard_provider();
    match board.name().as_ref() {
        "none" => {
            writeln!(
                stdout,
                "{}",
                "System clipboard provider: Not installed".red()
            )?;
            writeln!(
                stdout,
                "    {}",
                "For troubleshooting system clipboard issues, refer".red()
            )?;
            writeln!(stdout, "    {}",
                "https://github.com/helix-editor/helix/wiki/Troubleshooting#copypaste-fromto-system-clipboard-not-working"
            .red().underlined())?;
        }
        name => writeln!(stdout, "System clipboard provider: {}", name)?,
    }

    Ok(())
}

fn load_language_configurations() -> LanguageConfigurations  {
    LanguageConfigurations::merged().unwrap_or_else(|err| {
            let mut stderr = std::io::stderr().lock();
            writeln!(stderr,"{}: {}","Error parsing user language config".red(),err)?;
            writeln!(stderr, "{}", "Using default language config".yellow())?;
            LanguageConfigurations::default()
    }) 
}

pub fn languages_all() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let mut language_configurations = load_language_configurations();

    let mut headings = vec!["Language", "LSP", "DAP"];

    for feat in helix_treesitter::probe::TsFeature::all() {
        headings.push(feat.short_title())
    }

    let terminal_cols = crossterm::terminal::size().map(|(c, _)| c).unwrap_or(80);
    let column_width = terminal_cols as usize / headings.len();
    let is_terminal = std::io::stdout().is_tty();

    let column = |item: &str, color: Color| {
        let mut data = format!(
            "{:width$}",
            item.get(..column_width - 2)
                .map(|s| format!("{}…", s))
                .unwrap_or_else(|| item.to_string()),
            width = column_width,
        );
        if is_terminal {
            data = data.stylize().with(color).to_string();
        }

        // We can't directly use println!() because of
        // https://github.com/crossterm-rs/crossterm/issues/589
        let _ = crossterm::execute!(std::io::stdout(), Print(data));
    };

    for heading in headings {
        column(heading, Color::White);
    }
    writeln!(stdout)?;

    language_configurations
        .language
        .sort_unstable_by_key(|l| l.language_id.clone());

    let check_binary = |cmd: Option<String>| match cmd {
        Some(cmd) => match which::which(&cmd) {
            Ok(_) => column(&format!("✓ {}", cmd), Color::Green),
            Err(_) => column(&format!("✘ {}", cmd), Color::Red),
        },
        None => column("None", Color::Yellow),
    };

    for lang in &language_configurations.language {
        column(&lang.language_id, Color::Reset);

        let lsp = lang
            .language_server
            .as_ref()
            .map(|lsp| lsp.command.to_string());
        check_binary(lsp);

        let dap = lang.debugger.as_ref().map(|dap| dap.command.to_string());
        check_binary(dap);

        for ts_feat in helix_treesitter::probe::TsFeature::all() {
            match load_runtime_file(&lang.language_id, ts_feat.runtime_filename()).is_ok() {
                true => column("✓", Color::Green),
                false => column("✘", Color::Red),
            }
        }

        writeln!(stdout)?;
    }

    Ok(())
}

/// Display diagnostics pertaining to a particular language (LSP,
/// highlight queries, etc).
pub fn language(lang_str: String) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let language_configurations = load_language_configurations();
    let lang = match language_configurations
        .language
        .iter()
        .find(|l| l.language_id == lang_str)
    {
        Some(l) => l,
        None => {
            let msg = format!("Language '{}' not found", lang_str);
            writeln!(stdout, "{}", msg.red())?;
            let suggestions: Vec<&str> = language_configurations
                .language
                .iter()
                .filter(|l| l.language_id.starts_with(lang_str.chars().next().unwrap()))
                .map(|l| l.language_id.as_str())
                .collect();
            if !suggestions.is_empty() {
                let suggestions = suggestions.join(", ");
                writeln!(
                    stdout,
                    "Did you mean one of these: {} ?",
                    suggestions.yellow()
                )?;
            }
            return Ok(());
        }
    };

    probe_protocol(
        "language server",
        lang.language_server
            .as_ref()
            .map(|lsp| lsp.command.to_string()),
    )?;

    probe_protocol(
        "debug adapter",
        lang.debugger.as_ref().map(|dap| dap.command.to_string()),
    )?;


    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    for ts_feat in helix_treesitter::probe::TsFeature::all() {
        /// Display diagnostics about a feature that requires tree-sitter
        /// query files (highlights, textobjects, etc).
        let found = match load_runtime_file(lang, feature.runtime_filename()).is_ok() {
            true => "✓".green(),
            false => "✘".red(),
        };
        writeln!(stdout, "{} queries: {}", feature.short_title(), found)?;
    }
    Ok(())
}

/// Display diagnostics about LSP and DAP.
fn probe_protocol(protocol_name: &str, server_cmd: Option<String>) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    let cmd_name = match server_cmd {
        Some(ref cmd) => cmd.as_str().green(),
        None => "None".yellow(),
    };
    writeln!(stdout, "Configured {}: {}", protocol_name, cmd_name)?;

    if let Some(cmd) = server_cmd {
        let path = match which::which(&cmd) {
            Ok(path) => path.display().to_string().green(),
            Err(_) => format!("'{}' not found in $PATH", cmd).red(),
        };
        writeln!(stdout, "Binary for {}: {}", protocol_name, path)?;
    }

    Ok(())
}


pub fn print_health(health_arg: Option<String>) -> std::io::Result<()> {
    match health_arg.as_deref() {
        Some("languages") => languages_all()?,
        Some("clipboard") => clipboard()?,
        None | Some("all") => {
            general()?;
            clipboard()?;
            writeln!(std::io::stdout().lock())?;
            languages_all()?;
        }
        Some(lang) => language(lang.to_string())?,
    }
    Ok(())
}
