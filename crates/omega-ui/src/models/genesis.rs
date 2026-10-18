//! The Genesis and Mega Drive cart: one shell for both, as the real carts share one.
//!
//! Drawn 4 units to the millimetre from a retail Mega Drive cart photographed square on, and
//! checked against its width of 108.5 mm and its label of 80.5 mm. From the front it is a plain
//! rounded rectangle about 1.47 times as wide as it is tall, 434 by 296 units: a raised pillar
//! down each side that rolls off round the end, a label well between them 74% of the width that
//! runs up to the top edge, and under it a grid band for the console's name. The corners are
//! about 5 mm round.
//!
//! Behind the main shell, which is about 12.7 mm deep, is a narrower foot, 97 mm across, that
//! goes into the console, making the whole cart about 17 mm thick. The ends of the shell roll
//! off in a curve. The pillars are that curve: the front surface is flat across the label and
//! rolls back over about 13 mm at each end, about 10 mm deep, to the full width. This model has
//! that roll, with the whole cart as one slab of the full thickness behind it: the narrower foot
//! is not drawn, as a cart turned a few degrees as it sways shows little of it.

use std::sync::OnceLock;

use omega_gfx::MeshVertex;

use crate::cart::{Board, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};
use crate::spin::{SIDE_BACK_SHADE, SIDE_SAMPLE};

const SVG: &str = include_str!("../../assets/genesis_cart.svg");
const DETAIL_SVG: &str = include_str!("../../assets/genesis_cart_detail.svg");

/// How thick the cart is, as a share of its width: about 17 mm of its 108.5.
pub const THICK: f32 = 0.15;

/// How big the cart stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

/// The drawing's size in units, 4 to the millimetre: 108.5 by 74 mm.
const UNITS: (f32, f32) = (434.0, 296.0);

/// How round the corners are, in units: 5 mm, as a scanned real cart's are.
const RADIUS: f32 = 20.0;

/// The size of the face in pixels, about 2.75 to the millimetre as the SNES carts are, so the
/// cart stands at its right size beside them: even across its half so it stands on whole art
/// pixels in the middle of the row.
pub const W: u32 = 300;
pub const H: u32 = 204;

/// The label in the face's pixels: 80.5 mm by 53, from the pillar to the pillar and up to just
/// under the top edge, 1.8 mm, which leaves room for the bevel round its well.
pub const LABEL: (u32, u32, u32, u32) = (39, 5, 222, 145);

/// How far in from the outline a clear shell's plastic lightens at its rim, in pixels.
const RIM: u32 = 6;

/// The grid band under the label, in the face's rows, where the console's name is printed.
pub const BAND_TOP: u32 = LABEL.1 + LABEL.3;
pub const BAND_BOTTOM: u32 = 185;

/// How far in from the band's edges the name keeps, in pixels, and the biggest it is printed.
pub const BAND_PAD: u32 = 5;
pub const NAME_PX: f32 = 18.0;

/// The silver the name is printed in, as the chrome of the console's logo is.
pub const NAME_INK: [u8; 3] = [0xc9, 0xcd, 0xd8];

/// The console's name as a real cart of that console carries it on its band.
pub fn console_name(platform: omega_store::Platform) -> &'static str {
    match platform {
        omega_store::Platform::Genesis => "GENESIS",
        omega_store::Platform::MegaDrive => "MEGA DRIVE",
        _ => "",
    }
}

/// The cart's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    let (w, h) = UNITS;
    let r = RADIUS;
    let mut points = vec![(r, 0.0), (w - r, 0.0)];
    let corners = [
        (w - r, r, -90.0f32),
        (w - r, h - r, 0.0),
        (r, h - r, 90.0),
        (r, r, 180.0),
    ];
    for (i, (cx, cy, from)) in corners.into_iter().enumerate() {
        let at = |deg: f32| {
            let a = deg.to_radians();
            (cx + r * a.cos(), cy + r * a.sin())
        };
        if i > 0 {
            points.push(at(from));
        }
        points.push(at(from + 30.0));
        points.push(at(from + 60.0));
        points.push(at(from + 90.0));
    }
    // the last corner ends where the outline began
    points.pop();
    (points, UNITS)
}

/// The cart's silhouette `w` by `h`, as coverage: opaque inside the outline.
pub fn silhouette(w: u32, h: u32) -> Vec<u8> {
    rasterise_svg(SVG, w, h).unwrap_or_else(|| vec![255; (w * h) as usize])
}

fn mask() -> &'static [u8] {
    static MASK: OnceLock<Vec<u8>> = OnceLock::new();
    MASK.get_or_init(|| silhouette(W, H))
}

fn depth() -> &'static [u8] {
    static DEPTH: OnceLock<Vec<u8>> = OnceLock::new();
    DEPTH.get_or_init(|| depth_map(mask(), W as usize, H as usize))
}

fn detail() -> &'static Detail {
    static DETAIL: OnceLock<Detail> = OnceLock::new();
    DETAIL.get_or_init(|| rasterise_detail(DETAIL_SVG, W, H).unwrap_or_else(|| Detail::blank(W, H)))
}

/// How the cart's face is laid out. A clear shell shows the board inside it.
pub(crate) fn spec() -> Spec {
    Spec {
        w: W,
        h: H,
        label: LABEL,
        label_round: 0,
        mask: mask(),
        depth: depth(),
        detail: detail(),
        max_h: (LABEL.3 - 2 * PAD) as f32,
        rim: RIM,
        board: Board {
            x: (0.1, 0.9),
            top: 0.1,
            pins: (0.27, 0.73),
            contacts_from: 0.86,
            traces_from: 0.78,
        },
    }
}

