//! The boot sequence's timeline and what it draws, as functions of time alone: the B's piece
//! slides in, clicks home and carries on in one motion, the grid and the plate wipe in from
//! their middles, SLOT stamps in a letter at a time, and the sounds come on their beats.

use slot_b::audio::Sfx;
use slot_b::boot_seq::{
    frame_origin, glint_at, letters_shown, piece_offset, pulse, scene, sound, Art, Block, SEAT,
    SETTLE, SHINE, SLIDE, STAMP,
};
use slot_b_ui::boot::{GAP, LEFT, PC, PW, PX};

fn arts(blocks: &[Block]) -> Vec<Art> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Art { art, .. } => Some(*art),
            Block::Rect { .. } => None,
        })
        .collect()
}

#[test]
fn the_b_carries_on_at_the_speed_it_arrived() {
    let u = |t: f32| piece_offset(t).expect("the piece is on its way");
    let arriving = u(SEAT - 1.0) - u(SEAT);
    let leaving = u(SEAT) - u(SEAT + 1.0);
    assert!(
        (0.9..1.1).contains(&arriving),
        "{arriving} px/ms into the click"
    );
    assert!(
        (arriving - leaving).abs() < 0.05 * arriving,
        "{arriving} px/ms into the click and {leaving} out of it: not one motion"
    );
}

#[test]
fn the_piece_never_stops_or_turns_back_until_it_rests_where_the_plate_finishes() {
    assert_eq!(
        piece_offset(SLIDE[0] - 1.0),
        None,
        "it is not there before it slides"
    );
    assert_eq!(
        piece_offset(SLIDE[0]),
        Some(GAP as f32 + 125.0),
        "it starts off to the right"
    );
    assert_eq!(
        piece_offset(SEAT),
        Some(GAP as f32),
        "it clicks GAP pixels from the end"
    );
    let mut last = f32::MAX;
    for ms in (SLIDE[0] as i32)..(SETTLE[1] as i32) {
        let u = piece_offset(ms as f32).unwrap();
        assert!(
            u < last,
            "it stood still or turned back at {ms} ms: {u} after {last}"
        );
        last = u;
    }
    for ms in [SETTLE[1], 2_000.0, 3_000.0] {
        assert_eq!(
            piece_offset(ms),
            Some(0.0),
            "{ms} ms: it has not come to rest"
        );
    }
}

#[test]
fn slot_stamps_in_a_letter_at_a_time_and_all_of_it_stays() {
    assert_eq!(letters_shown(STAMP[0] - 1.0), None);
    for ms in (STAMP[0] as i32 + 1)..(STAMP[1] as i32) {
        let n = letters_shown(ms as f32).unwrap();
        // the letters before the newest are settled; the newest flickers on and off
        let progress = (ms as f32 - STAMP[0]) / (STAMP[1] - STAMP[0]);
        let newest = ((progress * 4.0) as i32 + 1).min(4);
        assert!(
            n == newest - 1 || n == newest,
            "{ms} ms: {n} letters, with {newest} the newest"
        );
    }
    for ms in [STAMP[1], 2_000.0, 3_000.0] {
        assert_eq!(letters_shown(ms), Some(4), "{ms} ms");
    }
}

#[test]
fn each_sound_falls_due_once_in_order_at_its_beat() {
    let mut heard = Vec::new();
    let mut before = f32::NEG_INFINITY;
    for frame in 0..=(4_000.0 / (1_000.0 / 60.0)) as u32 {
        let now = frame as f32 * 1_000.0 / 60.0;
        if let Some(s) = sound(before, now) {
            heard.push((s, now));
        }
        before = now;
    }
    let sounds: Vec<Sfx> = heard.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        sounds,
        [Sfx::BootRise, Sfx::BootStamp, Sfx::BootSlide, Sfx::BootSeat]
    );
    assert!(
        (heard[3].1 - SEAT).abs() < 17.0,
        "the click is on the click: {}",
        heard[3].1
    );
}

#[test]
fn the_grid_is_struck_a_little_as_the_plate_lands_and_a_lot_at_the_click() {
    assert_eq!(pulse(0.0), 0.0);
    let landed = pulse(1_260.0);
    assert!((0.4..0.5).contains(&landed), "{landed}");
    let clicked = pulse(SEAT + 1.0);
    assert!(clicked > 0.99, "{clicked}");
    assert!(pulse(3_000.0) < 0.06, "it has mostly faded by the fade out");
}

#[test]
fn nothing_is_drawn_before_it_starts_or_with_no_light() {
    assert!(scene(-10.0, 1.0, 640, 480).is_empty());
    assert!(
        scene(100.0, 1.0, 640, 480).is_empty(),
        "the grid has not started"
    );
    assert!(
        scene(2_000.0, 0.0, 640, 480).is_empty(),
        "no light, nothing to see"
    );
}

