use slot_b_store::display_title;

#[test]
fn the_tags_a_set_of_roms_puts_after_a_name_are_taken_off() {
    for (stem, want) in [
        ("Streets of Rage 2 (USA)", "Streets of Rage 2"),
        ("Streets of Rage (World) (Rev A)", "Streets of Rage"),
        ("Super Hang-On (World) (En,Ja) (Rev A)", "Super Hang-On"),
        (
            "QuackShot Starring Donald Duck (World) (Rev A)",
            "QuackShot Starring Donald Duck",
        ),
        ("Sonic The Hedgehog 2 (World)", "Sonic The Hedgehog 2"),
        ("Aladdin (Europe) (Beta)", "Aladdin"),
        ("Theme Park", "Theme Park"),
        ("Road Rash [!]", "Road Rash"),
        ("Bubsy II (USA) [b1]", "Bubsy II"),
    ] {
        assert_eq!(display_title(stem), want, "{stem}");
    }
}

#[test]
fn an_article_put_at_the_end_for_sorting_goes_back_to_the_front() {
    for (stem, want) in [
        ("Legend of Zelda, The (USA)", "The Legend of Zelda"),
        (
            "Adventures of Batman & Robin, The (USA)",
            "The Adventures of Batman & Robin",
        ),
        (
            "Lord of the Rings, The - Vol. 1 (USA)",
            "The Lord of the Rings - Vol. 1",
        ),
        (
            "Ren & Stimpy Show, The: Stimpy's Invention (USA)",
            "The Ren & Stimpy Show: Stimpy's Invention",
        ),
        ("Adventure, An (USA)", "An Adventure"),
    ] {
        assert_eq!(display_title(stem), want, "{stem}");
    }
    // a comma that is not an article stays
    assert_eq!(
        display_title("Earthworm Jim, Special Edition (USA)"),
        "Earthworm Jim, Special Edition"
    );
    assert_eq!(display_title("Theatre, Then (USA)"), "Theatre, Then");
}

#[test]
fn spacing_is_tidied_and_a_dangling_dash_is_dropped() {
    assert_eq!(display_title("  Sonic   3  (USA) "), "Sonic 3");
    assert_eq!(display_title("Sonic 3 - (USA)"), "Sonic 3");
    assert_eq!(display_title("Sonic 3 (USA) (Rev A)   (En)"), "Sonic 3");
}

#[test]
fn a_name_that_is_all_tags_is_given_back_as_it_was() {
    assert_eq!(display_title("(USA)"), "(USA)");
    assert_eq!(display_title("[BIOS] (USA)"), "[BIOS] (USA)");
    assert_eq!(display_title(""), "");
}

#[test]
fn unbalanced_brackets_do_not_swallow_the_name() {
    // a stray closing bracket is ignored; a stray opening one hides what follows, but never panics
    assert_eq!(display_title("Sonic) 3 (USA)"), "Sonic 3");
    assert_eq!(display_title("Sonic 3 (USA"), "Sonic 3");
}

#[test]
fn a_disc_name_without_its_tag_is_untouched() {
    assert_eq!(display_title("Parasite Eve (USA)"), "Parasite Eve");
    assert_eq!(display_title("Final Fantasy VII"), "Final Fantasy VII");
}
