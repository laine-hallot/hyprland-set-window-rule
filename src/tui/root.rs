use super::widgets::window_select::select_window;

use crate::hyprland_config::SelectWindowBy;
use crate::hyprland_config::WindowOptions;
use crate::wayland;
use crate::wayland::window_selector::Message;

use hyprland::data::*;
use hyprland::prelude::*;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use std::rc::Rc;
use std::time::Duration;

use color_eyre::Result;
use ratatui::Frame;

#[derive(Debug, Default, PartialEq)]
enum PageState {
    #[default]
    WindowSelect,
}

pub enum OrPrompt<T> {
    Args(T),
    Prompt,
}

struct Model {
    hovered_client: Option<Client>,
    running_state: RunningState,
    page: PageState,
    window_options: OrPrompt<WindowOptions>,
    select_by_list: OrPrompt<Rc<Vec<SelectWindowBy>>>,
}

impl Model {
    fn new(
        window_options: Option<WindowOptions>,
        select_by_list: Option<Rc<Vec<SelectWindowBy>>>,
    ) -> Self {
        return Self {
            hovered_client: None,
            running_state: RunningState::default(),
            page: PageState::default(),
            window_options: match window_options {
                Some(window_options) => OrPrompt::Args(window_options),
                None => OrPrompt::Prompt,
            },
            select_by_list: match select_by_list {
                Some(select_by_list) => OrPrompt::Args(select_by_list),
                None => OrPrompt::Prompt,
            },
        };
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq, Eq, Debug)]
pub struct WindowSelection {
    pub client: Client,
    pub window_options: WindowOptions,
    pub select_by_list: Rc<Vec<SelectWindowBy>>,
}

pub fn app(
    window_options: Option<WindowOptions>,
    select_by_list: Option<Rc<Vec<SelectWindowBy>>>,
) -> Result<Option<WindowSelection>> {
    tui::install_panic_hook();
    color_eyre::install()?;

    let monitors = Monitors::get()?;
    let clients = Clients::get()?;

    let selection_result = render(clients, monitors, window_options, select_by_list);

    tui::restore_terminal()?;
    return selection_result;
}

fn view(model: &mut Model, frame: &mut Frame) {
    match &model.page {
        PageState::WindowSelect => select_window(&model.hovered_client, frame),
    }
}

enum Messages {
    ClientUpdate(Client),
    RunningState(RunningState),
}

fn update(model: &mut Model, message: Option<Messages>) -> Option<()> {
    if let Some(message) = message {
        match message {
            Messages::ClientUpdate(client) => {
                model.hovered_client = Some(client.clone());
            }
            Messages::RunningState(running_state) => {
                model.running_state = running_state;
            }
        };
    }
    None
}

fn render(
    clients: Clients,
    monitors: Monitors,
    window_options: Option<WindowOptions>,
    select_by_list: Option<Rc<Vec<SelectWindowBy>>>,
) -> Result<Option<WindowSelection>> {
    let mut model = Model::new(window_options, select_by_list);

    let mut terminal = tui::init_terminal().expect("unable to create terminal ui");
    let mut window_select = wayland::window_selector::WindowSelect::new(clients, monitors);

    while model.running_state == RunningState::Running {
        if model.page == PageState::WindowSelect {
            match window_select.update() {
                Message::Done => {
                    window_select.clean_up();
                    if let (OrPrompt::Args(_), OrPrompt::Args(_), Some(_)) = (
                        &model.select_by_list,
                        &model.window_options,
                        &model.hovered_client,
                    ) {
                        update(&mut model, Some(Messages::RunningState(RunningState::Done)));
                    };
                }
                Message::HoveredClient(maybe_id) => {
                    if let Some(id) = maybe_id {
                        if let Ok(clients) = Clients::get() {
                            if let Some(client) = clients
                                .iter()
                                .find(|client| client.address.to_string() == id.to_string())
                            {
                                update(&mut model, Some(Messages::ClientUpdate(client.clone())));
                            }
                        }
                    }
                }
            }
        }

        if let Ok(current_update) = handle_event(&model) {
            update(&mut model, current_update);
        }

        terminal.draw(|f| view(&mut model, f))?;
    }
    if let (OrPrompt::Args(select_by_list), OrPrompt::Args(window_options), Some(client)) = (
        model.select_by_list,
        model.window_options,
        model.hovered_client,
    ) {
        return Ok(Some(WindowSelection {
            client,
            window_options: window_options,
            select_by_list: select_by_list,
        }));
    };
    Ok(None)
}

fn handle_event(_: &Model) -> color_eyre::Result<Option<Messages>> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Messages> {
    match key.code {
        KeyCode::Char('q') => Some(Messages::RunningState(RunningState::Done)),
        _ => None,
    }
}

mod tui {
    use ratatui::{
        Terminal,
        backend::{Backend, CrosstermBackend},
        crossterm::{
            ExecutableCommand,
            terminal::{
                EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
            },
        },
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}
