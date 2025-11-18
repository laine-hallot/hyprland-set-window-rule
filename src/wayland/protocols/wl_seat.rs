use wayland_client::{Connection, Dispatch, QueueHandle, WEnum, protocol::wl_seat};

impl Dispatch<wl_seat::WlSeat, ()> for super::State {
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
