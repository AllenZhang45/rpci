use std::collections::HashSet;
use std::io::Write;
use std::os::fd::{AsFd, FromRawFd};
use std::time::SystemTime;

use ime_probe::config::{CandidateLayout, Config};
use ime_probe::engine::{EngineOutput, HybridEngine};
use ime_probe::popup::{
    POPUP_DEFAULT_W, POPUP_SCALE, PopupRenderer, compose_lines, effective_layout, popup_height,
    popup_pixel_height, required_width,
};
use memmap2::MmapMut;
use smithay_client_toolkit::reexports::calloop::{
    EventLoop,
    channel::{Event as ChannelEvent, Sender, channel},
};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
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

struct State {
    _seat: wl_seat::WlSeat,
    im: ZwpInputMethodV2,
    grab: Option<ZwpInputMethodKeyboardGrabV2>,
    done_count: u32,
    candidate_count: usize,
    candidate_layout: CandidateLayout,
    popup_logical_width: u32,
    popup_logical_height: u32,
    shm: wl_shm::WlShm,
    config_mtime: Option<SystemTime>,
    _popup: ZwpInputPopupSurfaceV2,
    _popup_surface: wl_surface::WlSurface,
    popup_renderer: PopupRenderer,
    xkb_context: xkbcommon::xkb::Context,
    xkb_state: Option<xkbcommon::xkb::State>,
    mods: u32,
    forwarded_keys: HashSet<u32>,
    pending_taps: HashSet<u32>,
    deferred_ctrl: Option<u32>,
    ctrl_toggled: bool,
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
                reload_config_if_changed(state);
                if std::env::var_os("IME_NO_GRAB").is_none() && state.grab.is_none() {
                    state.grab = Some(proxy.grab_keyboard(qh, ()));
                    println!("  -> grab_keyboard");
                }
            }
            zwp_input_method_v2::Event::Unavailable => {
                eprintln!("another input method is already active on this seat; exiting");
                std::process::exit(1);
            }
            zwp_input_method_v2::Event::Deactivate => {
                state.rime.cancel();
                state.forwarded_keys.clear();
                state.pending_taps.clear();
                state.deferred_ctrl = None;
                state.ctrl_toggled = false;
                state.popup_renderer.draw("");
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

#[derive(Debug)]
enum Control {
    Toggle,
}

fn start_control_listener(sender: Sender<Control>) {
    let Some(path) = ime_probe::paths::control_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::remove_file(&path);
    let listener = match std::os::unix::net::UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("control socket failed: {err}");
            return;
        }
    };
    println!("control socket: {}", path.display());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            let mut buf = [0u8; 32];
            let Ok(size) = std::io::Read::read(&mut stream, &mut buf) else {
                continue;
            };
            let command = String::from_utf8_lossy(&buf[..size]);
            if command.trim() == "toggle" {
                let _ = sender.send(Control::Toggle);
            }
        }
    });
}

fn toggle_from_control(state: &mut State, source: &str) {
    state.rime.toggle_ascii();
    let label = state.rime.status_label();
    write_state(&label);
    state.popup_renderer.draw("");
    println!("{source} toggle -> {label}");
}

fn write_state(label: &str) {
    if let Some(path) = ime_probe::paths::state_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, label);
    }
}

fn config_mtime() -> Option<SystemTime> {
    let path = ime_probe::config::config_path()?;
    std::fs::metadata(path).ok()?.modified().ok()
}

/// Apply safe settings (candidate count and popup theme) when the config file
/// changes. Changing the language mode or Pinyin fallback still needs a full
/// restart because those are bound to RIME sessions.
fn reload_config_if_changed(state: &mut State) {
    let mtime = config_mtime();
    if mtime == state.config_mtime {
        return;
    }
    let config = Config::load();
    state.candidate_count = config.candidate_count;
    state.popup_renderer.set_preference(config.theme);
    state.config_mtime = mtime;
    println!(
        "config reloaded: candidates={}, theme={:?}",
        config.candidate_count, config.theme
    );
}

