mod hyprland_config;
mod shell_command;
mod system_info;
mod wayland;

use shell_command::commands::options_exec;
use shell_command::types::*;

use color_eyre::{Result, eyre};
use crossterm::event::{self, Event};
use eyre::Error;
use ratatui::{DefaultTerminal, Frame};

use clap::Parser;

fn main() -> Result<()> {
    //window_selector::create_window();
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if cli.version {
        options_exec::version();
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    return match &cli.command {
        Some(Commands::Generate {
            float,
            persistentsize,
            tile,
            fullscreen,
        }) => {
            /* color_eyre::install()?;
            let terminal = ratatui::init();
            let result = run(terminal);
            ratatui::restore(); */

            shell_command::commands::generate::exec(
                float.clone(),
                persistentsize.clone(),
                tile.clone(),
                fullscreen.clone(),
            );

            return Ok(()); //result;
        }
        None => Err(Error::msg("Unknown option")),
    };
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
