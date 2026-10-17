//! The Genesis and Mega Drive cart, which stands as a Super Famicom's in black until it has a
//! model of its own: every test here says what it is now, so a model of its own shows up.

use std::path::PathBuf;

use omega_store::{Cart, Core, Platform};
use omega_ui::{
    board_face, cart_box, cart_face, face_labels, model_of, seated_box, shelf_scale, shell_for,
    sockets_for, sockets_of, CartModel, SnesShell, BOARD_H, BOARD_W, GENESIS_SHELL, SFC_SHELL,
    SNES_SOCKETS,
};

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

#[test]
fn a_genesis_cart_stands_as_a_super_famicom_cart_for_now() {
    for p in BOTH {
        assert_eq!(
            model_of(&cart(p, "Sonic")),
            CartModel::Snes(SnesShell::Famicom),
            "{p:?}"
        );
    }
}

#[test]
fn the_genesis_and_the_mega_drive_share_a_model() {
    assert_eq!(
        model_of(&cart(Platform::Genesis, "Sonic")),
        model_of(&cart(Platform::MegaDrive, "Sonic"))
    );
}

#[test]
fn it_is_the_size_of_the_super_famicom_cart_on_the_shelf_and_in_the_handheld() {
    for p in BOTH {
        assert_eq!(cart_box(p), cart_box(Platform::Snes), "{p:?}");
        assert_eq!(seated_box(p), seated_box(Platform::Snes), "{p:?}");
        assert_eq!(shelf_scale(p), shelf_scale(Platform::Snes), "{p:?}");
    }
}

#[test]
fn it_is_black_where_a_super_famicom_cart_is_grey() {
    for p in BOTH {
        assert_eq!(shell_for(&cart(p, "Sonic")), GENESIS_SHELL, "{p:?}");
    }
    assert_ne!(GENESIS_SHELL.colour, SFC_SHELL.colour);
    assert!(
        GENESIS_SHELL.colour.iter().all(|c| *c < 0x40),
        "the Genesis cart is not dark"
    );
}

#[test]
fn a_snes_cart_is_still_grey() {
    assert_ne!(shell_for(&cart(Platform::Snes, "Zelda")), GENESIS_SHELL);
}

#[test]
fn its_label_is_cut_for_the_one_shell_it_wears_and_the_snes_for_both_of_its() {
    let sfc = face_labels(Platform::Genesis);
    assert_eq!(sfc.len(), 1, "{sfc:?}");
    assert_eq!(face_labels(Platform::MegaDrive), sfc);
    let snes = face_labels(Platform::Snes);
    assert_eq!(snes.len(), 2, "a SNES cart wears either shell: {snes:?}");
    assert!(snes.contains(&sfc[0]));
}

#[test]
fn it_opens_on_a_board_with_the_snes_sockets() {
    for p in BOTH {
        assert_eq!(sockets_for(p), SNES_SOCKETS, "{p:?}");
    }
    assert_eq!(sockets_of(Core::GenesisPlusGx), SNES_SOCKETS);
    let face = board_face(&cart(Platform::Genesis, "Sonic"));
    assert_eq!((face.w, face.h), (BOARD_W, BOARD_H));
}

#[test]
fn a_genesis_cart_with_no_label_wears_its_game_s_name() {
    let a = cart_face(&cart(Platform::Genesis, "Sonic The Hedgehog 2"));
    let b = cart_face(&cart(Platform::Genesis, "Columns"));
    assert_eq!((a.w, a.h), cart_box(Platform::Genesis));
    assert_ne!(a.rgba, b.rgba);
}
