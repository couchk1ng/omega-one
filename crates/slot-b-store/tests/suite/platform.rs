use slot_b_store::Platform;

#[test]
fn every_platform_has_a_directory() {
    let names: Vec<&str> = Platform::ALL.iter().map(|p| p.dir_name()).collect();
    assert_eq!(
        names,
        vec!["GBA", "GB", "GBC", "SNES", "PS1", "Genesis", "MD"]
    );
}

#[test]
fn every_platform_is_a_shelf_of_its_own() {
    assert_eq!(
        Platform::ALL,
        [
            Platform::Gba,
            Platform::Gb,
            Platform::Gbc,
            Platform::Snes,
            Platform::Ps1,
            Platform::Genesis,
            Platform::MegaDrive
        ]
    );
    for (i, p) in Platform::ALL.iter().enumerate() {
        assert!(
            !Platform::ALL[..i].contains(p),
            "{p:?} is named twice in the ring, so two shelves would hold one platform"
        );
    }
}

#[test]
fn each_platform_takes_only_its_own_extensions() {
    assert!(Platform::Gba.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(!Platform::Gba.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gb")));
    assert!(Platform::Gb.accepts(std::path::Path::new("Tetris.gbc")));
    assert!(!Platform::Gb.accepts(std::path::Path::new("Metroid Fusion.gba")));
    assert!(Platform::Gba.accepts(std::path::Path::new("Shrek.GBA")));
}

#[test]
fn the_genesis_and_the_mega_drive_take_the_same_files_and_no_others() {
    for p in [Platform::Genesis, Platform::MegaDrive] {
        for name in ["Sonic.md", "Sonic.gen", "Sonic.SMD", "Sonic.bin"] {
            assert!(p.accepts(std::path::Path::new(name)), "{p:?} {name}");
        }
        for name in [
            "Zelda.sfc",
            "Crash.cue",
            "Tetris.gb",
            "Fusion.gba",
            "notes.txt",
        ] {
            assert!(!p.accepts(std::path::Path::new(name)), "{p:?} {name}");
        }
    }
    assert!(!Platform::Snes.accepts(std::path::Path::new("Sonic.md")));
    assert!(!Platform::Ps1.accepts(std::path::Path::new("Sonic.bin")));
}

#[test]
fn the_genesis_and_the_mega_drive_are_one_machine_with_a_folder_each() {
    assert_eq!(Platform::Genesis.dir_name(), "Genesis");
    assert_eq!(Platform::MegaDrive.dir_name(), "MD");
    assert_eq!(Platform::Genesis.picture(), (320, 224));
    assert_eq!(Platform::MegaDrive.picture(), (320, 224));
    for p in Platform::ALL {
        assert_eq!(
            p.sega_16bit(),
            matches!(p, Platform::Genesis | Platform::MegaDrive),
            "{p:?}"
        );
    }
}

#[test]
fn the_genesis_keeps_its_own_x_y_and_shoulders_and_rewinds() {
    for p in [Platform::Genesis, Platform::MegaDrive] {
        assert!(!p.turbo_xy(), "X and Y are the game's own on a {p:?}");
        assert!(!p.owns_shoulders(), "L1 and R1 are the game's on a {p:?}");
        assert!(p.rewinds() && !p.game_owns_l2_r2(), "{p:?}");
        assert!(!p.colour_correction(), "{p:?}");
    }
}
