//! The Genesis and Mega Drive cart, one model for both, drawn from a retail Mega Drive cart
//! photographed square on: about 1.465 times as wide as it is tall, a label window 74% of its
//! width between two raised pillars, a grid band under it that carries the console's name.

use std::path::PathBuf;

use omega_store::{Cart, Core, Platform};
use omega_ui::{
    board_face, cart_box, cart_face, console_name, face_labels, genesis_silhouette, label_colour,
    label_text, model_box, model_of, seated_box, shelf_scale, shell_for, sockets_for, sockets_of,
    CartModel, BAND_BOTTOM, BAND_TOP, BOARD_H, BOARD_W, GENESIS_H, GENESIS_LABEL, GENESIS_SHELL,
    GENESIS_W, SNES_SOCKETS,
};

/// What the photograph of the real cart measured, as shares: its width over its height, the
/// label window's share of the width and of the height, and a pillar's share of the width.
const REAL_WIDE_OVER_TALL: f32 = 1.465;
const REAL_LABEL_OF_WIDTH: f32 = 0.74;
const REAL_LABEL_FOOT_OF_HEIGHT: f32 = 0.727;
const REAL_PILLAR_OF_WIDTH: f32 = 0.122;

fn cart(platform: Platform, title: &str) -> Cart {
    Cart {
        platform,
        stem: title.to_string(),
        rom: PathBuf::from(format!("Games/{}/{title}.md", platform.dir_name())),
        label: None,
        title: title.to_string(),
        code: String::new(),
        shell: None,
    }
}

const BOTH: [Platform; 2] = [Platform::Genesis, Platform::MegaDrive];

fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

#[test]
fn a_genesis_and_a_mega_drive_cart_have_a_model_of_their_own() {
    for p in BOTH {
        assert_eq!(model_of(&cart(p, "Sonic")), CartModel::Genesis, "{p:?}");
    }
    for p in Platform::ALL.into_iter().filter(|p| !p.sega_16bit()) {
        assert_ne!(model_of(&cart(p, "Sonic")), CartModel::Genesis, "{p:?}");
    }
}

#[test]
fn the_genesis_and_the_mega_drive_share_that_model() {
    assert_eq!(
        model_of(&cart(Platform::Genesis, "Sonic")),
        model_of(&cart(Platform::MegaDrive, "Sonic"))
    );
}

#[test]
fn the_face_is_in_the_proportions_of_the_real_cart() {
    let (w, h) = model_box(CartModel::Genesis);
    assert_eq!((w, h), (GENESIS_W, GENESIS_H));
    let ratio = w as f32 / h as f32;
    assert!(
        near(ratio, REAL_WIDE_OVER_TALL, 0.03),
        "{ratio:.3} wide for each tall, the real cart measured {REAL_WIDE_OVER_TALL}"
    );
}

#[test]
fn the_label_window_is_where_a_real_labels_is() {
    let (x, y, w, h) = GENESIS_LABEL;
    let (cw, ch) = (GENESIS_W as f32, GENESIS_H as f32);
    assert!(
        near(w as f32 / cw, REAL_LABEL_OF_WIDTH, 0.015),
        "{} of the width",
        w as f32 / cw
    );
    assert!(
        near((y + h) as f32 / ch, REAL_LABEL_FOOT_OF_HEIGHT, 0.02),
        "its foot is {} of the way down",
        (y + h) as f32 / ch
    );
    assert!(
        near(x as f32 / cw, REAL_PILLAR_OF_WIDTH, 0.02),
        "a pillar is {} of the width",
        x as f32 / cw
    );
    assert_eq!(
        x + w + x,
        GENESIS_W,
        "the label is not between two equal pillars"
    );
}

#[test]
fn the_label_is_80_5_millimetres_across_on_a_cart_108_5_wide() {
    let (_, _, w, _) = GENESIS_LABEL;
    let mm = w as f32 / GENESIS_W as f32 * 108.5;
    assert!(near(mm, 80.5, 1.0), "{mm:.1} mm");
}

