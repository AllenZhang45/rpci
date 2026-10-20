use std::time::Duration;

use cosmic_text::{
    Attrs, Buffer as TextBuffer, Color as TextColor, FontSystem, Metrics, Shaping, SwashCache,
};
use smithay_client_toolkit::reexports::calloop::EventLoop;
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_compositor, wl_output, wl_region, wl_registry, wl_shm, wl_surface},
};

const APPLET_W: u32 = 120;
const APPLET_H: u32 = 34;
const SCALE: u32 = 2;
const PIXEL_W: u32 = APPLET_W * SCALE;
const PIXEL_H: u32 = APPLET_H * SCALE;

fn main() {
    let conn = Connection::connect_to_env().expect("Wayland connection");
    let (globals, event_queue) = registry_queue_init(&conn).expect("registry");
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<Applet> = EventLoop::try_new().expect("event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .expect("wayland source");

    let compositor = CompositorState::bind(&globals, &qh).expect("compositor");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell");
    let shm = Shm::bind(&globals, &qh).expect("shm");

    let surface = compositor.create_surface(&qh);
    surface.set_buffer_scale(SCALE as i32);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
    window.set_title("COSMIC IME");
    window.set_app_id("com.github.alec.RpciImeApplet");
    window.set_min_size(Some((APPLET_W, APPLET_H)));

    // The applet is a status indicator, not an input target. The empty input
    // region keeps the applet itself from receiving pointer input; clicking
    // the panel background is still compositor focus behavior.
    let raw_compositor: wl_compositor::WlCompositor =
        globals.bind(&qh, 1..=6, ()).expect("raw compositor");
    let region = raw_compositor.create_region(&qh, ());
    window.wl_surface().set_input_region(Some(&region));
    region.destroy();
    window.commit();

    let pool = SlotPool::new((PIXEL_W * PIXEL_H * 4) as usize, &shm).expect("pool");
    let mut applet = Applet {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        window,
        pool,
        buffer: None,
        label: read_label(),
        configured: false,
        font_system: FontSystem::new(),
        swash_cache: SwashCache::new(),
    };

    loop {
        event_loop
            .dispatch(Duration::from_millis(250), &mut applet)
            .expect("dispatch");
        let label = read_label();
        if label != applet.label {
            applet.label = label;
            if applet.configured {
                applet.draw(&qh);
            }
        }
    }
}

struct Applet {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    window: Window,
    pool: SlotPool,
    buffer: Option<smithay_client_toolkit::shm::slot::Buffer>,
    label: String,
    configured: bool,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Applet {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        if !self.configured {
            return;
        }
        let stride = PIXEL_W as i32 * 4;
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    PIXEL_W as i32,
                    PIXEL_H as i32,
                    stride,
                    wl_shm::Format::Argb8888,
                )
                .expect("create buffer")
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second, canvas) = self
                    .pool
                    .create_buffer(
                        PIXEL_W as i32,
                        PIXEL_H as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second;
                canvas
            }
        };

        // Transparent background; panel already provides the surface.
        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0]);
        }

        let dark = ime_probe::popup::cosmic_is_dark().unwrap_or(true);
        let color = if dark {
            TextColor::rgb(0xF2, 0xF2, 0xF2)
        } else {
            TextColor::rgb(0x1E, 0x1E, 0x1E)
        };
        let mut text_buffer = TextBuffer::new(
            &mut self.font_system,
            Metrics::new(16.0 * SCALE as f32, 22.0 * SCALE as f32),
        );
        text_buffer.set_size(
            Some(PIXEL_W as f32 - 12.0 * SCALE as f32),
            Some(PIXEL_H as f32),
        );
        text_buffer.set_text(
            &self.label,
            &Attrs::new(),
            Shaping::Advanced,
            Some(cosmic_text::Align::Center),
        );
        {
            let Applet {
                font_system,
                swash_cache,
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
                        if xx < 0 || yy < 0 || xx >= PIXEL_W as i32 || yy >= PIXEL_H as i32 {
                            continue;
                        }
                        let idx = ((yy as usize) * PIXEL_W as usize + xx as usize) * 4;
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

        self.window
            .wl_surface()
            .damage_buffer(0, 0, PIXEL_W as i32, PIXEL_H as i32);
        buffer.attach_to(self.window.wl_surface()).expect("attach");
        self.window.commit();
    }
}

fn read_label() -> String {
    ime_probe::paths::state_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|label| label.trim().to_string())
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "IME".to_string())
}

impl ProvidesRegistryState for Applet {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

impl OutputHandler for Applet {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl CompositorHandler for Applet {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl WindowHandler for Applet {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {}
    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &Window,
        _configure: WindowConfigure,
        _serial: u32,
    ) {
        if !self.configured {
            self.configured = true;
            self.draw(qh);
        }
    }
}

impl ShmHandler for Applet {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for Applet {
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

impl Dispatch<wl_region::WlRegion, ()> for Applet {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for Applet {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_registry!(Applet);
smithay_client_toolkit::delegate_dispatch2!(Applet);
