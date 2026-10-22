use std::io::Write;
use std::os::fd::{AsFd, FromRawFd};

use ime_probe::config::ThemePreference;
use ime_probe::popup::{
    CandidateView, POPUP_SCALE, PopupRenderer, popup_pixel_height, required_width_with,
};
use memmap2::MmapMut;
use wayland_client::{
    QueueHandle,
    protocol::{wl_buffer, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_popup_surface_v2::ZwpInputPopupSurfaceV2;

use crate::State;

pub struct CandidateWindow {
    _popup: ZwpInputPopupSurfaceV2,
    shm: wl_shm::WlShm,
    renderer: PopupRenderer,
    has_buffer: bool,
    logical_width: u32,
    logical_height: u32,
}

impl CandidateWindow {
    pub fn new(
        popup: ZwpInputPopupSurfaceV2,
        surface: wl_surface::WlSurface,
        shm: wl_shm::WlShm,
        preference: ThemePreference,
    ) -> Self {
        let renderer = PopupRenderer::new(surface.clone(), preference);
        Self {
            _popup: popup,
            shm,
            renderer,
            has_buffer: false,
            logical_width: 0,
            logical_height: 0,
        }
    }

    pub fn set_preference(&mut self, preference: ThemePreference) {
        self.renderer.set_preference(preference);
    }

    pub fn show(&mut self, qh: &QueueHandle<State>, view: CandidateView) {
        if view.is_empty() {
            self.hide();
            return;
        }

        let desired_width = {
            let renderer = &mut self.renderer;
            required_width_with(
                &view.preedit,
                &view.candidates,
                view.page,
                view.layout,
                view.candidate_count,
                |text| renderer.measure_text_width(text),
            )
        };
        let desired_height = view.desired_height();

        if !self.has_buffer
            || desired_width != self.logical_width
            || desired_height != self.logical_height
        {
            let Some((pool, buffer, mmap)) = create_buffer(
                &self.shm,
                qh,
                desired_width * POPUP_SCALE,
                popup_pixel_height(desired_height),
            ) else {
                eprintln!("candidate window: failed to allocate {desired_width}x{desired_height}");
                self.hide();
                return;
            };
            self.renderer
                .resize(buffer, pool, mmap, desired_width, desired_height);
            self.logical_width = desired_width;
            self.logical_height = desired_height;
            self.has_buffer = true;
        }

        let lines = view.lines(self.logical_width);
        let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
        if refs.is_empty() {
            self.hide();
        } else {
            self.renderer.draw_lines(&refs);
        }
    }

    pub fn hide(&mut self) {
        if self.has_buffer {
            self.renderer.draw_lines(&[]);
        }
    }
}

fn create_buffer(
    shm: &wl_shm::WlShm,
    qh: &QueueHandle<State>,
    pixel_width: u32,
    pixel_height: u32,
) -> Option<(wl_shm_pool::WlShmPool, wl_buffer::WlBuffer, MmapMut)> {
    let stride = pixel_width as i32 * 4;
    let size = stride * pixel_height as i32;
    let cname = std::ffi::CString::new("ime-popup").ok()?;
    let fd = unsafe { libc::memfd_create(cname.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let data = vec![0u8; size as usize];
    file.write_all(&data).ok()?;
    let mmap = unsafe { MmapMut::map_mut(&file).ok()? };
    let pool = shm.create_pool(file.as_fd(), size, qh, ());
    let buffer = pool.create_buffer(
        0,
        pixel_width as i32,
        pixel_height as i32,
        stride,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    Some((pool, buffer, mmap))
}
