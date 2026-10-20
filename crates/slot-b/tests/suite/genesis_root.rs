//! The folders a fresh card is given for the Genesis and the Mega Drive.

use slot_b::root;

#[test]
fn every_folder_a_platform_uses_is_made_for_the_genesis_and_the_mega_drive() {
    for dir in ["Genesis", "MD"] {
        for parent in ["Games", "Labels", "Overlays", "Saves", "States"] {
            let path = format!("{parent}/{dir}");
            assert!(
                root::DIRS.contains(&path.as_str()),
                "{path} is not made on a new card"
            );
        }
    }
}

#[test]
fn a_fresh_card_has_a_folder_for_every_platform_in_every_group() {
    for platform in slot_b_store::Platform::ALL {
        for parent in ["Games", "Labels", "Saves", "States"] {
            let path = format!("{parent}/{}", platform.dir_name());
            assert!(
                root::DIRS.contains(&path.as_str()),
                "{path} is not made on a new card"
            );
        }
    }
}

#[test]
fn ensure_makes_them() {
    let d = tempfile::tempdir().unwrap();
    root::ensure(d.path());
    for path in [
        "Games/Genesis",
        "Games/MD",
        "Saves/Genesis",
        "States/MD",
        "Labels/Genesis",
    ] {
        assert!(d.path().join(path).is_dir(), "{path}");
    }
}

#[test]
fn no_folder_is_named_twice() {
    let mut seen = std::collections::HashSet::new();
    for dir in root::DIRS {
        assert!(seen.insert(dir), "{dir} is listed twice");
    }
}
