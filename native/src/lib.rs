//! Native desktop/Android frontend. All game assets are embedded at compile time.
mod audio;
mod render;
mod save;
mod text;
mod ui;

use diner_dash_engine::{
    game::{GameConfig, GameManager, GameMode, TableId},
    save::SaveManager,
};
use macroquad::prelude::*;
use render::NativeRenderer;
use text::draw_text;
use ui::{Action, Ui};
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

const CREDIT: &str = "Made by Nirusaki";
const W: f32 = 800.;
const H: f32 = 600.;

fn config() -> Conf {
    #[cfg(target_os = "linux")]
    let linux_backend = {
        let args: Vec<String> = std::env::args().collect();
        let force_wayland = args.iter().any(|a| a == "--wayland" || a == "-wayland")
            || std::env::var("DINER_DASH_BACKEND")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false);
        let force_x11 = args.iter().any(|a| a == "--x11" || a == "-x11")
            || std::env::var("DINER_DASH_BACKEND")
                .map(|v| v.eq_ignore_ascii_case("x11"))
                .unwrap_or(false);

        if force_wayland {
            macroquad::miniquad::conf::LinuxBackend::WaylandOnly
        } else if force_x11 {
            macroquad::miniquad::conf::LinuxBackend::X11Only
        } else if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            macroquad::miniquad::conf::LinuxBackend::WaylandWithX11Fallback
        } else {
            macroquad::miniquad::conf::LinuxBackend::X11WithWaylandFallback
        }
    };

    Conf {
        window_title: format!("Diner Dash - {CREDIT}"),
        window_width: 960,
        window_height: 720,
        high_dpi: true,
        icon: Some(macroquad::miniquad::conf::Icon {
            small: *include_bytes!("../../packaging/icon16.rgba"),
            medium: *include_bytes!("../../packaging/icon32.rgba"),
            big: *include_bytes!("../../packaging/icon64.rgba"),
        }),
        #[cfg(target_os = "linux")]
        platform: macroquad::miniquad::conf::Platform {
            linux_backend,
            wayland_decorations:
                macroquad::miniquad::conf::WaylandDecorations::ServerWithLibDecorFallback,
            linux_wm_class: "diner-dash",
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn run() {
    #[cfg(not(target_os = "android"))]
    {
        if std::env::args().any(|a| a == "--version") {
            println!("Diner Dash {} — {CREDIT}", env!("CARGO_PKG_VERSION"));
            return;
        }
        if std::env::args().any(|a| a == "--help" || a == "-h") {
            println!("Diner Dash {} — {CREDIT}", env!("CARGO_PKG_VERSION"));
            println!("Usage: diner-dash [OPTIONS]");
            println!();
            println!("Options:");
            println!("  --wayland       Force native Wayland backend");
            println!("  --x11           Force X11 backend");
            println!("  --version       Print version information");
            println!("  --help          Print this help message");
            println!();
            println!("Environment variables:");
            println!("  DINER_DASH_BACKEND=wayland|x11  Select windowing backend");
            return;
        }
    }
    #[cfg(target_os = "linux")]
    install_x11_handlers();
    macroquad::Window::from_config(config(), game_loop());
    #[cfg(not(target_os = "android"))]
    std::process::exit(0);
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn x11_nonfatal_error_handler(
    _display: *mut std::ffi::c_void,
    event: *mut std::ffi::c_void,
) -> std::os::raw::c_int {
    if !event.is_null() {
        let error_code = *(((event as *const u8).add(32)) as *const u8);
        if error_code == 3 || error_code == 4 || error_code == 8 || error_code == 9 {
            libc::_exit(0);
        }
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn x11_io_error_handler(_display: *mut std::ffi::c_void) -> std::os::raw::c_int {
    libc::_exit(0);
}

#[cfg(target_os = "linux")]
pub fn install_x11_handlers() {
    unsafe {
        let lib = libc::dlopen(
            b"libX11.so.6\0".as_ptr() as *const _,
            libc::RTLD_LAZY | libc::RTLD_GLOBAL,
        );
        if !lib.is_null() {
            type XSetErrorHandlerFn = unsafe extern "C" fn(
                Option<
                    unsafe extern "C" fn(
                        *mut std::ffi::c_void,
                        *mut std::ffi::c_void,
                    ) -> std::os::raw::c_int,
                >,
            ) -> *mut std::ffi::c_void;
            type XSetIOErrorHandlerFn = unsafe extern "C" fn(
                Option<unsafe extern "C" fn(*mut std::ffi::c_void) -> std::os::raw::c_int>,
            ) -> *mut std::ffi::c_void;

            let sym_err = libc::dlsym(lib, b"XSetErrorHandler\0".as_ptr() as *const _);
            if !sym_err.is_null() {
                let set_handler: XSetErrorHandlerFn = std::mem::transmute(sym_err);
                set_handler(Some(x11_nonfatal_error_handler));
            }
            let sym_io = libc::dlsym(lib, b"XSetIOErrorHandler\0".as_ptr() as *const _);
            if !sym_io.is_null() {
                let set_io: XSetIOErrorHandlerFn = std::mem::transmute(sym_io);
                set_io(Some(x11_io_error_handler));
            }
        }
        extern "C" fn handle_signal(_: libc::c_int) {
            unsafe {
                libc::_exit(0);
            }
        }
        libc::signal(libc::SIGINT, handle_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handle_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, handle_signal as *const () as libc::sighandler_t);
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn quad_main() {
    run();
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Menu,
    Playing,
    Paused,
    Levels,
    Endless,
    Upgrades,
    Help,
    Result,
    Comic,
    Prelevel,
    Difficulty,
    Settings,
    Scores,
    Login,
    ResetConfirm,
}

pub struct App {
    pub game: GameManager,
    pub profile: SaveManager,
    pub options: save::GameOptions,
    pub screen: Screen,
    pub previous_screen: Screen,
    pub has_session: bool,
    pub stage: u32,
    pub difficulty: u32,
    pub level: u32,
    pub help_page: u8,
    pub comic_pages: Vec<String>,
    pub comic_index: usize,
    pub ending_comic: bool,
    pub notice: String,
    pub error: String,
    pub login_selected: Option<usize>,
    pub login_editing: bool,
    pub login_input: String,
    pub scores_tab: u8,
    pub scores_recent_rank: Option<usize>,
}

impl App {
    fn new() -> Self {
        let mut profile = SaveManager::new();
        let restored = save::load(&mut profile);
        let has_session = restored.is_some();
        let options = save::get_options();
        let active_profile = save::get_active_profile();
        let profiles = save::list_profiles();
        let active_idx = profiles.iter().position(|p| p == &active_profile);
        Self {
            game: restored.unwrap_or_else(|| GameManager::new(GameConfig::default())),
            profile,
            options,
            screen: Screen::Menu,
            previous_screen: Screen::Menu,
            has_session,
            stage: 1,
            difficulty: 1,
            level: 1,
            help_page: 1,
            comic_pages: Vec::new(),
            comic_index: 0,
            ending_comic: false,
            notice: String::new(),
            error: String::new(),
            login_selected: active_idx,
            login_editing: false,
            login_input: String::new(),
            scores_tab: 0,
            scores_recent_rank: None,
        }
    }
    fn checkpoint(&mut self) {
        if self.has_session {
            self.error = save::store(&self.profile, &self.game)
                .err()
                .map(|e| format!("Save failed: {e}"))
                .unwrap_or_default();
        }
    }
    fn start(&mut self, mode: GameMode, stage: u32, level: u32) {
        let total = self.profile.get_career_progress().total_score;
        match self
            .game
            .init(mode, stage, level, self.difficulty)
            .and_then(|_| self.game.start_level())
        {
            Ok(()) => {
                if mode == GameMode::Career {
                    self.game.total_score = total;
                }
                self.screen = Screen::Playing;
                self.has_session = true;
                self.profile.record_game_started();
                self.checkpoint();
            }
            Err(e) => self.error = e.to_string(),
        }
    }
    fn resume(&mut self) {
        self.screen = if self.game.level_complete.is_some() {
            Screen::Result
        } else {
            Screen::Playing
        };
    }
}

async fn game_loop() {
    #[cfg(target_os = "linux")]
    install_x11_handlers();
    text::init();
    prevent_quit();
    simulate_mouse_with_touch(true);
    let mut app = App::new();
    let mut renderer = NativeRenderer::new();
    let mut ui = Ui::new();
    let mut audio = audio::Audio::new();
    audio.set_music(app.options.music);
    audio.set_sfx(app.options.sfx);
    let mut last_save = get_time();
    let mut fullscreen = app.options.fullscreen;
    if fullscreen {
        set_fullscreen(true);
    }
    loop {
        if is_quit_requested() {
            app.checkpoint();
            #[cfg(not(target_os = "android"))]
            std::process::exit(0);
            #[cfg(target_os = "android")]
            break;
        }
        if is_key_pressed(KeyCode::F11) {
            fullscreen = !fullscreen;
            app.options.fullscreen = fullscreen;
            set_fullscreen(fullscreen);
            save::save_options(&app.options);
        }

        let (scale, offset) = if app.options.stretch_display {
            (
                vec2(screen_width() / W, screen_height() / H),
                vec2(0., 0.),
            )
        } else {
            let s = (screen_width() / W).min(screen_height() / H);
            (
                vec2(s, s),
                vec2(
                    (screen_width() - W * s) / 2.,
                    (screen_height() - H * s) / 2.,
                ),
            )
        };

        let (mx, my) = mouse_position();
        let pointer = vec2((mx - offset.x) / scale.x, (my - offset.y) / scale.y);
        let click = is_mouse_button_pressed(MouseButton::Left);
        let dt = get_frame_time().min(0.1);

        if app.screen == Screen::Login && app.login_editing {
            while let Some(c) = get_char_pressed() {
                if (c.is_alphanumeric() || c == ' ' || c == '_' || c == '-') && app.login_input.len() < 12 {
                    app.login_input.push(c);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                app.login_input.pop();
            }
            if is_key_pressed(KeyCode::Enter) {
                app.action(Action::CommitProfile);
            }
        }

        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Back) {
            app.screen = match app.screen {
                Screen::Playing => {
                    app.game.drag = None;
                    app.checkpoint();
                    Screen::Paused
                }
                Screen::Paused => Screen::Playing,
                Screen::Settings | Screen::Scores | Screen::Login => app.previous_screen,
                _ => Screen::Menu,
            };
        }
        #[cfg(target_os = "android")]
        if save::PAUSE_REQUESTED.swap(false, std::sync::atomic::Ordering::Relaxed)
            && app.screen == Screen::Playing
        {
            app.game.drag = None;
            app.screen = Screen::Paused;
            app.checkpoint();
        }
        if app.screen == Screen::Playing {
            let pos = diner_dash_engine::math::Vec2::new(pointer.x, pointer.y);
            if click
                && Rect::new(0., 0., 800., 600.).contains(pointer)
                && !ui.blocks_game(&app, pointer)
                && app.game.pointer_down(pos)
            {
                audio
                    .effect("assets/audio/sfx_food_ready_1_snd_252.wav")
                    .await;
            }
            app.game.pointer_move(pos);
            if touches().iter().any(|t| t.phase == TouchPhase::Cancelled) {
                app.game
                    .pointer_up(diner_dash_engine::math::Vec2::new(-1000., -1000.));
            }
            if is_mouse_button_released(MouseButton::Left) && app.game.pointer_up(pos) {
                audio
                    .effect("assets/audio/sfx_seat_people_snd_243.wav")
                    .await;
                app.checkpoint();
            }
            for (i, key) in [
                KeyCode::Key1,
                KeyCode::Key2,
                KeyCode::Key3,
                KeyCode::Key4,
                KeyCode::Key5,
                KeyCode::Key6,
                KeyCode::Key7,
                KeyCode::Key8,
            ]
            .iter()
            .enumerate()
            {
                if is_key_pressed(*key) {
                    app.game.click_table(TableId(i as u64 + 1));
                }
            }
            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                app.game.click_kitchen();
            }
            if is_key_pressed(KeyCode::P) {
                app.game.click_podium();
            }
            let before = (
                app.game.ready_orders.len(),
                app.game.customers_served,
                app.game.customers_left,
            );
            if let Some(result) = app.game.update(dt) {
                let active_player = save::get_active_profile();
                if result.success {
                    app.profile.record_level_complete(&result);
                    if result.expert {
                        app.profile.record_expert(result.stage, result.level);
                    }
                    if app.game.mode == GameMode::Career {
                        app.profile.save_career_progress(
                            app.game.stage,
                            app.game.level,
                            app.game.total_score,
                            app.game.stars,
                            app.game.stars,
                        );
                        let rank = save::record_high_score("career", &active_player, app.game.total_score);
                        app.scores_recent_rank = rank;
                    }
                    audio.effect("assets/audio/sfx_build_rest_331.wav").await;
                } else {
                    app.profile.record_level_failed();
                    audio.effect("assets/audio/sfx_lose_heart_1_238.wav").await;
                }
                if app.game.mode == GameMode::Endless {
                    app.profile
                        .submit_endless_score(app.difficulty, result.score);
                    let diff_key = match app.difficulty {
                        1 => "endless_easy",
                        2 => "endless_medium",
                        _ => "endless_hard",
                    };
                    let rank = save::record_high_score(diff_key, &active_player, result.score);
                    app.scores_recent_rank = rank;
                }
                app.screen = Screen::Result;
                app.checkpoint();
            } else if app.game.customers_left > before.2 {
                audio.effect("assets/audio/sfx_lose_heart_1_238.wav").await;
            } else if app.game.customers_served > before.1 {
                audio
                    .effect("assets/audio/sfx_bring_check_1_snd_270.wav")
                    .await;
            } else if app.game.ready_orders.len() > before.0 {
                audio
                    .effect("assets/audio/sfx_food_ready_1_snd_252.wav")
                    .await;
            }
            if get_time() - last_save > 2. {
                app.checkpoint();
                last_save = get_time();
            }
        }

        audio
            .music(
                if !matches!(app.screen, Screen::Playing | Screen::Prelevel) {
                    0
                } else {
                    app.game.stage
                },
            )
            .await;

        clear_background(BLACK);
        let vp = if app.options.stretch_display {
            Some((
                0,
                0,
                (screen_width() * screen_dpi_scale()) as i32,
                (screen_height() * screen_dpi_scale()) as i32,
            ))
        } else {
            Some((
                (offset.x * screen_dpi_scale()) as i32,
                (offset.y * screen_dpi_scale()) as i32,
                (W * scale.x * screen_dpi_scale()) as i32,
                (H * scale.y * screen_dpi_scale()) as i32,
            ))
        };
        set_camera(&Camera2D {
            target: vec2(W / 2., H / 2.),
            zoom: vec2(2. / W, 2. / H),
            viewport: vp,
            ..Default::default()
        });

        if app.has_session {
            app.game.render(&mut renderer.commands);
            renderer.flush();
        } else {
            draw_rectangle(0., 0., W, H, Color::from_rgba(25, 38, 53, 255));
        }

        let action = ui.draw(&app, &mut renderer, pointer, click, get_time());
        if ui.hover_changed {
            audio.effect("assets/audio/sfx_rollover_1_316.wav").await;
        }
        if let Some(action) = action {
            audio
                .effect("assets/audio/sfx_food_ready_1_snd_252.wav")
                .await;
            if action == Action::Quit {
                app.checkpoint();
                #[cfg(not(target_os = "android"))]
                std::process::exit(0);
                #[cfg(target_os = "android")]
                break;
            }
            if let Action::ToggleMusic(val) = action {
                app.options.music = val;
                audio.set_music(val);
                save::save_options(&app.options);
            } else if let Action::ToggleSfx(val) = action {
                app.options.sfx = val;
                audio.set_sfx(val);
                save::save_options(&app.options);
            } else if let Action::ToggleFullscreen(val) = action {
                app.options.fullscreen = val;
                set_fullscreen(val);
                save::save_options(&app.options);
            } else if let Action::ToggleStretch(val) = action {
                app.options.stretch_display = val;
                save::save_options(&app.options);
            } else {
                app.action(action);
            }
        }
        if !app.error.is_empty() {
            draw_text(&app.error, 16., 580., 18., RED);
        }
        set_default_camera();
        next_frame().await;
    }
}

impl App {
    fn unlocked(&self, stage: u32, level: u32) -> bool {
        let p = self.profile.get_career_progress();
        (stage, level)
            <= (p.furthest_stage, p.furthest_level).max((p.current_stage, p.current_level))
    }
    fn begin_career(&mut self) {
        self.ending_comic = false;
        self.comic_pages = match (self.stage, self.level) {
            (1, 1) => vec!["intro-01-color", "intro-02-color", "intro-03-color"],
            (5, 1) => vec!["transformation-01-color", "transformation-02-colorB"],
            _ => Vec::new(),
        }
        .into_iter()
        .map(|s| format!("gfx.screens/{s}"))
        .collect();
        self.comic_index = 0;
        if self.comic_pages.is_empty() {
            self.prelevel();
        } else {
            self.screen = Screen::Comic;
        }
    }
    fn prelevel(&mut self) {
        self.start(GameMode::Career, self.stage, self.level);
        if self.error.is_empty() {
            self.screen = Screen::Prelevel;
        }
    }
    fn action(&mut self, action: Action) {
        match action {
            Action::Menu => {
                self.checkpoint();
                self.screen = Screen::Menu;
            }
            Action::Career => {
                let p = self.profile.get_career_progress();
                self.stage = p.current_stage;
                self.level = p.current_level;
                self.screen = Screen::Levels;
            }
            Action::Endless => self.screen = Screen::Endless,
            Action::Help(page) => {
                self.help_page = page;
                self.screen = Screen::Help;
            }
            Action::Settings => {
                self.previous_screen = self.screen;
                self.screen = Screen::Settings;
            }
            Action::CloseSettings => {
                save::save_options(&self.options);
                self.screen = self.previous_screen;
            }
            Action::Scores => {
                self.previous_screen = self.screen;
                self.scores_tab = 0;
                self.screen = Screen::Scores;
            }
            Action::CloseScores => {
                self.screen = self.previous_screen;
            }
            Action::ScoresTab(tab) => {
                self.scores_tab = tab;
            }
            Action::ScoresArrow(dir) => {
                if dir < 0 && self.scores_tab > 0 {
                    self.scores_tab -= 1;
                } else if dir > 0 && self.scores_tab < 3 {
                    self.scores_tab += 1;
                }
            }
            Action::Login => {
                self.previous_screen = self.screen;
                let profiles = save::list_profiles();
                let active = save::get_active_profile();
                self.login_selected = profiles.iter().position(|p| p == &active);
                self.login_editing = false;
                self.login_input.clear();
                self.screen = Screen::Login;
            }
            Action::CloseLogin => {
                self.login_editing = false;
                self.login_input.clear();
                self.screen = self.previous_screen;
            }
            Action::SelectProfile(idx) => {
                self.login_selected = Some(idx);
                self.login_editing = false;
            }
            Action::EditProfile(_) => {
                self.login_editing = true;
                self.login_input.clear();
            }
            Action::CommitProfile => {
                let name = self.login_input.trim().to_string();
                if !name.is_empty() {
                    if save::create_profile(&name) {
                        self.login_editing = false;
                        self.login_input.clear();
                        let profiles = save::list_profiles();
                        self.login_selected = profiles.iter().position(|p| p == &name);
                    }
                }
            }
            Action::PlayProfile => {
                let profiles = save::list_profiles();
                if let Some(idx) = self.login_selected {
                    if idx < profiles.len() {
                        save::set_active_profile(&profiles[idx]);
                        self.profile = SaveManager::new();
                        if let Some(restored) = save::load(&mut self.profile) {
                            self.game = restored;
                            self.has_session = true;
                        } else {
                            self.game = GameManager::new(GameConfig::default());
                            self.has_session = false;
                        }
                        self.screen = self.previous_screen;
                    }
                }
            }
            Action::DeleteProfile => {
                let profiles = save::list_profiles();
                if let Some(idx) = self.login_selected {
                    if idx < profiles.len() && profiles.len() > 1 {
                        let name = &profiles[idx];
                        if save::delete_profile(name) {
                            self.login_selected = Some(0);
                            self.profile = SaveManager::new();
                            if let Some(restored) = save::load(&mut self.profile) {
                                self.game = restored;
                                self.has_session = true;
                            } else {
                                self.game = GameManager::new(GameConfig::default());
                                self.has_session = false;
                            }
                        }
                    }
                }
            }
            Action::Resume | Action::Play => self.resume(),
            Action::Pause => {
                self.game.drag = None;
                self.checkpoint();
                self.screen = Screen::Paused;
            }
            Action::Select(s, l) => {
                self.stage = s;
                self.level = l;
            }
            Action::StartCareer => self.begin_career(),
            Action::Difficulty(s) => {
                self.stage = s;
                self.screen = Screen::Difficulty;
            }
            Action::StartEndless(d) => {
                self.difficulty = d;
                self.start(GameMode::Endless, self.stage, 1);
            }
            Action::CloseDifficulty => self.screen = Screen::Endless,
            Action::ComicPrevious => self.comic_index = self.comic_index.saturating_sub(1),
            Action::ComicNext => {
                if self.comic_index + 1 < self.comic_pages.len() {
                    self.comic_index += 1;
                } else if self.ending_comic {
                    self.screen = Screen::Levels;
                } else {
                    self.prelevel();
                }
            }
            Action::NextShift => {
                if let Some(r) = self.game.level_complete.clone() {
                    if r.end_game {
                        if r.success {
                            self.comic_pages = ["end-01-color", "end-02-color", "end-03-color"]
                                .iter()
                                .map(|s| format!("gfx.screens/{s}"))
                                .collect();
                            self.comic_index = 0;
                            self.ending_comic = true;
                            self.screen = Screen::Comic;
                        } else {
                            self.screen = Screen::Levels;
                        }
                    } else if self.game.mode == GameMode::Career {
                        self.stage = self.game.stage;
                        self.level = self.game.level;
                        self.begin_career();
                    } else {
                        self.start(GameMode::Endless, self.game.stage, 1);
                    }
                }
            }
            Action::Save | Action::Sound => {
                self.checkpoint();
                self.notice = "Saved.".into();
            }
            Action::Load => {
                if let Some(game) = save::load(&mut self.profile) {
                    self.game = game;
                    self.has_session = true;
                    self.resume();
                } else {
                    self.notice = "No saved game found.".into();
                }
            }
            Action::Export => {
                self.notice =
                    save::export(&self.profile).unwrap_or_else(|e| format!("Export failed: {e}"))
            }
            Action::Import => match save::import(&mut self.profile) {
                Ok(()) => {
                    let p = self.profile.get_career_progress();
                    self.stage = p.current_stage;
                    self.level = p.current_level;
                    self.start(GameMode::Career, self.stage, self.level);
                    self.screen = Screen::Prelevel;
                }
                Err(e) => self.notice = format!("Import failed: {e}"),
            },
            Action::ResetConfirm => self.screen = Screen::ResetConfirm,
            Action::Reset => match save::reset() {
                Ok(()) => {
                    self.profile = SaveManager::new();
                    self.has_session = false;
                    self.game = GameManager::new(GameConfig::default());
                    self.stage = 1;
                    self.level = 1;
                    self.screen = Screen::Menu;
                }
                Err(e) => self.error = e.to_string(),
            },
            Action::Upgrades => self.screen = Screen::Upgrades,
            Action::Upgrade(key) => {
                match self.game.purchase_upgrade(&key) {
                    Ok(()) => {
                        self.checkpoint();
                        self.screen = Screen::Playing;
                    }
                    Err(e) => self.error = e.to_string(),
                };
            }
            Action::Quit
            | Action::ToggleMusic(_)
            | Action::ToggleSfx(_)
            | Action::ToggleFullscreen(_)
            | Action::ToggleStretch(_) => {}
        }
    }
}
