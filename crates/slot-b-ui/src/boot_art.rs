//! The boot sequence's art, drawn once into a few small faces: the grid, the plate in its two
//! pieces, SLOT's lettering, the B and its shadow, and the glint. The sequence only places,
//! clips and tints them, so a frame costs about a dozen draw items instead of a thousand.
//!
//! Everything is laid out in logical pixels, each drawn two screen pixels wide: a 320x240 frame
//! with the plate in the middle of it. The plate is a silver octagon cut in two. SLOT's piece
//! has a notch in its right end; the B's piece has a tab that fits it. The B's piece slides in,
//! clicks home at `GAP` pixels from where it finishes, and carries on into SLOT's piece, which
//! is the same silver, so the two make one plate with no seam.

use crate::UndoFace;

/// The frame the plate is laid out in.
pub const FRAME_W: i32 = 320;
pub const FRAME_H: i32 = 240;

/// The finished plate: where it starts, its size and how far its corners are cut.
pub const PX: i32 = 45;
pub const PW: i32 = 230;
pub const PY: i32 = 92;
pub const PH: i32 = 56;
pub const PC: i32 = 14;

/// Where SLOT's piece ends, once the B's piece has touched it.
pub const XSL: i32 = 227;
/// How far the B's piece carries on into SLOT's piece after the click.
pub const GAP: i32 = 18;
/// How far its tab reaches into SLOT's notch, and the rows the two occupy.
pub const ND: i32 = 12;
pub const NY0: i32 = 17;
pub const NY1: i32 = 39;

/// A face's size: its pixels, each a logical pixel.
pub const LEFT: [i32; 4] = [PX - 2, PY - 2, XSL + 3 - (PX - 2), PH + 6];
pub const LETTERS: [i32; 4] = [WX, WY, 4 * ADVANCE - WORD_GAP + EMBOSS, 7 * S + EMBOSS];
pub const PIECE: [i32; 4] = [XSL - ND, PY - 2, 81, PH + 4];
pub const SEAM: [i32; 4] = [XSL - ND - 2, PY - 2, ND + 2 + 2, PH + 4];
pub const SHADOW: [i32; 4] = [XSL + 3, PY + 4, 67, PH];
pub const GLYPH: [i32; 4] = [WX + B_AT + GAP, WY, 6 * S + EMBOSS, 7 * S + EMBOSS];
pub const GLINT: [i32; 4] = [-GLINT_HALF, 0, 2 * GLINT_HALF, PH];

/// The lettering: each font pixel is `S` logical pixels, letters `ADVANCE` apart, and a word's
/// first letter stands at `WX`, on `WY`.
const S: i32 = 5;
const WORD_GAP: i32 = 5;
const SPACE: i32 = 14;
const ADVANCE: i32 = 6 * S + WORD_GAP;
const EMBOSS: i32 = 2;
/// The word is "SLOT B": 184 wide. The B stands after SLOT and a space.
const B_AT: i32 = 4 * ADVANCE + SPACE;
const WORD_W: i32 = B_AT + 6 * S;
pub const WX: i32 = FRAME_W / 2 - WORD_W / 2;
pub const WY: i32 = PY + (PH - 7 * S) / 2;
/// How far the glint's band reaches either side of its middle.
const GLINT_HALF: i32 = 40;

const GLYPHS: [(char, [&str; 7]); 5] = [
    (
        'S',
        [
            ".#####", "##....", "##....", ".####.", "....##", "....##", "#####.",
        ],
    ),
    (
        'L',
        [
            "##....", "##....", "##....", "##....", "##....", "##....", "######",
        ],
    ),
    (
        'O',
        [
            ".####.", "##..##", "##..##", "##..##", "##..##", "##..##", ".####.",
        ],
    ),
    (
        'T',
        [
            "######", "..##..", "..##..", "..##..", "..##..", "..##..", "..##..",
        ],
    ),
    (
        'B',
        [
            "#####.", "##..##", "##..##", "#####.", "##..##", "##..##", "#####.",
        ],
    ),
];

const SILVER_TOP: [f32; 3] = [244.0, 249.0, 252.0];
const SILVER_MID: [f32; 3] = [214.0, 226.0, 234.0];
const SILVER_LOW: [f32; 3] = [154.0, 174.0, 188.0];
const OUTLINE: [u8; 3] = [0x56, 0x62, 0x6e];

/// A face placed in the frame: its top left, in logical pixels.
pub struct ArtFace {
    pub face: UndoFace,
    pub x: i32,
    pub y: i32,
}

