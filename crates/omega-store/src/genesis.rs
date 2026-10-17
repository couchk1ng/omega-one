//! The title in a Mega Drive or Genesis ROM header.

use std::io::Read;
use std::path::Path;

const HEADER: usize = 0x180;
const MAGIC_AT: usize = 0x100;
const DOMESTIC: std::ops::Range<usize> = 0x120..0x150;
const OVERSEAS: std::ops::Range<usize> = 0x150..0x180;

/// The game's name from the header, preferring the overseas one. A ROM in the interleaved
/// `.smd` format, or one with no header, has none.
pub fn title(rom: &Path) -> Option<String> {
    let mut header = [0u8; HEADER];
    std::fs::File::open(rom)
        .and_then(|mut f| f.read_exact(&mut header))
        .ok()?;
    title_in(&header)
}

pub fn title_in(header: &[u8]) -> Option<String> {
    if header.len() < HEADER || &header[MAGIC_AT..MAGIC_AT + 4] != b"SEGA" {
        return None;
    }
    [OVERSEAS, DOMESTIC]
        .into_iter()
        .find_map(|range| clean(&header[range]))
}

fn clean(field: &[u8]) -> Option<String> {
    let text: String = field
        .iter()
        .map(|&b| match b {
            0x20..=0x7e => b as char,
            _ => ' ',
        })
        .collect();
    let words: Vec<&str> = text.split_whitespace().collect();
    (!words.is_empty()).then(|| words.join(" "))
}
