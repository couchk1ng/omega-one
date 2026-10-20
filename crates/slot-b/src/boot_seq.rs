//! The boot sequence, as a function of time: a grid wipes in from the middle, SLOT's piece of
//! a silver plate wipes in with a notch cut in its end, SLOT stamps in a letter at a time, and
//! the B's piece slides in from the right, speeding up until its tab bottoms out in the notch
//! with a click. It does not stop there: it carries on at the speed it arrived, braking, into
//! SLOT's piece, and the two make one plate with no seam. A glint crosses it.
//!
//! `scene` says what to draw at a moment, as pieces of the art `slot_b_ui::boot_art` made,
//! each placed, clipped and tinted; nothing here knows of the GPU. Times are in ms from the
//! start of the sequence.

use slot_b_ui::boot::{
    FRAME_H, FRAME_W, GAP, GLINT, GLYPH, LEFT, LETTERS, PC, PH, PIECE, PW, PX, PY, SEAM, SHADOW,
    WX, XSL,
};

use crate::audio::Sfx;

/// When each part of the sequence runs, in ms.
pub const GRID: [f32; 2] = [250.0, 1000.0];
pub const PLATE: [f32; 2] = [900.0, 1250.0];
pub const STAMP: [f32; 2] = [1250.0, 1600.0];
pub const SLIDE: [f32; 2] = [1650.0, 1900.0];
/// The click: the B's tab bottoms out in SLOT's notch.
pub const SEAT: f32 = 1900.0;
/// The B's piece carrying on after the click, to rest.
pub const SETTLE: [f32; 2] = [1900.0, 1972.0];
pub const SHINE: [f32; 2] = [2050.0, 2600.0];

/// How long the finished plate is held, from the start, before the sequence fades out.
pub const HOLD_MS: f32 = 3000.0;

/// How far off to the right the B's piece starts, past where it clicks, in logical pixels.
const START_BEHIND: f32 = 125.0;

/// The letters of SLOT, how far apart they stand and how wide the lettering is with its emboss.
const LETTER_COUNT: i32 = 4;
const ADVANCE: i32 = 35;
const LETTER_GAP: i32 = 5;
const EMBOSS: i32 = 2;

/// `t` from 0 at `a` to 1 at `b`, held outside them.
pub fn prog(t: f32, a: f32, b: f32) -> f32 {
    ((t - a) / (b - a)).clamp(0.0, 1.0)
}

/// Fast off the start, slow onto the end.
fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

fn ease_out_quart(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(4)
}

/// How much the grid has been struck bright, 0 to 1: a little as the plate lands, a lot at the
/// click, fading.
pub fn pulse(t: f32) -> f32 {
    let landed = if t > PLATE[1] {
        0.5 * (-(t - PLATE[1]) / 380.0).exp()
    } else {
        0.0
    };
    let clicked = if t > SEAT {
        (-(t - SEAT) / 380.0).exp()
    } else {
        0.0
    };
    landed.max(clicked)
}

/// How far the B's piece is from where it finishes, in logical pixels: not there yet before it
/// slides, `GAP` plus the rest of its slide until the click, then `GAP` down to nothing.
pub fn piece_offset(t: f32) -> Option<f32> {
    if t < SLIDE[0] {
        return None;
    }
    if t > SEAT {
        return Some(GAP as f32 * (1.0 - ease_out_quart(prog(t, SETTLE[0], SETTLE[1]))));
    }
    let p = prog(t, SLIDE[0], SLIDE[1]);
    Some(GAP as f32 + START_BEHIND * (1.0 - p * p))
}

/// How many of SLOT's letters show, the newest flickering in; None while none have started.
pub fn letters_shown(t: f32) -> Option<i32> {
    let p = prog(t, STAMP[0], STAMP[1]);
    if p <= 0.0 {
        return None;
    }
    let newest = ((p * LETTER_COUNT as f32) as i32 + 1).min(LETTER_COUNT);
    let flick = (t / 40.0).floor() as i64 % 2 == 0;
    Some(newest - 1 + i32::from(p >= 1.0 || flick))
}