/// The cart's width in millimetres, which its roll is measured against.
const WIDTH_MM: f32 = 108.5;

/// How wide each end's roll is across the face, and how far back it goes, in millimetres: the
/// width of a pillar, and a little under the depth of the main shell, from the cross-section
/// of a real cart seen from its connector end.
pub const ROLL_MM: (f32, f32) = (13.0, 10.0);

/// How much darker the right end of the face is where it has turned all the way back, the end
/// the light falls away from.
const ROLL_SHADE: f32 = 0.2;

/// How many steps the curve of each end is cut into.
const ROLL_STEPS: usize = 6;

/// Rows the face is cut into: it does not curve up and down.
const ROWS: usize = 2;

/// How far back the front surface is at `x` units across the drawing, as a share of the cart's
/// width: nothing across the flat of the label, and curving back to `ROLL_MM.1` at each edge, a
/// quarter of an ellipse that leaves the flat level and meets the side square.
pub fn roll_z(x: f32) -> f32 {
    let (across, deep) = ROLL_MM;
    let units_per_mm = UNITS.0 / WIDTH_MM;
    let d = x.min(UNITS.0 - x).max(0.0) / units_per_mm;
    if d >= across {
        return 0.0;
    }
    let s = (across - d) / across;
    deep / WIDTH_MM * (1.0 - (1.0 - s * s).max(0.0).sqrt())
}

/// The columns the face is cut at, as shares of its width: the roll of each end in equal
/// steps of its angle, so the curve is finer where it turns, and the flat between.
fn columns() -> Vec<f32> {
    let (across, _) = ROLL_MM;
    let flat = across / WIDTH_MM;
    let mut left: Vec<f32> = (0..=ROLL_STEPS)
        .rev()
        .map(|j| {
            let t = std::f32::consts::FRAC_PI_2 * j as f32 / ROLL_STEPS as f32;
            flat * (1.0 - t.sin())
        })
        .collect();
    // from the edge in to the start of the flat
    left.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    let mut all = left.clone();
    all.extend(left.iter().rev().map(|u| 1.0 - u));
    all
}

/// The cart's model: its face, a flat across the label that rolls back at each end, whole texture
/// laid over it so the picture seen from the front is the face as drawn, and its sides, a wall
/// for each cut of its outline from where the face is there back to `THICK`, each taking its
/// colour from the face just inside its edge. Sides come first, so the face covers them.
pub(crate) fn mesh() -> Vec<MeshVertex> {
    let (points, (uw, uh)) = outline();
    let cols = columns();
    let mut ring: Vec<(f32, f32)> = Vec::new();
    for (i, &a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        ring.push(a);
        // a straight top or bottom edge is cut where the face is, so its wall follows the roll
        if (a.1 - b.1).abs() < 1e-3 && (b.0 - a.0).abs() > 1.0 {
            let (lo, hi) = (a.0.min(b.0), a.0.max(b.0));
            let mut cuts: Vec<f32> = cols
                .iter()
                .map(|u| u * uw)
                .filter(|x| *x > lo + 0.5 && *x < hi - 0.5)
                .collect();
            cuts.sort_by(|p, q| p.total_cmp(q));
            if b.0 < a.0 {
                cuts.reverse();
            }
            ring.extend(cuts.into_iter().map(|x| (x, a.1)));
        }
    }
    let mut out = Vec::new();
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % ring.len()];
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-3 {
            continue;
        }
        let normal = [dy / len, -dx / len, 0.0];
        let sample = |p: (f32, f32)| {
            [
                (p.0 - normal[0] * SIDE_SAMPLE) / uw,
                (p.1 - normal[1] * SIDE_SAMPLE) / uh,
            ]
        };
        let corner = |p: (f32, f32), z: f32, shade: f32| MeshVertex {
            xyz: [p.0 / uw, p.1 / uh, z],
            normal,
            uv: sample(p),
            shade,
        };
        let (a0, b0) = (corner(a, roll_z(a.0), 1.0), corner(b, roll_z(b.0), 1.0));
        let (a1, b1) = (
            corner(a, THICK, SIDE_BACK_SHADE),
            corner(b, THICK, SIDE_BACK_SHADE),
        );
        out.extend([a0, b0, b1, a0, b1, a1]);
    }
    let rows: Vec<f32> = (0..=ROWS).map(|r| r as f32 / ROWS as f32).collect();
    let at = |u: f32, v: f32| {
        let z = roll_z(u * uw);
        // The renderer folds a polygon away by testing each vertex's own normal against the eye,
        // so every vertex of a polygon has to agree. The face keeps the one normal of a flat
        // face, and the roll is shaded by the vertex's light instead: the end the light falls
        // away from, the right, darkens as it turns back.
        let shade = if u < 0.5 {
            1.0
        } else {
            1.0 - ROLL_SHADE * (z / (ROLL_MM.1 / WIDTH_MM))
        };
        MeshVertex {
            xyz: [u, v, z],
            normal: [0.0, 0.0, -1.0],
            uv: [u, v],
            shade,
        }
    };
    for r in 0..ROWS {
        for c in 0..cols.len() - 1 {
            let (p, q, s, t) = (
                at(cols[c], rows[r]),
                at(cols[c + 1], rows[r]),
                at(cols[c + 1], rows[r + 1]),
                at(cols[c], rows[r + 1]),
            );
            out.extend([p, q, s, p, s, t]);
        }
    }
    out
}