/// All the sequence's art.
pub struct BootArt {
    /// Grid lines in white, fainter towards the corners, as large as the panel in logical
    /// pixels; the sequence tints it.
    pub grid: UndoFace,
    /// SLOT's piece of the plate, with its shadow, outline, face and the notch.
    pub left: ArtFace,
    /// SLOT, in black on the plate, with the pale emboss behind it.
    pub letters: ArtFace,
    /// The B's piece, as it stands where it touches SLOT's: outline, face and tab.
    pub piece: ArtFace,
    /// The outline along the B's piece's cut edge and its tab, shown only while the pieces
    /// are apart.
    pub seam: ArtFace,
    /// The B's piece's shadow.
    pub shadow: ArtFace,
    /// The blue B, as it stands where the piece touches SLOT's.
    pub glyph: ArtFace,
    /// The glint's slanted band, in white, about its middle.
    pub glint: ArtFace,
}

/// The silver of the plate on its row `y`.
fn face_colour(y: i32) -> [u8; 3] {
    if y == 0 {
        return [255, 255, 255];
    }
    if y == PH - 1 {
        return [111, 128, 140];
    }
    let t = y as f32 / (PH - 1) as f32;
    let (a, b, u) = if t < 0.5 {
        (SILVER_TOP, SILVER_MID, t / 0.5)
    } else {
        (SILVER_MID, SILVER_LOW, (t - 0.5) / 0.5)
    };
    let mix = |i: usize| (a[i] + (b[i] - a[i]) * u).round() as u8;
    [mix(0), mix(1), mix(2)]
}

/// How far the plate's cut corners take its row `y` in.
pub fn inset(y: i32) -> i32 {
    0i32.max(PC - y).max(PC - (PH - 1 - y))
}

/// The same for the outline's row `r`, two rows above the plate's first.
fn inset_outline(r: i32) -> i32 {
    0i32.max(PC + 2 - r).max(PC + 2 - (PH + 3 - r))
}

fn notch(y: i32) -> bool {
    (NY0..NY1).contains(&y)
}

/// Pixels as premultiplied floats, painted over one another as the compositor would.
struct Canvas {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    px: Vec<[f32; 4]>,
}

impl Canvas {
    fn new(rect: [i32; 4]) -> Canvas {
        Canvas {
            x0: rect[0],
            y0: rect[1],
            w: rect[2],
            h: rect[3],
            px: vec![[0.0; 4]; (rect[2] * rect[3]) as usize],
        }
    }

    /// `rgb` at opacity `a` over the rectangle, clipped to the canvas.
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, rgb: [u8; 3], a: f32) {
        for yy in (y - self.y0).max(0)..(y + h - self.y0).min(self.h) {
            for xx in (x - self.x0).max(0)..(x + w - self.x0).min(self.w) {
                let p = &mut self.px[(yy * self.w + xx) as usize];
                for (c, &v) in p.iter_mut().zip(&rgb) {
                    *c = v as f32 / 255.0 * a + *c * (1.0 - a);
                }
                p[3] = a + p[3] * (1.0 - a);
            }
        }
    }

    fn span(&mut self, x0: i32, x1: i32, y: i32, rgb: [u8; 3], a: f32) {
        self.rect(x0, y, x1 - x0, 1, rgb, a);
    }

    fn into_face(self) -> UndoFace {
        let mut rgba = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            let a = p[3];
            for c in &p[..3] {
                let v = if a > 0.0 { c / a } else { 0.0 };
                rgba.push((v * 255.0).round().clamp(0.0, 255.0) as u8);
            }
            rgba.push((a * 255.0).round().clamp(0.0, 255.0) as u8);
        }
        UndoFace {
            rgba,
            w: self.w as u32,
            h: self.h as u32,
        }
    }
}

fn placed(c: Canvas) -> ArtFace {
    let (x, y) = (c.x0, c.y0);
    ArtFace {
        face: c.into_face(),
        x,
        y,
    }
}

/// The font pixels of the letters `from..to` of "SLOT B", each as (x, y, top): where it
/// stands in the word, and whether the pixel above it is clear.
fn cells(letters: &[char], first_at: i32) -> Vec<(i32, i32, bool)> {
    let mut out = Vec::new();
    for (k, ch) in letters.iter().enumerate() {
        let glyph = &GLYPHS.iter().find(|(c, _)| c == ch).expect("a glyph").1;
        let at = first_at + k as i32 * ADVANCE;
        for (r, row) in glyph.iter().enumerate() {
            for (c, b) in row.bytes().enumerate() {
                if b == b'#' {
                    let top = r == 0 || glyph[r - 1].as_bytes()[c] != b'#';
                    out.push((at + c as i32 * S, r as i32 * S, top));
                }
            }
        }
    }
    out
}

