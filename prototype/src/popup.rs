use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use std::path::PathBuf;

use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer, wl_shm_pool, wl_surface};

use crate::config::{CandidateLayout, ThemePreference};
use crate::engine::PageInfo;

/// Logical size in surface coordinates.
pub const POPUP_MIN_W: u32 = 260;
pub const POPUP_MAX_W: u32 = 760;
pub const POPUP_DEFAULT_W: u32 = 520;
pub const POPUP_SINGLE_H: u32 = 48;
pub const POPUP_TWO_ROW_H: u32 = 96;
/// Render at 2x for HiDPI; the compositor scales down on 1x outputs.
pub const POPUP_SCALE: u32 = 2;
/// COSMIC HIG-style corner radius in logical pixels.
const POPUP_RADIUS: f32 = 10.0 * POPUP_SCALE as f32;

pub fn effective_layout(layout: CandidateLayout, candidate_count: usize) -> CandidateLayout {
    match layout {
        CandidateLayout::Auto if candidate_count > 5 => CandidateLayout::TwoRow,
        CandidateLayout::Auto => CandidateLayout::Single,
        other => other,
    }
}

pub fn popup_height(layout: CandidateLayout, candidate_count: usize) -> u32 {
    match effective_layout(layout, candidate_count) {
        CandidateLayout::Single | CandidateLayout::Auto => POPUP_SINGLE_H,
        CandidateLayout::TwoRow => POPUP_TWO_ROW_H,
    }
}

pub fn popup_pixel_height(logical_height: u32) -> u32 {
    logical_height * POPUP_SCALE
}

fn approx_text_width(text: &str) -> f32 {
    text.chars()
        .map(|c| if c.is_ascii() { 9.0 } else { 17.0 })
        .sum()
}

pub fn required_width(
    preedit: &str,
    candidates: &[String],
    page: PageInfo,
    layout: CandidateLayout,
    candidate_count: usize,
) -> u32 {
    let candidate_text = candidates
        .iter()
        .take(candidate_count)
        .enumerate()
        .map(|(i, candidate)| format!("{}.{}", i + 1, candidate))
        .collect::<Vec<_>>()
        .join("  ");
    let page_text = page.label().unwrap_or_default();
    let first_line = if preedit.is_empty() {
        page_text.clone()
    } else if page_text.is_empty() {
        preedit.to_string()
    } else {
        format!("{preedit}   {page_text}")
    };
    let content = match effective_layout(layout, candidate_count) {
        CandidateLayout::TwoRow => {
            approx_text_width(&candidate_text).max(approx_text_width(&first_line))
        }
        _ => approx_text_width(&format!("{preedit}   {candidate_text}   {page_text}")),
    };
    ((content + 32.0).ceil() as u32).clamp(POPUP_MIN_W, POPUP_MAX_W)
}

fn pad_page_right(mut line: String, page: &str, width: u32) -> String {
    if page.is_empty() {
        return line;
    }
    let space = approx_text_width(" ");
    let target = width as f32 - 16.0 - approx_text_width(page);
    let current = approx_text_width(&line);
    if current + space < target {
        let spaces = (((target - current) / space).floor() as usize).max(1);
        line.push_str(&" ".repeat(spaces));
    } else if !line.is_empty() {
        line.push_str("  ");
    }
    line.push_str(page);
    line
}

pub fn compose_lines(
    preedit: &str,
    candidates: &[String],
    page: PageInfo,
    layout: CandidateLayout,
    candidate_count: usize,
    width: u32,
) -> Vec<String> {
    let candidate_text = candidates
        .iter()
        .take(candidate_count)
        .enumerate()
        .map(|(i, candidate)| format!("{}.{}", i + 1, candidate))
        .collect::<Vec<_>>()
        .join("  ");
    let page_text = page.label().unwrap_or_default();

    match effective_layout(layout, candidate_count) {
        CandidateLayout::Single | CandidateLayout::Auto => {
            let mut line = String::new();
            if !preedit.is_empty() {
                line.push_str(preedit);
                line.push_str("   ");
            }
            if !page_text.is_empty() {
                line.push_str(&page_text);
                line.push_str("   ");
            }
            line.push_str(&candidate_text);
            if line.is_empty() {
                Vec::new()
            } else {
                vec![line]
            }
        }
        CandidateLayout::TwoRow => {
            let first = pad_page_right(preedit.to_string(), &page_text, width);
            if first.is_empty() && candidate_text.is_empty() {
                Vec::new()
            } else {
                vec![first, candidate_text]
            }
        }
    }
}

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

