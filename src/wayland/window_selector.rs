use super::buffer_surface::{BaseSurfaceBuffer, BufferSurface, ClientRegion, HasOutput, InProcess};
use super::protocols::State;

use hyprland::data::{Client as HyClient, Clients as HyClients, Monitors as HyMonitors};
use hyprland::shared::{Address, WorkspaceId};
use wayland_client::EventQueue;

use std::collections::HashMap;

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat, wl_shm},
};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

#[derive(Debug)]
pub enum Message {
    Done,
    HoveredClient(Option<Address>),
}

pub struct Running {
    state: State,
    event_queue: EventQueue<State>,
    client_regions: Vec<ClientRegion>,
    mapped_client_id_and_client: HashMap<Address, HyClient>,
}

pub enum Desu {
    Running(Running),
    Done,
}

pub struct WindowSelect {
    pub stuff: Desu,
}
impl WindowSelect {
    pub fn update(self: &mut Self) -> Message {
        return match &mut self.stuff {
            Desu::Running(stuff) => {
                return update_running(
                    &mut stuff.state,
                    &mut stuff.event_queue,
                    &mut stuff.client_regions,
                    &mut stuff.mapped_client_id_and_client,
                );
            }
            Desu::Done => Message::Done,
        };
    }
    pub fn clean_up(self: &mut Self) {
        match &mut self.stuff {
            Desu::Running(stuff) => return clean_up_running(&mut stuff.state),
            Desu::Done => {}
        };
        self.stuff = Desu::Done;
    }
    pub fn new(clients: HyClients, monitors: HyMonitors) -> Self {
        let (wl_state, client_regions) = create_state_and_region_bounds(&clients, &monitors);
        return Self {
            stuff: Desu::Running(Running {
                state: wl_state,
                client_regions,
                event_queue: create_wayland_window_select(),
                mapped_client_id_and_client: index_client_id(&clients),
            }),
        };
    }
}

fn index_client_id(clients: &HyClients) -> HashMap<Address, HyClient> {
    return HashMap::<Address, HyClient>::from_iter(
        clients
            .iter()
            .map(|client| (client.address.clone(), client.clone())),
    );
}

fn update_running(
    state: &mut State,
    event_queue: &mut EventQueue<State>,
    client_regions: &mut Vec<ClientRegion>,
    mapped_client_id_and_client: &mut HashMap<Address, HyClient>,
) -> Message {
    if state.running {
        event_queue.blocking_dispatch(state).expect("wayland loop");
        if let (Some(pointer_position), Some((pointer_monitor_id, _))) =
            (state.pointer_position, state.pointer_surface.clone())
        {
            let hovered_client_region = client_regions.iter().find(|client| {
                let pointer_x = pointer_position.0.trunc() as i16;
                let pointer_y = pointer_position.1.trunc() as i16;
                let x = client.at.0 < pointer_x && pointer_x < (client.at.0 + client.size.0);
                let y = client.at.1 < pointer_y && pointer_y < (client.at.1 + client.size.1);
                if let Some(client_monitor) = &client.monitor {
                    return x && y && client_monitor.to_string() == pointer_monitor_id;
                }
                return false;
            });
            let hovered_client = match hovered_client_region {
                Some(hovered_client_region) => {
                    mapped_client_id_and_client.get(&hovered_client_region.client_id)
                }
                None => None,
            };
            return match hovered_client {
                Some(client) => Message::HoveredClient(Some(client.address.clone())),
                None => Message::HoveredClient(None),
            };
        };
        return Message::HoveredClient(None);
    } else {
        return Message::Done;
    }
}