#[test]
fn it_stands_beside_the_snes_carts_at_the_same_scale() {
    let px_per_mm_genesis = GENESIS_W as f32 / 108.5;
    let (snes_w, _) = cart_box(Platform::Snes);
    let px_per_mm_snes = snes_w as f32 / 130.0;
    assert!(
        near(px_per_mm_genesis, px_per_mm_snes, 0.15),
        "{px_per_mm_genesis:.2} against {px_per_mm_snes:.2} pixels to the millimetre"
    );
    for p in BOTH {
        assert_eq!(cart_box(p), (GENESIS_W, GENESIS_H), "{p:?}");
        assert_eq!(shelf_scale(p), 1.0, "{p:?}");
    }
}

#[test]
fn it_is_seated_at_its_own_proportions() {
    let (w, h) = seated_box(Platform::Genesis);
    assert_eq!(seated_box(Platform::MegaDrive), (w, h));
    assert!(
        near(w as f32 / h as f32, REAL_WIDE_OVER_TALL, 0.03),
        "{w}x{h}"
    );
}

#[test]
fn the_silhouette_is_a_rounded_rectangle() {
    let mask = genesis_silhouette(GENESIS_W, GENESIS_H);
    let at = |x: u32, y: u32| mask[(y * GENESIS_W + x) as usize];
    // the corners are cut away, the middle of every edge is not
    for (x, y) in [
        (0, 0),
        (GENESIS_W - 1, 0),
        (0, GENESIS_H - 1),
        (GENESIS_W - 1, GENESIS_H - 1),
    ] {
        assert!(at(x, y) < 128, "the corner {x},{y} is square");
    }
    for (x, y) in [
        (GENESIS_W / 2, 0),
        (GENESIS_W / 2, GENESIS_H - 1),
        (0, GENESIS_H / 2),
        (GENESIS_W - 1, GENESIS_H / 2),
        (GENESIS_W / 2, GENESIS_H / 2),
    ] {
        assert!(at(x, y) > 200, "the edge or middle at {x},{y} is cut away");
    }
}

#[test]
fn the_corners_are_about_five_millimetres_round() {
    let mask = genesis_silhouette(GENESIS_W, GENESIS_H);
    let px_per_mm = GENESIS_W as f32 / 108.5;
    // on the top row, the first opaque pixel is a corner's radius in from the side
    let first = (0..GENESIS_W).find(|&x| mask[x as usize] > 128).unwrap() as f32;
    let radius_mm = first / px_per_mm;
    assert!(
        radius_mm > 1.0 && radius_mm < 5.5,
        "{radius_mm:.1} mm in along the top row"
    );
    // 3 px down, the edge has come most of the way in
    let lower = (0..GENESIS_W)
        .find(|&x| mask[(3 * GENESIS_W + x) as usize] > 128)
        .unwrap() as f32;
    assert!(lower < first, "the corner does not curve");
}

#[test]
fn it_is_black_where_a_super_famicom_cart_is_grey() {
    for p in BOTH {
        assert_eq!(shell_for(&cart(p, "Sonic")), GENESIS_SHELL, "{p:?}");
    }
    assert!(
        GENESIS_SHELL.colour.iter().all(|c| *c < 0x40),
        "the Genesis cart is not dark"
    );
}

#[test]
fn its_label_is_cut_to_the_one_size_and_the_snes_carts_keep_theirs() {
    let (_, _, w, h) = GENESIS_LABEL;
    for p in BOTH {
        assert_eq!(face_labels(p), vec![(w, h)], "{p:?}");
    }
    assert_eq!(face_labels(Platform::Snes).len(), 2);
    assert!(!face_labels(Platform::Snes).contains(&(w, h)));
}

#[test]
fn it_opens_on_a_board_with_the_snes_sockets_for_now() {
    for p in BOTH {
        assert_eq!(sockets_for(p), SNES_SOCKETS, "{p:?}");
    }
    assert_eq!(sockets_of(Core::GenesisPlusGx), SNES_SOCKETS);
    let face = board_face(&cart(Platform::Genesis, "Sonic"));
    assert_eq!((face.w, face.h), (BOARD_W, BOARD_H));
}

