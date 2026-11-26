#![windows_subsystem = "windows"]
mod chat;
use chat::Line;
use configparser::ini::Ini;
use eframe::egui::{self, Color32, ViewportCommand as Cmd};
use std::{collections::VecDeque, path::PathBuf, sync::mpsc, time::Duration};
use tray_icon::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

fn path(name: &str) -> PathBuf {
    std::env::current_exe().unwrap().with_file_name(name)
}
struct Settings {
    channel: String,
    font: u32,
    opacity: u32,
    limit: u32,
    retry: bool,
    delay: u32,
    size: [u32; 2],
    position: Option<[f32; 2]>,
}
impl Settings {
    fn read(ini: &Ini) -> Self {
        let num = |s, k, d, min, max| {
            ini.getuint(s, k)
                .ok()
                .flatten()
                .unwrap_or(d)
                .clamp(min, max) as u32
        };
        let position = ini.get("Window", "Position").and_then(|s| {
            let mut p = s
                .strip_prefix("@Point(")?
                .strip_suffix(')')?
                .split_whitespace();
            Some([p.next()?.parse().ok()?, p.next()?.parse().ok()?])
        });
        Self {
            channel: ini.get("Chat", "Channel").unwrap_or_default(),
            font: num("Style", "FontSize", 14, 8, 72),
            opacity: num("Style", "BgOpacity", 40, 0, 100),
            limit: num("Chat", "MaxMessages", 100, 1, 2000),
            retry: ini
                .getbool("Connection", "AutoReconnect")
                .ok()
                .flatten()
                .unwrap_or(true),
            delay: num("Connection", "ReconnectInterval", 10000, 1000, 300000),
            size: [
                num("Window", "Width", 350, 200, 3840),
                num("Window", "Height", 600, 150, 2160),
            ],
            position,
        }
    }
    fn save(&self) -> std::io::Result<()> {
        let mut ini = Ini::new();
        for (s, k, v) in [
            ("Chat", "Channel", self.channel.clone()),
            ("Chat", "MaxMessages", self.limit.to_string()),
            ("Style", "FontSize", self.font.to_string()),
            ("Style", "BgOpacity", self.opacity.to_string()),
            ("Connection", "AutoReconnect", self.retry.to_string()),
            ("Connection", "ReconnectInterval", self.delay.to_string()),
            ("Window", "Width", self.size[0].to_string()),
            ("Window", "Height", self.size[1].to_string()),
        ] {
            ini.set(s, k, Some(v));
        }
        if let Some([x, y]) = self.position {
            ini.set(
                "Window",
                "Position",
                Some(format!("@Point({} {})", x as i32, y as i32)),
            );
        }
        ini.write(path("config-v2.ini"))
    }
}
enum Action {
    Options,
    Toggle,
    Quit,
}
struct App {
    cfg: Settings,
    options: bool,
    visible: bool,
    error: String,
    lines: VecDeque<Line>,
    rx: mpsc::Receiver<Line>,
    stop: Option<mpsc::Sender<()>>,
    actions: mpsc::Receiver<Action>,
    _tray: TrayIcon,
}
impl App {
    fn connect(&mut self) {
        self.stop.take();
        self.lines.clear();
        let (tx, rx) = mpsc::channel();
        self.rx = rx;
        if self.cfg.channel.is_empty() {
            return;
        }
        let (stop, signal) = mpsc::channel();
        self.stop = Some(stop);
        chat::start(
            self.cfg.channel.clone(),
            self.cfg.retry,
            self.cfg.delay as u64,
            tx,
            signal,
        );
    }
    fn new(
        ctx: &egui::Context,
        cfg: Settings,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let menu = Menu::new();
        let options = MenuItem::new("Options", true, None);
        let toggle = MenuItem::new("Afficher / masquer", true, None);
        let quit = MenuItem::new("Quitter", true, None);
        menu.append_items(&[&options, &toggle, &quit])?;
        let (tx, actions) = mpsc::channel();
        let options_id = options.id().clone();
        let quit_id = quit.id().clone();
        let sender = tx.clone();
        let wake = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == options_id {
                Action::Options
            } else if event.id == quit_id {
                Action::Quit
            } else {
                Action::Toggle
            };
            let _ = sender.send(action);
            wake.request_repaint();
        }));
        let wake = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = tx.send(Action::Toggle);
                wake.request_repaint();
            }
        }));
        let mut pixels = Vec::new();
        for y in 0..32 {
            for x in 0..32 {
                let white = (6..26).contains(&x) && (7..22).contains(&y)
                    || ((7..12).contains(&x) && (21..26).contains(&y));
                pixels.extend(if white {
                    [255, 255, 255, 255]
                } else {
                    [104, 62, 180, 255]
                });
            }
        }
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Overlay Twitch Chat")
            .with_icon(tray_icon::Icon::from_rgba(pixels, 32, 32)?)
            .build()?;
        let mut app = Self {
            options: cfg.channel.is_empty(),
            visible: true,
            error: String::new(),
            lines: VecDeque::new(),
            rx: mpsc::channel().1,
            stop: None,
            actions,
            _tray: tray,
            cfg,
        };
        app.connect();
        Ok(app)
    }
}
impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.; 4]
    }
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        for action in self.actions.try_iter() {
            match action {
                Action::Quit => ctx.send_viewport_cmd(Cmd::Close),
                Action::Options => {
                    self.options = true;
                    self.visible = true;
                }
                Action::Toggle => self.visible = !self.visible,
            }
        }
        ctx.send_viewport_cmd(Cmd::MousePassthrough(!self.options || !self.visible));
        self.lines.extend(self.rx.try_iter());
        while self.lines.len() > self.cfg.limit as usize {
            self.lines.pop_front();
        }
        ctx.request_repaint_after(Duration::from_millis(150));
        if !self.visible {
            return;
        }
        let bg = if self.options {
            Color32::from_gray(30)
        } else {
            Color32::from_black_alpha((self.cfg.opacity * 255 / 100) as u8)
        };
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg).inner_margin(6))
            .show(ctx, |ui| {
                if self.options {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if ui
                            .add(
                                egui::Label::new("?  D�placer l'overlay")
                                    .sense(egui::Sense::drag()),
                            )
                            .drag_started()
                        {
                            ctx.send_viewport_cmd(Cmd::StartDrag);
                        }
                        ui.heading("Options Twitch");
                        ui.label("Cha�ne Twitch");
                        ui.text_edit_singleline(&mut self.cfg.channel);
                        ui.add(egui::Slider::new(&mut self.cfg.font, 8..=72).text("Police"));
                        ui.add(egui::Slider::new(&mut self.cfg.opacity, 0..=100).text("Fond (%)"));
                        ui.add(egui::Slider::new(&mut self.cfg.limit, 1..=2000).text("Messages"));
                        ui.add(
                            egui::Slider::new(&mut self.cfg.size[0], 200..=3840).text("Largeur"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.cfg.size[1], 150..=2160).text("Hauteur"),
                        );
                        ui.checkbox(&mut self.cfg.retry, "Reconnexion automatique");
                        ui.add(
                            egui::DragValue::new(&mut self.cfg.delay)
                                .range(1000..=300000)
                                .suffix(" ms"),
                        );
                        if ui.button("R�initialiser").clicked() {
                            self.cfg = Settings::read(&Ini::new());
                        }
                        if ui.button("Appliquer").clicked() {
                            self.cfg.channel = self
                                .cfg
                                .channel
                                .trim()
                                .trim_start_matches('#')
                                .to_ascii_lowercase();
                            if self.cfg.channel.is_empty()
                                || !self
                                    .cfg
                                    .channel
                                    .bytes()
                                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                            {
                                self.error = "Entre un identifiant de cha�ne Twitch valide.".into();
                            } else {
                                self.cfg.position = ctx
                                    .input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]));
                                match self.cfg.save() {
                                    Err(e) => self.error = e.to_string(),
                                    Ok(()) => {
                                        self.error.clear();
                                        ctx.send_viewport_cmd(Cmd::InnerSize(egui::vec2(
                                            self.cfg.size[0] as f32,
                                            self.cfg.size[1] as f32,
                                        )));
                                        self.options = false;
                                        self.connect();
                                    }
                                }
                            }
                        }
                        ui.label(&self.error);
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for Line(name, color, msg) in &self.lines {
                                let mut job = egui::text::LayoutJob::default();
                                for (s, c) in [
                                    (format!("{name} : "), *color),
                                    (msg.clone(), Color32::WHITE),
                                ] {
                                    job.append(
                                        &s,
                                        0.,
                                        egui::TextFormat {
                                            font_id: egui::FontId::proportional(
                                                self.cfg.font as f32,
                                            ),
                                            color: c,
                                            ..Default::default()
                                        },
                                    );
                                }
                                ui.label(job);
                            }
                        });
                }
            });
    }
}
fn main() -> eframe::Result<()> {
    let mut ini = Ini::new();
    let file = path("config-v2.ini");
    let _ = ini.load(if file.exists() {
        file
    } else {
        path("config.ini")
    });
    let cfg = Settings::read(&ini);
    let mut viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_always_on_top()
        .with_taskbar(false)
        .with_transparent(true)
        .with_inner_size([cfg.size[0] as f32, cfg.size[1] as f32]);
    if let Some(pos) = cfg.position {
        viewport = viewport.with_position(pos);
    }
    eframe::run_native(
        "Overlay Twitch Chat",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(App::new(&cc.egui_ctx, cfg)?))),
    )
}
