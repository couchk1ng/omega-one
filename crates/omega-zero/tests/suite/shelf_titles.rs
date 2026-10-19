//! Y on the shelf puts the selected game's name in a pill above it, and takes it away again.

use crate::common;

use omega_gfx::out_w;
use omega_input::{Action, Btn};
use omega_store::{write_app_state, AppState, Cart, Platform};
use omega_ui::kit::action::BAR;
use omega_ui::{rest_y, shelf_box, stage_y, sway_reach, Draw, TexId, SPRITE_PX, TOP_BAR_H};
use omega_zero::app::{App, Phase};
use omega_zero::screens::controls::ControlsView;

const FACE: usize = 9100;
const FACE_W: u32 = 80;

fn app_with(platform: Platform, stems: &[&str]) -> App {
    App::new(
        stems
            .iter()
            .map(|stem| Cart {
                platform,
                stem: (*stem).to_string(),
                rom: format!("Games/{}/{stem}.md", platform.dir_name()).into(),
                label: None,
                code: String::new(),
                shell: None,
                title: stem.to_uppercase(),
            })
            .collect(),
    )
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn settle(a: &mut App, seconds: f32) {
    for _ in 0..(seconds * 60.0) as usize {
        a.update(1.0 / 60.0);
    }
}

/// What the app draws, with the caption's face given to it as the front end would.
fn drawn(a: &mut App) -> Vec<Draw> {
    a.set_shelf_title_face(TexId::from_raw(FACE), FACE_W);
    let mut out = Vec::new();
    a.draw(&mut out);
    out
}

fn caption(out: &[Draw]) -> Option<(f32, f32, f32, f32, f32)> {
    out.iter().find_map(|d| match *d {
        Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha,
        } if tex == TexId::from_raw(FACE) => Some((x, y, w, h, alpha)),
        _ => None,
    })
}

#[test]
fn titles_are_off_until_y_is_pressed() {
    let a = app_with(Platform::Genesis, &["Streets of Rage 2 (USA)"]);
    assert!(!a.titles_on());
    assert_eq!(a.title_text(), None);
}

#[test]
fn y_turns_the_titles_on_and_another_y_turns_them_off() {
    let mut a = app_with(Platform::Genesis, &["Streets of Rage 2 (USA)"]);
    press(&mut a, Btn::Y);
    assert!(a.titles_on());
    assert_eq!(a.title_text().as_deref(), Some("Streets of Rage 2"));
    press(&mut a, Btn::Y);
    assert!(!a.titles_on());
    assert_eq!(a.title_text(), None);
}

#[test]
fn the_caption_follows_the_selection_as_the_shelf_is_browsed() {
    let mut a = app_with(
        Platform::MegaDrive,
        &[
            "Aladdin (Europe)",
            "Legend of Zelda, The (USA)",
            "Super Hang-On (World) (En,Ja) (Rev A)",
        ],
    );
    press(&mut a, Btn::Y);
    assert_eq!(a.title_text().as_deref(), Some("Aladdin"));
    for want in ["The Legend of Zelda", "Super Hang-On"] {
        press(&mut a, Btn::Right);
        settle(&mut a, 1.0);
        assert_eq!(a.title_text().as_deref(), Some(want));
    }
}

#[test]
fn y_works_on_every_system_and_nothing_else_changes_on_the_shelf() {
    for platform in Platform::ALL {
        let mut a = app_with(platform, &["Game One (USA)", "Game Two (Europe)"]);
        press(&mut a, Btn::Y);
        assert!(a.titles_on(), "{platform:?}");
        assert!(
            matches!(a.phase(), Phase::Shelf),
            "{platform:?} left the shelf: {:?}",
            a.phase()
        );
        assert_eq!(
            a.selected_stem(),
            Some("Game One (USA)"),
            "{platform:?} moved the selection"
        );
    }
}

/// An app booted on a card whose clock was set, so it comes up on the shelf.
fn booted(stems: &[&str]) -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(stems);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let app = App::boot(d.path());
    (d, app)
}

#[test]
fn y_is_left_alone_while_the_core_picker_is_open_and_works_again_once_it_is_closed() {
    // GBA has two cores, so X opens the picker; Y must not change the titles under it
    let (_d, mut a) = booted(&["Emerald", "Zzz"]);
    assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
    press(&mut a, Btn::X);
    assert!(a.core_picker().is_some(), "X did not open the picker");
    press(&mut a, Btn::Y);
    assert!(
        !a.titles_on(),
        "Y turned the titles on with the core picker open"
    );
    assert!(a.core_picker().is_some(), "Y closed the picker");
    a.apply(Action::GbaDown(Btn::B));
    a.update(0.4);
    assert!(a.core_picker().is_none());
    press(&mut a, Btn::Y);
    assert!(a.titles_on(), "Y did nothing once the picker was closed");
}

