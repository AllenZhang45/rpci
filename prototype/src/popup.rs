use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer, wl_shm_pool, wl_surface};

pub const POPUP_W: u32 = 520;
pub const POPUP_H: u32 = 48;

pub struct PopupRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    surface: wl_surface::WlSurface,
    _buffer: wl_buffer::WlBuffer,
    _pool: wl_shm_pool::WlShmPool,
    mmap: MmapMut,
}

impl PopupRenderer {
    pub fn new(
        surface: wl_surface::WlSurface,
        buffer: wl_buffer::WlBuffer,
        pool: wl_shm_pool::WlShmPool,
        mmap: MmapMut,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let text_buffer = Buffer::new(&mut font_system, Metrics::new(17.0, 22.0));
        Self {
            font_system,
            swash_cache: SwashCache::new(),
            text_buffer,
            surface,
            _buffer: buffer,
            _pool: pool,
            mmap,
        }
    }

    fn save_debug_png(canvas: &[u8], width: usize, height: usize) {
        let mut rgba = Vec::with_capacity(width * height * 4);
        for px in canvas.chunks_exact(4) {
            rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
        if let Ok(file) = std::fs::File::create("/tmp/ime-popup.png") {
            let mut encoder =
                png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            if let Ok(mut writer) = encoder.write_header() {
                let _ = writer.write_image_data(&rgba);
            }
        }
    }

    pub fn draw(&mut self, text: &str) {
        let width = POPUP_W as usize;
        let height = POPUP_H as usize;
        let mut canvas = vec![0u8; width * height * 4];

        if text.is_empty() {
            // Transparent: make the popup invisible.
            Self::save_debug_png(&canvas, width, height);
            self.mmap.copy_from_slice(&canvas);
            self.surface
                .damage_buffer(0, 0, POPUP_W as i32, POPUP_H as i32);
            self.surface.commit();
            return;
        }

        // wl_shm ARGB8888 on little-endian: B, G, R, A
        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&[0x20, 0x20, 0x20, 0xF2]);
        }

        self.text_buffer
            .set_size(Some(width as f32 - 20.0), Some(height as f32));
        self.text_buffer
            .set_text(text, &Attrs::new(), Shaping::Advanced, None);

        let color = Color::rgb(0xEF, 0xEF, 0xEF);
        {
            let PopupRenderer {
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

        Self::save_debug_png(&canvas, width, height);
        self.mmap.copy_from_slice(&canvas);
        self.surface
            .damage_buffer(0, 0, POPUP_W as i32, POPUP_H as i32);
        self.surface.commit();
    }
}
