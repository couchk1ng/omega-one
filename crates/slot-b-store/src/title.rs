//! A game's name as the shelf can show it above the cart: from the ROM's file name, with the tags
//! in brackets that a set of ROMs puts after a name taken off, and "Legend of Zelda, The" turned
//! back into "The Legend of Zelda".

/// `stem`, a ROM's file name without its extension, as a title to read: "Streets of Rage 2
/// (USA)" is "Streets of Rage 2", and "Super Hang-On (World) (En,Ja) (Rev A)" is "Super
/// Hang-On". A name that is all tags, or that has nothing left, is given back as it was.
pub fn display_title(stem: &str) -> String {
    let mut kept = String::with_capacity(stem.len());
    let (mut round, mut square) = (0usize, 0usize);
    for c in stem.chars() {
        match c {
            '(' => round += 1,
            ')' => round = round.saturating_sub(1),
            '[' => square += 1,
            ']' => square = square.saturating_sub(1),
            _ if round == 0 && square == 0 => kept.push(c),
            _ => {}
        }
    }
    let spaced = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = spaced
        .trim_matches(|c: char| c == '-' || c == ' ')
        .to_string();
    if title.is_empty() {
        return stem.trim().to_string();
    }
    move_article(&title)
}

/// "Legend of Zelda, The" and "Lord of the Rings, The - Vol. 1" put their article at the end,
/// as a sorted list does. It goes back to the front.
fn move_article(title: &str) -> String {
    for article in ["The", "A", "An"] {
        let tail = format!(", {article}");
        if let Some(head) = title.strip_suffix(tail.as_str()) {
            return format!("{article} {head}");
        }
        for sep in [" - ", ": "] {
            let needle = format!("{tail}{sep}");
            if let Some(at) = title.find(needle.as_str()) {
                return format!("{article} {}{}", &title[..at], &title[at + tail.len()..]);
            }
        }
    }
    title.to_string()
}