fn forward_key(state: &mut State, qh: &QueueHandle<State>, keycode: u32, pressed: bool) {
    // Enter (and keypad Enter) are sent as a complete tap. Re-grabbing between
    // press and release can make the application miss the key-up, so the
    // editor never inserts a newline.
    if matches!(keycode, 28 | 96) {
        if !pressed {
            return;
        }
        if let Some(grab) = state.grab.take() {
            grab.release();
        }
        state.vk.key(0, keycode, 1);
        state.vk.key(0, keycode, 0);
        state.grab = Some(state.im.grab_keyboard(qh, ()));
        println!("PASSTHROUGH keycode={keycode} tap");
        return;
    }

    if let Some(grab) = state.grab.take() {
        grab.release();
    }
    state.vk.key(0, keycode, if pressed { 1 } else { 0 });
    state.grab = Some(state.im.grab_keyboard(qh, ()));
    println!("PASSTHROUGH keycode={keycode} pressed={pressed}");
}

fn forward_tap(state: &mut State, qh: &QueueHandle<State>, keycode: u32) {
    // Send a complete tap only after the physical key is released. Forwarding
    // while the same physical key is still down makes COSMIC drop the synthetic
    // event on real keyboards.
    if let Some(grab) = state.grab.take() {
        grab.release();
    }
    state.vk.key(0, keycode, 1);
    state.vk.key(0, keycode, 0);
    state.grab = Some(state.im.grab_keyboard(qh, ()));
    println!("PASSTHROUGH tap keycode={keycode}");
}

