use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
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
/// COSMIC HIG-style corner radius in logical pixels.
const POPUP_RADIUS: f32 = 10.0 * POPUP_SCALE as f32;

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

impl Theme {
    fn detect(preference: ThemePreference) -> Self {
        let env = std::env::var("IME_POPUP_THEME").ok();
        theme_from(env.as_deref(), preference, cosmic_is_dark())
    }

    fn background(self) -> [u8; 4] {
        match self {
            Theme::Dark => [0x1E, 0x1E, 0x1E, 0xF2],
            Theme::Light => [0xFA, 0xFA, 0xFA, 0xF2],
        }
    }

    fn text_color(self) -> Color {
        match self {
            Theme::Dark => Color::rgb(0xF2, 0xF2, 0xF2),
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

    /// Re-detect the theme after the user changes it in settings.
    pub fn set_preference(&mut self, preference: ThemePreference) {
        self.theme = Theme::detect(preference);
    }

    pub fn draw(&mut self, text: &str) {
        let width = POPUP_PIXEL_W as usize;
        let height = POPUP_PIXEL_H as usize;
        let mut canvas = vec![0u8; width * height * 4];

        if text.is_empty() {
            // Transparent: make the popup invisible.
            Self::save_debug_png(&canvas, width, height);
            self.mmap.copy_from_slice(&canvas);
            self.surface.attach(Some(&self._buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, POPUP_PIXEL_W as i32, POPUP_PIXEL_H as i32);
            self.surface.commit();
            return;
        }

        // wl_shm ARGB8888 on little-endian: B, G, R, A.
        // Draw a rounded COSMIC-HIG-like surface; outside the radius stays transparent.
        let background = self.theme.background();
        for y in 0..height {
            for x in 0..width {
                let coverage = rounded_coverage(
                    x as f32 + 0.5,
                    y as f32 + 0.5,
                    width as f32,
                    height as f32,
                    POPUP_RADIUS,
                );
                if coverage == 0 {
                    continue;
                }
                let alpha = (background[3] as u16 * coverage as u16 / 255) as u8;
                let idx = (y * width + x) * 4;
                canvas[idx] = background[0];
                canvas[idx + 1] = background[1];
                canvas[idx + 2] = background[2];
                canvas[idx + 3] = alpha;
            }
        }

        self.text_buffer
            .set_size(Some(width as f32 - 24.0), Some(height as f32));
        self.text_buffer.set_text(
            text,
            &Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
            None,
        );

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
        self.surface.attach(Some(&self._buffer), 0, 0);
        self.surface
            .damage_buffer(0, 0, POPUP_PIXEL_W as i32, POPUP_PIXEL_H as i32);
        self.surface.commit();
    }
}

fn rounded_coverage(px: f32, py: f32, width: f32, height: f32, radius: f32) -> u8 {
    const SAMPLES: [(f32, f32); 4] = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)];
    let mut inside = 0;
    for (sx, sy) in SAMPLES {
        let x = px - 0.5 + sx;
        let y = py - 0.5 + sy;
        if inside_rounded_rect(x, y, width, height, radius) {
            inside += 1;
        }
    }
    (inside * 255 / 4) as u8
}

fn inside_rounded_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> bool {
    if x < 0.0 || y < 0.0 || x >= width || y >= height {
        return false;
    }
    let nearest_x = x.min(width - x);
    let nearest_y = y.min(height - y);
    if nearest_x < radius && nearest_y < radius {
        let dx = radius - nearest_x;
        let dy = radius - nearest_y;
        dx * dx + dy * dy <= radius * radius
    } else {
        true
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
    fn rounded_corners_are_transparent() {
        assert_eq!(rounded_coverage(0.5, 0.5, 100.0, 48.0, 10.0), 0);
        assert_eq!(rounded_coverage(50.0, 24.0, 100.0, 48.0, 10.0), 255);
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
