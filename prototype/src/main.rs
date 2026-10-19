use std::collections::HashSet;
use std::io::Write;
use std::os::fd::{AsFd, FromRawFd};

use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use memmap2::MmapMut;

use rime_api::{Session, Traits};
use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2::{self, ZwpInputMethodKeyboardGrabV2},
    zwp_input_method_manager_v2::{self, ZwpInputMethodManagerV2},
    zwp_input_method_v2::{self, ZwpInputMethodV2},
    zwp_input_popup_surface_v2::{self, ZwpInputPopupSurfaceV2},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::{self, ZwpVirtualKeyboardManagerV1},
    zwp_virtual_keyboard_v1::{self, ZwpVirtualKeyboardV1},
};

const POPUP_W: u32 = 520;
const POPUP_H: u32 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Wubi,
    Pinyin,
}

struct EngineOutput {
    mode: Mode,
    preedit: String,
    candidates: Vec<String>,
    commit: Option<String>,
    passthrough: bool,
}

const MOD_CTRL: u32 = 1 << 2;
const MOD_ALT: u32 = 1 << 3;
const MOD_SUPER: u32 = 1 << 6;

fn should_toggle(keysym: u32, mods: u32) -> bool {
    (keysym == 0x20 && mods & MOD_CTRL != 0) || keysym == 0xffbf
}

fn is_punctuation(keysym: u32) -> bool {
    matches!(keysym, 0x21..=0x2f | 0x3a..=0x40 | 0x5b..=0x60 | 0x7b..=0x7e)
}

fn should_passthrough(keysym: u32, text: &str, mods: u32, composing: bool) -> bool {
    if mods & (MOD_CTRL | MOD_ALT | MOD_SUPER) != 0 {
        return true;
    }
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    if is_punctuation(keysym) {
        return false;
    }
    match keysym {
        0x20 | 0x31..=0x39 => !composing,
        0xff08 => !composing,
        0xff1b | 0xff0d => !composing,
        _ => true,
    }
}

fn choose_mode(letters_len: usize, wubi_candidate_count: usize) -> Mode {
    if letters_len > 4 || wubi_candidate_count == 0 {
        Mode::Pinyin
    } else {
        Mode::Wubi
    }
}

struct HybridEngine {
    _traits: Traits,
    wubi: Session,
    pinyin: Session,
    letters: String,
    mode: Mode,
    ascii_mode: bool,
}

impl HybridEngine {
    fn new() -> Self {
        let shared_dir = std::env::var("IME_RIME_SHARED_DIR")
            .unwrap_or_else(|_| "/usr/share/rime-data".to_string());
        let user_dir =
            std::env::var("IME_RIME_USER_DIR").unwrap_or_else(|_| "/tmp/rime-user".to_string());
        let log_dir = format!("{user_dir}/log");

        let mut traits = Traits::new();
        traits
            .set_shared_data_dir(&shared_dir)
            .set_user_data_dir(&user_dir)
            .set_distribution_name("ime-probe")
            .set_distribution_code_name("ime-probe")
            .set_distribution_version("0.0.1")
            .set_log_dir(&log_dir)
            .set_min_log_level(3);

        rime_api::setup(&mut traits);
        rime_api::initialize(&mut traits);
        let deploy = rime_api::full_deploy_and_wait();
        println!("RIME deploy: {deploy:?}");

        let wubi = rime_api::create_session().expect("wubi session");
        wubi.select_schema("wubi86").expect("select wubi86");
        let pinyin = rime_api::create_session().expect("pinyin session");
        pinyin
            .select_schema("pinyin_simp")
            .expect("select pinyin_simp");

        Self {
            _traits: traits,
            wubi,
            pinyin,
            letters: String::new(),
            mode: Mode::Wubi,
            ascii_mode: false,
        }
    }

    fn feed(&self, session: &Session, keysym: u32) {
        session.process_key(rime_api::KeyEvent::new(keysym, 0));
    }

    fn candidates_of(&self, session: &Session) -> (String, Vec<String>) {
        if let Some(ctx) = session.context() {
            let preedit = ctx.composition().preedit.unwrap_or_default().to_string();
            let candidates = ctx
                .menu()
                .candidates
                .iter()
                .map(|c| c.text.to_string())
                .collect();
            (preedit, candidates)
        } else {
            (String::new(), Vec::new())
        }
    }

    fn recur_mode(&mut self) {
        let (_, wubi_candidates) = self.candidates_of(&self.wubi);
        self.mode = choose_mode(self.letters.len(), wubi_candidates.len());
    }