/// How lit the joint's flash is, from the click.
pub fn flash(t: f32) -> f32 {
    if t > SEAT {
        (-(t - SEAT) / 60.0).exp()
    } else {
        0.0
    }
}

/// Where the glint's middle is, in logical pixels along the frame, while it crosses.
pub fn glint_at(t: f32) -> Option<f32> {
    let p = prog(t, SHINE[0], SHINE[1]);
    (p > 0.0 && p < 1.0).then(|| -30.0 + (FRAME_W as f32 + 40.0 + 30.0) * ease_out(p))
}

/// The sound that falls due in (`before`, `now`], if one does. They are 250 ms or more apart.
pub fn sound(before: f32, now: f32) -> Option<Sfx> {
    [
        (250.0, Sfx::BootRise),
        (STAMP[0], Sfx::BootStamp),
        (SLIDE[0], Sfx::BootSlide),
        (SEAT, Sfx::BootSeat),
    ]
    .into_iter()
    .find(|&(at, _)| before < at && at <= now)
    .map(|(_, sfx)| sfx)
}

/// A piece of the art.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Art {
    Grid,
    Left,
    Letters,
    Piece,
    Seam,
    Shadow,
    Glyph,
    Glint,
}

/// One thing to draw, in screen pixels.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Block {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        colour: [f32; 4],
    },
    Art {
        art: Art,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        /// Multiplies the art's colours: the sequence's brightness in the first three, the
        /// part's opacity in the last.
        colour: [f32; 4],
        /// Only what falls in this screen rectangle is drawn.
        clip: Option<[f32; 4]>,
    },
}

/// Where the 320x240 frame's top left stands on a `w` by `h` screen: the frame in the middle,
/// each of its pixels two screen pixels.
pub fn frame_origin(w: u32, h: u32) -> (i32, i32) {
    ((w as i32 - 2 * FRAME_W) / 2, (h as i32 - 2 * FRAME_H) / 2)
}

