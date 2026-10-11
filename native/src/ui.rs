//! Native rendering of the same artwork, coordinates and animation timing as wasm/app.js.
use crate::{
    render::NativeRenderer,
    text::{draw_text, measure_text},
    App, Screen,
};
use diner_dash_engine::game::GameMode;
use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Menu,
    Career,
    Endless,
    Help(u8),
    Settings,
    Scores,
    Login,
    Quit,
    Resume,
    Pause,
    Select(u32, u32),
    StartCareer,
    Difficulty(u32),
    StartEndless(u32),
    CloseDifficulty,
    ComicPrevious,
    ComicNext,
    Play,
    NextShift,
    Sound,
    ToggleMusic(bool),
    ToggleSfx(bool),
    ToggleFullscreen(bool),
    ToggleStretch(bool),
    ScoresTab(u8),
    ScoresArrow(i32),
    SelectProfile(usize),
    EditProfile(usize),
    CommitProfile,
    DeleteProfile,
    PlayProfile,
    CloseSettings,
    CloseScores,
    CloseLogin,
    Save,
    Load,
    Export,
    Import,
    ResetConfirm,
    Reset,
    Upgrades,
    Upgrade(String),
}

#[derive(Clone, Deserialize)]
struct Frame {
    key: String,
    rect: [f32; 4],
    z: i32,
    anim: String,
}

#[derive(Clone, Deserialize)]
struct Node {
    id: String,
    frames: Vec<Frame>,
}

#[derive(Deserialize)]
struct Layouts {
    title: Vec<Node>,
    career: Vec<Node>,
}

#[derive(Deserialize)]
struct Asset {
    path: String,
    width: f32,
    height: f32,
}

struct Element {
    key: String,
    rect: [f32; 4],
    z: i32,
    action: Option<Action>,
    hover: String,
    enabled: bool,
    flip: bool,
    text: Option<(String, u16, Color)>,
}

impl Element {
    fn art(key: &str, rect: [f32; 4], z: i32) -> Self {
        Self {
            key: key.into(),
            rect,
            z,
            action: None,
            hover: String::new(),
            enabled: true,
            flip: false,
            text: None,
        }
    }
    fn button(key: &str, rect: [f32; 4], z: i32, hover: &str, action: Action) -> Self {
        let mut e = Self::art(key, rect, z);
        e.action = Some(action);
        e.hover = hover.into();
        e
    }
    fn text(text: &str, rect: [f32; 4], z: i32, size: u16, color: Color) -> Self {
        let mut e = Self::art("", rect, z);
        e.text = Some((text.into(), size, color));
        e
    }
}

