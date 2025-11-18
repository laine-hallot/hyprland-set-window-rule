use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_pointer};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1::{self, Shape as CursorShape},
    wp_cursor_shape_manager_v1,
};

impl Dispatch<wl_pointer::WlPointer, ()> for super::State {
    fn event(
        state: &mut Self,
        pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                serial, surface, ..
            } => {
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

                state.pointer_surface = Some(surface);
            }
            wl_pointer::Event::Leave { .. } => {
                state.pointer_surface = None;
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_position = Some((surface_x, surface_y));
            }
            wl_pointer::Event::Button { .. } => {
                println!("Clicked - Exiting");
                state.running = false;
            }
            _ => (),
        }
    }
}