fn clean_up_running(state: &mut State) {
    state
        .buffer_surfaces
        .iter_mut()
        .for_each(|(_, bfs)| match bfs {
            BufferSurface::Pre(pre) => {
                pre.monitor_clients.clear();
            }
            BufferSurface::InProcess(in_process) => {
                in_process.base_surface.destroy();
                in_process.buffer.destroy();
            }
            BufferSurface::HasOutput(has_output) => {
                has_output.wlr_surface.destroy();
                has_output.base_surface.destroy();
                has_output.buffer.destroy();
                has_output.wayland_output.release();
            }
            BufferSurface::ReadyToDraw(ready_to_draw) => {
                ready_to_draw.wlr_surface.destroy();
                ready_to_draw.base_surface.destroy();
                ready_to_draw.buffer.destroy();
                ready_to_draw.wayland_output.release();
            }
        });

    if let Some(shm) = &state.shm {
        shm.release();
    }
    state.shm = None;

    if let Some(cursor_shape_manager) = &state.cursor_shape_manager {
        cursor_shape_manager.destroy();
    }
    state.cursor_shape_manager = None;

    if let Some(layer_shell) = &state.layer_shell {
        layer_shell.destroy();
    }
    state.layer_shell = None;

    state.buffer_surfaces.clear();
    state.compositor = None;

    state.pointer_position = None;

    state.pointer_surface = None;
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    state.compositor = Some(compositor);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    if let Some(compositor) = &state.compositor {
                        state.buffer_surfaces.iter_mut().for_each(|(_, bfs)| {
                            match bfs {
                                BufferSurface::Pre(pre) => {
                                    let update =
                                        InProcess::from((pre.clone(), &shm, qh, compositor));
                                    *bfs = BufferSurface::InProcess(update);
                                }
                                _ => (),
                            };
                        });
                    }

                    state.shm = Some(shm);
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "wl_pointer" => {
                    registry.bind::<wl_pointer::WlPointer, _, _>(name, 1, qh, ());
                }
                "wl_keyboard" => {
                    registry.bind::<wl_keyboard::WlKeyboard, _, _>(name, 1, qh, ());
                }
                "wp_cursor_shape_manager_v1" => {
                    let manager = registry
                        .bind::<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, _, _>(
                            name,
                            1,
                            qh,
                            (),
                        );
                    state.cursor_shape_manager = Some(manager)
                }
                "wp_cursor_shape_device_v1" => {
                    registry.bind::<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    );
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 1, qh, ());
                    if let Some(layer_shell) = state.layer_shell.as_ref() {
                        /* why did they make it so the output object doesn't have an ID on its own!?!?!?!
                            I could get the ID from an event but wl_output events only fire once you've
                            attached a surface to the output so im forced to attach stuff here and guess
                            which display it is.
                            If the is language had WeakMap i could at least us that to match outputs in the event
                            that gets fired after initially attaching but oh well \(;-;)/
                        */
                        if let Some(buffer_surface) =
                            state.buffer_surfaces.get(&state.output_index.to_string())
                        {
                            match buffer_surface {
                                BufferSurface::InProcess(in_process) => {
                                    let in_process = in_process.clone();

                                    let has_output =
                                        HasOutput::from((in_process, layer_shell, &output, qh));
                                    *state
                                        .buffer_surfaces
                                        .get_mut(&state.output_index.to_string())
                                        .unwrap() = BufferSurface::HasOutput(has_output);
                                    state.output_index += 1;
                                }
                                _ => (),
                            }
                        }
                    }
                }
                "zwlr_layer_shell_v1" => {
                    let zwlr_layer = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    );
                    state.layer_shell = Some(zwlr_layer);
                }
                "zwlr_layer_surface_v1" => {
                    registry
                        .bind::<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, _, _>(name, 1, qh, None);
                }
                _ => {}
            }
        }
    }
}

fn create_state_and_region_bounds<'c>(
    clients: &'c HyClients,
    monitors: &HyMonitors,
) -> (State, Vec<ClientRegion>) {
    let active_workspaces_ids: Vec<WorkspaceId> = monitors
        .iter()
        .map(|monitor| monitor.active_workspace.id)
        .collect();

    let clients: Vec<HyClient> = clients
        .iter()
        .filter_map(|client| {
            if client.mapped && active_workspaces_ids.contains(&client.workspace.id) {
                return Some(client.clone());
            }
            return None;
        })
        .collect();

    let client_regions = clients.iter().map(|client| {
        if let Some(client_monitor_id) = client.monitor {
            if let Some(monitor) = monitors
                .iter()
                .find(|monitor| monitor.id == client_monitor_id)
            {
                let relative_x = (client.at.0 as i32) - monitor.x;
                let relative_y = (client.at.1 as i32) - monitor.y;
                return ClientRegion {
                    at: (relative_x as i16, relative_y as i16),
                    size: client.size.clone(),
                    monitor: Some(client_monitor_id.to_string()),
                    client_id: client.address.clone(),
                };
            }
        }
        return ClientRegion {
            at: client.at.clone(),
            size: client.size.clone(),
            monitor: None,
            client_id: client.address.clone(),
        };
    });

    let buffer_surfaces = HashMap::from_iter(monitors.iter().map(|monitor| {
        let monitor_clients: Vec<ClientRegion> = client_regions
            .clone()
            .filter(|client| match client.monitor.clone() {
                Some(client_monitor) => monitor.id.to_string() == client_monitor,
                None => false,
            })
            .collect();

        return (
            monitor.id.to_string(),
            BufferSurface::Pre(BaseSurfaceBuffer {
                monitor_id: monitor.id.to_string(),
                monitor_size: (monitor.width, monitor.height),
                monitor_clients: monitor_clients.clone(),
            }),
        );
    }));

    return (
        State {
            running: true,
            buffer_surfaces,
            cursor_shape_manager: None,
            layer_shell: None,
            shm: None,
            compositor: None,
            // this sucks but so do i, it seems like output ids start at 0 and count up
            output_index: 0,
            pointer_position: None,
            pointer_surface: None,
        },
        client_regions.collect(),
    );
}

fn create_wayland_window_select() -> EventQueue<State> {
    let conn = Connection::connect_to_env().unwrap();

    let event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    return event_queue;
}
