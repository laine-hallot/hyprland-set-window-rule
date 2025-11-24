use super::window_info_row::window_info_row;
use hyprland::data::Client;
use ratatui::prelude::*;

use ratatui::Frame;

pub fn select_window(hovered_client: &Option<Client>, frame: &mut Frame) {
    let layout = Layout::new(
        Direction::Vertical,
        [Constraint::Length(4), Constraint::Fill(1)],
    )
    .flex(layout::Flex::Start)
    .spacing(0)
    .split(frame.area());

    frame.render_widget(
        window_info_row(
            "Select a window: ",
            match &hovered_client {
                Some(selected_client) => selected_client.title.as_str(),
                None => "",
            },
        ),
        layout[0],
    );

    if let Some(selected_client) = &hovered_client {
        let details = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .flex(layout::Flex::Start)
        .spacing(0)
        .split(layout[1]);

        frame.render_widget(
            window_info_row("Title: ", selected_client.title.as_str()),
            details[0],
        );
        frame.render_widget(
            window_info_row("Class: ", selected_client.class.as_str()),
            details[1],
        );
        frame.render_widget(
            window_info_row("Initial Title: ", selected_client.initial_title.as_str()),
            details[2],
        );
        frame.render_widget(
            window_info_row("Initial Class: ", selected_client.initial_class.as_str()),
            details[3],
        );
    }
}
