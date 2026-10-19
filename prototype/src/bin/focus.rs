use cosmic_protocols::toplevel_info::v1::client::{
    zcosmic_toplevel_handle_v1::{self, ZcosmicToplevelHandleV1},
    zcosmic_toplevel_info_v1::{self, ZcosmicToplevelInfoV1},
};
use cosmic_protocols::toplevel_management::v1::client::zcosmic_toplevel_manager_v1::{
    self, ZcosmicToplevelManagerV1,
};
use wayland_client::{
    Connection, Dispatch, QueueHandle, event_created_child,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};

#[derive(Default)]
struct State {
    seat: Option<wl_seat::WlSeat>,
    info: Option<ZcosmicToplevelInfoV1>,
    manager: Option<ZcosmicToplevelManagerV1>,
    toplevels: Vec<Toplevel>,
}

struct Toplevel {
    handle: ZcosmicToplevelHandleV1,
    title: Option<String>,
    app_id: Option<String>,
    state: Vec<u32>,
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

impl Dispatch<ZcosmicToplevelInfoV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZcosmicToplevelInfoV1,
        event: zcosmic_toplevel_info_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zcosmic_toplevel_info_v1::Event::Toplevel { toplevel } = event {
            state.toplevels.push(Toplevel {
                handle: toplevel,
                title: None,
                app_id: None,
                state: Vec::new(),
            });
        }
    }

    event_created_child!(
        State,
        ZcosmicToplevelInfoV1,
        [
            zcosmic_toplevel_info_v1::EVT_TOPLEVEL_OPCODE => (ZcosmicToplevelHandleV1, ()),
        ]
    );
}

impl Dispatch<ZcosmicToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        handle: &ZcosmicToplevelHandleV1,
        event: zcosmic_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zcosmic_toplevel_handle_v1::Event::Title { title } => {
                if let Some(t) = state.toplevels.iter_mut().find(|t| &t.handle == handle) {
                    t.title = Some(title);
                }
            }
            zcosmic_toplevel_handle_v1::Event::AppId { app_id } => {
                if let Some(t) = state.toplevels.iter_mut().find(|t| &t.handle == handle) {
                    t.app_id = Some(app_id);
                }
            }
            zcosmic_toplevel_handle_v1::Event::State { state: raw_state } => {
                if let Some(t) = state.toplevels.iter_mut().find(|t| &t.handle == handle) {
                    t.state = raw_state
                        .chunks_exact(4)
                        .map(|c| u32::from_ne_bytes(c.try_into().unwrap()))
                        .collect();
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZcosmicToplevelManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZcosmicToplevelManagerV1,
        _: zcosmic_toplevel_manager_v1::Event,
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
    let info: ZcosmicToplevelInfoV1 = globals.bind(&qh, 1..=1, ())?;
    let manager: ZcosmicToplevelManagerV1 = globals.bind(&qh, 1..=4, ())?;

    let mut state = State {
        seat: Some(seat),
        info: Some(info),
        manager: Some(manager),
        toplevels: Vec::new(),
    };

    queue.roundtrip(&mut state)?;
    queue.roundtrip(&mut state)?;

    for t in &state.toplevels {
        println!(
            "toplevel app_id={:?} title={:?} state={:?}",
            t.app_id, t.title, t.state
        );
    }

    let target = state
        .toplevels
        .iter()
        .find(|t| {
            let hay = format!("{:?} {:?}", t.app_id, t.title).to_lowercase();
            hay.contains("cosmicterm")
                || hay.contains("terminal")
                || hay.contains("终端")
                || hay.contains("cosmic-edit")
        })
        .map(|t| (t.handle.clone(), t.title.clone()));

    match target {
        Some((handle, title)) => {
            state
                .manager
                .as_ref()
                .unwrap()
                .activate(&handle, state.seat.as_ref().unwrap());
            conn.flush()?;
            queue.roundtrip(&mut state)?;
            println!("activated {title:?}");
            queue.roundtrip(&mut state)?;
            for t in &state.toplevels {
                println!(
                    "after: app_id={:?} title={:?} state={:?}",
                    t.app_id, t.title, t.state
                );
            }
        }
        None => println!("no terminal/editor toplevel found"),
    }

    Ok(())
}
