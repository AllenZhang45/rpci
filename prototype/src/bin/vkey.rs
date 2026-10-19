use std::io::Write;
use std::os::fd::{AsFd, FromRawFd, OwnedFd};

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::{self, ZwpVirtualKeyboardManagerV1},
    zwp_virtual_keyboard_v1::{self, ZwpVirtualKeyboardV1},
};

struct State {
    _seat: wl_seat::WlSeat,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<State>(&conn)?;
    let qh = queue.handle();

    let seat: wl_seat::WlSeat = globals.bind(&qh, 1..=9, ())?;
    let manager: ZwpVirtualKeyboardManagerV1 = globals.bind(&qh, 1..=1, ())?;
    let vk = manager.create_virtual_keyboard(&seat, &qh, ());
    let mut state = State { _seat: seat, vk };

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
    .ok_or("failed to create keymap")?;
    let text = keymap.get_as_string(xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1);

    let cname = std::ffi::CString::new("vkey")?;
    let fd = unsafe { libc::memfd_create(cname.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err("memfd_create failed".into());
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    file.write_all(text.as_bytes())?;
    let owned: OwnedFd = file.into();

    state.vk.keymap(1, owned.as_fd(), text.len() as u32);

    let mode = std::env::args().nth(1).unwrap_or_default();
    if mode == "alt-tab" {
        state.vk.modifiers(8, 0, 0, 0);
        state.vk.key(0, 56, 1);
        state.vk.key(0, 15, 1);
        state.vk.key(0, 15, 0);
        state.vk.key(0, 56, 0);
        state.vk.modifiers(0, 0, 0, 0);
        println!("sent alt+tab");
    } else if mode == "f1" {
        state.vk.key(0, 59, 1);
        state.vk.key(0, 59, 0);
        println!("sent F1");
    } else if mode == "preview" {
        for keycode in [17u32, 16, 23, 21] {
            state.vk.key(0, keycode, 1);
            state.vk.key(0, keycode, 0);
        }
        println!("sent preview sequence wqiy");
    } else if mode == "pinyin" {
        for keycode in [49u32, 23, 35, 30, 24, 57] {
            state.vk.key(0, keycode, 1);
            state.vk.key(0, keycode, 0);
        }
        println!("sent pinyin sequence nihao + space");
    } else if mode == "wubi" {
        for keycode in [17u32, 16, 23, 21, 57] {
            state.vk.key(0, keycode, 1);
            state.vk.key(0, keycode, 0);
        }
        println!("sent wubi sequence wqiy + space");
    } else if mode == "abc" {
        for keycode in [30u32, 48, 46] {
            state.vk.key(0, keycode, 1);
            state.vk.key(0, keycode, 0);
        }
        println!("sent keys a, b, c");
    } else {
        state.vk.key(0, 30, 1);
        state.vk.key(0, 30, 0);
        println!("sent key 'a' (keycode 30)");
    }
    conn.flush()?;
    queue.roundtrip(&mut state)?;

    Ok(())
}
