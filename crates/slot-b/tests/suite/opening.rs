//! The opening: the boot sequence, played over the shelf by a boot that lands on it. The logo
//! fades to black, the grid, the plate and the B's click play for three seconds, the whole
//! thing fades out, and the shelf fades in. A press cuts it short and goes no further.

use crate::common;

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use slot_b::app::{App, Phase};
use slot_b::audio::Sfx;
use slot_b::boot_seq::{SEAT, SLIDE, STAMP};
use slot_b::opening::{
    Facts, OpeningArt, CUT_OUT_MS, CUT_REVEAL_MS, LEAD_MS, OUT_MS, REVEAL_MS, SCRIPT_MS,
};
use slot_b_gfx::{out_h, out_w};
use slot_b_input::{Action, Btn};
use slot_b_power::parse_baseos_release;
use slot_b_store::{write_app_state, AppState};
use slot_b_ui::{Draw, TexId};
use tempfile::TempDir;

/// A frame at 60 fps, in seconds.
const FRAME: f32 = 1.0 / 60.0;

const MEMINFO: &str = "MemTotal:        1003916 kB\nMemFree:          812340 kB\nMemAvailable:     893120 kB\nBuffers:            1024 kB\n";

fn booted_on_the_shelf() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let state = AppState {
        clock_set: true,
        ..AppState::default()
    };
    write_app_state(d.path(), &state).unwrap();
    let (a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    assert!(matches!(a.phase(), Phase::Shelf));
    (d, a)
}

fn opened() -> (TempDir, App) {
    let (d, mut a) = booted_on_the_shelf();
    a.start_opening(&Facts::default());
    assert!(a.opening_shown());
    (d, a)
}

const FRAME_MS: f32 = FRAME * 1000.0;

/// Runs `a` on `ms`, a frame at a time, gathering the sounds.
fn run_for(a: &mut App, ms: f32) -> Vec<Sfx> {
    let mut heard = Vec::new();
    for _ in 0..(ms / FRAME_MS).round() as u32 {
        a.update(FRAME);
        heard.extend(a.take_sfx());
    }
    heard
}

/// Runs an opening not yet drawn on to `ms` from its first frame.
fn run_to(a: &mut App, ms: f32) -> Vec<Sfx> {
    run_for(a, ms + FRAME_MS)
}

#[test]
fn the_facts_are_the_handhelds_own() {
    let release = parse_baseos_release("BASEOS_TARGET=rg35xxsp\nBASEOS_MODEL=\"RG35XX SP\"\n");
    let f = Facts::from_parts(Some(release), Some(MEMINFO));
    assert_eq!(
        f,
        Facts {
            model: "RG35XX SP".into(),
            mem_mb: 1024,
            avail_mb: 872,
            battery_mah: Some(3300),
        },
        "the kernel's 980 MB are the 1 GB fitted"
    );
}

#[test]
fn a_battery_nobody_checked_goes_untold_and_memory_rounds_up_to_its_chips() {
    let release = parse_baseos_release("BASEOS_TARGET=rg40xxh\n");
    let f = Facts::from_parts(
        Some(release),
        Some("MemTotal: 2012345 kB\nMemAvailable: 1700000 kB\n"),
    );
    assert_eq!(
        f.model, "RG40XXH",
        "the target names it when no model is given"
    );
    assert_eq!(f.battery_mah, None);
    assert_eq!((f.mem_mb, f.avail_mb), (2048, 1660));
}

#[test]
fn a_computer_standing_in_tells_the_rg35xx_sps_facts() {
    assert_eq!(Facts::from_parts(None, None), Facts::default());
    let f = Facts::default();
    assert_eq!(f.model, "RG35XX SP");
    assert_eq!((f.mem_mb, f.battery_mah), (1024, Some(3300)));
}

/// The art, as the frontend would have uploaded it: a texture id for each piece.
fn art() -> OpeningArt {
    let tex = TexId::from_raw;
    OpeningArt {
        grid: tex(1),
        left: tex(2),
        letters: tex(3),
        piece: tex(4),
        seam: tex(5),
        shadow: tex(6),
        glyph: tex(7),
        glint: tex(8),
    }
}

/// What `a` draws, with the art handed to it.
fn drawn(a: &mut App) -> Vec<Draw> {
    a.set_opening_art(art());
    let mut out = Vec::new();
    a.draw(&mut out);
    out
}

