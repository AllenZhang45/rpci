use std::time::Duration;

use cosmic_text::{
    Attrs, Buffer as TextBuffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use ime_probe::backup::{self, KEEP_BACKUPS, default_paths};
use ime_probe::config::{Config, config_path};
use ime_probe::settings_model::SettingsModel;
use smithay_client_toolkit::activation::{ActivationHandler, ActivationState, RequestData};
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    },
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
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
};

const KEY_ESCAPE: u32 = 0xff1b;
const KEY_RETURN: u32 = 0xff0d;
const KEY_UP: u32 = 0xff52;
const KEY_DOWN: u32 = 0xff54;
const KEY_LEFT: u32 = 0xff51;
const KEY_RIGHT: u32 = 0xff53;
const KEY_S: u32 = 0x73;
const KEY_B: u32 = 0x62;
const KEY_R: u32 = 0x72;

const WIDTH: u32 = 560;
const HEIGHT: u32 = 380;
const BACKGROUND: [u8; 4] = [0x24, 0x24, 0x24, 0xff];
const HIGHLIGHT: [u8; 4] = [0x3a, 0x3a, 0x3a, 0xff];

fn main() {
    let conn = Connection::connect_to_env().expect("wayland connection");
    let (globals, event_queue) = registry_queue_init(&conn).expect("registry");
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<SettingsApp> = EventLoop::try_new().expect("event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle)
        .expect("wayland source");

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm");
    let xdg_activation = ActivationState::bind(&globals, &qh).ok();

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
    window.set_title("COSMIC IME 设置");
    window.set_app_id("io.github.alec.ImeSettings");
    window.set_min_size(Some((WIDTH, HEIGHT)));
    window.commit();

    if let Some(activation) = xdg_activation.as_ref() {
        activation.request_token(
            &qh,
            RequestData {
                seat_and_serial: None,
                surface: Some(window.wl_surface().clone()),
                app_id: Some("io.github.alec.ImeSettings".to_string()),
                udata: (),
            },
        );
    }

    let pool = SlotPool::new((WIDTH * HEIGHT * 4) as usize, &shm).expect("slot pool");
    let mut font_system = FontSystem::new();
    let text_buffer = TextBuffer::new(&mut font_system, Metrics::new(16.0, 30.0));
    let config = Config::load();
    let config_path = config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(未找到)".to_string());

    let mut app = SettingsApp {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        xdg_activation,
        exit: false,
        first_configure: true,
        pool,
        width: WIDTH,
        height: HEIGHT,
        window,
        keyboard: None,
        keyboard_focus: false,
        loop_handle: event_loop.handle(),
        model: SettingsModel::new(config),
        config_path,
        font_system,
        text_buffer,
        swash_cache: SwashCache::new(),
    };

    loop {
        event_loop
            .dispatch(Duration::from_millis(16), &mut app)
            .expect("dispatch");
        if app.exit {
            break;
        }
    }
}

struct SettingsApp {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    xdg_activation: Option<ActivationState>,
    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    window: Window,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    loop_handle: LoopHandle<'static, SettingsApp>,
    model: SettingsModel,
    config_path: String,
    font_system: FontSystem,
    text_buffer: TextBuffer,
    swash_cache: SwashCache,
}

impl SettingsApp {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let width = self.width.max(WIDTH);
        let height = self.height.max(HEIGHT);
        let stride = (width * 4) as i32;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        for px in canvas.chunks_exact_mut(4) {
            px.copy_from_slice(&BACKGROUND);
        }

        let selected_y = 120 + self.model.selected as u32 * 30;
        fill_rect(
            canvas,
            width,
            16,
            selected_y.saturating_sub(4),
            width - 32,
            28,
            HIGHLIGHT,
        );

        let lines = self.model.lines(&self.config_path);
        let text = lines.join("\n");
        self.text_buffer
            .set_size(Some(width as f32 - 40.0), Some(height as f32 - 20.0));
        self.text_buffer.set_text(
            &text,
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
            Some(cosmic_text::Align::Left),
        );

        {
            let SettingsApp {
                font_system,
                text_buffer,
                swash_cache,
                ..
            } = self;
            let mut borrowed = text_buffer.borrow_with(font_system);
            borrowed.draw(
                swash_cache,
                Color::rgb(0xE8, 0xE8, 0xE8),
                |x, y, gw, gh, c| {
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
                            let idx = ((yy as usize) * width as usize + xx as usize) * 4;
                            let b = (c.b() as u32 * a + canvas[idx] as u32 * inv) / 255;
                            let g = (c.g() as u32 * a + canvas[idx + 1] as u32 * inv) / 255;
                            let r = (c.r() as u32 * a + canvas[idx + 2] as u32 * inv) / 255;
                            canvas[idx] = b as u8;
                            canvas[idx + 1] = g as u8;
                            canvas[idx + 2] = r as u8;
                            canvas[idx + 3] = 0xFF;
                        }
                    }
                },
            );
        }

        save_debug_png(canvas, width, height);
        buffer
            .attach_to(self.window.wl_surface())
            .expect("buffer attach");
        self.window
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.window.commit();
    }

    fn save(&mut self) {
        match self.model.config.save() {
            Ok(path) => self.model.mark_saved(&path),
            Err(err) => self.model.mark_error(err),
        }
    }

    fn backup(&mut self) {
        match backup::backup_now(&default_paths(), KEEP_BACKUPS) {
            Ok(path) => self.model.status = format!("已备份到 {}", path.display()),
            Err(err) => self.model.mark_error(err),
        }
    }

    fn reload(&mut self) {
        self.model.mark_reloaded(Config::load());
    }
}

