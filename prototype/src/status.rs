use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer, wl_shm_pool, wl_surface};

use crate::config::ThemePreference;
use crate::popup::theme_colors;

/// Logical size of the status chip.
pub const STATUS_W: u32 = 140;
pub const STATUS_H: u32 = 28;
pub const STATUS_SCALE: u32 = 2;
pub const STATUS_PIXEL_W: u32 = STATUS_W * STATUS_SCALE;
pub const STATUS_PIXEL_H: u32 = STATUS_H * STATUS_SCALE;

pub struct StatusRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    surface: wl_surface::WlSurface,
    buffer: wl_buffer::WlBuffer,
    _pool: wl_shm_pool::WlShmPool,
    mmap: MmapMut,
    text: String,
    configured: bool,
    preference: ThemePreference,
}

impl StatusRenderer {
    pub fn new(
        surface: wl_surface::WlSurface,
        buffer: wl_buffer::WlBuffer,
        pool: wl_shm_pool::WlShmPool,
        mmap: MmapMut,
        preference: ThemePreference,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let text_buffer = Buffer::new(
            &mut font_system,
            Metrics::new(16.0 * STATUS_SCALE as f32, 22.0 * STATUS_SCALE as f32),
        );
        surface.set_buffer_scale(STATUS_SCALE as i32);
        Self {
            font_system,
            swash_cache: SwashCache::new(),
            text_buffer,
            surface,
            buffer,
            _pool: pool,
            mmap,
            text: String::new(),
            configured: false,
            preference,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        if self.configured {
            self.draw();
        }
    }

    pub fn configure(&mut self) {
        self.configured = true;
        self.draw();
    }

    fn draw(&mut self) {
        if !self.configured {
            return;
        }

        let width = STATUS_PIXEL_W as usize;
        let height = STATUS_PIXEL_H as usize;
        let mut canvas = vec![0u8; width * height * 4];
        let (background, color) = theme_colors(self.preference);

        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&background);
        }

        self.text_buffer
            .set_size(Some(width as f32 - 12.0), Some(height as f32));
        self.text_buffer.set_text(
            &self.text,
            &Attrs::new(),
            Shaping::Advanced,
            Some(cosmic_text::Align::Center),
        );

        {
            let StatusRenderer {
                font_system,
                swash_cache,
                text_buffer,
                ..
            } = self;
            let mut borrowed = text_buffer.borrow_with(font_system);
            borrowed.draw(swash_cache, color, |x, y, gw, gh, c| {
                let a = c.a() as u32;
                if a == 0 {
                    return;
                }
                let inv = 255 - a;
                for yy in y..y + gh as i32 {
                    for xx in x..x + gw as i32 {
                        if xx < 0 || yy < 0 || xx >= width as i32 || yy >= height as i32 {
                            continue;
                        }
                        let idx = ((yy as usize) * width + xx as usize) * 4;
                        let b = (c.b() as u32 * a + canvas[idx] as u32 * inv) / 255;
                        let g = (c.g() as u32 * a + canvas[idx + 1] as u32 * inv) / 255;
                        let r = (c.r() as u32 * a + canvas[idx + 2] as u32 * inv) / 255;
                        canvas[idx] = b as u8;
                        canvas[idx + 1] = g as u8;
                        canvas[idx + 2] = r as u8;
                        canvas[idx + 3] = 0xFF;
                    }
                }
            });
        }

        save_debug_png(&canvas, width, height);
        self.mmap.copy_from_slice(&canvas);
        self.surface.attach(Some(&self.buffer), 0, 0);
        self.surface
            .damage_buffer(0, 0, STATUS_PIXEL_W as i32, STATUS_PIXEL_H as i32);
        self.surface.commit();
    }
}

fn save_debug_png(canvas: &[u8], width: usize, height: usize) {
    let mut rgba = Vec::with_capacity(width * height * 4);
    for px in canvas.chunks_exact(4) {
        rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
    if let Ok(file) = std::fs::File::create("/tmp/ime-status.png") {
        let mut encoder =
            png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(&rgba);
        }
    }
}