#[test]
fn the_grid_wipes_in_from_the_middle_of_the_screen() {
    let blocks = scene(500.0, 1.0, 720, 480);
    let Some(Block::Art {
        art: Art::Grid,
        x,
        y,
        w,
        h,
        clip: Some(c),
        ..
    }) = blocks.first().copied()
    else {
        panic!("the grid is not first, clipped: {blocks:?}");
    };
    assert_eq!(
        (x, y, w, h),
        (0.0, 0.0, 720.0, 480.0),
        "the grid is the whole screen's"
    );
    assert!(
        (c[0] + c[2] / 2.0 - 360.0).abs() < 0.5 && (c[1] + c[3] / 2.0 - 240.0).abs() < 0.5,
        "{c:?}"
    );
    assert!(c[2] < 720.0, "it has not reached the sides yet: {c:?}");
    let whole = scene(1_100.0, 1.0, 720, 480);
    assert!(
        matches!(
            whole.first(),
            Some(Block::Art {
                art: Art::Grid,
                clip: None,
                ..
            })
        ),
        "it is whole once wiped in"
    );
}

#[test]
fn the_plate_wipes_in_from_the_middle_of_slots_piece() {
    let blocks = scene(1_000.0, 1.0, 640, 480);
    let Some(Block::Art { clip: Some(c), .. }) = blocks
        .iter()
        .find(|b| matches!(b, Block::Art { art: Art::Left, .. }))
        .copied()
    else {
        panic!("SLOT's piece is not wiped in: {blocks:?}");
    };
    let middle = 2.0 * (PX + 227) as f32 / 2.0 + 0.0;
    assert!(
        (c[0] + c[2] / 2.0 - middle).abs() < 1.0,
        "{c:?} is not centred on {middle}"
    );
    assert!(c[2] < 2.0 * LEFT[2] as f32, "it is already whole: {c:?}");
}

#[test]
fn the_plate_stands_in_the_middle_of_every_panel() {
    for (w, h) in [(640, 480), (720, 480), (720, 720)] {
        let (ox, oy) = frame_origin(w, h);
        assert_eq!((ox, oy), ((w as i32 - 640) / 2, (h as i32 - 480) / 2));
        let blocks = scene(2_700.0, 1.0, w, h);
        let Some(Block::Art { x, y, .. }) = blocks
            .iter()
            .find(|b| matches!(b, Block::Art { art: Art::Left, .. }))
            .copied()
        else {
            panic!("no SLOT's piece at {w}x{h}");
        };
        assert_eq!(
            (x, y),
            ((ox + 2 * LEFT[0]) as f32, (oy + 2 * LEFT[1]) as f32),
            "{w}x{h}"
        );
        // the plate's own middle is the screen's
        assert_eq!(ox + 2 * (PX + PW / 2), w as i32 / 2, "{w}x{h}");
    }
}

#[test]
fn light_dims_the_plate_and_leaves_the_shadow_black() {
    let blocks = scene(2_700.0, 0.4, 640, 480);
    for b in &blocks {
        let Block::Art { art, colour, .. } = b else {
            continue;
        };
        if *art != Art::Grid {
            assert_eq!(&colour[..3], &[0.4, 0.4, 0.4], "{art:?}");
        } else {
            assert!(
                colour[..3].iter().all(|c| *c <= 0.4),
                "the grid is tinted bluish and dimmed: {colour:?}"
            );
        }
    }
}

#[test]
fn the_glint_crosses_once_and_is_cut_to_the_plates_corners() {
    assert_eq!(glint_at(SHINE[0]), None);
    assert!(glint_at(2_300.0).is_some());
    assert_eq!(glint_at(SHINE[1]), None);
    let (ox, _) = frame_origin(640, 480);
    let blocks = scene(2_300.0, 1.0, 640, 480);
    let glint: Vec<[f32; 4]> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::Art {
                art: Art::Glint,
                clip: Some(c),
                ..
            } => Some(*c),
            _ => None,
        })
        .collect();
    assert_eq!(glint.len(), 7, "three stairs up, the middle, three down");
    assert_eq!(
        glint[3][0],
        (ox + 2 * PX) as f32,
        "the middle is the whole plate"
    );
    assert_eq!(
        glint[0][0],
        (ox + 2 * (PX + PC)) as f32,
        "the top is cut in by the corners"
    );
    assert_eq!(
        glint[0][2], glint[6][2],
        "the top and the bottom stairs are the same"
    );
}

#[test]
fn the_pieces_are_drawn_in_the_order_they_overlap() {
    // SLOT's piece, then SLOT, then the B's trail, shadow, piece, seam and B on top
    let blocks = scene(1_850.0, 1.0, 640, 480);
    assert_eq!(
        arts(&blocks),
        [
            Art::Grid,
            Art::Left,
            Art::Letters,
            Art::Piece,
            Art::Piece,
            Art::Shadow,
            Art::Piece,
            Art::Seam,
            Art::Glyph
        ]
    );
}
