use super::chime::{self, BOOT_RISE_S, BOOT_SEAT_S, BOOT_SLIDE_S, BOOT_STAMP_S, KEY_S, READING_S};

const INSERT: &[u8] = include_bytes!("../../assets/insert.pcm");
const EJECT: &[u8] = include_bytes!("../../assets/eject.pcm");
const ASSET_HZ: f32 = 48_000.0;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sfx {
    Insert,
    Eject,
    /// A key typed in the start notice: 440 Hz with every harmonic, gone in a breath.
    Key,
    /// A file read in the start notice: a fast warble of A5, C6 and F5 that dies away.
    Reading,
    /// The boot sequence's power-on: a low triangle wave rising under the grid.
    BootRise,
    /// SLOT stamping in: a soft thump as the plate lands and a rising note for each letter.
    BootStamp,
    /// The B's piece sliding in: a sine sweeping up as it speeds.
    BootSlide,
    /// The click as the B's tab bottoms out, a thunk, and a chord that blooms and rings.
    BootSeat,
}

impl Sfx {
    pub fn lead(self) -> f32 {
        match self {
            Sfx::Insert => 0.097,
            Sfx::Eject => 0.021,
            Sfx::Key
            | Sfx::Reading
            | Sfx::BootRise
            | Sfx::BootStamp
            | Sfx::BootSlide
            | Sfx::BootSeat => 0.0,
        }
    }

    pub fn tail(self) -> f32 {
        let secs = match self {
            Sfx::Insert => INSERT.len() as f32 / 2.0 / ASSET_HZ,
            Sfx::Eject => EJECT.len() as f32 / 2.0 / ASSET_HZ,
            Sfx::Key => KEY_S,
            Sfx::Reading => READING_S,
            Sfx::BootRise => BOOT_RISE_S,
            Sfx::BootStamp => BOOT_STAMP_S,
            Sfx::BootSlide => BOOT_SLIDE_S,
            Sfx::BootSeat => BOOT_SEAT_S,
        };
        secs - self.lead()
    }

    pub fn render(self, sample_rate: u32) -> Vec<i16> {
        let pcm = match self {
            Sfx::Insert => INSERT,
            Sfx::Eject => EJECT,
            Sfx::Key
            | Sfx::Reading
            | Sfx::BootRise
            | Sfx::BootStamp
            | Sfx::BootSlide
            | Sfx::BootSeat => return stereo(chime::render(self, sample_rate)),
        };
        stereo(resampled(pcm, sample_rate))
    }
}

/// `mono` as the frames the sink takes, the same sample to both ears.
fn stereo(mono: Vec<f32>) -> Vec<i16> {
    let mut out = Vec::with_capacity(mono.len() * 2);
    for v in mono {
        let s = v.clamp(-32768.0, 32767.0) as i16;
        out.push(s);
        out.push(s);
    }
    out
}

fn resampled(pcm: &[u8], sample_rate: u32) -> Vec<f32> {
    let src: Vec<f32> = pcm
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32)
        .collect();
    if sample_rate == ASSET_HZ as u32 || src.is_empty() {
        return src;
    }
    let ratio = sample_rate as f32 / ASSET_HZ;
    let n = (src.len() as f32 * ratio) as usize;
    (0..n)
        .map(|i| {
            let x = i as f32 / ratio;
            let a = x as usize;
            let f = x - a as f32;
            let b = (a + 1).min(src.len() - 1);
            src[a] * (1.0 - f) + src[b] * f
        })
        .collect()
}