fn is_modifier_key(keycode: u32) -> bool {
    matches!(keycode, 29 | 97 | 42 | 54 | 56 | 100 | 125 | 126 | 58)
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
                let is_ctrl = matches!(key, 29 | 97);

                // Ctrl+Space must be consumed before the application sees Ctrl.
                // Forwarding a half-pressed Ctrl and then swallowing Space drops
                // text-input focus on COSMIC. Defer Ctrl and toggle directly.
                if pressed && is_ctrl {
                    state.deferred_ctrl = Some(key);
                    return;
                }
                if !pressed && is_ctrl {
                    if state.ctrl_toggled {
                        state.ctrl_toggled = false;
                        state.deferred_ctrl = None;
                        // Clear Ctrl from the virtual keyboard too; otherwise a
                        // physical Ctrl can remain logically pressed in the
                        // focused client after Ctrl+Space.
                        state.mods &= !4;
                        state.vk.modifiers(state.mods, 0, 0, 0);
                    } else if state.deferred_ctrl.take().is_some() {
                        // Bare Ctrl is not forwarded.
                    } else if state.forwarded_keys.remove(&key) {
                        forward_key(state, qh, key, false);
                    }
                    return;
                }

                if pressed {
                    if key == 57 && state.deferred_ctrl.is_some() {
                        state.deferred_ctrl = None;
                        state.ctrl_toggled = true;
                        toggle_from_control(state, "CTRL+SPACE");
                        return;
                    }
                    if let Some(ctrl_key) = state.deferred_ctrl.take() {
                        state.forwarded_keys.insert(ctrl_key);
                        forward_key(state, qh, ctrl_key, true);
                    }

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
                        page,
                    } = state.rime.handle(sym_raw, &text, mods);

                    if passthrough {
                        if is_modifier_key(key) {
                            state.forwarded_keys.insert(key);
                            forward_key(state, qh, key, true);
                        } else {
                            state.pending_taps.insert(key);
                        }
                        return;
                    }

                    let label = state.rime.status_label();
                    write_state(&label);

                    if let Some(commit) = commit {
                        if !commit.is_empty() {
                            state.im.commit_string(commit.clone());
                            state.im.commit(state.done_count);
                            println!("COMMIT = {commit:?}");
                        }
                    }

                    let desired_width = required_width(
                        &preedit,
                        &candidates,
                        page,
                        state.candidate_layout,
                        state.candidate_count,
                    );
                    let desired_height =
                        popup_height(state.candidate_layout, state.candidate_count);
                    if desired_width != state.popup_logical_width
                        || desired_height != state.popup_logical_height
                    {
                        if let Ok((pool, buffer, mmap)) = create_popup_buffer(
                            &state.shm,
                            qh,
                            desired_width * POPUP_SCALE,
                            popup_pixel_height(desired_height),
                        ) {
                            state.popup_renderer.resize(
                                buffer,
                                pool,
                                mmap,
                                desired_width,
                                desired_height,
                            );
                            state.popup_logical_width = desired_width;
                            state.popup_logical_height = desired_height;
                        }
                    }
                    let lines = compose_lines(
                        &preedit,
                        &candidates,
                        page,
                        state.candidate_layout,
                        state.candidate_count,
                        state.popup_logical_width,
                    );
                    let line_refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
                    if line_refs.is_empty() {
                        state.popup_renderer.draw("");
                    } else {
                        state.popup_renderer.draw_lines(&line_refs);
                    }

                    println!(
                        "KEY keycode={key} sym={name} mode={mode:?} preedit={preedit:?} candidates={candidates:?}"
                    );
                } else if state.pending_taps.remove(&key) {
                    forward_tap(state, qh, key);
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
    pixel_width: u32,
    pixel_height: u32,
) -> Result<(wl_shm_pool::WlShmPool, wl_buffer::WlBuffer, MmapMut), Box<dyn std::error::Error>> {
    let stride = pixel_width as i32 * 4;
    let size = stride * pixel_height as i32;

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
        pixel_width as i32,
        pixel_height as i32,
        stride,
        wl_shm::Format::Argb8888,
        qh,
        (),
    );
    Ok((pool, buffer, mmap))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init::<State>(&conn)?;
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

    let config = Config::load();
    let popup_layout = effective_layout(config.candidate_layout, config.candidate_count);
    let popup_logical_height = popup_height(popup_layout, config.candidate_count);
    let popup_pixel_h = popup_pixel_height(popup_logical_height);
    let popup_surface = compositor.create_surface(&qh, ());
    let popup = im.get_input_popup_surface(&popup_surface, &qh, ());
    let (pool, buffer, mmap) =
        create_popup_buffer(&shm, &qh, POPUP_DEFAULT_W * POPUP_SCALE, popup_pixel_h)?;
    let popup_renderer = PopupRenderer::new(
        popup_surface.clone(),
        buffer.clone(),
        pool,
        mmap,
        config.theme,
        POPUP_DEFAULT_W,
        popup_logical_height,
    );

    popup_surface.attach(Some(&buffer), 0, 0);
    popup_surface.damage_buffer(
        0,
        0,
        (POPUP_DEFAULT_W * POPUP_SCALE) as i32,
        popup_pixel_h as i32,
    );
    popup_surface.commit();

    println!(
        "bound input method + popup surface ({POPUP_DEFAULT_W}x{popup_logical_height}, {popup_layout:?}); waiting for activate..."
    );
    let mut state = State {
        _seat: seat,
        im,
        grab: None,
        done_count: 0,
        candidate_count: config.candidate_count,
        candidate_layout: config.candidate_layout,
        popup_logical_width: POPUP_DEFAULT_W,
        popup_logical_height: popup_logical_height,
        shm: shm.clone(),
        config_mtime: config_mtime(),
        _popup: popup,
        _popup_surface: popup_surface,
        popup_renderer,
        xkb_context: xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS),
        xkb_state: None,
        mods: 0,
        forwarded_keys: HashSet::new(),
        pending_taps: HashSet::new(),
        deferred_ctrl: None,
        ctrl_toggled: false,
        rime: HybridEngine::new(&config),
        vk,
    };

    let label = state.rime.status_label();
    write_state(&label);

    let mut event_loop: EventLoop<State> = EventLoop::try_new()?;
    let handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(handle.clone())?;

    let (sender, receiver) = channel::<Control>();
    handle
        .insert_source(receiver, |event, _, state| {
            if let ChannelEvent::Msg(Control::Toggle) = event {
                toggle_from_control(state, "control");
            }
        })
        .map_err(|err| format!("control channel: {err}"))?;
    start_control_listener(sender);

    event_loop.run(None, &mut state, |_| {})?;
    Ok(())
}
