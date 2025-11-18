use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_output};

impl Dispatch<wl_output::WlOutput, ()> for super::State {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            /*
                this is where my output matching code would go if this world were nicer
            */
            _ => (),
        }
    }
}
