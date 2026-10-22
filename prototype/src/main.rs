mod candidate_window;

use candidate_window::CandidateWindow;

use std::io::Write;
use std::os::fd::{AsFd, FromRawFd};
use std::time::SystemTime;

use ime_probe::config::{CandidateLayout, Config};
use ime_probe::engine::HybridEngine;
use ime_probe::input_session::{Action, InputSession, KeyEvent};
use ime_probe::popup::CandidateView;

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
    config_mtime: Option<SystemTime>,
    candidate_window: CandidateWindow,
    xkb_context: xkbcommon::xkb::Context,
    xkb_state: Option<xkbcommon::xkb::State>,
    mods: u32,
    input: InputSession,
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
                let actions = state.input.cancel();
                apply_actions(state, qh, actions);
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

fn toggle_from_control(state: &mut State, qh: &QueueHandle<State>, source: &str) {
    let actions = state.input.toggle_ascii();
    let label = state.input.status_label();
    apply_actions(state, qh, actions);
    println!("{source} toggle -> {label}");
}

fn apply_actions(state: &mut State, qh: &QueueHandle<State>, actions: Vec<Action>) {
    for action in actions {
        match action {
            Action::Forward { keycode, pressed } => forward_key(state, qh, keycode, pressed),
            Action::Tap { keycode } => forward_tap(state, qh, keycode),
            Action::Commit(commit) => {
                state.im.commit_string(commit.clone());
                state.im.commit(state.done_count);
                println!("COMMIT = {commit:?}");
            }
            Action::ShowCandidates {
                preedit,
                candidates,
                page,
            } => state.candidate_window.show(
                qh,
                CandidateView {
                    preedit,
                    candidates,
                    page,
                    layout: state.candidate_layout,
                    candidate_count: state.candidate_count,
                },
            ),
            Action::Hide => state.candidate_window.hide(),
            Action::Status(label) => write_state(&label),
            Action::SetModifiers(mods) => {
                state.mods = mods;
                state.vk.modifiers(mods, 0, 0, 0);
            }
        }
    }
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
    state.candidate_window.set_preference(config.theme);
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

                let (sym, name, text) = {
                    let Some(xkb_state) = state.xkb_state.as_ref() else {
                        return;
                    };
                    let sym = xkb_state.key_get_one_sym((key + 8).into());
                    let name = xkbcommon::xkb::keysym_get_name(sym);
                    let text = xkb_state.key_get_utf8((key + 8).into());
                    (sym, name, text)
                };

                println!("KEY keycode={key} sym={name} pressed={pressed}");
                let actions = state.input.handle(KeyEvent {
                    keycode: key,
                    pressed,
                    is_ctrl,
                    keysym: u32::from(sym),
                    text,
                    mods: state.mods,
                });
                apply_actions(state, qh, actions);
            }
            _ => {}
        }
    }
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
    let popup_surface = compositor.create_surface(&qh, ());
    let popup = im.get_input_popup_surface(&popup_surface, &qh, ());
    let candidate_window = CandidateWindow::new(popup, popup_surface, shm.clone(), config.theme);

    println!("bound input method + candidate window; waiting for activate...");
    let mut state = State {
        _seat: seat,
        im,
        grab: None,
        done_count: 0,
        candidate_count: config.candidate_count,
        candidate_layout: config.candidate_layout,
        config_mtime: config_mtime(),
        candidate_window,
        xkb_context: xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS),
        xkb_state: None,
        mods: 0,
        input: InputSession::new(Box::new(HybridEngine::new(&config))),
        vk,
    };

    let label = state.input.status_label();
    write_state(&label);

    let mut event_loop: EventLoop<State> = EventLoop::try_new()?;
    let handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(handle.clone())?;

    let (sender, receiver) = channel::<Control>();
    let control_qh = qh.clone();
    handle
        .insert_source(receiver, move |event, _, state| {
            if let ChannelEvent::Msg(Control::Toggle) = event {
                toggle_from_control(state, &control_qh, "control");
            }
        })
        .map_err(|err| format!("control channel: {err}"))?;
    start_control_listener(sender);

    event_loop.run(None, &mut state, |_| {})?;
    Ok(())
}
