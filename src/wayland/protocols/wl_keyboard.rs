use wayland_client::{
    protocol::wl_keyboard,
    {Connection, Dispatch, QueueHandle},
};

impl Dispatch<wl_keyboard::WlKeyboard, ()> for super::State {
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
