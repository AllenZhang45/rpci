use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use std::path::PathBuf;

use memmap2::MmapMut;
use wayland_client::protocol::{wl_buffer, wl_shm_pool, wl_surface};

use crate::config::{CandidateLayout, ThemePreference};
use crate::engine::PageInfo;

/// Logical size in surface coordinates.
pub const POPUP_MIN_W: u32 = 200;
pub const POPUP_MAX_W: u32 = 760;
pub const POPUP_DEFAULT_W: u32 = 520;
pub const POPUP_SCALE: u32 = 2;
/// Actual text metrics used by cosmic-text, in logical pixels.
pub const LINE_HEIGHT: u32 = 22;
pub const PADDING_X: u32 = 12;
pub const PADDING_Y: u32 = 8;
pub const LINE_GAP: u32 = 6;
/// COSMIC HIG-style corner radius in logical pixels.
const POPUP_RADIUS: f32 = 10.0 * POPUP_SCALE as f32;

pub fn effective_layout(layout: CandidateLayout, candidate_count: usize) -> CandidateLayout {
    match layout {
        CandidateLayout::Auto if candidate_count > 5 => CandidateLayout::TwoRow,
        CandidateLayout::Auto => CandidateLayout::Single,
        other => other,
    }
}

pub fn height_for_lines(lines: usize) -> u32 {
    let lines = lines.max(1) as u32;
    PADDING_Y * 2 + lines * LINE_HEIGHT + (lines.saturating_sub(1) as u32) * LINE_GAP
}

pub fn popup_height(layout: CandidateLayout, candidate_count: usize) -> u32 {
    match effective_layout(layout, candidate_count) {
        CandidateLayout::Single | CandidateLayout::Auto => height_for_lines(1),
        CandidateLayout::TwoRow => height_for_lines(2),
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

fn candidate_text(candidates: &[String], candidate_count: usize) -> String {
    candidates
        .iter()
        .take(candidate_count)
        .enumerate()
        .map(|(i, candidate)| format!("{}.{}", i + 1, candidate))
        .collect::<Vec<_>>()
        .join("  ")
}

pub fn required_width(
    preedit: &str,
    candidates: &[String],
    page: PageInfo,
    layout: CandidateLayout,
    candidate_count: usize,
) -> u32 {
    let candidates = candidate_text(candidates, candidate_count);
    let page_text = page.label().unwrap_or_default();
    let nav = page.nav();
    let first_line = match (preedit.is_empty(), page_text.is_empty()) {
        (true, true) => String::new(),
        (true, false) => page_text.clone(),
        (false, true) => preedit.to_string(),
        (false, false) => format!("{preedit}   {page_text}"),
    };
    let content = match effective_layout(layout, candidate_count) {
        CandidateLayout::TwoRow => {
            let second = format!("{candidates}   {nav}");
            approx_text_width(&second).max(approx_text_width(&first_line))
        }
        _ => approx_text_width(&format!("{preedit}   {candidates}   {page_text}   {nav}")),
    };
    ((content + 32.0).ceil() as u32).clamp(POPUP_MIN_W, POPUP_MAX_W)
}

fn pad_page_right(mut line: String, right: &str, width: u32) -> String {
    if right.is_empty() {
        return line;
    }
    let space = approx_text_width(" ");
    let target = width as f32 - 16.0 - approx_text_width(right);
    let current = approx_text_width(&line);
    if current + space < target {
        let spaces = (((target - current) / space).floor() as usize).max(1);
        line.push_str(&" ".repeat(spaces));
    } else if !line.is_empty() {
        line.push_str("  ");
    }
    line.push_str(right);
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
    let candidates = candidate_text(candidates, candidate_count);
    let page_text = page.label().unwrap_or_default();
    let nav = page.nav();

    if preedit.is_empty() && candidates.is_empty() {
        return Vec::new();
    }

    match effective_layout(layout, candidate_count) {
        CandidateLayout::Single | CandidateLayout::Auto => {
            let mut line = String::new();
            if !preedit.is_empty() {
                line.push_str(preedit);
                line.push_str("   ");
            }
            line.push_str(&candidates);
            let right = format!("{page_text}{nav}");
            let line = pad_page_right(line, &right, width);
            if line.is_empty() {
                Vec::new()
            } else {
                vec![line]
            }
        }
        CandidateLayout::TwoRow => {
            let first = pad_page_right(preedit.to_string(), &page_text, width);
            let second = pad_page_right(candidates, nav, width);
            if first.is_empty() && second.is_empty() {
                Vec::new()
            } else {
                vec![first, second]
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

    fn separator(self) -> [u8; 4] {
        match self {
            Theme::Dark => [0x80, 0x80, 0x80, 0xB0],
            Theme::Light => [0x60, 0x60, 0x60, 0xB0],
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
        let mut canvas = vec![0u8; width * height * 4];

        if lines.iter().all(|line| line.is_empty()) {
            Self::save_debug_png(&canvas, width, height);
            self.mmap.copy_from_slice(&canvas);
            self.surface.attach(Some(&self._buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, self.pixel_width as i32, self.pixel_height as i32);
            self.surface.commit();
            return;
        }

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

        let scale = POPUP_SCALE as usize;
        let pad_x = PADDING_X as usize * scale;
        let pad_y = PADDING_Y as usize * scale;
        let line_h = LINE_HEIGHT as usize * scale;
        let gap = LINE_GAP as usize * scale;

        // A contrasting separator between the two rows, with extra breathing room.
        if lines.len() > 1 {
            let separator_y = pad_y + line_h + gap / 2;
            let color = self.theme.separator();
            for x in pad_x..width.saturating_sub(pad_x) {
                let idx = (separator_y * width + x) * 4;
                canvas[idx] = color[0];
                canvas[idx + 1] = color[1];
                canvas[idx + 2] = color[2];
                canvas[idx + 3] = color[3];
            }
        }

        let color = self.theme.text_color();
        for (line_index, line) in lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            let offset_y = pad_y + line_index * (line_h + gap);
            self.text_buffer.set_size(
                Some((width.saturating_sub(pad_x * 2)) as f32),
                Some(line_h as f32),
            );
            self.text_buffer.set_text(
                line,
                &Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
                None,
            );
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
                            let px = xx + pad_x as i32;
                            let py = yy + offset_y as i32;
                            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                                continue;
                            }
                            let idx = ((py as usize) * width + px as usize) * 4;
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
        assert_eq!(
            popup_height(CandidateLayout::Single, 9),
            height_for_lines(1)
        );
        assert_eq!(
            popup_height(CandidateLayout::TwoRow, 1),
            height_for_lines(2)
        );
        assert_eq!(
            height_for_lines(2),
            PADDING_Y * 2 + LINE_HEIGHT * 2 + LINE_GAP
        );
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
        assert_eq!(popup_height(CandidateLayout::Auto, 6), height_for_lines(2));
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
        assert!(lines[1].starts_with("1.你  2.您"));
        assert!(lines[1].ends_with("‹ ›"));
    }

    #[test]
    fn empty_composition_hides_page_indicator() {
        let page = PageInfo {
            page_no: 0,
            page_size: 5,
            is_last_page: false,
        };
        let lines = compose_lines("", &[], page, CandidateLayout::TwoRow, 6, 200);
        assert!(lines.is_empty());
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
