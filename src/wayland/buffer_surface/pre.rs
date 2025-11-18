use wayland_client::protocol::wl_output;

#[derive(Debug, Clone)]
pub struct Pre {
    pub window_buffer: Vec<bool>,
    pub monitor_id: String,
    pub wayland_output: Option<wl_output::WlOutput>,
    pub monitor_size: (u16, u16),
}