/// The glyph pixels drawn as the plate wears them: the pale emboss behind, the ink, and a thin
/// edge of `edge` along the top of each stroke.
fn paint_glyphs(
    c: &mut Canvas,
    cells: &[(i32, i32, bool)],
    at: (i32, i32),
    body: [u8; 3],
    edge: [u8; 3],
) {
    for &(x, y, _) in cells {
        c.rect(
            at.0 + x + EMBOSS,
            at.1 + y + EMBOSS,
            S,
            S,
            [255, 255, 255],
            0.8,
        );
    }
    for &(x, y, _) in cells {
        c.rect(at.0 + x, at.1 + y, S, S, body, 1.0);
    }
    for &(x, y, top) in cells {
        if top {
            c.rect(at.0 + x, at.1 + y, S, EMBOSS, edge, 1.0);
        }
    }
}

fn left() -> ArtFace {
    let mut c = Canvas::new(LEFT);
    for y in 0..PH {
        c.span(PX + inset(y) + 3, XSL + 3, PY + 4 + y, [0, 0, 0], 0.55);
    }
    for r in 0..PH + 4 {
        c.span(PX - 2 + inset_outline(r), XSL + 2, PY - 2 + r, OUTLINE, 1.0);
    }
    for y in 0..PH {
        let end = if notch(y) { XSL - ND } else { XSL };
        c.span(PX + inset(y), end, PY + y, face_colour(y), 1.0);
    }
    // the recess the tab slides into: shadowed above and to the left, open on the right
    c.rect(XSL - ND, PY + NY0, ND, NY1 - NY0, [0x4b, 0x58, 0x65], 1.0);
    c.rect(XSL - ND, PY + NY0, ND, 2, [0x2e, 0x38, 0x42], 1.0);
    c.rect(XSL - ND, PY + NY0, 2, NY1 - NY0, [0x2e, 0x38, 0x42], 1.0);
    c.rect(XSL - ND, PY + NY1 - 1, ND, 1, [255, 255, 255], 0.8);
    placed(c)
}

fn letters() -> ArtFace {
    let mut c = Canvas::new(LETTERS);
    paint_glyphs(
        &mut c,
        &cells(&['S', 'L', 'O', 'T'], 0),
        (WX, WY),
        [0x0a, 0x0d, 0x14],
        [0x2f, 0x5d, 0xb0],
    );
    placed(c)
}

/// The B's piece's row `r` of outline, starting where its face does.
fn piece_row_start(r: i32) -> i32 {
    if notch(r - 2) {
        XSL - ND
    } else {
        XSL
    }
}

fn piece() -> ArtFace {
    let mut c = Canvas::new(PIECE);
    for r in 0..PH + 4 {
        c.span(
            piece_row_start(r),
            PX + PW + 2 - inset_outline(r) + GAP,
            PY - 2 + r,
            OUTLINE,
            1.0,
        );
    }
    for y in 0..PH {
        let start = if notch(y) { XSL - ND } else { XSL };
        c.span(start, PX + PW - inset(y) + GAP, PY + y, face_colour(y), 1.0);
    }
    placed(c)
}

fn seam() -> ArtFace {
    let mut c = Canvas::new(SEAM);
    for r in 0..PH + 4 {
        let start = piece_row_start(r);
        c.span(start - 2, start, PY - 2 + r, OUTLINE, 1.0);
    }
    placed(c)
}

fn shadow() -> ArtFace {
    let mut c = Canvas::new(SHADOW);
    for y in 0..PH {
        c.span(
            XSL + 3,
            PX + PW - inset(y) + 3 + GAP,
            PY + 4 + y,
            [0, 0, 0],
            0.55,
        );
    }
    placed(c)
}

fn glyph() -> ArtFace {
    let mut c = Canvas::new(GLYPH);
    let b = cells(&['B'], 0);
    paint_glyphs(
        &mut c,
        &b,
        (WX + B_AT + GAP, WY),
        [0x1c, 0x62, 0xd8],
        [0x79, 0xad, 0xff],
    );
    placed(c)
}

fn glint() -> ArtFace {
    let mut c = Canvas::new(GLINT);
    for y in 0..PH {
        let mid = (y as f32 - PH as f32 / 2.0) * 0.9;
        let band = |c: &mut Canvas, a: f32, b: f32, al: f32| {
            let (s, e) = ((mid + a).round() as i32, (mid + b).round() as i32);
            c.span(s, e, y, [255, 255, 255], al);
        };
        band(&mut c, -14.0, -5.0, 0.2);
        band(&mut c, -5.0, 5.0, 0.5);
        band(&mut c, 5.0, 14.0, 0.2);
    }
    placed(c)
}

