use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use std::path::PathBuf;

use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer, wl_shm_pool, wl_surface};

use crate::config::ThemePreference;

/// Logical size in surface coordinates.
pub const POPUP_W: u32 = 520;
pub const POPUP_H: u32 = 48;
/// Render at 2x for HiDPI; the compositor scales down on 1x outputs.
pub const POPUP_SCALE: u32 = 2;
pub const POPUP_PIXEL_W: u32 = POPUP_W * POPUP_SCALE;
pub const POPUP_PIXEL_H: u32 = POPUP_H * POPUP_SCALE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme {
    Dark,
    Light,
}

fn theme_from(env: Option<&str>, preference: ThemePreference, is_dark: Option<bool>) -> Theme {
    match env {
        Some("light") => return Theme::Light,
        Some("dark") => return Theme::Dark,
        _ => {}
    }
    match preference {
        ThemePreference::Light => Theme::Light,
        ThemePreference::Dark => Theme::Dark,
        ThemePreference::Auto => match is_dark {
            Some(false) => Theme::Light,
            _ => Theme::Dark,
        },
    }
}

fn cosmic_is_dark() -> Option<bool> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    let path = config.join("cosmic/com.system76.CosmicTheme.Mode/v1/is_dark");
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn theme_colors(preference: ThemePreference) -> ([u8; 4], Color) {
    let theme = Theme::detect(preference);
    (theme.background(), theme.text_color())
}

impl Theme {
    fn detect(preference: ThemePreference) -> Self {
        let env = std::env::var("IME_POPUP_THEME").ok();
        theme_from(env.as_deref(), preference, cosmic_is_dark())
    }

    fn background(self) -> [u8; 4] {
        match self {
            Theme::Dark => [0x20, 0x20, 0x20, 0xF2],
            Theme::Light => [0xF4, 0xF4, 0xF4, 0xF2],
        }
    }

    fn text_color(self) -> Color {
        match self {
            Theme::Dark => Color::rgb(0xEF, 0xEF, 0xEF),
            Theme::Light => Color::rgb(0x1E, 0x1E, 0x1E),
        }
    }
}

pub struct PopupRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    surface: wl_surface::WlSurface,
    _buffer: wl_buffer::WlBuffer,
    _pool: wl_shm_pool::WlShmPool,
    mmap: MmapMut,
    theme: Theme,
}

impl PopupRenderer {
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
            Metrics::new(17.0 * POPUP_SCALE as f32, 22.0 * POPUP_SCALE as f32),
        );
        surface.set_buffer_scale(POPUP_SCALE as i32);
        Self {
            font_system,
            swash_cache: SwashCache::new(),
            text_buffer,
            surface,
            _buffer: buffer,
            _pool: pool,
            mmap,
            theme: Theme::detect(preference),
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
        let width = POPUP_PIXEL_W as usize;
        let height = POPUP_PIXEL_H as usize;
        let mut canvas = vec![0u8; width * height * 4];

        if text.is_empty() {
            // Transparent: make the popup invisible.
            Self::save_debug_png(&canvas, width, height);
            self.mmap.copy_from_slice(&canvas);
            self.surface
                .damage_buffer(0, 0, POPUP_PIXEL_W as i32, POPUP_PIXEL_H as i32);
            self.surface.commit();
            return;
        }

        // wl_shm ARGB8888 on little-endian: B, G, R, A
        let background = self.theme.background();
        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&background);
        }

        self.text_buffer
            .set_size(Some(width as f32 - 20.0), Some(height as f32));
        self.text_buffer
            .set_text(text, &Attrs::new(), Shaping::Advanced, None);

        let color = self.theme.text_color();
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
            .damage_buffer(0, 0, POPUP_PIXEL_W as i32, POPUP_PIXEL_H as i32);
        self.surface.commit();
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn env_override() {
        assert_eq!(
            theme_from(Some("light"), ThemePreference::Dark, Some(true)),
            Theme::Light
        );
        assert_eq!(
            theme_from(Some("dark"), ThemePreference::Light, Some(false)),
            Theme::Dark
        );
    }

    #[test]
    fn config_fallback() {
        assert_eq!(
            theme_from(None, ThemePreference::Auto, Some(false)),
            Theme::Light
        );
        assert_eq!(
            theme_from(None, ThemePreference::Light, Some(true)),
            Theme::Light
        );
        assert_eq!(
            theme_from(None, ThemePreference::Dark, Some(false)),
            Theme::Dark
        );
        assert_eq!(theme_from(None, ThemePreference::Auto, None), Theme::Dark);
    }
}
