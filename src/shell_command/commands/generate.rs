use crate::wayland;
use hyprland::data::*;
use hyprland::prelude::*;
use ratatui::{prelude::*, widgets::*};
use std::rc::Rc;

pub fn exec(float: bool, _persistentsize: bool, tile: bool, _fullscreen: bool) {
    if let Ok(monitors) = Monitors::get() {
        if let Ok(clients) = Clients::get() {
            let selected_client =
                wayland::window_selector::create_window(&clients, Rc::new(monitors)); //select_window(&clients);
            match selected_client {
                Some(client) => {
                    println!("Config: ");
                    if float {
                        println!("windowrule = float, initialTitle:{}", client.initial_title);
                    }
                    if tile {
                        println!("windowrule = tile, initialTitle:{}", client.initial_title);
                    }
                }
                _ => (),
            }
        }
    }
}