#[test]
fn the_console_names_are_what_a_real_cart_of_each_carries() {
    assert_eq!(console_name(Platform::Genesis), "GENESIS");
    assert_eq!(console_name(Platform::MegaDrive), "MEGA DRIVE");
    for p in Platform::ALL.into_iter().filter(|p| !p.sega_16bit()) {
        assert_eq!(console_name(p), "", "{p:?}");
    }
}

/// The share of the pixels in rows `from..to` of a face that are light, like printed silver.
fn light_in(face: &omega_ui::CartFace, from: u32, to: u32) -> usize {
    (from..to)
        .flat_map(|y| (0..face.w).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let i = ((y * face.w + x) * 4) as usize;
            face.rgba[i..i + 3].iter().map(|c| *c as u32).sum::<u32>() / 3 > 150
        })
        .count()
}

#[test]
fn the_name_is_printed_on_the_grid_band_and_nowhere_else() {
    let g = cart_face(&cart(Platform::Genesis, "Sonic"));
    let m = cart_face(&cart(Platform::MegaDrive, "Sonic"));
    assert!(
        light_in(&g, BAND_TOP, BAND_BOTTOM) > 30,
        "no name on the Genesis band"
    );
    assert!(
        light_in(&m, BAND_TOP, BAND_BOTTOM) > 30,
        "no name on the Mega Drive band"
    );
    // the two carts differ on the band, and only there
    for y in 0..g.h {
        for x in 0..g.w {
            let i = ((y * g.w + x) * 4) as usize;
            let same = g.rgba[i..i + 4] == m.rgba[i..i + 4];
            if !(BAND_TOP..BAND_BOTTOM).contains(&y) {
                assert!(same, "the carts differ at {x},{y}, outside the band");
            }
        }
    }
    assert_ne!(
        g.rgba, m.rgba,
        "a Genesis cart and a Mega Drive cart look the same"
    );
}

#[test]
fn the_name_is_centred_on_the_cart() {
    for p in BOTH {
        let f = cart_face(&cart(p, "Sonic"));
        let mut lit: Vec<u32> = Vec::new();
        for y in BAND_TOP..BAND_BOTTOM {
            for x in 0..f.w {
                let i = ((y * f.w + x) * 4) as usize;
                if f.rgba[i..i + 3].iter().map(|c| *c as u32).sum::<u32>() / 3 > 150 {
                    lit.push(x);
                }
            }
        }
        let (a, b) = (
            *lit.iter().min().unwrap() as f32,
            *lit.iter().max().unwrap() as f32,
        );
        assert!(
            near((a + b) / 2.0, f.w as f32 / 2.0, 3.0),
            "{p:?}: {a}..{b} on a face {} across",
            f.w
        );
    }
}

#[test]
fn the_label_art_goes_where_the_window_is_and_leaves_the_pillars_black() {
    let f = cart_face(&cart(Platform::Genesis, "Sonic"));
    let (x, y, _, _) = GENESIS_LABEL;
    let px = |x: u32, y: u32| {
        let i = ((y * f.w + x) * 4) as usize;
        [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2]]
    };
    // a corner of the window, clear of the title the label has printed in its middle
    let c = cart(Platform::Genesis, "Sonic");
    let want = label_colour(&label_text(&c));
    let inside = px(x + 6, y + 6);
    assert_eq!(inside, want, "the label window is not the label's colour");
    for (cx, cy) in [(x / 2, f.h / 2), (f.w - x / 2, f.h / 2)] {
        assert!(
            px(cx, cy).iter().all(|c| *c < 0x50),
            "a pillar at {cx},{cy} is not black"
        );
    }
}

#[test]
fn a_cart_of_another_console_is_not_given_a_name() {
    for p in [Platform::Gba, Platform::Snes] {
        let f = cart_face(&cart(p, "Sonic"));
        assert_eq!((f.w, f.h), cart_box(p));
    }
}