pub fn cosmic_is_dark() -> Option<bool> {
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
    pixel_width: u32,
    pixel_height: u32,
    logical_width: u32,
    logical_height: u32,
}

impl PopupRenderer {
    pub fn new(
        surface: wl_surface::WlSurface,
        buffer: wl_buffer::WlBuffer,
        pool: wl_shm_pool::WlShmPool,
        mmap: MmapMut,
        preference: ThemePreference,
        logical_width: u32,
        logical_height: u32,
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
            pixel_width: logical_width * POPUP_SCALE,
            pixel_height: popup_pixel_height(logical_height),
            logical_width,
            logical_height,
        }
    }

    pub fn resize(
        &mut self,
        buffer: wl_buffer::WlBuffer,
        pool: wl_shm_pool::WlShmPool,
        mmap: MmapMut,
        logical_width: u32,
        logical_height: u32,
    ) {
        self._buffer = buffer;
        self._pool = pool;
        self.mmap = mmap;
        self.pixel_width = logical_width * POPUP_SCALE;
        self.pixel_height = popup_pixel_height(logical_height);
        self.logical_width = logical_width;
        self.logical_height = logical_height;
    }

    pub fn pixel_width(&self) -> u32 {
        self.pixel_width
    }

    pub fn pixel_height(&self) -> u32 {
        self.pixel_height
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
        self.draw_lines(&[text]);
    }

    pub fn draw_lines(&mut self, lines: &[&str]) {
        let width = self.pixel_width as usize;
        let height = self.pixel_height as usize;
        let text = lines
            .iter()
            .filter(|line| !line.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        let mut canvas = vec![0u8; width * height * 4];

        if text.is_empty() {
            // Transparent: make the popup invisible.
            Self::save_debug_png(&canvas, width, height);
            self.mmap.copy_from_slice(&canvas);
            self.surface.attach(Some(&self._buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, self.pixel_width as i32, self.pixel_height as i32);
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
            &text,
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
            .damage_buffer(0, 0, self.pixel_width as i32, self.pixel_height as i32);
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
mod layout_tests {
    use super::*;

    #[test]
    fn single_is_default_height() {
        assert_eq!(popup_height(CandidateLayout::Single, 9), POPUP_SINGLE_H);
        assert_eq!(popup_height(CandidateLayout::TwoRow, 1), POPUP_TWO_ROW_H);
    }

    #[test]
    fn auto_uses_two_rows_for_large_pages() {
        assert_eq!(
            effective_layout(CandidateLayout::Auto, 5),
            CandidateLayout::Single
        );
        assert_eq!(
            effective_layout(CandidateLayout::Auto, 6),
            CandidateLayout::TwoRow
        );
        assert_eq!(popup_height(CandidateLayout::Auto, 6), POPUP_TWO_ROW_H);
    }

    #[test]
    fn two_row_separates_preedit_from_candidates() {
        let candidates = vec!["你".to_string(), "您".to_string()];
        let page = PageInfo {
            page_no: 1,
            page_size: 5,
            is_last_page: true,
        };
        let lines = compose_lines("wqiy", &candidates, page, CandidateLayout::TwoRow, 6, 520);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("wqiy"));
        assert!(lines[0].ends_with("第 2 页"));
        assert_eq!(lines[1], "1.你  2.您");
    }

    #[test]
    fn single_row_keeps_one_line() {
        let candidates = vec!["你".to_string(), "您".to_string()];
        let lines = compose_lines(
            "wqiy",
            &candidates,
            PageInfo::default(),
            CandidateLayout::Single,
            6,
            520,
        );
        assert_eq!(lines, vec!["wqiy   1.你  2.您"]);
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
