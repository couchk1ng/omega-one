//! The Genesis and Mega Drive cart, one model for both, drawn from a retail Mega Drive cart
//! photographed square on: about 1.465 times as wide as it is tall, a label window 74% of its
//! width between two raised pillars, a grid band under it that carries the console's name.

use std::path::PathBuf;

use slot_b_store::{Cart, Core, Platform};
use slot_b_ui::{
    board_face, cart_box, cart_face, cart_mesh, console_name, face_labels, genesis_roll_z,
    genesis_silhouette, label_colour, label_text, model_box, model_of, seated_box, shelf_scale,
    shell_for, sockets_for, sockets_of, CartModel, BAND_BOTTOM, BAND_LINE, BAND_PANEL, BAND_TOP,
    BOARD_H, BOARD_W, GENESIS_ART_H, GENESIS_H, GENESIS_LABEL, GENESIS_ROLL_MM, GENESIS_SHELL,
    GENESIS_W, SNES_SOCKETS,
};

/// What the photograph of the real cart measured, as shares: its width over its height, the
/// label window's share of the width and of the height, and a pillar's share of the width.
const REAL_WIDE_OVER_TALL: f32 = 1.465;
const REAL_LABEL_OF_WIDTH: f32 = 0.74;
const REAL_ART_FOOT_OF_HEIGHT: f32 = 0.727;
const REAL_RECESS_FOOT_OF_HEIGHT: f32 = 0.90;
/// A scan of a real label from ScreenScraper, and the recess measured on the cart: both are
/// wider than they are tall by about this much.
const REAL_SCAN_WIDE_OVER_TALL: f32 = 1.289;
const REAL_RECESS_WIDE_OVER_TALL: f32 = 1.205;
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
fn the_label_window_is_the_whole_recess_where_a_real_labels_is() {
    let (x, y, w, h) = GENESIS_LABEL;
    let (cw, ch) = (GENESIS_W as f32, GENESIS_H as f32);
    assert!(
        near(w as f32 / cw, REAL_LABEL_OF_WIDTH, 0.015),
        "{} of the width",
        w as f32 / cw
    );
    assert!(
        near((y + h) as f32 / ch, REAL_RECESS_FOOT_OF_HEIGHT, 0.03),
        "the recess ends {} of the way down",
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
    // it is the shape of a real label: between the recess measured and a scan of a label
    let shape = w as f32 / h as f32;
    assert!(
        shape > REAL_RECESS_WIDE_OVER_TALL - 0.03 && shape < REAL_SCAN_WIDE_OVER_TALL + 0.03,
        "{shape:.3} wide for each tall, between {REAL_RECESS_WIDE_OVER_TALL} and {REAL_SCAN_WIDE_OVER_TALL}"
    );
}

#[test]
fn the_art_ends_where_the_grid_band_of_a_real_label_begins() {
    let (_, y, _, _) = GENESIS_LABEL;
    let foot = (y + GENESIS_ART_H) as f32 / GENESIS_H as f32;
    assert!(
        near(foot, REAL_ART_FOOT_OF_HEIGHT, 0.03),
        "the art ends {foot} of the way down"
    );
    assert_eq!(BAND_TOP, y + GENESIS_ART_H);
    assert_eq!(BAND_BOTTOM, y + GENESIS_LABEL.3);
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
fn light_in(face: &slot_b_ui::CartFace, from: u32, to: u32) -> usize {
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

// ---- the roll of the ends: the front surface curves back to the full width, as a real cart's does

const MM: f32 = 108.5;

/// The vertices of the face: the ones laid over the texture so that where each is drawn from the
/// front is where it is in the texture. A wall's take their colour from just inside its edge.
fn face_vertices() -> Vec<slot_b_gfx::MeshVertex> {
    cart_mesh(CartModel::Genesis)
        .into_iter()
        .filter(|v| v.uv == [v.xyz[0], v.xyz[1]])
        .collect()
}

#[test]
fn the_face_is_flat_across_the_label_and_rolls_back_at_both_ends() {
    let (across, deep) = GENESIS_ROLL_MM;
    let face = face_vertices();
    assert!(!face.is_empty());
    for v in &face {
        let u = v.xyz[0];
        if (across / MM..=1.0 - across / MM).contains(&u) {
            assert_eq!(v.xyz[2], 0.0, "the face is not flat at {u}");
        }
    }
    let deepest = face.iter().map(|v| v.xyz[2]).fold(0.0f32, f32::max);
    assert!(
        (deepest * MM - deep).abs() < 0.05,
        "{} mm back at the edge",
        deepest * MM
    );
    for edge in [0.0, 1.0] {
        let z = face
            .iter()
            .filter(|v| v.xyz[0] == edge)
            .map(|v| v.xyz[2])
            .fold(0.0f32, f32::max);
        assert!(
            (z * MM - deep).abs() < 0.05,
            "the edge at {edge} is {} mm back",
            z * MM
        );
    }
}

#[test]
fn the_roll_is_as_wide_as_a_pillar_and_the_same_at_both_ends() {
    let (across, _) = GENESIS_ROLL_MM;
    let units_per_mm = 434.0 / MM;
    assert_eq!(genesis_roll_z(across * units_per_mm), 0.0);
    assert!(genesis_roll_z((across - 0.5) * units_per_mm) > 0.0);
    for mm in [0.0, 1.0, 3.0, 6.0, 9.0, 12.0] {
        let (left, right) = (
            genesis_roll_z(mm * units_per_mm),
            genesis_roll_z((MM - mm) * units_per_mm),
        );
        assert!(
            (left - right).abs() < 1e-5,
            "{mm} mm in: {left} against {right}"
        );
    }
    // it is as wide as a pillar is on the face
    assert!(
        (across / MM - 39.0 / 300.0).abs() < 0.02,
        "{} of the width",
        across / MM
    );
}

#[test]
fn the_roll_leaves_the_flat_level_and_meets_the_edge_steep() {
    let (across, deep) = GENESIS_ROLL_MM;
    let units_per_mm = 434.0 / MM;
    let depth_at = |mm: f32| genesis_roll_z(mm * units_per_mm) * MM;
    // never comes forward going out towards the edge
    let steps: Vec<f32> = (0..=26)
        .map(|i| depth_at(across - i as f32 * across / 26.0))
        .collect();
    assert!(steps.windows(2).all(|w| w[1] >= w[0] - 1e-4), "{steps:?}");
    // a millimetre past the start of the roll it has barely left the flat
    assert!(
        depth_at(across - 1.0) < 0.25,
        "{} mm back a mm in",
        depth_at(across - 1.0)
    );
    // a millimetre from the edge it has gone most of the way back
    assert!(
        depth_at(1.0) > 0.6 * deep,
        "{} mm back a mm from the edge",
        depth_at(1.0)
    );
}

#[test]
fn seen_from_the_front_the_face_is_the_texture_as_drawn() {
    for v in face_vertices() {
        assert_eq!(
            [v.xyz[0], v.xyz[1]],
            v.uv,
            "the face is stretched at {:?}",
            v.xyz
        );
    }
    // and it spans the whole of the drawing
    let face = face_vertices();
    let (lo, hi) = face.iter().fold((1.0f32, 0.0f32), |(l, h), v| {
        (l.min(v.xyz[0]), h.max(v.xyz[0]))
    });
    assert_eq!((lo, hi), (0.0, 1.0));
}

#[test]
fn every_vertex_of_the_face_has_the_one_normal_of_a_flat_face() {
    for v in face_vertices() {
        assert_eq!(v.normal, [0.0, 0.0, -1.0], "at {:?}", v.xyz);
    }
}

#[test]
fn every_polygon_of_the_mesh_has_one_normal() {
    // the renderer folds a polygon away by testing each of its vertices' normals against the eye,
    // so one whose vertices disagree is stretched across the screen as it turns
    for model in CartModel::ALL.into_iter().filter(|m| *m != CartModel::Disc) {
        for (i, tri) in cart_mesh(model).chunks(3).enumerate() {
            assert!(
                tri[0].normal == tri[1].normal && tri[1].normal == tri[2].normal,
                "{model:?}: polygon {i} has different normals at its corners: {:?}",
                tri.iter().map(|v| v.normal).collect::<Vec<_>>()
            );
        }
    }
}

/// The model vertex shader's own test for a vertex facing away from the eye, for a model drawn
/// `w` by `h` and spun `yaw` about its upright middle.
fn faces_away(v: &slot_b_gfx::MeshVertex, w: f32, h: f32, yaw: f32) -> bool {
    let (lx, ly) = (v.xyz[0] * w - w * 0.5, v.xyz[1] * h - h * 0.5);
    let depth = v.xyz[2] * w;
    let (s, c) = yaw.sin_cos();
    let across = lx * c - depth * s;
    let back = lx * s + depth * c;
    let n = [
        v.normal[0] * c - v.normal[2] * s,
        v.normal[1],
        v.normal[0] * s + v.normal[2] * c,
    ];
    n[0] * across + n[1] * ly + n[2] * (back + slot_b_gfx::FOCAL) >= 0.0
}

#[test]
fn no_polygon_is_part_towards_the_eye_and_part_away_at_any_angle_of_the_sway() {
    // the shelf sways a cart 20 degrees either way and leans it up to 20 more into a scroll
    for model in CartModel::ALL {
        let (w, h) = model_box(model);
        let mesh = cart_mesh(model);
        for step in -140..=140 {
            let yaw = step as f32 * 0.005;
            let torn = mesh
                .chunks(3)
                .filter(|tri| {
                    let away: Vec<bool> = tri
                        .iter()
                        .map(|v| faces_away(v, w as f32, h as f32, yaw))
                        .collect();
                    away.iter().any(|a| *a) && away.iter().any(|a| !*a)
                })
                .count();
            assert_eq!(
                torn, 0,
                "{model:?} at {yaw:.3} rad has {torn} polygons stretched across the screen"
            );
        }
    }
}

#[test]
fn the_roll_is_shaded_on_the_end_the_light_falls_away_from_and_nowhere_else() {
    let (_, deep) = GENESIS_ROLL_MM;
    for v in face_vertices() {
        let (u, z) = (v.xyz[0], v.xyz[2]);
        assert!(v.shade > 0.0 && v.shade <= 1.0, "{} at {u}", v.shade);
        if u < 0.5 || z == 0.0 {
            assert_eq!(v.shade, 1.0, "the face is shaded at {u}");
        } else {
            let want = 1.0 - 0.2 * (z * MM / deep);
            assert!(
                (v.shade - want).abs() < 1e-4,
                "{} at {u}, wanted {want}",
                v.shade
            );
        }
    }
    let darkest = face_vertices()
        .iter()
        .map(|v| v.shade)
        .fold(1.0f32, f32::min);
    assert!(darkest > 0.75 && darkest < 0.9, "{darkest}");
}

#[test]
fn the_sides_run_from_the_face_back_to_the_full_thickness() {
    let thick = CartModel::Genesis.thick();
    let walls: Vec<_> = cart_mesh(CartModel::Genesis)
        .into_iter()
        .filter(|v| v.normal[2] == 0.0)
        .collect();
    assert!(!walls.is_empty());
    for v in &walls {
        let front = genesis_roll_z(v.xyz[0] * 434.0);
        let on_front = (v.xyz[2] - front).abs() < 1e-4;
        let on_back = (v.xyz[2] - thick).abs() < 1e-4;
        assert!(
            on_front || on_back,
            "{:?} is neither on the face nor at the back",
            v.xyz
        );
    }
    assert!(
        walls.iter().any(|v| (v.xyz[2] - thick).abs() < 1e-4),
        "nothing reaches the back"
    );
}

#[test]
fn the_mesh_stays_inside_its_box_and_is_still_few_polygons() {
    let mesh = cart_mesh(CartModel::Genesis);
    let thick = CartModel::Genesis.thick();
    assert_eq!(mesh.len() % 3, 0);
    for v in &mesh {
        assert!(
            v.xyz.iter().all(|c| c.is_finite()) && v.normal.iter().all(|c| c.is_finite()),
            "{v:?}"
        );
        assert!(
            (0.0..=1.0).contains(&v.xyz[0]) && (0.0..=1.0).contains(&v.xyz[1]),
            "{:?}",
            v.xyz
        );
        assert!((0.0..=thick + 1e-4).contains(&v.xyz[2]), "{:?}", v.xyz);
    }
    assert!(
        mesh.len() / 3 < 400,
        "{} triangles is not few",
        mesh.len() / 3
    );
}

#[test]
fn no_other_model_has_a_rolled_face() {
    for model in CartModel::ALL
        .into_iter()
        .filter(|m| !matches!(m, CartModel::Genesis | CartModel::Disc))
    {
        for v in cart_mesh(model).into_iter().filter(|v| v.normal[2] < -0.2) {
            assert_eq!(v.xyz[2], 0.0, "{model:?} has a face that is not flat");
        }
    }
}

// ---- a scan of a real label is the whole of it, and nothing of ours goes over it

fn write_scan(dir: &std::path::Path, w: u32, h: u32, rgb: [u8; 3]) -> PathBuf {
    let path = dir.join("scan.png");
    let mut data = Vec::with_capacity((w * h * 3) as usize);
    for _ in 0..w * h {
        data.extend_from_slice(&rgb);
    }
    let f = std::fs::File::create(&path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&data).unwrap();
    path
}

fn pixel(f: &slot_b_ui::CartFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * f.w + x) * 4) as usize;
    [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2]]
}