pub struct Ui {
    layouts: Layouts,
    manifest: HashMap<String, Asset>,
    hovered: Option<Action>,
    pub hover_changed: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            layouts: serde_json::from_str(include_str!("../../packaging/menu_layouts.json"))
                .unwrap(),
            manifest: serde_json::from_slice(
                crate::asset("assets/data/sprite_manifest.json").unwrap(),
            )
            .unwrap(),
            hovered: None,
            hover_changed: false,
        }
    }
    fn frame<'a>(&self, node: &'a Node, time: f64) -> &'a Frame {
        let first = &node.frames[0];
        let i = if first.anim == "blink" {
            let cycle = (time * 1000.) as usize % 2200;
            if cycle < node.frames.len() * 60 {
                cycle / 60
            } else {
                0
            }
        } else {
            (time * 1000. / 80.) as usize % node.frames.len()
        };
        &node.frames[i]
    }
    fn elements(&self, app: &App, time: f64) -> Vec<Element> {
        let mut out = Vec::new();
        match app.screen {
            Screen::Menu => {
                for node in &self.layouts.title {
                    let f = self.frame(node, time);
                    match node.id.as_str() {
                        "user_bg" => {
                            out.push(Element::art(&f.key, f.rect, f.z));
                        }
                        "username_txt" => {
                            let name = crate::save::get_active_profile();
                            out.push(Element::text(&name, f.rect, f.z + 1, 14, WHITE));
                        }
                        "login_button" => {
                            out.push(Element::button(
                                &f.key,
                                f.rect,
                                f.z,
                                "gfx.screens/button_login_on",
                                Action::Login,
                            ));
                        }
                        "career_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Career);
                            out.push(e);
                        }
                        "endless_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Endless);
                            out.push(e);
                        }
                        "help_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Help(1));
                            out.push(e);
                        }
                        "options_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Settings);
                            out.push(e);
                        }
                        "hiscore_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Scores);
                            out.push(e);
                        }
                        "quit_button" => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            e.action = Some(Action::Quit);
                            out.push(e);
                        }
                        _ => {
                            let mut e = Element::art(&f.key, f.rect, f.z);
                            e.hover = format!("{}_on", f.key);
                            out.push(e);
                        }
                    }
                }
            }
            Screen::Levels => {
                for node in &self.layouts.career {
                    let id = &node.id;
                    if [
                        "const_",
                        "const_anim_",
                        "statue_1",
                        "statue_2",
                        "fadeout",
                        "sparkle_",
                        "lockall_",
                        "unlockall_",
                    ]
                    .iter()
                    .any(|p| id.starts_with(p))
                    {
                        continue;
                    }
                    let f = self.frame(node, time);
                    let mut e = Element::art(&f.key, f.rect, f.z);
                    let can_continue = app.has_session
                        && app.game.level_complete.is_none()
                        && app.game.mode == GameMode::Career
                        && (app.stage, app.level) == (app.game.stage, app.game.level);
                    match id.as_str() {
                        "menu_button" => e.action = Some(Action::Menu),
                        "play_button" => {
                            if !can_continue {
                                continue;
                            }
                            e.action = Some(Action::Resume);
                            e.hover = "gfx.screens/button_continue_on".into();
                        }
                        "newgame_button" => {
                            if can_continue {
                                continue;
                            }
                            e.action = Some(Action::StartCareer);
                            e.hover = "gfx.screens/button_new_game_up".into();
                        }
                        "statue_cover" => {
                            if !app.unlocked(5, 1) {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    if id.starts_with("lvlbutton_") {
                        let nums: Vec<u32> = id
                            .split('_')
                            .skip(1)
                            .filter_map(|n| n.parse().ok())
                            .collect();
                        if nums.len() == 2 {
                            let (s, l) = (nums[0], nums[1]);
                            if !app.unlocked(s, l) {
                                continue;
                            }
                            let chosen = (app.stage, app.level) == (s, l);
                            let continuing = app.has_session
                                && app.game.level_complete.is_none()
                                && app.game.mode == GameMode::Career
                                && (s, l) == (app.game.stage, app.game.level);
                            let key = if continuing {
                                if chosen {
                                    "lvlcontinue_selected"
                                } else {
                                    "lvlcontinue_normal"
                                }
                            } else if app.profile.is_expert(s, l) {
                                if chosen {
                                    "leveldot_star_selected"
                                } else {
                                    "leveldot_star"
                                }
                            } else if chosen {
                                "leveldot_selected"
                            } else {
                                "leveldot_normal"
                            };
                            e.key = format!("gfx.screens/{key}");
                            e.z = 100;
                            e.action = Some(Action::Select(s, l));
                            e.enabled = true;
                        }
                    }
                    if let Some(s) = id.strip_prefix("rest_").and_then(|n| n.parse::<u32>().ok()) {
                        if s > 1 && !app.unlocked(s, 1) {
                            continue;
                        }
                    }
                    out.push(e);
                }
            }
            Screen::Endless | Screen::Difficulty => {
                out.push(Element::art(
                    "gfx.screens/background",
                    [0., 0., 800., 600.],
                    1,
                ));
                out.push(Element::art(
                    "gfx.screens/Title_Endless",
                    [5., 0., 535., 69.],
                    17,
                ));
                out.push(Element::button(
                    "gfx.screens/button_back_to_menu",
                    [686., 6., 797., 34.],
                    18,
                    "gfx.screens/button_back_to_menu_on",
                    Action::Menu,
                ));
                for (s, key, rect, z) in [
                    (1, "r1", [53., 105., 305., 283.], 2),
                    (2, "RT_2", [486., 57., 789., 330.], 7),
                    (3, "RT_3", [591., 337., 800., 590.], 5),
                    (4, "RT_4", [0., 318., 321., 600.], 3),
                    (5, "RT_5", [279., 225., 524., 398.], 11),
                ] {
                    if app.unlocked(s, 1) {
                        out.push(Element::button(
                            &format!("gfx.screens/{key}"),
                            rect,
                            z,
                            "",
                            Action::Difficulty(s),
                        ));
                    }
                }
                if app.screen == Screen::Difficulty {
                    for e in &mut out {
                        e.action = None;
                    }
                    out.push(Element::art("@dim", [0., 0., 800., 600.], 199));
                    out.push(Element::art(
                        "gfx.screens/hsp_wind",
                        [175., 137., 638., 489.],
                        201,
                    ));
                    out.push(Element::art(
                        "gfx.screens/ENDLESS_MSG",
                        [262., 271., 544., 326.],
                        202,
                    ));
                    for (d, rect) in [
                        (1, [206., 352., 319., 394.]),
                        (2, [331., 353., 475., 393.]),
                        (3, [486., 354., 599., 396.]),
                    ] {
                        out.push(Element::button(
                            &format!("gfx.screens/diff_{d}_off"),
                            rect,
                            203,
                            &format!("gfx.screens/diff_{d}_on"),
                            Action::StartEndless(d),
                        ));
                    }
                    out.push(Element::button(
                        "gfx.screens/hsp_menu",
                        [302., 412., 507., 451.],
                        204,
                        "gfx.screens/hsp_menu_on",
                        Action::CloseDifficulty,
                    ));
                }
            }
            Screen::Settings => {
                out.push(Element::art("@dim", [0., 0., 800., 600.], 200));
                out.push(Element::art(
                    "gfx.screens/hsp_wind",
                    [174., 107., 637., 459.],
                    201,
                ));
                out.push(Element::art(
                    "gfx.screens/options_lbl",
                    [279., 221., 538., 268.],
                    202,
                ));

                // Music
                out.push(Element::art(
                    "gfx.screens/opt_music_lbl2",
                    [305., 276., 402., 294.],
                    203,
                ));
                if app.options.music {
                    out.push(Element::art(
                        "gfx.screens/opt_on_hilite",
                        [424., 274., 468., 296.],
                        204,
                    ));
                } else {
                    out.push(Element::button(
                        "gfx.screens/opt_on",
                        [424., 274., 468., 296.],
                        204,
                        "gfx.screens/opt_on_hilite",
                        Action::ToggleMusic(true),
                    ));
                }
                out.push(Element::text(
                    "/",
                    [464., 274., 485., 296.],
                    204,
                    18,
                    WHITE,
                ));
                if !app.options.music {
                    out.push(Element::art(
                        "gfx.screens/opt_off_hilite",
                        [481., 274., 531., 296.],
                        204,
                    ));
                } else {
                    out.push(Element::button(
                        "gfx.screens/opt_off",
                        [481., 274., 531., 296.],
                        204,
                        "gfx.screens/opt_off_hilite",
                        Action::ToggleMusic(false),
                    ));
                }

                // SFX
                out.push(Element::art(
                    "gfx.screens/sfx_lbl",
                    [277., 316., 401., 334.],
                    203,
                ));
                if app.options.sfx {
                    out.push(Element::art(
                        "gfx.screens/opt_on_hilite",
                        [424., 314., 468., 336.],
                        204,
                    ));
                } else {
                    out.push(Element::button(
                        "gfx.screens/opt_on",
                        [424., 314., 468., 336.],
                        204,
                        "gfx.screens/opt_on_hilite",
                        Action::ToggleSfx(true),
                    ));
                }
                out.push(Element::text(
                    "/",
                    [464., 314., 485., 336.],
                    204,
                    18,
                    WHITE,
                ));
                if !app.options.sfx {
                    out.push(Element::art(
                        "gfx.screens/opt_off_hilite",
                        [481., 314., 531., 336.],
                        204,
                    ));
                } else {
                    out.push(Element::button(
                        "gfx.screens/opt_off",
                        [481., 314., 531., 336.],
                        204,
                        "gfx.screens/opt_off_hilite",
                        Action::ToggleSfx(false),
                    ));
                }

                // Fullscreen (Desktop) / Stretch Display (Android / Desktop)
                out.push(Element::art(
                    "gfx.screens/fs_lbl",
                    [240., 356., 405., 374.],
                    203,
                ));
                let is_fullscreen_or_stretch = if cfg!(target_os = "android") {
                    app.options.stretch_display
                } else {
                    app.options.fullscreen
                };
                if is_fullscreen_or_stretch {
                    out.push(Element::art(
                        "gfx.screens/opt_on_hilite",
                        [424., 354., 468., 376.],
                        204,
                    ));
                } else {
                    let action = if cfg!(target_os = "android") {
                        Action::ToggleStretch(true)
                    } else {
                        Action::ToggleFullscreen(true)
                    };
                    out.push(Element::button(
                        "gfx.screens/opt_on",
                        [424., 354., 468., 376.],
                        204,
                        "gfx.screens/opt_on_hilite",
                        action,
                    ));
                }
                out.push(Element::text(
                    "/",
                    [464., 354., 485., 376.],
                    204,
                    18,
                    WHITE,
                ));
                if !is_fullscreen_or_stretch {
                    out.push(Element::art(
                        "gfx.screens/opt_off_hilite",
                        [481., 354., 531., 376.],
                        204,
                    ));
                } else {
                    let action = if cfg!(target_os = "android") {
                        Action::ToggleStretch(false)
                    } else {
                        Action::ToggleFullscreen(false)
                    };
                    out.push(Element::button(
                        "gfx.screens/opt_off",
                        [481., 354., 531., 376.],
                        204,
                        "gfx.screens/opt_off_hilite",
                        action,
                    ));
                }

                // OK Button
                out.push(Element::button(
                    "gfx.screens/opt_ok",
                    [329., 387., 486., 436.],
                    205,
                    "gfx.screens/opt_ok_rollover",
                    Action::CloseSettings,
                ));
            }
            Screen::Scores => {
                out.push(Element::art(
                    "gfx.screens/board",
                    [0., 0., 800., 600.],
                    1,
                ));
                out.push(Element::art(
                    "gfx.screens/high_scoreS",
                    [270., 59., 509., 86.],
                    2,
                ));
                out.push(Element::art(
                    "gfx.screens/line_h_long",
                    [58., 94., 743., 114.],
                    3,
                ));

                if app.scores_tab > 0 {
                    out.push(Element::button(
                        "gfx.screens/hs_arrow",
                        [149., 124., 195., 157.],
                        10,
                        "gfx.screens/hs_arrow_on",
                        Action::ScoresArrow(-1),
                    ));
                }
                if app.scores_tab < 3 {
                    let mut r_arr = Element::button(
                        "gfx.screens/hs_arrow",
                        [590., 123., 636., 157.],
                        10,
                        "gfx.screens/hs_arrow_on",
                        Action::ScoresArrow(1),
                    );
                    r_arr.flip = true;
                    out.push(r_arr);
                }

                let tab_name = match app.scores_tab {
                    0 => "Flo's career",
                    1 => "endless (easy)",
                    2 => "endless (medium)",
                    _ => "endless (hard)",
                };
                out.push(Element::text(
                    tab_name,
                    [207., 124., 573., 161.],
                    11,
                    20,
                    WHITE,
                ));

                let high_scores = crate::save::get_high_scores();
                let list = match app.scores_tab {
                    0 => &high_scores.career,
                    1 => &high_scores.endless_easy,
                    2 => &high_scores.endless_medium,
                    _ => &high_scores.endless_hard,
                };

                for (i, entry) in list.iter().enumerate().take(10) {
                    let y = 174. + i as f32 * 31.;
                    let color = if app.scores_recent_rank == Some(i + 1) {
                        Color::from_rgba(92, 255, 255, 255)
                    } else {
                        WHITE
                    };
                    out.push(Element::text(
                        &format!("{}. {}", i + 1, entry.name),
                        [120., y, 430., y + 25.],
                        20,
                        16,
                        color,
                    ));
                    out.push(Element::text(
                        &format!("{}", entry.score),
                        [480., y, 680., y + 25.],
                        20,
                        16,
                        color,
                    ));
                }

                out.push(Element::button(
                    "gfx.screens/Back_to_Menu_off2",
                    [315., 490., 488., 531.],
                    25,
                    "gfx.screens/Back_to_Menu_on2",
                    Action::CloseScores,
                ));
            }
            Screen::Login => {
                out.push(Element::art(
                    "gfx.screens/board",
                    [0., 0., 800., 600.],
                    1,
                ));
                out.push(Element::art(
                    "gfx.screens/profile_overlay",
                    [47., 45., 738., 104.],
                    2,
                ));

                let profiles = crate::save::list_profiles();
                let ct = profiles.len();

                for i in 0..10 {
                    let y = 105. + i as f32 * 40.;
                    let box_rect = [246., y, 528., y + 41.];
                    let text_rect = [260., y + 8., 516., y + 34.];
                    let z = 10 + i as i32 * 2;

                    if i < ct {
                        if app.login_selected == Some(i) {
                            out.push(Element::art("gfx.screens/login_selected", box_rect, z));
                            out.push(Element::text(
                                &profiles[i],
                                text_rect,
                                z + 1,
                                15,
                                Color::from_rgba(128, 255, 255, 255),
                            ));
                        } else {
                            out.push(Element::button(
                                "gfx.screens/login_normal",
                                box_rect,
                                z,
                                "gfx.screens/login_rollover",
                                Action::SelectProfile(i),
                            ));
                            out.push(Element::text(&profiles[i], text_rect, z + 1, 15, WHITE));
                        }
                    } else if i == ct && ct < 10 {
                        if app.login_editing {
                            out.push(Element::art("gfx.screens/login_selected", box_rect, z));
                            let cursor = if (time * 3.0) as usize % 2 == 0 { "_" } else { "" };
                            out.push(Element::text(
                                &format!("{}{}", app.login_input, cursor),
                                text_rect,
                                z + 1,
                                15,
                                Color::from_rgba(255, 255, 128, 255),
                            ));
                            out.push(Element::button(
                                "gfx.screens/login_ok_up",
                                [539., y, 607., y + 42.],
                                z + 1,
                                "",
                                Action::CommitProfile,
                            ));
                        } else {
                            out.push(Element::button(
                                "gfx.screens/login_normal",
                                box_rect,
                                z,
                                "gfx.screens/login_rollover",
                                Action::EditProfile(i),
                            ));
                            out.push(Element::text(
                                "[New Player]",
                                text_rect,
                                z + 1,
                                15,
                                Color::from_rgba(255, 255, 128, 255),
                            ));
                        }
                    } else {
                        let mut e = Element::art("gfx.screens/login_normal", box_rect, z);
                        e.enabled = false;
                        out.push(e);
                    }
                }

                out.push(Element::button(
                    "gfx.screens/login_cancel",
                    [108., 509., 260., 554.],
                    50,
                    "gfx.screens/login_cancel_up",
                    Action::CloseLogin,
                ));

                let can_action = app.login_selected.is_some() && !app.login_editing;
                if can_action && profiles.len() > 1 {
                    out.push(Element::button(
                        "gfx.screens/login_delete",
                        [325., 509., 477., 554.],
                        50,
                        "gfx.screens/login_delete_up",
                        Action::DeleteProfile,
                    ));
                }
                if can_action {
                    out.push(Element::button(
                        "gfx.screens/login_play",
                        [545., 509., 697., 554.],
                        50,
                        "gfx.screens/login_play_up",
                        Action::PlayProfile,
                    ));
                }
            }
            Screen::Help => {
                out.push(Element::art(
                    "gfx.screens/help_frame",
                    [0., 0., 800., 600.],
                    1,
                ));
                let pages = [
                    [80., 67., 709., 529.],
                    [72., 66., 728., 549.],
                    [72., 67., 715., 541.],
                    [72., 49., 714., 510.],
                    [73., 68., 716., 534.],
                    [67., 49., 728., 543.],
                ];
                let credits = app.help_page == 6;
                let key = if credits {
                    "gfx.screens/help_CREDITS_PAGE".into()
                } else {
                    format!("gfx.screens/help_page_{}", app.help_page)
                };
                out.push(Element::art(&key, pages[(app.help_page - 1) as usize], 3));
                out.push(Element::art(
                    if credits {
                        "gfx.screens/help_title_CREDITS"
                    } else {
                        "gfx.screens/help_title_instructions"
                    },
                    if credits {
                        [331., 3., 474., 44.]
                    } else {
                        [282., 2., 517., 43.]
                    },
                    4,
                ));
                let key = if credits { "instruction" } else { "credits" };
                out.push(Element::button(
                    &format!("gfx.screens/help_button_{key}"),
                    [416., 542., 593., 586.],
                    6,
                    &format!("gfx.screens/help_button_{key}_up"),
                    Action::Help(if credits { 1 } else { 6 }),
                ));
                out.push(Element::button(
                    "gfx.screens/help_button_backtomenu",
                    [206., 541., 383., 586.],
                    10,
                    "gfx.screens/help_button_backtomenu_up",
                    Action::Menu,
                ));
                if !credits && app.help_page > 1 {
                    out.push(Element::button(
                        "gfx.screens/help_arrow",
                        [21., 523., 120., 584.],
                        12,
                        "gfx.screens/help_arrow_up",
                        Action::Help(app.help_page - 1),
                    ));
                }
                if !credits && app.help_page < 5 {
                    let mut e = Element::button(
                        "gfx.screens/help_arrow",
                        [673., 524., 773., 584.],
                        11,
                        "gfx.screens/help_arrow_up",
                        Action::Help(app.help_page + 1),
                    );
                    e.flip = true;
                    out.push(e);
                }
            }
            Screen::Comic => {
                out.push(Element::art(
                    "gfx.screens/help_frame",
                    [0., 0., 800., 600.],
                    1,
                ));
                out.push(Element::art(
                    &app.comic_pages[app.comic_index],
                    [80., 60., 720., 540.],
                    3,
                ));
                if app.comic_index > 0 {
                    out.push(Element::button(
                        "gfx.screens/help_arrow_up",
                        [28., 537., 127., 598.],
                        5,
                        "gfx.screens/help_arrow",
                        Action::ComicPrevious,
                    ));
                }
                let mut e = Element::button(
                    "gfx.screens/help_arrow_up",
                    [675., 537., 774., 598.],
                    6,
                    "gfx.screens/help_arrow",
                    Action::ComicNext,
                );
                e.flip = true;
                out.push(e);
            }
            Screen::Prelevel => {
                out.push(Element::art("@dim", [0., 0., 800., 600.], 0));
                out.push(Element::art(
                    "gfx.screens/endshop_bg",
                    [65., 26., 700., 567.],
                    1,
                ));
                let (doodle, rect, text_rect) = match (app.game.stage, app.game.level) {
                    (1, 1) => (
                        "younggirl_63",
                        [582., 156., 659., 383.],
                        [200., 113., 578., 441.],
                    ),
                    (1, 6) => (
                        "foodcritic_65",
                        [578., 146., 671., 386.],
                        [200., 113., 574., 441.],
                    ),
                    (1, 7) => (
                        "oldman_66",
                        [589., 123., 671., 352.],
                        [200., 113., 584., 441.],
                    ),
                    (2, 6) => (
                        "businesswoman_67",
                        [587., 123., 671., 387.],
                        [200., 113., 582., 441.],
                    ),
                    _ => ("", [0.; 4], [200., 113., 664., 441.]),
                };
                if !doodle.is_empty() {
                    out.push(Element::art(&format!("doodles/{doodle}"), rect, 2));
                }
                let text = match (app.game.stage, app.game.level) {
                    (1, 1) => "Flo is opening her first diner! Help her seat guests, take orders, serve hot dishes, collect checks, and clean dishes before customers lose patience.",
                    (1, 6) => "The Food Critic is in town! Critics are very demanding but leave huge tips when satisfied. Give them top priority!",
                    (1, 7) => "Grandpa is visiting! Senior citizens take longer to eat and tip generously if treated with respect.",
                    (2, 6) => "Businesswomen are busy! They want quick service with zero waiting, so keep Flo moving fast.",
                    _ => "Get ready for a busy dinner rush! Work cleanly, match seat colors for big multipliers, and keep every table happy.",
                };
                out.push(Element::text(
                    text,
                    text_rect,
                    3,
                    20,
                    Color::from_rgba(25, 38, 53, 255),
                ));
                out.push(Element::button(
                    "gfx.screens/endshop_play",
                    [493., 478., 658., 536.],
                    4,
                    "gfx.screens/endshop_play_on",
                    Action::Play,
                ));
            }
            Screen::Playing => {
                out.push(Element::button(
                    "gfx.screens/pauseButton_off",
                    [742., 561., 791., 589.],
                    250,
                    "gfx.screens/pauseButton_on",
                    Action::Pause,
                ));
                if app.game.show_upgrade_button {
                    out.push(Element::button(
                        "@button:Upgrades",
                        [540., 562., 665., 590.],
                        251,
                        "",
                        Action::Upgrades,
                    ));
                }
            }
            Screen::Result => {
                if let Some(r) = &app.game.level_complete {
                    out.push(Element::art("@dim", [0., 0., 800., 600.], 5));
                    out.push(Element::art(
                        "gfx.screens/endGame_window",
                        [259., 39., 737., 564.],
                        6,
                    ));
                    out.push(Element::art(
                        if r.success {
                            "gfx.screens/flo_win"
                        } else {
                            "gfx.screens/flo_lose"
                        },
                        if r.success {
                            [1., 14., 345., 581.]
                        } else {
                            [113., 19., 327., 589.]
                        },
                        15,
                    ));
                    if r.expert {
                        out.push(Element::art(
                            "gfx.screens/expert_star",
                            [469., 16., 616., 181.],
                            35,
                        ));
                    }
                    if r.success {
                        if let Some(doodle) = &r.doodle {
                            out.push(Element::art("doodles/noise", [547., 292., 714., 503.], 32));
                            out.push(Element::art(
                                &format!("doodles/{doodle}"),
                                [560., 334., 701., 494.],
                                33,
                            ));
                        }
                    }
                    let ink = Color::from_rgba(21, 39, 53, 255);
                    out.push(Element::text(
                        &r.text,
                        [289., 230., 532., 370.],
                        20,
                        15,
                        ink,
                    ));
                    if app.game.mode == GameMode::Career {
                        out.push(Element::text(
                            &r.customers_served.to_string(),
                            [288., 195., 357., 222.],
                            21,
                            16,
                            ink,
                        ));
                        out.push(Element::text(
                            &r.customers_left.to_string(),
                            [373., 195., 442., 222.],
                            22,
                            16,
                            ink,
                        ));
                        out.push(Element::text(
                            &format!("{}-{}", r.stage, r.level),
                            [461., 192., 524., 219.],
                            23,
                            16,
                            ink,
                        ));
                    }
                    out.push(Element::text(
                        &r.score.to_string(),
                        [434., 443., 526., 464.],
                        25,
                        18,
                        ink,
                    ));
                    out.push(Element::text(
                        &app.game.total_score.max(r.score).to_string(),
                        [392., 487., 528., 508.],
                        27,
                        18,
                        ink,
                    ));
                    let key = if r.end_game {
                        "quit"
                    } else if r.success {
                        "continue"
                    } else {
                        "try_again"
                    };
                    out.push(Element::button(
                        &format!("gfx.screens/{key}_normal"),
                        [299., 384., 500., 441.],
                        40,
                        &format!("gfx.screens/{key}_rollover"),
                        Action::NextShift,
                    ));
                }
            }
            _ => self.panel(app, &mut out),
        }
        out.sort_by_key(|e| e.z);
        out
    }
    fn panel(&self, app: &App, out: &mut Vec<Element>) {
        out.push(Element::art("@panel-dim", [0., 0., 800., 600.], 300));
        out.push(Element::art("@panel", [120., 110., 680., 510.], 301));
        let title = match app.screen {
            Screen::Paused => "Shift paused",
            Screen::ResetConfirm => "Reset progress?",
            _ => "Choose an upgrade",
        };
        out.push(Element::text(
            title,
            [145., 140., 655., 190.],
            302,
            25,
            Color::from_rgba(246, 196, 83, 255),
        ));
        let mut buttons = Vec::new();
        let message = match app.screen {
            Screen::Paused => {
                buttons.push(("Resume".into(), Action::Resume));
                buttons.push(("Options".into(), Action::Settings));
                buttons.push(("Main menu".into(), Action::Menu));
                "Take a breather. Your shift is saved.".into()
            }
            Screen::ResetConfirm => {
                buttons.push(("Cancel".into(), Action::Settings));
                buttons.push(("Reset".into(), Action::Reset));
                "This removes your saved career and current shift.".into()
            }
            _ => {
                for (key, value) in &app.game.upgrades.powerups {
                    if !value.is_empty() {
                        buttons.push((key.clone(), Action::Upgrade(key.clone())));
                    }
                }
                if !app.game.upgrades.tables.is_empty() {
                    buttons.push(("table".into(), Action::Upgrade("table".into())));
                }
                buttons.sort_by(|a, b| a.0.cmp(&b.0));
                buttons.push(("Back".into(), Action::Resume));
                String::new()
            }
        };
        out.push(Element::text(
            &message,
            [140., 195., 660., 285.],
            302,
            17,
            Color::from_rgba(255, 241, 207, 255),
        ));
        for (i, (label, action)) in buttons.into_iter().enumerate() {
            let x = 145. + (i % 3) as f32 * 175.;
            let y = 300. + (i / 3) as f32 * 55.;
            out.push(Element::button(
                &format!("@button:{label}"),
                [x, y, x + 160., y + 44.],
                303,
                "",
                action,
            ));
        }
    }
    pub fn draw(
        &mut self,
        app: &App,
        renderer: &mut NativeRenderer,
        pointer: Vec2,
        click: bool,
        time: f64,
    ) -> Option<Action> {
        let items = self.elements(app, time);
        let hovered = items
            .iter()
            .rev()
            .find(|e| e.enabled && e.action.is_some() && rect(e.rect).contains(pointer));
        let current = hovered.and_then(|e| e.action.clone());
        self.hover_changed = current.is_some() && current != self.hovered;
        self.hovered = current.clone();
        for e in items {
            let r = rect(e.rect);
            let hot = e.enabled && e.action.is_some() && e.action == current;
            if let Some((text, size, color)) = &e.text {
                text_box(text, r, *size, *color);
                continue;
            }
            match e.key.as_str() {
                "@dim" => draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0., 0., 0., 0.5)),
                "@panel-dim" => {
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(7, 25, 39, 187))
                }
                "@panel" => {
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::from_rgba(24, 48, 63, 238));
                    draw_rectangle_lines(
                        r.x,
                        r.y,
                        r.w,
                        r.h,
                        2.,
                        Color::from_rgba(246, 196, 83, 255),
                    );
                }
                key if key.starts_with("@button:") => {
                    draw_rectangle(
                        r.x,
                        r.y,
                        r.w,
                        r.h,
                        if hot {
                            Color::from_rgba(62, 89, 101, 255)
                        } else {
                            Color::from_rgba(38, 61, 73, 255)
                        },
                    );
                    draw_rectangle_lines(
                        r.x,
                        r.y,
                        r.w,
                        r.h,
                        1.,
                        Color::from_rgba(105, 125, 133, 255),
                    );
                    text_box(&key[8..], r, 16, Color::from_rgba(255, 241, 207, 255));
                }
                _ => {
                    let key = if hot && self.manifest.contains_key(&e.hover) {
                        &e.hover
                    } else {
                        &e.key
                    };
                    let path = self
                        .manifest
                        .get(key)
                        .map(|a| a.path.as_str())
                        .unwrap_or(key);
                    let tint = if !e.enabled {
                        Color::new(1., 1., 1., 0.4)
                    } else if hot {
                        Color::new(1.15, 1.15, 1.15, 1.)
                    } else {
                        WHITE
                    };
                    renderer.image(path, r, e.flip, tint);
                }
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            show_mouse(false);
            let key = if is_mouse_button_down(MouseButton::Left) {
                "gfx.screens/cursor_click_1"
            } else {
                "gfx.screens/cursor_arrow"
            };
            if let Some(a) = self.manifest.get(key) {
                renderer.image(
                    &a.path,
                    Rect::new(pointer.x, pointer.y, a.width, a.height),
                    false,
                    WHITE,
                );
            }
        }
        if click {
            current
        } else {
            None
        }
    }
    pub fn blocks_game(&self, app: &App, pointer: Vec2) -> bool {
        self.elements(app, 0.)
            .iter()
            .any(|e| e.action.is_some() && rect(e.rect).contains(pointer))
    }
}

fn rect(r: [f32; 4]) -> Rect {
    Rect::new(r[0], r[1], r[2] - r[0], r[3] - r[1])
}

fn text_box(text: &str, r: Rect, size: u16, color: Color) {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.into()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty() && measure_text(&candidate, None, size, 1.).width > r.w - 16. {
                lines.push(line);
                line = word.into();
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    let spacing = size as f32 * 1.42;
    let start = r.y + (r.h - lines.len() as f32 * spacing) / 2. + size as f32;
    for (i, line) in lines.iter().enumerate() {
        let x = r.x + (r.w - measure_text(line, None, size, 1.).width) / 2.;
        draw_text(line, x, start + i as f32 * spacing, size as f32, color);
    }
}
