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
#[derive(Clone)]
struct Settings {
    completed: bool,
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
            completed: ini.get("App", "FirstLaunchCompleted").is_some(),
            channel: ini.get("Chat", "Channel").unwrap_or_default(),
            font: num("Style", "FontSize", 14, 8, 24),
            opacity: num("Style", "BgOpacity", 40, 0, 100),
            limit: num("Chat", "MaxMessages", 100, 10, 500),
            retry: ini
                .getbool("Connection", "AutoReconnect")
                .ok()
                .flatten()
                .unwrap_or(true),
            delay: num("Connection", "ReconnectInterval", 10000, 5000, 60000),
            size: [
                num("Window", "Width", 350, 200, 800),
                num("Window", "Height", 600, 200, 1200),
            ],
            position,
        }
    }
    fn save(&self) -> std::io::Result<()> {
        let mut ini = Ini::new_cs();
        if self.completed {
            ini.set("App", "FirstLaunchCompleted", Some("true".into()));
        }
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
        ini.write(path("config.ini"))
    }
}
enum Action {
    About,
    Options,
    Toggle,
    Quit,
}
struct App {
    cfg: Settings,
    draft: Settings,
    tab: usize,
    about: bool,
    prompt: u8,
    policy: std::sync::Arc<std::sync::Mutex<(bool, u64)>>,
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
        chat::start(self.cfg.channel.clone(), self.policy.clone(), tx, signal);
    }
    fn new(
        ctx: &egui::Context,
        cfg: Settings,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let menu = Menu::new();
        let options = MenuItem::new("Options", true, None);
        let toggle = MenuItem::new("À propos", true, None);
        let quit = MenuItem::new("Quitter", true, None);
        let separator = tray_icon::menu::PredefinedMenuItem::separator();
        menu.append_items(&[&options, &toggle, &separator, &quit])?;
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
                Action::About
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
        let pixels = [128u8, 0, 128, 255].repeat(32 * 32);
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Overlay Twitch Chat")
            .with_icon(tray_icon::Icon::from_rgba(pixels, 32, 32)?)
            .build()?;
        let mut app = Self {
            draft: cfg.clone(),
            tab: 0,
            about: false,
            prompt: 0,
            policy: std::sync::Arc::new(std::sync::Mutex::new((cfg.retry, cfg.delay as u64))),
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
                Action::About => self.about = true,
                Action::Options => {
                    if !self.options {
                        self.draft = self.cfg.clone();
                        self.tab = 0;
                        self.prompt = 0;
                        self.error.clear();
                    }
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
        let initializing = self.cfg.position.is_none();
        if initializing {
            let x = ctx.input(|i| {
                i.viewport()
                    .monitor_size
                    .map(|s| s.x - self.cfg.size[0] as f32 - 20.)
                    .unwrap_or(50.)
            });
            ctx.send_viewport_cmd(Cmd::OuterPosition(egui::pos2(x, 20.)));
            self.cfg.position = Some([x, 20.]);
            if let Err(e) = self.cfg.save() {
                self.error = e.to_string();
            }
        }
        if self.options {
            if !initializing
                && let Some(pos) =
                    ctx.input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]))
            {
                if self.cfg.position != Some(pos) {
                    self.cfg.position = Some(pos);
                    if let Err(e) = self.cfg.save() {
                        self.error = e.to_string();
                    }
                }
            }
            ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("options"),egui::ViewportBuilder::default().with_title("Options").with_inner_size([400.,360.]).with_always_on_top(),|dialog,_|{
                if dialog.input(|i|i.viewport().close_requested()){if self.cfg.channel.is_empty()&&!self.cfg.completed{dialog.send_viewport_cmd(Cmd::CancelClose);self.prompt=2;}else{self.options=false;}}
                egui::CentralPanel::default().show(dialog,|ui|{
                    if self.prompt!=0{
                        ui.label(if self.prompt==1{"Réinitialiser tous les paramètres et redémarrer l’application ?"}else{"Aucune chaîne configurée. Voulez-vous quitter l’application ?"});
                        if ui.button("Non").clicked(){if self.prompt==2{self.cfg.completed=true;if let Err(e)=self.cfg.save(){self.error=e.to_string();}self.options=false;}self.prompt=0;}
                        if ui.button("Oui").clicked(){if self.prompt==2{ctx.send_viewport_cmd_to(egui::ViewportId::ROOT,Cmd::Close);}else{
                            let reset=Settings::read(&Ini::new());match reset.save().and_then(|_|std::process::Command::new(std::env::current_exe()?).spawn()){Ok(_)=>ctx.send_viewport_cmd_to(egui::ViewportId::ROOT,Cmd::Close),Err(e)=>self.error=e.to_string()}self.prompt=0;
                        }}return;
                    }
                    ui.label("Vous pouvez déplacer le chat avec le clic gauche pendant que cette fenêtre est ouverte.");
                    if self.cfg.channel.is_empty(){ui.colored_label(Color32::LIGHT_RED,"Veuillez configurer une chaîne Twitch pour commencer.");}
                    ui.horizontal(|ui|{for (i,title) in ["Général","Apparence","Connexion"].iter().enumerate(){ui.selectable_value(&mut self.tab,i,*title);}});
                    ui.separator();
                    let d=&mut self.draft;
                    match self.tab{
                        0=>{ui.label("Chaîne Twitch");ui.text_edit_singleline(&mut d.channel);ui.add(egui::Slider::new(&mut d.limit,10..=500).text("Nb max. de messages"));},
                        1=>{ui.add(egui::Slider::new(&mut d.font,8..=24).text("Taille de la police"));ui.add(egui::Slider::new(&mut d.opacity,0..=100).text("Opacité (%)"));ui.add(egui::Slider::new(&mut d.size[0],200..=800).text("Largeur"));ui.add(egui::Slider::new(&mut d.size[1],200..=1200).text("Hauteur"));},
                        _=>{ui.checkbox(&mut d.retry,"Reconnexion automatique");ui.label("Intervalle de reconnexion");ui.add_enabled(d.retry,egui::DragValue::new(&mut d.delay).range(5000..=60000).speed(1000).suffix(" ms"));}
                    }
                    ui.separator();let file=path("config.ini");ui.label(file.display().to_string());
                    if ui.button("Ouvrir le dossier").clicked(){if let Err(e)=std::process::Command::new("explorer.exe").arg(file.parent().unwrap()).spawn(){self.error=e.to_string();}}
                    ui.horizontal(|ui|{
                        if ui.button("Réinitialiser").clicked(){self.prompt=1;}
                        if ui.add_enabled(self.cfg.completed||!self.cfg.channel.is_empty(),egui::Button::new("Annuler")).clicked(){self.options=false;}
                        if ui.button("OK").clicked(){
                            self.draft.channel=self.draft.channel.trim().to_string();
                            if self.draft.channel.is_empty()||self.draft.channel.contains(['\r','\n']){self.error="Vous devez entrer un nom de chaîne Twitch.".into();return;}
                            self.draft.position=self.cfg.position;self.draft.completed=true;
                            if let Err(e)=self.draft.save(){self.error=e.to_string();return;}
                            let changed=self.cfg.channel!=self.draft.channel;self.cfg=self.draft.clone();
                            *self.policy.lock().unwrap()=(self.cfg.retry,self.cfg.delay as u64);
                            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT,Cmd::InnerSize(egui::vec2(self.cfg.size[0] as f32,self.cfg.size[1] as f32)));
                            self.options=false;if changed{self.connect();}
                        }
                    });ui.label(&self.error);
                });
            });
        }
        if self.about {
            ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("about"),egui::ViewportBuilder::default().with_title("À propos").with_inner_size([350.,150.]).with_always_on_top(),|v,_|{egui::CentralPanel::default().show(v,|ui|{ui.heading("Overlay Twitch Chat");ui.label(concat!("Version ",env!("CARGO_PKG_VERSION")));ui.label("Développé par Bougli");ui.label("Un overlay pour afficher le chat Twitch par dessus n’importe quelle application.");if ui.button("OK").clicked()||v.input(|i|i.viewport().close_requested()){self.about=false;}});});
        }
        if !self.visible {
            return;
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(Color32::from_black_alpha((self.cfg.opacity*255/100) as u8)).inner_margin(5)).show(ctx,|ui|{
            if self.options && ctx.input(|i|i.pointer.primary_pressed()){ctx.send_viewport_cmd(Cmd::StartDrag);}
            if self.cfg.channel.is_empty(){ui.label("Aucune chaîne configurée. Veuillez configurer une chaîne Twitch via l’icône de la barre des tâches.");}
                    egui::ScrollArea::vertical()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).stick_to_bottom(true)
                        .show(ui, |ui| {
                            for Line(name, color, msg, moderator) in &self.lines {
                                let mut job = egui::text::LayoutJob::default();
                                if name.is_empty(){ui.label(egui::RichText::new(msg.as_str()).italics().size(self.cfg.font as f32));continue;}
                                for (s, c) in [
                                    (if *moderator {"● ".into()}else{String::new()},Color32::GREEN),
                                    (format!("{name}: "), *color),
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
            });
    }
}
fn main() -> eframe::Result<()> {
    let mut ini = Ini::new();
    let file = path("config.ini");
    let _ = ini.load(if file.exists() {
        file
    } else {
        path("config-v2.ini")
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
