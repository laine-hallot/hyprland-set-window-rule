use std::{
    fs::File,
    io::Write,
    os::fd::AsFd,
    time::{SystemTime, UNIX_EPOCH},
};

use super::super::protocols::State;

use super::pre::Pre;
use wayland_client::{
    QueueHandle,
    protocol::{wl_buffer, wl_compositor, wl_shm, wl_surface},
};

#[derive(Debug, Clone)]
pub struct InProcess {
    pub window_buffer: Vec<bool>,
    pub monitor_id: String,
    pub size: (u16, u16),
    pub buffer: wl_buffer::WlBuffer,
    pub base_surface: wl_surface::WlSurface,
}

impl
    From<(
        Pre,
        &wl_shm::WlShm,
        &QueueHandle<State>,
        &wl_compositor::WlCompositor,
    )> for InProcess
{
    fn from(
        (pre, shm, qh, compositor): (
            Pre,
            &wl_shm::WlShm,
            &QueueHandle<State>,
            &wl_compositor::WlCompositor,
        ),
    ) -> Self {
        InProcess {
            window_buffer: pre.window_buffer.clone(),
            monitor_id: pre.monitor_id.clone(),
            size: pre.monitor_size,
            buffer: create_surface_buffer(&shm, qh, (1, 1)),
            base_surface: create_base_surface(compositor, qh),
        }
    }
}

fn create_surface_buffer(
    shm: &wl_shm::WlShm,
    qh: &QueueHandle<State>,
    size: (u16, u16),
) -> wl_buffer::WlBuffer {
    let (init_w, init_h) = size;

    let mut file = tempfile::tempfile().unwrap();

    draw_empty(&mut file, (init_w as u32, init_h as u32));
    let pool = shm.create_pool(file.as_fd(), init_w as i32 * init_h as i32 * 4, qh, ());
    let buffer = pool.create_buffer(
        0,
        init_w as i32,
        init_h as i32,
        (init_w * 4) as i32,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    return buffer.clone();
}

fn create_base_surface(
    compositor: &wl_compositor::WlCompositor,
    qh: &QueueHandle<State>,
) -> wl_surface::WlSurface {
    return compositor.create_surface(qh, ());
}

const BG_COLOR: [u8; 4] = [0x00 as u8, 0x00 as u8, 0x00 as u8, 0x00 as u8];

fn draw_empty(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    let mut buf = std::io::BufWriter::new(tmp);
    for _ in 0..(buf_x * buf_y) {
        buf.write_all(&BG_COLOR).unwrap();
    }
    buf.flush().unwrap();

    let end = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    /* println!(
        "Blank - surface buffer: {}ms",
        start.abs_diff(end).as_millis()
    ); */
}