    fn active(&self) -> &Session {
        match self.mode {
            Mode::Wubi => &self.wubi,
            Mode::Pinyin => &self.pinyin,
        }
    }

    fn reset(&mut self) {
        self.feed(&self.wubi, 0xff1b);
        self.feed(&self.pinyin, 0xff1b);
        self.letters.clear();
        self.mode = Mode::Wubi;
    }

    fn handle(&mut self, keysym: u32, text: &str, mods: u32) -> EngineOutput {
        if should_toggle(keysym, mods) {
            self.reset();
            self.ascii_mode = !self.ascii_mode;
            let label = if self.ascii_mode { "English" } else { "中文" };
            return EngineOutput {
                mode: self.mode,
                preedit: label.to_string(),
                candidates: Vec::new(),
                commit: None,
                passthrough: false,
            };
        }

        if self.ascii_mode {
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
            };
        }

        if should_passthrough(keysym, text, mods, !self.letters.is_empty()) {
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
            };
        }

        if !text.is_empty() && text.chars().all(|c| c.is_ascii_alphabetic()) {
            self.feed(&self.wubi, keysym);
            self.feed(&self.pinyin, keysym);
            self.letters.push_str(text);
            self.recur_mode();
            let (preedit, candidates) = self.candidates_of(self.active());
            return EngineOutput {
                mode: self.mode,
                preedit,
                candidates,
                commit: None,
                passthrough: false,
            };
        }

        if is_punctuation(keysym) {
            self.feed(self.active(), keysym);
            let commit = self.active().commit().map(|c| c.text().to_string());
            let passthrough = commit.is_none();
            if commit.is_some() {
                self.letters.clear();
                self.mode = Mode::Wubi;
            }
            return EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit,
                passthrough,
            };
        }

        match keysym {
            0x20 | 0x31..=0x39 => {
                self.feed(self.active(), keysym);
                let commit = self.active().commit().map(|c| c.text().to_string());
                if commit.is_some() {
                    self.reset();
                }
                EngineOutput {
                    mode: self.mode,
                    preedit: String::new(),
                    candidates: Vec::new(),
                    commit,
                    passthrough: false,
                }
            }
            0xff08 => {
                self.feed(&self.wubi, keysym);
                self.feed(&self.pinyin, keysym);
                self.letters.pop();
                self.recur_mode();
                let (preedit, candidates) = self.candidates_of(self.active());
                EngineOutput {
                    mode: self.mode,
                    preedit,
                    candidates,
                    commit: None,
                    passthrough: false,
                }
            }
            0xff1b | 0xff0d => {
                self.reset();
                EngineOutput {
                    mode: Mode::Wubi,
                    preedit: String::new(),
                    candidates: Vec::new(),
                    commit: None,
                    passthrough: false,
                }
            }
            _ => EngineOutput {
                mode: self.mode,
                preedit: String::new(),
                candidates: Vec::new(),
                commit: None,
                passthrough: true,
            },
        }
    }
}

impl Drop for HybridEngine {
    fn drop(&mut self) {
        let _ = self.wubi.close();
        let _ = self.pinyin.close();
        rime_api::finalize();
    }
}

struct State {
    _seat: wl_seat::WlSeat,
    im: ZwpInputMethodV2,
    grab: Option<ZwpInputMethodKeyboardGrabV2>,
    done_count: u32,
    _popup: ZwpInputPopupSurfaceV2,
    _popup_surface: wl_surface::WlSurface,
    popup_renderer: PopupRenderer,
    xkb_context: xkbcommon::xkb::Context,
    xkb_state: Option<xkbcommon::xkb::State>,
    mods: u32,
    forwarded_keys: HashSet<u32>,
    rime: HybridEngine,
    vk: ZwpVirtualKeyboardV1,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardManagerV1,
        _: zwp_virtual_keyboard_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardV1,
        _: zwp_virtual_keyboard_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpInputMethodManagerV2,
        _: zwp_input_method_manager_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputPopupSurfaceV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpInputPopupSurfaceV2,
        event: zwp_input_popup_surface_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        println!("POPUP event: {event:?}");
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        println!("IM event: {event:?}");
        match event {
            zwp_input_method_v2::Event::Activate => {
                if std::env::var_os("IME_NO_GRAB").is_none() && state.grab.is_none() {
                    state.grab = Some(proxy.grab_keyboard(qh, ()));
                    println!("  -> grab_keyboard");
                }
            }
            zwp_input_method_v2::Event::Deactivate => {
                state.forwarded_keys.clear();
                if let Some(grab) = state.grab.take() {
                    grab.release();
                    println!("  -> released grab");
                }
            }
            zwp_input_method_v2::Event::Done => {
                state.done_count += 1;
            }
            _ => {}
        }
    }
}