/// The texture of each of the sequence's art pieces in `out`, in the order they are drawn.
fn textures(out: &[Draw]) -> Vec<u32> {
    let known = art();
    out.iter()
        .filter_map(|d| match d {
            Draw::Spun { tex, .. } => [
                known.grid,
                known.left,
                known.letters,
                known.piece,
                known.seam,
                known.shadow,
                known.glyph,
                known.glint,
            ]
            .iter()
            .position(|t| t == tex)
            .map(|i| i as u32 + 1),
            _ => None,
        })
        .collect()
}

#[test]
fn the_logo_fades_to_black_before_the_sequence_starts() {
    let (_d, mut a) = opened();
    a.update(FRAME);
    assert_eq!(
        a.opening_levels(),
        Some((1.0, 0.0)),
        "the first frame is black"
    );
    run_for(&mut a, LEAD_MS - 20.0);
    assert_eq!(
        a.opening_levels(),
        Some((1.0, 0.0)),
        "the logo's fade is over black"
    );
    run_for(&mut a, 20.0 + 100.0);
    assert_eq!(
        a.opening_levels(),
        Some((1.0, 1.0)),
        "the sequence is lit from its first moment"
    );
}

#[test]
fn the_four_sounds_come_in_order_at_their_beats() {
    let (_d, mut a) = opened();
    let mut heard = Vec::new();
    for n in 0..(6_000.0 / FRAME_MS) as u32 {
        a.update(FRAME);
        let at = n as f32 * FRAME_MS - LEAD_MS;
        heard.extend(a.take_sfx().map(|s| (s, at)));
    }
    let want = [
        (Sfx::BootRise, 250.0),
        (Sfx::BootStamp, STAMP[0]),
        (Sfx::BootSlide, SLIDE[0]),
        (Sfx::BootSeat, SEAT),
    ];
    assert_eq!(heard.len(), 4, "{heard:?}");
    for ((sfx, at), (want_sfx, want_at)) in heard.iter().zip(want) {
        assert_eq!(*sfx, want_sfx);
        assert!(
            (at - want_at).abs() <= FRAME_MS * 2.0,
            "{sfx:?} sounded at {at}, not {want_at}"
        );
    }
}

#[test]
fn it_fades_out_to_black_then_the_shelf_fades_in_and_it_is_over() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + SCRIPT_MS + OUT_MS / 2.0);
    let (veil, ink) = a.opening_levels().unwrap();
    assert_eq!(veil, 1.0, "the shelf shows before the sequence is gone");
    assert!(ink > 0.0 && ink < 0.6, "the sequence fades out: {ink}");
    run_for(&mut a, OUT_MS / 2.0 + REVEAL_MS / 2.0);
    let (veil, ink) = a.opening_levels().unwrap();
    assert_eq!(ink, 0.0);
    assert!(veil > 0.0 && veil < 0.6, "the black lifts: {veil}");
    run_for(&mut a, REVEAL_MS / 2.0 + 50.0);
    assert!(!a.opening_shown(), "still playing after it has ended");
    assert!(matches!(a.phase(), Phase::Shelf));
}

#[test]
fn a_press_cuts_it_short_and_goes_no_further() {
    let (_d, mut a) = opened();
    // before the B's piece slides in or clicks: a cut must keep both sounds from ever coming
    run_to(&mut a, LEAD_MS + 1_500.0);
    let before = a.selected_stem().map(str::to_string);
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::ShelfRight);
    a.apply(Action::Insert);
    a.apply(Action::GbaUp(Btn::Right));
    assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
    assert_eq!(a.selected_stem().map(str::to_string), before);
    // the slide, due 150 ms after the cut, must not be heard
    let heard = run_for(&mut a, CUT_OUT_MS + 20.0);
    assert!(
        heard.is_empty(),
        "the sequence played on after the cut: {heard:?}"
    );
    assert_eq!(
        a.opening_levels().map(|(_, ink)| ink),
        Some(0.0),
        "the sequence is still lit"
    );
    run_for(&mut a, CUT_REVEAL_MS + 40.0);
    assert!(
        !a.opening_shown(),
        "the shelf is not in half a second after the press"
    );
    assert!(matches!(a.phase(), Phase::Shelf));
    assert_eq!(a.selected_stem().map(str::to_string), before);
}

#[test]
fn the_art_is_asked_for_once_the_opening_starts_and_not_after_it_has_it() {
    let (_d, mut a) = booted_on_the_shelf();
    assert!(!a.opening_wants_art(), "no opening, no art");
    a.start_opening(&Facts::default());
    assert!(a.opening_wants_art());
    a.set_opening_art(art());
    assert!(!a.opening_wants_art());
}

