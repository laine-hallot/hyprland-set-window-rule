use hyprland::{
    ctl::output,
    data::{LayerClient, LayerDisplay, Layers},
};
use std::{fs::File, os::unix::io::AsFd};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_display, wl_keyboard, wl_output, wl_pointer, wl_registry,
        wl_seat, wl_shm, wl_shm_pool, wl_surface,
    },
};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Shape as CursorShape;
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1, wp_cursor_shape_manager_v1,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols_wlr::layer_shell::{
    v1::client::zwlr_layer_shell_v1::{self, Layer},
    v1::client::zwlr_layer_surface_v1,
};

struct DisplayMode {
    width: i32,
    height: i32,
}

struct State {
    running: bool,
    base_surface: Option<wl_surface::WlSurface>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    cursor_shape_manager: Option<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    wlr_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    output: Option<wl_output::WlOutput>,
    has_init_wlr: bool,
    display_mode: Option<DisplayMode>,
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
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());

                    let (init_w, init_h) = (320, 240);

                    let mut file = tempfile::tempfile().unwrap();
                    draw(&mut file, (init_w, init_h));
                    let pool = shm.create_pool(file.as_fd(), (init_w * init_h * 4) as i32, qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        init_w as i32,
                        init_h as i32,
                        (init_w * 4) as i32,
                        wl_shm::Format::Argb8888,
                        qh,
                        (),
                    );
                    state.buffer = Some(buffer.clone());

                    /* if state.configured {
                        let surface = state.base_surface.as_ref().unwrap();
                        surface.attach(Some(&buffer), 0, 0);
                        surface.commit();
                    } */
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "wl_pointer" => {
                    registry.bind::<wl_pointer::WlPointer, _, _>(name, 1, qh, ());
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
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    state.wm_base = Some(wm_base);
                }
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, _, _>(name, 1, qh, ());
                    state.output = Some(output);
                }
                "zwlr_layer_shell_v1" => {
                    let zwlr_layer = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    );
                    println!("created layer_shell");
                    state.layer_shell = Some(zwlr_layer);

                    let zwlr_layer = state.layer_shell.as_ref().unwrap();
                    let base_surface = state.base_surface.as_ref().unwrap();

                    state.wlr_surface = Some(zwlr_layer.get_layer_surface(
                        base_surface,
                        state.output.as_ref(),
                        Layer::Overlay,
                        "selection".to_string(),
                        &qh,
                        (),
                    ));
                    state.init_wlr_surface();
                    //base_surface.commit();
                }
                "zwlr_layer_surface_v1" => {
                    println!("zwlr_layer_surface_v1");
                    registry.bind::<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    );
                }
                _ => {}
            }
        }
    }
}

// Ignore events from these object types in this example.
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);
delegate_noop!(State: ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);

fn draw(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    use std::{cmp::min, io::Write};
    let mut buf = std::io::BufWriter::new(tmp);
    for y in 0..buf_y {
        for x in 0..buf_x {
            let a = 0xEE;
            let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
            buf.write_all(&[b as u8, g as u8, r as u8, a as u8])
                .unwrap();
        }
    }
    buf.flush().unwrap();
}

impl State {
    fn init_wlr_surface(&mut self) {
        if !self.has_init_wlr {
            println!("init_wlr_surface");
            let wlr_surface = self.wlr_surface.as_ref().unwrap();
            let base_surface = self.base_surface.as_ref().unwrap();

            wlr_surface.set_size(300, 200);
            wlr_surface.set_anchor(zwlr_layer_surface_v1::Anchor::Top);
            wlr_surface.set_anchor(zwlr_layer_surface_v1::Anchor::Left);
            wlr_surface.set_exclusive_zone(0);

            base_surface.commit();
            self.has_init_wlr = true;
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Description { description } => {
                println!("Description: {}", description)
            }
            wl_output::Event::Done => println!("Done"),
            geometry @ wl_output::Event::Geometry { .. } => println!("Geometry, {:#?}", geometry),
            mode @ wl_output::Event::Mode { width, height, .. } => {
                println!("Mode, {:#?}", mode);

                if let Some(ref buffer) = state.buffer {
                    let base_surface = state.base_surface.as_ref().unwrap();

                    println!("committing first actual draw");
                    base_surface.attach(Some(buffer), width, height);
                    base_surface.commit();
                }
                state.display_mode = Some(DisplayMode { width, height });
            }
            wl_output::Event::Name { name } => println!("Name, {name}"),
            wl_output::Event::Scale { factor } => println!("Scale, {factor}"),
            _ => (),
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("XdgWmBase");
        match event {
            xdg_wm_base::Event::Ping { serial } => {
                wm_base.pong(serial);
            }
            _ => (),
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close = event {
            state.running = false;
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_seat::Event::Capabilities {
                capabilities: WEnum::Value(capabilities),
            } => {
                if capabilities.contains(wl_seat::Capability::Keyboard) {
                    seat.get_keyboard(qh, ());
                }
                if capabilities.contains(wl_seat::Capability::Pointer) {
                    seat.get_pointer(qh, ());
                }
            }
            _ => (),
        };
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn event(
        state: &mut Self,
        pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_pointer::Event::Enter {
            serial,
            surface: _surface,
            surface_x: _surface_x,
            surface_y: _surface_y,
        } = event
        {
            println!("Pointer entered surface");
            let device = wp_cursor_shape_manager_v1::WpCursorShapeManagerV1::get_pointer(
                &state
                    .cursor_shape_manager
                    .clone()
                    .expect("failed to clone state.cursor_shape_manager"),
                pointer,
                qh,
                (),
            );
            wp_cursor_shape_device_v1::WpCursorShapeDeviceV1::set_shape(
                &device,
                serial,
                CursorShape::Crosshair,
            );
        }
    }
}

impl Dispatch<wp_cursor_shape_manager_v1::WpCursorShapeManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        _: wp_cursor_shape_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("WpCursorShapeManagerV1");
    }
}

impl Dispatch<wp_cursor_shape_device_v1::WpCursorShapeDeviceV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &wp_cursor_shape_device_v1::WpCursorShapeDeviceV1,
        _: wp_cursor_shape_device_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("WpCursorShapeDeviceV1");
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                state.running = false;
            }
        }
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            conf @ zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                dbg!(&conf);
                let base_surface = state.base_surface.as_ref().unwrap();
                if state.display_mode.is_some() {
                    let display_mode = state.display_mode.as_ref().unwrap();

                    layer_surface.set_size(display_mode.width as u32, display_mode.height as u32);
                } else {
                    layer_surface.set_size(width, height);
                }

                base_surface.commit();
                layer_surface.ack_configure(serial);
            }
            zwlr_layer_surface_v1::Event::Closed => todo!(),
            _ => todo!(),
        };
    }
}

pub fn create_window() -> () {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State {
        running: true,
        base_surface: None,
        buffer: None,
        wm_base: None,
        cursor_shape_manager: None,
        layer_shell: None,
        wlr_surface: None,
        output: None,
        has_init_wlr: false,
        display_mode: None,
    };

    println!("Starting the example window app, press <ESC> to quit.");

    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .expect("window loop");
    }
}
