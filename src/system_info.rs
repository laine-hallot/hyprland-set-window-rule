use std::path::{Path, PathBuf};

use color_eyre::eyre::{self, WrapErr};
use directories::{ProjectDirs, UserDirs};

pub fn get_data_dir() -> eyre::Result<PathBuf> {
    let directory = if let Ok(s) = std::env::var("RATATUI_TEMPLATE_DATA") {
        PathBuf::from(s)
    } else if let Some(proj_dirs) = ProjectDirs::from("", "", "ratatui-template") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        return Err(eyre::eyre!(
            "Unable to find data directory for ratatui-template"
        ));
    };
    Ok(directory)
}

pub fn get_config_dir() -> eyre::Result<PathBuf> {
    let directory = if let Ok(s) = std::env::var("RATATUI_TEMPLATE_CONFIG") {
        PathBuf::from(s)
    } else if let Some(proj_dirs) = ProjectDirs::from("", "", "ratatui-template") {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        return Err(eyre::eyre!(
            "Unable to find config directory for ratatui-template"
        ));
    };
    Ok(directory)
}

pub fn get_hyprland_dir() -> eyre::Result<PathBuf> {
    let directory = if let Ok(s) = std::env::var("HYPRLAND_CONFIG_DIR") {
        PathBuf::from(s)
    } else if let Some(proj_dirs) = ProjectDirs::from("", "", "hypr") {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        return Err(eyre::eyre!(
            "Unable to find data directory for ratatui-template"
        ));
    };
    Ok(directory)
}
pub fn get_window_rules_dir() -> eyre::Result<PathBuf> {
    let hyprland_dir = get_hyprland_dir();
    return match hyprland_dir {
        Ok(hyprland_dir) => {
            let directory = if let Ok(var_dir) = std::env::var("WINDOW_RULE_DIR") {
                Path::join(&hyprland_dir, var_dir)
            } else {
                Path::join(&hyprland_dir, "window-rules/")
            };
            Ok(directory)
        }
        Err(err) => Err(err),
    };
}
