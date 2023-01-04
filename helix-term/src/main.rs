use anyhow::{Context, Error, Result};
use crossterm::event::EventStream;
mod help;
use crate::{
    application::Application,
    args::Args,
    config::Config,
    help::help,
    health,
};

use helix_loader::VERSION_AND_GIT_HASH;
use helix_core::syntax::LanguageConfigurations;
use std::path::PathBuf;

fn main() -> Result<()> {
    let exit_code = main_impl()?;
    std::process::exit(exit_code);
}

#[tokio::main]
async fn main_impl() -> Result<i32> {
    let args = Args::parse_args().context("faild to parse arguments")?;
    setup_logging(args.log_file, args.verbosity).context("failed to initialize logging")?;

    // IMPROVEMENT: minor, section could be placed into a cleaner match statement
    // if args is made into an enum.
    if args.display_help {
        print!("{}", help());
        return Ok(0);
    }
    if args.display_version {
        println!("helix {}", VERSION_AND_GIT_HASH);
        return Ok(0);
    }
    if args.health {
        if let Err(err) = health::print_health(args.health_arg) {
            // Piping to for example `head -10` requires special handling:
            // https://stackoverflow.com/a/65760807/7115678
            if err.kind() != std::io::ErrorKind::BrokenPipe {
                return Err(err.into());
            }
        }
        return Ok(0);
    }
    if args.fetch_grammars {
        helix_treesitter::grammar::fetch_grammars()?;
        return Ok(0);
    }
    if args.build_grammars {
        helix_treesitter::grammar::build_grammars(None)?;
        return Ok(0);
    }

    helix_loader::setup_config_file(args.config_file);
    let config = Config::load_user_config().unwrap_or_else(|err| {
        eprintln!("Bad config: {}", err);
        eprintln!("Press <ENTER> to continue with default config");
        let wait_for_enter = std::io::stdin().read(&mut []);
        Config::default()
    });

    let language_configurations = LanguageConfigurations::merged().unwrap_or_else(|err| {
        eprintln!("Bad language config: {}", err);
        eprintln!("Press <ENTER> to continue with default language config");
        let wait_for_enter = std::io::stdin().read(&mut []);
        LanguageConfigurations::default()
    });

    // TODO: use the thread local executor to spawn the application task separately from the work pool
    let mut app = Application::new(args, config, language_configurations)
        .context("unable to create new application")?;

    let exit_code = app.run(&mut EventStream::new()).await?;
    Ok(exit_code)
}

fn setup_logging(logpath: Option<PathBuf>, verbosity: u64) -> Result<()> {
    helix_loader::setup_log_file(logpath); 
    let logpath = helix_loader::log_file();

    let mut base_config = fern::Dispatch::new();
    base_config = match verbosity {
        0 => base_config.level(log::LevelFilter::Warn),
        1 => base_config.level(log::LevelFilter::Info),
        2 => base_config.level(log::LevelFilter::Debug),
        _3_or_more => base_config.level(log::LevelFilter::Trace),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} [{}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(logpath)?);

    base_config.chain(file_config).apply()?;

    Ok(())
}