#[test]
fn the_black_comes_first_and_nothing_else_is_drawn_until_the_sequence_starts() {
    let (_d, mut a) = opened();
    run_to(&mut a, 200.0);
    let out = drawn(&mut a);
    let (w, h) = (out_w() as f32, out_h() as f32);
    assert_eq!(
        out,
        [Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: [0.0, 0.0, 0.0, 1.0]
        }],
        "the logo's fade is over black"
    );
}

#[test]
fn the_finished_plate_is_six_pieces_of_art_over_the_black_and_nothing_under_it_is_drawn() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + 2_800.0);
    let out = drawn(&mut a);
    let (w, h) = (out_w() as f32, out_h() as f32);
    assert_eq!(
        out[0],
        Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: [0.0, 0.0, 0.0, 1.0]
        }
    );
    // the grid, SLOT's piece, SLOT, the B's shadow, the B's piece and the B: no seam, no
    // trail, no flash and no glint by now
    assert_eq!(textures(&out), [1, 2, 3, 6, 4, 7], "{out:?}");
    assert_eq!(out.len(), 7);
    let Draw::Spun {
        x,
        y,
        w: tw,
        h: th,
        colour,
        turn,
        yaw,
        clip,
        ..
    } = out[2]
    else {
        panic!("SLOT's piece is not a spun draw: {:?}", out[2]);
    };
    let (ox, oy) = ((out_w() as i32 - 640) / 2, (out_h() as i32 - 480) / 2);
    assert_eq!(
        (x, y),
        (
            (ox + 2 * slot_b_ui::boot::LEFT[0]) as f32,
            (oy + 2 * slot_b_ui::boot::LEFT[1]) as f32
        )
    );
    assert_eq!(
        (tw, th),
        (
            2.0 * slot_b_ui::boot::LEFT[2] as f32,
            2.0 * slot_b_ui::boot::LEFT[3] as f32
        )
    );
    assert_eq!(
        (colour, turn, yaw, clip),
        ([1.0, 1.0, 1.0, 1.0], 0.0, 0.0, None),
        "plain, whole and full bright"
    );
}

#[test]
fn fading_out_dims_everything_by_the_same_amount() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + SCRIPT_MS + OUT_MS / 2.0);
    let (_, ink) = a.opening_levels().unwrap();
    assert!(ink > 0.2 && ink < 0.8, "{ink}");
    let out = drawn(&mut a);
    let mut dimmed = 0;
    for d in &out[1..] {
        let (colour, is_grid) = match d {
            Draw::Spun { colour, tex, .. } => (*colour, *tex == art().grid),
            Draw::Rect { colour, .. } => (*colour, false),
            other => panic!("something else is drawn: {other:?}"),
        };
        if is_grid {
            // tinted bluish, then dimmed: no brighter than the dimming allows
            assert!(colour[..3].iter().all(|c| *c <= ink), "{d:?}");
        } else {
            assert_eq!(&colour[..3], &[ink, ink, ink], "{d:?}");
        }
        dimmed += 1;
    }
    assert!(dimmed >= 6, "the plate was not there to dim: {out:?}");
}

#[test]
fn while_the_pieces_are_apart_their_seam_shows_and_a_trail_follows_the_b() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + 1_850.0);
    let out = drawn(&mut a);
    let arts = textures(&out);
    assert!(
        arts.contains(&5),
        "the seam is not drawn while they are apart: {arts:?}"
    );
    assert_eq!(
        arts.iter().filter(|&&t| t == 4).count(),
        3,
        "the B's piece and its two ghosts: {arts:?}"
    );
    let ghosts: Vec<f32> = out
        .iter()
        .filter_map(|d| match d {
            Draw::Spun { tex, colour, .. } if *tex == art().piece && colour[3] < 1.0 => {
                Some(colour[3])
            }
            _ => None,
        })
        .collect();
    assert_eq!(ghosts, [0.07, 0.12]);
}

#[test]
fn the_click_lights_the_joint_under_the_b_and_the_seam_is_gone_once_they_have_met() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + SEAT + 40.0);
    let out = drawn(&mut a);
    let glow = out
        .iter()
        .filter(|d| matches!(d, Draw::Rect { colour, .. } if colour[3] < 1.0))
        .count();
    assert_eq!(glow, 3, "a soft glow of three rects: {out:?}");
    let arts = textures(&out);
    assert!(
        !arts.contains(&5),
        "the seam shows with the pieces together: {arts:?}"
    );
    assert_eq!(arts.last(), Some(&7), "the B is not on top of the glow");
}

