use color_eyre::Result;
use hyprland::data::Client;
use hyprlang::Hyprland;
use regex::Regex;
use std::{fs, path::Path, rc::Rc};

use crate::system_info::get_window_rules_dir;

#[derive(PartialEq, Eq, Debug)]
pub enum SelectWindowBy {
    Title,
    Class,
    InitialClass,
    InitialTitle,
}

#[derive(PartialEq, Eq, Debug)]
pub enum WindowPlacement {
    Float,
    Tile,
}
#[derive(PartialEq, Eq, Debug)]
pub struct WindowOptions {
    pub window_placement: WindowPlacement,
    pub fullscreen: bool,
}

pub fn create_window_rule_config(
    client: Client,
    cli_options: &WindowOptions,
    select_by_list: Rc<Vec<SelectWindowBy>>,
) -> Result<()> {
    // Create Hyprland config (handlers auto-registered)
    let mut hypr = Hyprland::new();
    let config = hypr.config_mut();

    let window_position = match cli_options.window_placement {
        WindowPlacement::Float => "float",
        WindowPlacement::Tile => "tile",
    };

    let selector = select_by_list
        .iter()
        .map(|select_by| match select_by {
            SelectWindowBy::Title => format!("title:({})", client.title),
            SelectWindowBy::Class => format!("class:({})", client.class),
            SelectWindowBy::InitialClass => format!("initialClass:({})", client.initial_class),
            SelectWindowBy::InitialTitle => format!("initialTitle:({})", client.initial_title),
        })
        .collect::<Vec<String>>()
        .join(", ");

    config.set_string("windowrule", format!("{window_position}, {selector}"));

    let regex = Regex::new(r"(?m)\W+").unwrap();
    let name = format!(
        "{:.8}-{:.8}.conf",
        regex.replace_all(&client.title, ""),
        regex.replace_all(&client.class, "")
    );

    let window_rules_path = get_window_rules_dir()?;
    if !fs::exists(&window_rules_path)? {
        fs::create_dir_all(&window_rules_path)?;
    }

    let file_path = Path::join(&window_rules_path, name);
    println!(
        "Writing \"windowrule {window_position}, {selector}\" to {}",
        file_path.to_string_lossy()
    );
    config.save_as(file_path)?;

    Ok(())
}
