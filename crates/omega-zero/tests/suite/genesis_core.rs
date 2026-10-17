//! Runs the real Genesis Plus GX core through omega-zero's own host. Skipped where the core has not
//! been built for this host, like the other tests that open a core.

use std::path::PathBuf;

use omega_retro::{ButtonMask, LibretroCore, RetroCore, MAX_FRAME_H, MAX_FRAME_W};
use omega_store::Core;

use crate::common;

fn core_path() -> Option<PathBuf> {
    let p = common::repo_root().join(format!(
        "vendor/genesis_plus_gx_libretro.{}",
        std::env::consts::DLL_EXTENSION
    ));
    p.exists().then_some(p)
}

/// The smallest ROM the core will run: a header, a reset vector, and a program that turns the
/// display on in the video chip's Mega Drive mode, then its 40 column mode, and waits. The chip
/// ignores the column setting until the Mega Drive mode is on.
fn rom(regions: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x400];
    rom[0..4].copy_from_slice(&0x00ff_0000u32.to_be_bytes());
    for vector in 1..64 {
        rom[vector * 4..vector * 4 + 4].copy_from_slice(&0x0000_0200u32.to_be_bytes());
    }
    rom[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    rom[0x110..0x180].fill(b' ');
    rom[0x150..0x150 + 9].copy_from_slice(b"TEST ROM ");
    rom[0x1a0..0x1a4].copy_from_slice(&0u32.to_be_bytes());
    rom[0x1a4..0x1a8].copy_from_slice(&0x0000_03ffu32.to_be_bytes());
    rom[0x1f0..0x1f0 + regions.len()].copy_from_slice(regions);
    let program: [u8; 18] = [
        0x33, 0xfc, 0x81, 0x64, 0x00, 0xc0, 0x00, 0x04, // move.w #$8164,$c00004
        0x33, 0xfc, 0x8c, 0x81, 0x00, 0xc0, 0x00, 0x04, // move.w #$8c81,$c00004
        0x60, 0xfe, // bra.s *
    ];
    rom[0x200..0x200 + program.len()].copy_from_slice(&program);
    rom
}

fn run_in(regions: &[u8], options: &[(&str, &str)]) -> Option<LibretroCore> {
    let path = core_path().or_else(|| {
        eprintln!("no Genesis Plus GX core on this host, skipping");
        None
    })?;
    let d = tempfile::tempdir().unwrap();
    let rom_path = d.path().join("Test.md");
    std::fs::write(&rom_path, rom(regions)).unwrap();
    let mut core = LibretroCore::open_with(&path, d.path(), d.path()).expect("open the core");
    omega_zero::core::apply_core_options(&mut core, Core::GenesisPlusGx, "auto", false, false);
    for (key, value) in options {
        core.set_option(key, value);
    }
    core.load(&rom_path).expect("load the rom");
    for _ in 0..40 {
        core.run_frame(ButtonMask::default());
    }
    Some(core)
}

fn run(options: &[(&str, &str)]) -> Option<LibretroCore> {
    run_in(b"JUE", options)
}

#[test]
fn the_core_hands_over_a_whole_forty_column_picture() {
    let _g = common::core_lock();
    let Some(core) = run(&[]) else { return };
    assert_eq!(core.video_size(), (320, 224));
    assert_eq!(core.video_xrgb8888().len(), 320 * 224 * 4);
}

#[test]
fn the_core_runs_at_sixty_hertz_with_forty_four_thousand_samples_a_second() {
    let _g = common::core_lock();
    let Some(mut core) = run(&[]) else {
        return;
    };
    let av = core.av_info();
    assert!((av.fps - 59.92).abs() < 0.1, "{} frames a second", av.fps);
    assert_eq!(av.sample_rate, 44_100.0);
    assert!(
        !core.take_audio().is_empty(),
        "the core made no sound at all"
    );
}

#[test]
fn forcing_the_video_chip_to_fifty_hertz_makes_the_core_a_fifty_hertz_one() {
    let _g = common::core_lock();
    let Some(core) = run(&[("genesis_plus_gx_vdp_mode", "50hz")]) else {
        return;
    };
    let fps = core.av_info().fps;
    assert!((fps - 50.0).abs() < 0.5, "{fps} frames a second");
}

#[test]
fn forcing_sixty_hertz_keeps_the_core_at_sixty() {
    let _g = common::core_lock();
    let Some(core) = run(&[("genesis_plus_gx_vdp_mode", "60hz")]) else {
        return;
    };
    assert!((core.av_info().fps - 59.92).abs() < 0.1);
}

#[test]
fn the_picture_is_never_bigger_than_omega_zero_takes() {
    let _g = common::core_lock();
    let Some(core) = run(&[]) else { return };
    let (w, h) = core.video_size();
    assert!(w <= MAX_FRAME_W && h <= MAX_FRAME_H, "{w}x{h}");
}

#[test]
fn a_european_only_rom_runs_at_fifty_hertz_without_being_asked() {
    let _g = common::core_lock();
    let Some(core) = run_in(b"E  ", &[]) else {
        return;
    };
    let fps = core.av_info().fps;
    assert!((fps - 50.0).abs() < 0.5, "{fps} frames a second");
}

#[test]
fn a_world_rom_runs_at_sixty() {
    let _g = common::core_lock();
    let Some(core) = run_in(b"JUE", &[]) else {
        return;
    };
    assert!((core.av_info().fps - 59.92).abs() < 0.1);
}

#[test]
fn every_option_omega_zero_sets_on_it_is_one_the_core_declares() {
    let _g = common::core_lock();
    let Some(path) = core_path() else { return };
    let core = LibretroCore::open(&path).expect("open the core");
    let mut core = core;
    omega_zero::core::apply_core_options(&mut core, Core::GenesisPlusGx, "auto", false, false);
    for (key, value) in omega_zero::core::core_options(Core::GenesisPlusGx, "auto", false, false) {
        let declared = core.declared_options();
        let values = declared
            .get(key)
            .unwrap_or_else(|| panic!("the core declares no {key}"));
        assert!(
            values.iter().any(|v| *v == value),
            "{key} takes {values:?}, not {value}"
        );
    }
}
