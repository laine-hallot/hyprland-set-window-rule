use std::rc::Rc;

use color_eyre::Result;

use crate::hyprland_config::{WindowOptions, WindowPlacement, create_window_rule_config};
use crate::shell_command::types::SelectWindowBy;
use crate::tui::root;

pub fn exec(
    float: &bool,
    tile: &bool,
    fullscreen: &bool,
    select_by_list: &Vec<SelectWindowBy>,
) -> Result<()> {
    let window_options = WindowOptions {
        fullscreen: fullscreen.clone(),
        window_placement: match (float, tile) {
            (true, false) => WindowPlacement::Float,
            (false, true) => WindowPlacement::Tile,
            _ => panic!("--float and --tile are mutually exclusive options"),
        },
    };

    let select_by_list: Vec<crate::hyprland_config::SelectWindowBy> = select_by_list
        .iter()
        .map(|select_by| match select_by {
            SelectWindowBy::Title => crate::hyprland_config::SelectWindowBy::Title,
            SelectWindowBy::Class => crate::hyprland_config::SelectWindowBy::Class,
            SelectWindowBy::InitialClass => crate::hyprland_config::SelectWindowBy::InitialClass,
            SelectWindowBy::InitialTitle => crate::hyprland_config::SelectWindowBy::InitialTitle,
        })
        .collect();
    let selected_client = root::app(Some(window_options), Some(Rc::new(select_by_list)))?;

    match selected_client {
        Some(selection) => {
            create_window_rule_config(
                selection.client.clone(),
                &selection.window_options,
                selection.select_by_list,
            )
            .expect("Failed generating config");
        }
        None => {}
    };

    Ok(())
}
