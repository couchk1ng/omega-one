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
//! off in a long curve about 19 mm across. This model keeps the whole cart as one slab of the
//! full thickness, so neither the foot nor the roll of the ends is drawn: the face is what the
//! shelf shows, and a cart turned a few degrees as it sways shows little of either.

use std::sync::OnceLock;

use crate::cart::{Board, Spec, PAD};
use crate::silhouette::{depth_map, rasterise_detail, rasterise_svg, Detail};

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