fn forward_key(state: &mut State, qh: &QueueHandle<State>, keycode: u32, pressed: bool) {
    if let Some(grab) = state.grab.take() {
        grab.release();
    }
    state.vk.key(0, keycode, if pressed { 1 } else { 0 });
    state.grab = Some(state.im.grab_keyboard(qh, ()));
    println!("PASSTHROUGH keycode={keycode} pressed={pressed}");
}

impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodKeyboardGrabV2,
        event: zwp_input_method_keyboard_grab_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { fd, size, .. } => {
                let keymap = unsafe {
                    xkbcommon::xkb::Keymap::new_from_fd(
                        &state.xkb_context,
                        fd,
                        size as usize,
                        xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS,
                    )
                };
                match keymap {
                    Ok(Some(keymap)) => {
                        state.xkb_state = Some(xkbcommon::xkb::State::new(&keymap));
                        println!("xkb keymap loaded ({size} bytes)");
                    }
                    Ok(None) => println!("xkb keymap: null"),
                    Err(err) => println!("xkb keymap error: {err}"),
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                state.mods = mods_depressed | mods_latched | mods_locked;
                if let Some(xkb_state) = &mut state.xkb_state {
                    xkb_state.update_mask(mods_depressed, mods_latched, mods_locked, group, 0, 0);
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Key {
                key,
                state: key_state,
                ..
            } => {
                let pressed = matches!(&key_state, WEnum::Value(wl_keyboard::KeyState::Pressed));
                if pressed {
                    let (sym, name, text) = {
                        let Some(xkb_state) = state.xkb_state.as_ref() else {
                            return;
                        };
                        let sym = xkb_state.key_get_one_sym((key + 8).into());
                        let name = xkbcommon::xkb::keysym_get_name(sym);
                        let text = xkb_state.key_get_utf8((key + 8).into());
                        (sym, name, text)
                    };
                    let sym_raw = u32::from(sym);
                    let mods = state.mods;

                    let EngineOutput {
                        mode,
                        preedit,
                        candidates,
                        commit,
                        passthrough,
                    } = state.rime.handle(sym_raw, &text, mods);

                    if passthrough {
                        state.forwarded_keys.insert(key);
                        forward_key(state, qh, key, true);
                        return;
                    }

                    if let Some(commit) = commit {
                        if !commit.is_empty() {
                            state.im.commit_string(commit.clone());
                            state.im.commit(state.done_count);
                            println!("COMMIT = {commit:?}");
                        }
                    }

                    let mut popup_text = String::new();
                    if !preedit.is_empty() {
                        popup_text.push_str(&preedit);
                        popup_text.push_str("   ");
                    }
                    for (i, candidate) in candidates.iter().take(6).enumerate() {
                        if i > 0 {
                            popup_text.push_str("  ");
                        }
                        popup_text.push_str(&format!("{}.{}", i + 1, candidate));
                    }
                    state.popup_renderer.draw(&popup_text);

                    println!(
                        "KEY keycode={key} sym={name} mode={mode:?} preedit={preedit:?} candidates={candidates:?}"
                    );
                } else if state.forwarded_keys.remove(&key) {
                    forward_key(state, qh, key, false);
                }
            }
            _ => {}
        }
    }
}