/// What to draw at `t`, over black, on a `w` by `h` screen, with everything `level` as bright
/// as it is meant to be (fading it to 0 fades the sequence out).
pub fn scene(t: f32, level: f32, w: u32, h: u32) -> Vec<Block> {
    let mut out = Vec::new();
    if t < 0.0 || level <= 0.0 {
        return out;
    }
    let (ox, oy) = frame_origin(w, h);
    let x = |lx: f32| ox as f32 + 2.0 * lx;
    let y = |ly: f32| oy as f32 + 2.0 * ly;
    let grey = |a: f32| [level, level, level, a];
    let art =
        |art: Art, face: [i32; 4], dx: f32, colour: [f32; 4], clip: Option<[f32; 4]>| Block::Art {
            art,
            x: x(face[0] as f32 + dx),
            y: y(face[1] as f32),
            w: 2.0 * face[2] as f32,
            h: 2.0 * face[3] as f32,
            colour,
            clip,
        };

    // the grid wipes in from the middle, as the preview's grew from it, and is struck bright
    // at the landing and the click
    let reveal = ease_out(prog(t, GRID[0], GRID[1]));
    if reveal > 0.0 {
        let (hx, hy) = (
            reveal * 1.25 * w as f32 / 2.0,
            reveal * 1.25 * h as f32 / 2.0,
        );
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        let glow = pulse(t);
        let mix = |a: f32, b: f32| (a + (b - a) * (glow * 0.6).clamp(0.0, 1.0)) / 255.0 * level;
        out.push(Block::Art {
            art: Art::Grid,
            x: 0.0,
            y: 0.0,
            w: w as f32,
            h: h as f32,
            colour: [
                mix(44.0, 70.0),
                mix(50.0, 110.0),
                mix(64.0, 190.0),
                0.75 + 0.25 * glow,
            ],
            clip: (reveal < 1.0).then_some([cx - hx, cy - hy, 2.0 * hx, 2.0 * hy]),
        });
    }

    // SLOT's piece wipes in from its middle
    let wipe = ease_out(prog(t, PLATE[0], PLATE[1]));
    if wipe <= 0.0 {
        return out;
    }
    let middle = (PX + XSL) as f32 / 2.0;
    let half = wipe * ((XSL - PX) as f32 / 2.0 + 3.0);
    let clip = (wipe < 1.0).then(|| {
        [
            x(middle - half),
            y(LEFT[1] as f32),
            4.0 * half,
            2.0 * LEFT[3] as f32,
        ]
    });
    out.push(art(Art::Left, LEFT, 0.0, grey(1.0), clip));

    // S, L, O, T stamp in
    if let Some(n) = letters_shown(t).filter(|&n| n > 0) {
        let width = n * ADVANCE - LETTER_GAP + EMBOSS;
        let clip = [
            x(WX as f32),
            y(LETTERS[1] as f32),
            2.0 * width as f32,
            2.0 * LETTERS[3] as f32,
        ];
        out.push(art(Art::Letters, LETTERS, 0.0, grey(1.0), Some(clip)));
    }

    // the B's piece
    if let Some(u) = piece_offset(t) {
        let du = u.round() - GAP as f32;
        if t < SEAT {
            // a faint trail behind it while it slides
            for (behind, a) in [(16.0, 0.07), (8.0, 0.12)] {
                out.push(art(Art::Piece, PIECE, du + behind, grey(a), None));
            }
        }
        // its shadow falls from the cut edge outward, never over SLOT's piece
        let left = x((XSL + 3) as f32);
        let right = x(SHADOW[0] as f32 + du) + 2.0 * SHADOW[2] as f32;
        if right > left {
            let clip = [
                left,
                y(SHADOW[1] as f32),
                right - left,
                2.0 * SHADOW[3] as f32,
            ];
            out.push(art(Art::Shadow, SHADOW, du, grey(1.0), Some(clip)));
        }
        out.push(art(Art::Piece, PIECE, du, grey(1.0), None));
        // the outline along its cut edge shows only while the pieces are apart
        let apart = ((u - GAP as f32) / 6.0).clamp(0.0, 1.0);
        if apart > 0.0 {
            out.push(art(Art::Seam, SEAM, du, grey(apart), None));
        }
        // the flash at the joint, a soft glow, under the B
        let lit = flash(t);
        if lit > 0.03 {
            for (width, a) in [(32.0, 0.16), (20.0, 0.26), (10.0, 0.4)] {
                out.push(Block::Rect {
                    x: x(XSL as f32 - width / 2.0),
                    y: y(PY as f32 + 5.0),
                    w: 2.0 * width,
                    h: 2.0 * (PH as f32 - 10.0),
                    colour: grey(a * lit),
                });
            }
        }
        out.push(art(Art::Glyph, GLYPH, du, grey(1.0), None));
    }

    // the glint across the finished plate, kept inside its cut corners by stairs
    if let Some(at) = glint_at(t) {
        let at = at.round();
        let stairs: [(i32, i32, i32); 7] = [
            (0, 5, PC),
            (5, 10, PC - 5),
            (10, 14, PC - 10),
            (14, PH - 14, 0),
            (PH - 14, PH - 10, PC - 10),
            (PH - 10, PH - 5, PC - 5),
            (PH - 5, PH, PC),
        ];
        for (from, to, inset) in stairs {
            let clip = [
                x((PX + inset) as f32),
                y((PY + from) as f32),
                2.0 * (PW - 2 * inset) as f32,
                2.0 * (to - from) as f32,
            ];
            out.push(art(
                Art::Glint,
                [GLINT[0], PY, GLINT[2], GLINT[3]],
                at,
                grey(1.0),
                Some(clip),
            ));
        }
    }
    out
}