impl CompositorHandler for SettingsApp {
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

impl OutputHandler for SettingsApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl WindowHandler for SettingsApp {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(WIDTH);
        self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(HEIGHT);
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
        let _ = conn;
    }
}

impl ActivationHandler for SettingsApp {
    type RequestUdata = ();
    fn new_token(&mut self, token: String, _data: &RequestData<()>) {
        if let Some(activation) = self.xdg_activation.as_ref() {
            activation.activate::<SettingsApp>(self.window.wl_surface(), token);
        }
    }
}

impl SeatHandler for SettingsApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = Some(
                self.seat_state
                    .get_keyboard_with_repeat(
                        qh,
                        &seat,
                        None,
                        self.loop_handle.clone(),
                        Box::new(|_, _, _| {}),
                    )
                    .expect("keyboard"),
            );
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard {
            if let Some(keyboard) = self.keyboard.take() {
                keyboard.release();
            }
        }
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for SettingsApp {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        self.keyboard_focus = true;
    }
    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
        self.keyboard_focus = false;
    }
    fn press_key(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        let mut handled = true;
        match event.keysym.raw() {
            KEY_ESCAPE => self.exit = true,
            KEY_UP => self.model.move_selection(-1),
            KEY_DOWN => self.model.move_selection(1),
            KEY_LEFT => self.model.adjust(-1),
            KEY_RIGHT => self.model.adjust(1),
            KEY_RETURN => self.model.toggle(),
            KEY_S => self.save(),
            KEY_B => self.backup(),
            KEY_R => self.reload(),
            _ => handled = false,
        }
        if handled && !self.exit {
            self.draw(qh);
        }
    }
    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }
    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }
    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: Modifiers,
        _: RawModifiers,
        _: u32,
    ) {
    }
}

impl ShmHandler for SettingsApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

fn fill_rect(canvas: &mut [u8], width: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for yy in y..y + h {
        for xx in x..x + w {
            if xx >= width {
                continue;
            }
            let idx = (yy * width + xx) as usize * 4;
            if idx + 4 <= canvas.len() {
                canvas[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }
}

fn save_debug_png(canvas: &[u8], width: u32, height: u32) {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for px in canvas.chunks_exact(4) {
        rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
    if let Ok(file) = std::fs::File::create("/tmp/ime-settings.png") {
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(&rgba);
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for SettingsApp {
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

delegate_registry!(SettingsApp);

impl ProvidesRegistryState for SettingsApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}

smithay_client_toolkit::delegate_dispatch2!(SettingsApp);