fn create_popup_buffer(
    shm: &wl_shm::WlShm,
    qh: &QueueHandle<State>,
) -> Result<(wl_shm_pool::WlShmPool, wl_buffer::WlBuffer, MmapMut), Box<dyn std::error::Error>> {
    let stride = POPUP_W as i32 * 4;
    let size = stride * POPUP_H as i32;

    let cname = std::ffi::CString::new("ime-popup")?;
    let fd = unsafe { libc::memfd_create(cname.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err("memfd_create failed".into());
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };

    let data = vec![0u8; size as usize];
    file.write_all(&data)?;

    let mmap = unsafe { MmapMut::map_mut(&file)? };
    let pool = shm.create_pool(file.as_fd(), size, qh, ());
    let buffer = pool.create_buffer(
        0,
        POPUP_W as i32,
        POPUP_H as i32,
        stride,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    Ok((pool, buffer, mmap))
}

struct PopupRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    surface: wl_surface::WlSurface,
    _buffer: wl_buffer::WlBuffer,
    _pool: wl_shm_pool::WlShmPool,
    mmap: MmapMut,
}

impl PopupRenderer {
    fn new(
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

    fn draw(&mut self, text: &str) {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<State>(&conn)?;
    let qh = event_queue.handle();

    let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=9, ())?;
    let compositor: wl_compositor::WlCompositor = globals.bind(&qh, 1..=5, ())?;
    let shm: wl_shm::WlShm = globals.bind(&qh, 1..=1, ())?;
    let manager: ZwpInputMethodManagerV2 = globals.bind(&qh, 1..=1, ())?;
    let im = manager.get_input_method(&seat, &qh, ());

    let vk_manager: ZwpVirtualKeyboardManagerV1 = globals.bind(&qh, 1..=1, ())?;
    let vk = vk_manager.create_virtual_keyboard(&seat, &qh, ());
    {
        let ctx = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);
        let keymap = xkbcommon::xkb::Keymap::new_from_names(
            &ctx,
            "",
            "",
            "us",
            "",
            None,
            xkbcommon::xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .expect("virtual keyboard keymap");
        let text = keymap.get_as_string(xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1);
        let cname = std::ffi::CString::new("ime-vk")?;
        let fd = unsafe { libc::memfd_create(cname.as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err("virtual keyboard memfd_create failed".into());
        }
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        file.write_all(text.as_bytes())?;
        vk.keymap(1, file.as_fd(), text.len() as u32);
    }

    let popup_surface = compositor.create_surface(&qh, ());
    let popup = im.get_input_popup_surface(&popup_surface, &qh, ());
    let (pool, buffer, mmap) = create_popup_buffer(&shm, &qh)?;

    popup_surface.attach(Some(&buffer), 0, 0);
    popup_surface.damage_buffer(0, 0, POPUP_W as i32, POPUP_H as i32);
    popup_surface.commit();
    let popup_renderer = PopupRenderer::new(popup_surface.clone(), buffer, pool, mmap);

    println!("bound input method + popup surface ({POPUP_W}x{POPUP_H}); waiting for activate...");
    let mut state = State {
        _seat: seat,
        im,
        grab: None,
        done_count: 0,
        _popup: popup,
        _popup_surface: popup_surface,
        popup_renderer,
        xkb_context: xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS),
        xkb_state: None,
        mods: 0,
        forwarded_keys: HashSet::new(),
        rime: HybridEngine::new(),
        vk,
    };

    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_after_five_letters() {
        assert_eq!(choose_mode(4, 2), Mode::Wubi);
        assert_eq!(choose_mode(5, 2), Mode::Pinyin);
    }

    #[test]
    fn fallback_without_wubi_candidates() {
        assert_eq!(choose_mode(2, 0), Mode::Pinyin);
        assert_eq!(choose_mode(2, 1), Mode::Wubi);
    }
}

#[cfg(test)]
mod passthrough_tests {
    use super::*;

    #[test]
    fn ctrl_shortcuts_pass_through() {
        assert!(should_passthrough(0x63, "c", MOD_CTRL, false));
        assert!(should_passthrough(0x76, "v", MOD_CTRL, true));
    }

    #[test]
    fn plain_letters_are_handled() {
        assert!(!should_passthrough(0x77, "w", 0, false));
        assert!(!should_passthrough(0x57, "W", 0, true));
    }

    #[test]
    fn navigation_keys_pass_through() {
        assert!(should_passthrough(0xff53, "", 0, true));
        assert!(should_passthrough(0xffbe, "", 0, false));
    }

    #[test]
    fn space_and_digits_depend_on_composition() {
        assert!(should_passthrough(0x20, " ", 0, false));
        assert!(!should_passthrough(0x20, " ", 0, true));
        assert!(should_passthrough(0x31, "1", 0, false));
        assert!(!should_passthrough(0x31, "1", 0, true));
    }
}

#[cfg(test)]
mod punctuation_tests {
    use super::*;

    #[test]
    fn punctuation_is_handled_unless_shortcut() {
        assert!(!should_passthrough(0x2e, ".", 0, false));
        assert!(!should_passthrough(0x3f, "?", 0, true));
        assert!(should_passthrough(0x2e, ".", MOD_CTRL, false));
    }

    #[test]
    fn punctuation_ranges() {
        assert!(is_punctuation(0x2e));
        assert!(is_punctuation(0x3f));
        assert!(!is_punctuation(0x20));
        assert!(!is_punctuation(0x61));
    }
}

#[cfg(test)]
mod toggle_tests {
    use super::*;

    #[test]
    fn ctrl_space_and_f2_toggle() {
        assert!(should_toggle(0x20, MOD_CTRL));
        assert!(should_toggle(0xffbf, 0));
        assert!(!should_toggle(0x20, 0));
        assert!(!should_toggle(0x61, 0));
    }
}