#[test]
fn a_label_scan_fills_the_whole_recess_with_nothing_printed_over_it() {
    let d = tempfile::tempdir().unwrap();
    // a scan of a real label comes about 1.29 wide for each tall, so is cropped a little to fit
    let want = [200, 40, 40];
    let scan = write_scan(d.path(), 238, 190, want);
    for p in BOTH {
        let mut c = cart(p, "Aladdin");
        c.label = Some(scan.clone());
        let f = cart_face(&c);
        let (x, y, w, h) = GENESIS_LABEL;
        for py in y..y + h {
            for px in x..x + w {
                assert_eq!(
                    pixel(&f, px, py),
                    want,
                    "{p:?}: the label is not all there at {px},{py}"
                );
            }
        }
    }
}

#[test]
fn a_label_scan_makes_a_genesis_and_a_mega_drive_cart_the_same() {
    // the console's name is on the scan, in the logo, so the faces are one
    let d = tempfile::tempdir().unwrap();
    let scan = write_scan(d.path(), 238, 190, [30, 90, 200]);
    let (mut g, mut m) = (
        cart(Platform::Genesis, "Aladdin"),
        cart(Platform::MegaDrive, "Aladdin"),
    );
    g.label = Some(scan.clone());
    m.label = Some(scan);
    assert_eq!(cart_face(&g).rgba, cart_face(&m).rgba);
}

#[test]
fn a_generated_label_has_a_dark_panel_with_a_grid_for_the_name_to_stand_on() {
    let f = cart_face(&cart(Platform::Genesis, "Sonic"));
    let (x, _, _, _) = GENESIS_LABEL;
    // between the lines of the grid the panel, on them the line, well clear of the name
    assert_eq!(pixel(&f, x + 2, BAND_TOP + 1), BAND_PANEL);
    assert_eq!(pixel(&f, x + 5, BAND_TOP + 1), BAND_LINE);
    assert_eq!(pixel(&f, x + 2, BAND_TOP + 4), BAND_LINE);
}