/// Grid lines, a line through the middle of the `w` by `h` panel and one every 16 logical
/// pixels either way, in white that thins towards the corners as the preview's did.
fn grid(w: u32, h: u32) -> UndoFace {
    let (lw, lh) = ((w / 2) as i32, (h / 2) as i32);
    let (cx, cy) = (lw / 2, lh / 2);
    let reach = 1.075 * ((lw as f32 / 2.0).hypot(lh as f32 / 2.0));
    let mut alpha = vec![0.0f32; (lw * lh) as usize];
    let strength = |x: i32, y: i32| {
        let d = ((x - cx) as f32).hypot((y - cy) as f32);
        1.0 - 0.55 * (d / reach).clamp(0.0, 1.0)
    };
    for y in 0..lh {
        for x in 0..lw {
            let on_v = (x - cx).rem_euclid(16) == 0;
            let on_h = (y - cy).rem_euclid(16) == 0;
            let a = strength(x, y);
            let i = (y * lw + x) as usize;
            // two lines crossing are painted twice, as the preview painted them
            alpha[i] = match (on_v, on_h) {
                (true, true) => 1.0 - (1.0 - a) * (1.0 - a),
                (true, false) | (false, true) => a,
                (false, false) => 0.0,
            };
        }
    }
    let mut rgba = Vec::with_capacity(alpha.len() * 4);
    for a in alpha {
        rgba.extend_from_slice(&[255, 255, 255, (a * 255.0).round() as u8]);
    }
    UndoFace {
        rgba,
        w: lw as u32,
        h: lh as u32,
    }
}

/// All the boot sequence's art, for a `w` by `h` panel.
pub fn boot_art(w: u32, h: u32) -> BootArt {
    BootArt {
        grid: grid(w, h),
        left: left(),
        letters: letters(),
        piece: piece(),
        seam: seam(),
        shadow: shadow(),
        glyph: glyph(),
        glint: glint(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(p: &ArtFace, x: i32, y: i32) -> [u8; 4] {
        let i = (((y - p.y) * p.face.w as i32 + (x - p.x)) * 4) as usize;
        [
            p.face.rgba[i],
            p.face.rgba[i + 1],
            p.face.rgba[i + 2],
            p.face.rgba[i + 3],
        ]
    }

    #[test]
    fn each_face_is_the_size_its_constant_says() {
        let a = boot_art(640, 480);
        for (name, p, r) in [
            ("left", &a.left, LEFT),
            ("letters", &a.letters, LETTERS),
            ("piece", &a.piece, PIECE),
            ("seam", &a.seam, SEAM),
            ("shadow", &a.shadow, SHADOW),
            ("glyph", &a.glyph, GLYPH),
            ("glint", &a.glint, GLINT),
        ] {
            assert_eq!(
                (p.x, p.y, p.face.w as i32, p.face.h as i32),
                (r[0], r[1], r[2], r[3]),
                "{name}"
            );
            assert_eq!(
                p.face.rgba.len(),
                (r[2] * r[3] * 4) as usize,
                "{name}'s pixels"
            );
        }
        assert_eq!((a.grid.w, a.grid.h), (320, 240));
        let big = boot_art(720, 720);
        assert_eq!((big.grid.w, big.grid.h), (360, 360));
    }

    #[test]
    fn the_word_is_centred_and_its_b_stands_where_the_plate_finishes() {
        assert_eq!((WX, WY), (68, 102));
        // the B's glyph, once the piece has carried on GAP pixels, is 154 pixels along the word
        assert_eq!(WX + B_AT, 222);
        assert_eq!(WORD_W, 184);
    }

    #[test]
    fn the_two_pieces_make_one_plate_once_the_b_has_carried_on() {
        // the B's piece, moved GAP pixels left, lays its face over SLOT's, and on each row
        // the faces meet or overlap with the same silver, so nothing shows between them
        let a = boot_art(640, 480);
        for y in 0..PH {
            let left_end = if notch(y) { XSL - ND } else { XSL };
            let piece_start = (if notch(y) { XSL - ND } else { XSL }) - GAP;
            assert!(piece_start <= left_end, "row {y}: the pieces leave a gap");
            let want = face_colour(y);
            let l = px(&a.left, left_end - 1, PY + y);
            assert_eq!(
                [l[0], l[1], l[2], l[3]],
                [want[0], want[1], want[2], 255],
                "row {y}: SLOT's silver"
            );
            let p = px(&a.piece, piece_start + GAP, PY + y);
            assert_eq!(
                [p[0], p[1], p[2], p[3]],
                [want[0], want[1], want[2], 255],
                "row {y}: the B's silver"
            );
        }
    }

    #[test]
    fn the_grid_has_a_line_through_the_middle_and_thins_to_the_corners() {
        let a = boot_art(640, 480);
        let at = |x: usize, y: usize| a.grid.rgba[(y * a.grid.w as usize + x) * 4 + 3];
        assert!(
            (at(160, 100) as i32 - 242).abs() <= 1,
            "a vertical line near the middle"
        );
        assert_eq!(at(161, 100), 0, "between lines");
        assert!(
            at(0, 0) < at(160, 120),
            "the corner is fainter than the middle"
        );
    }
}