#[test]
fn the_controls_panel_lists_y_as_title_on_every_shelf() {
    for platform in Platform::ALL {
        let view = ControlsView::new(platform);
        let shelf = &view.groups[0];
        assert_eq!(shelf.name, "SHELF");
        assert!(
            shelf.lines.contains(&("Y", "TITLE")),
            "{platform:?}: {:?}",
            shelf.lines
        );
    }
}

#[test]
fn with_titles_off_no_caption_is_drawn_even_with_a_face_baked() {
    let mut a = app_with(Platform::Genesis, &["Streets of Rage 2 (USA)"]);
    assert!(caption(&drawn(&mut a)).is_none());
}

#[test]
fn the_caption_is_centred_below_the_top_bar_and_clear_of_the_cart() {
    for platform in [Platform::Genesis, Platform::MegaDrive, Platform::Snes] {
        let mut a = app_with(platform, &["Streets of Rage 2 (USA)"]);
        press(&mut a, Btn::Y);
        settle(&mut a, 0.5);
        let (x, y, w, h, alpha) =
            caption(&drawn(&mut a)).unwrap_or_else(|| panic!("{platform:?}: no caption"));
        assert!(
            (x + w / 2.0 - out_w() as f32 / 2.0).abs() <= SPRITE_PX as f32,
            "{platform:?}: not centred: {x}+{w}/2"
        );
        assert_eq!(h, (BAR * SPRITE_PX) as f32, "{platform:?}");
        assert_eq!(w, (FACE_W * SPRITE_PX) as f32, "{platform:?}");
        assert!(
            y >= stage_y() + TOP_BAR_H,
            "{platform:?}: the caption is under the top bar: {y}"
        );
        let (cw, ch) = shelf_box(platform);
        let cart_top = rest_y(ch) - sway_reach(cw, ch);
        assert!(
            y + h <= cart_top,
            "{platform:?}: the caption reaches into the cart: {} against {cart_top}",
            y + h
        );
        assert_eq!(alpha, 1.0, "{platform:?}: not fully in after half a second");
    }
}

#[test]
fn the_caption_fades_in_when_y_turns_the_titles_on() {
    let mut a = app_with(Platform::Genesis, &["Streets of Rage 2 (USA)"]);
    press(&mut a, Btn::Y);
    let first = caption(&drawn(&mut a)).unwrap().4;
    settle(&mut a, 0.08);
    let part = caption(&drawn(&mut a)).unwrap().4;
    settle(&mut a, 0.5);
    let whole = caption(&drawn(&mut a)).unwrap().4;
    assert!(first < part && part < whole, "{first} {part} {whole}");
    assert_eq!(whole, 1.0);
    assert!(first < 0.2, "it starts nearly clear: {first}");
}

#[test]
fn the_caption_is_not_drawn_while_a_cart_goes_in() {
    let mut a = app_with(Platform::Genesis, &["Streets of Rage 2 (USA)"]);
    press(&mut a, Btn::Y);
    settle(&mut a, 0.5);
    assert!(caption(&drawn(&mut a)).is_some());
    a.apply(Action::GbaDown(Btn::A));
    a.apply(Action::GbaUp(Btn::A));
    settle(&mut a, 0.2);
    assert!(
        matches!(a.phase(), Phase::Inserting { .. }),
        "{:?}",
        a.phase()
    );
    assert!(
        caption(&drawn(&mut a)).is_none(),
        "the caption stayed up as the cart went in"
    );
}

#[test]
fn a_long_name_still_gets_a_caption_the_front_end_will_shorten() {
    let mut a = app_with(
        Platform::Genesis,
        &["Disney's Aladdin Special Collector's Edition Director's Cut Remastered (USA)"],
    );
    press(&mut a, Btn::Y);
    assert_eq!(
        a.title_text().as_deref(),
        Some("Disney's Aladdin Special Collector's Edition Director's Cut Remastered")
    );
}

#[test]
fn titles_are_remembered_across_a_reboot_and_off_is_remembered_too() {
    let (d, mut a) = booted(&["Emerald", "Zzz"]);
    assert!(!a.titles_on(), "titles on a card that was never asked");
    press(&mut a, Btn::Y);
    assert!(a.titles_on());
    drop(a);

    let mut a = App::boot(d.path());
    assert!(a.titles_on(), "titles did not come back after a reboot");
    assert_eq!(a.title_text().as_deref(), Some("Emerald"));
    press(&mut a, Btn::Y);
    assert!(!a.titles_on());
    drop(a);

    let a = App::boot(d.path());
    assert!(!a.titles_on(), "turning them off was not remembered");
}
