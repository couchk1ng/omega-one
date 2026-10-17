use omega_store::genesis;

fn header(overseas: &str, domestic: &str) -> Vec<u8> {
    let mut h = vec![0u8; 0x200];
    h[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    h[0x120..0x150].fill(b' ');
    h[0x150..0x180].fill(b' ');
    h[0x120..0x120 + domestic.len()].copy_from_slice(domestic.as_bytes());
    h[0x150..0x150 + overseas.len()].copy_from_slice(overseas.as_bytes());
    h
}

#[test]
fn the_overseas_title_is_preferred() {
    let h = header("SONIC THE HEDGEHOG", "ソニック");
    assert_eq!(genesis::title_in(&h).as_deref(), Some("SONIC THE HEDGEHOG"));
}

#[test]
fn the_domestic_title_is_used_when_there_is_no_overseas_one() {
    let h = header("", "ALEX KIDD");
    assert_eq!(genesis::title_in(&h).as_deref(), Some("ALEX KIDD"));
}

#[test]
fn runs_of_padding_collapse_into_single_spaces() {
    let h = header("STREETS  OF     RAGE 2", "");
    assert_eq!(genesis::title_in(&h).as_deref(), Some("STREETS OF RAGE 2"));
}

#[test]
fn a_rom_without_the_sega_marker_has_no_title() {
    let mut h = header("NOT A MEGA DRIVE GAME", "");
    h[0x100..0x104].copy_from_slice(b"XXXX");
    assert_eq!(genesis::title_in(&h), None);
}

#[test]
fn a_header_of_blanks_has_no_title() {
    assert_eq!(genesis::title_in(&header("", "")), None);
}

#[test]
fn a_file_too_short_for_a_header_has_no_title() {
    let d = tempfile::tempdir().unwrap();
    let rom = d.path().join("Short.md");
    std::fs::write(&rom, [0u8; 64]).unwrap();
    assert_eq!(genesis::title(&rom), None);
    assert_eq!(genesis::title_in(&[0u8; 64]), None);
}

#[test]
fn a_rom_file_is_read_from_disk() {
    let d = tempfile::tempdir().unwrap();
    let rom = d.path().join("Columns.gen");
    std::fs::write(&rom, header("COLUMNS", "")).unwrap();
    assert_eq!(genesis::title(&rom).as_deref(), Some("COLUMNS"));
}

mod shelf {
    use crate::common::tmp_root;
    use omega_store::{scan, Platform};

    fn rom(title: &str) -> Vec<u8> {
        super::header(title, "")
    }

    fn write(d: &tempfile::TempDir, rel: &str, bytes: &[u8]) {
        let path = d.path().join("Games").join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn the_genesis_and_md_folders_are_two_shelves_of_one_machine() {
        let d = tmp_root();
        write(&d, "Genesis/Sonic 2.md", &rom("SONIC THE HEDGEHOG 2"));
        write(&d, "MD/Streets of Rage.gen", &rom("STREETS OF RAGE"));
        let carts = scan(d.path()).unwrap();
        let find = |p: Platform| carts.iter().find(|c| c.platform == p).unwrap();
        assert_eq!(find(Platform::Genesis).stem, "Sonic 2");
        assert_eq!(find(Platform::MegaDrive).stem, "Streets of Rage");
        assert_eq!(carts.len(), 2);
    }

    #[test]
    fn a_cart_takes_its_title_from_the_rom_header() {
        let d = tmp_root();
        write(&d, "Genesis/sonic2.md", &rom("SONIC THE HEDGEHOG 2"));
        let cart = scan(d.path()).unwrap().into_iter().next().unwrap();
        assert_eq!(cart.title, "SONIC THE HEDGEHOG 2");
        assert!(cart.code.is_empty());
    }

    #[test]
    fn every_extension_the_machine_takes_is_shelved_and_nothing_else() {
        let d = tmp_root();
        for name in ["a.md", "b.GEN", "c.smd", "d.bin"] {
            write(&d, &format!("Genesis/{name}"), &rom("X"));
        }
        for name in ["e.sfc", "f.cue", "g.txt", ".hidden.md"] {
            write(&d, &format!("Genesis/{name}"), &rom("X"));
        }
        let mut stems: Vec<String> = scan(d.path())
            .unwrap()
            .into_iter()
            .filter(|c| c.platform == Platform::Genesis)
            .map(|c| c.stem)
            .collect();
        stems.sort();
        assert_eq!(stems, ["a", "b", "c", "d"]);
    }

    #[test]
    fn a_label_in_one_folder_is_not_used_by_the_other() {
        let d = tmp_root();
        write(&d, "Genesis/Sonic.md", &rom("SONIC"));
        write(&d, "MD/Sonic.md", &rom("SONIC"));
        let labels = d.path().join("Labels/Genesis");
        std::fs::create_dir_all(&labels).unwrap();
        std::fs::write(labels.join("Sonic.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        for cart in scan(d.path()).unwrap() {
            assert_eq!(
                cart.label.is_some(),
                cart.platform == Platform::Genesis,
                "{:?}",
                cart.platform
            );
        }
    }

    #[test]
    fn the_genesis_has_its_own_core_and_no_other_core_runs_it() {
        use omega_store::Core;
        for p in [Platform::Genesis, Platform::MegaDrive] {
            assert_eq!(Core::for_platform(p), &[Core::GenesisPlusGx]);
            assert_eq!(Core::default_for(p), Core::GenesisPlusGx);
            for other in [
                Core::Mgba,
                Core::Gpsp,
                Core::Snes9x,
                Core::Supafaust,
                Core::PcsxRearmed,
            ] {
                assert!(!other.runs(p), "{other:?} runs {p:?}");
            }
        }
        for p in Platform::ALL.into_iter().filter(|p| !p.sega_16bit()) {
            assert!(!Core::GenesisPlusGx.runs(p), "{p:?}");
        }
        assert_eq!(Core::GenesisPlusGx.as_str(), "genesis_plus_gx");
        assert_eq!(Core::GenesisPlusGx.library(), "genesis_plus_gx");
        assert_eq!(Core::parse(" Genesis_Plus_GX "), Some(Core::GenesisPlusGx));
        assert!(
            Core::GenesisPlusGx.text().len() <= 6,
            "its chip has room for six letters"
        );
    }
}