#[test]
fn the_glint_is_kept_inside_the_plates_cut_corners_by_stairs() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + 2_300.0);
    let out = drawn(&mut a);
    let glint: Vec<[f32; 4]> = out
        .iter()
        .filter_map(|d| match d {
            Draw::Spun {
                tex, clip: Some(c), ..
            } if *tex == art().glint => Some(*c),
            _ => None,
        })
        .collect();
    assert_eq!(glint.len(), 7);
    use slot_b_ui::boot::{PC, PH, PW, PX};
    let (ox, oy) = ((out_w() as i32 - 640) / 2, (out_h() as i32 - 480) / 2);
    // the middle stair is the whole plate; those above and below it narrow towards its corners
    assert_eq!(glint[3][0], (ox + 2 * PX) as f32);
    assert_eq!(glint[3][2], (2 * PW) as f32);
    assert_eq!(glint[0][0], (ox + 2 * (PX + PC)) as f32);
    assert_eq!(glint[0][2], (2 * (PW - 2 * PC)) as f32);
    assert!(glint[0][2] < glint[1][2] && glint[1][2] < glint[2][2] && glint[2][2] < glint[3][2]);
    assert_eq!(
        (glint[0][1], glint[6][1] + glint[6][3]),
        (
            (oy + 2 * slot_b_ui::boot::PY) as f32,
            (oy + 2 * (slot_b_ui::boot::PY + PH)) as f32
        )
    );
}

#[test]
fn a_slow_frame_slows_the_opening_rather_than_skipping_part_of_it() {
    let (_d, mut a) = opened();
    a.update(FRAME);
    a.update(3.0);
    a.update(3.0);
    let (veil, ink) = a.opening_levels().unwrap();
    assert_eq!(
        (veil, ink),
        (1.0, 0.0),
        "two slow frames ran through the logo's fade"
    );
}

#[test]
fn a_press_with_the_shelf_showing_through_ends_it_and_is_the_shelfs() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + SCRIPT_MS + OUT_MS + 100.0);
    assert!(a.opening_shown());
    a.apply(Action::GbaDown(Btn::Start));
    assert!(!a.opening_shown());
    assert!(
        matches!(a.phase(), Phase::Settings { .. }),
        "START did not open Settings: {:?}",
        a.phase()
    );
}

#[test]
fn power_and_the_lid_end_it_at_once_and_brightness_works_through_it() {
    let (_d, mut a) = opened();
    run_to(&mut a, LEAD_MS + 2_000.0);
    a.apply(Action::BrightnessUp);
    a.apply(Action::VolumeDown);
    a.apply(Action::LidOpen);
    assert_eq!(
        a.opening_levels(),
        Some((1.0, 1.0)),
        "a level key cut it short"
    );
    a.apply(Action::PowerPress);
    assert!(!a.opening_shown(), "POWER waited for the opening");
}

#[test]
fn the_logo_fades_over_the_black_on_top_of_it() {
    let (_d, mut a) = booted_on_the_shelf();
    let logo = TexId::from_raw(900);
    a.set_boot_logo(logo, out_w(), out_h());
    a.start_opening(&Facts::default());
    a.update(FRAME);
    let mut out = Vec::new();
    a.draw(&mut out);
    assert!(matches!(
        out.first(),
        Some(Draw::Rect {
            colour: [0.0, 0.0, 0.0, 1.0],
            ..
        })
    ));
    assert!(
        matches!(out.last(), Some(Draw::Tex { tex, alpha, .. }) if *tex == logo && *alpha == 1.0),
        "the logo is not whole on top: {out:?}"
    );
}

#[test]
fn only_a_boot_onto_the_shelf_plays_it() {
    let d = tmp_root_with_carts(&["Emerald"]);
    write_app_state(d.path(), &AppState::default()).unwrap();
    let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    assert!(matches!(a.phase(), Phase::SetClock { .. }));
    a.start_opening(&Facts::default());
    assert!(!a.opening_shown(), "the clock's first boot played it");
    let d = tmp_root_with_carts(&["Emerald"]);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            cart: Some("Emerald".into()),
            ..AppState::default()
        },
    )
    .unwrap();
    let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    a.start_opening(&Facts::default());
    assert!(!a.opening_shown(), "a boot back into a game played it");
}
