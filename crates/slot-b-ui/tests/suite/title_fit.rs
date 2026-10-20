use slot_b_ui::kit::pixel::{fit_text, text_width, ADVANCE};

#[test]
fn a_name_that_fits_is_left_alone() {
    assert_eq!(fit_text("Streets of Rage 2", 336), "Streets of Rage 2");
    assert_eq!(fit_text("", 336), "");
}

#[test]
fn a_name_exactly_as_wide_as_the_room_is_left_alone() {
    let name = "Sonic The Hedgehog 3";
    assert_eq!(fit_text(name, text_width(name)), name);
}

#[test]
fn a_name_too_wide_is_cut_and_ends_in_dots_that_fit() {
    let long = "Disney's Aladdin Special Collector's Edition Director's Cut Remastered";
    for room in [120u32, 200, 336] {
        let fitted = fit_text(long, room);
        assert!(fitted.ends_with("..."), "{fitted:?} is not marked as cut");
        assert!(
            text_width(&fitted) <= room,
            "{fitted:?} is {} wide in {room}",
            text_width(&fitted)
        );
        assert!(
            long.starts_with(fitted.trim_end_matches('.')),
            "{fitted:?} is not the start of the name"
        );
        // as much as fits: the next real letter would not
        let kept = fitted.trim_end_matches('.');
        let mut next = kept.len();
        while long[next..].starts_with(' ') {
            next += 1;
        }
        let more = format!("{}...", &long[..next + 1]);
        assert!(
            text_width(&more) > room,
            "{fitted:?} left room for another letter"
        );
    }
}

#[test]
fn a_cut_name_does_not_end_in_a_space_before_its_dots() {
    let fitted = fit_text(
        "Super Street Fighter II Turbo",
        text_width("Super Street Fighter...") + 1,
    );
    assert!(!fitted.contains(" ..."), "{fitted:?}");
}

#[test]
fn a_room_too_small_for_even_the_dots_still_gives_back_something() {
    assert_eq!(fit_text("Sonic", 2), "...");
    assert!(text_width("...") >= 3 * ADVANCE as u32 - 3);
}
