//! The boot sequence's sounds and the start notice's. The start notice's are after Mega Man
//! X5's boot screen, measured off it: a beep for each key and a warble for each file read,
//! square waves made for the rate they play at, softened above 8 kHz as the PlayStation's were.
//! The boot sequence's are four clips, one for each beat of it: a low rise under the grid, a
//! thump and a rising note for each letter, a sweep as the B's piece slides, and a click, a
//! thunk and a ringing chord as it seats. They are sine and FM tones, not softened.

use super::Sfx;

/// How long each sound runs, in seconds.
pub(super) const KEY_S: f32 = 0.16;
pub(super) const READING_S: f32 = 0.55;
pub(super) const BOOT_RISE_S: f32 = 1.01;
pub(super) const BOOT_STAMP_S: f32 = 0.72;
pub(super) const BOOT_SLIDE_S: f32 = 0.26;
pub(super) const BOOT_SEAT_S: f32 = 1.46;

/// How loud each of the start notice's peaks, as a share of full scale.
const KEY_LEVEL: f32 = 0.2;
const READING_LEVEL: f32 = 0.2;

/// The boot sequence's clips are mixed so the loudest, the seat, peaks just under the 9,000 the
/// opening's sounds may reach, to sit under the carts'. Their balance is the preview's, which
/// was louder, with its loudest moment at 0.8 of full scale: a gain of 1.68.
const BOOT_GAIN: f32 = 0.57;

/// X5's file warble: A5, C6, F5, each held 45 ms.
const READING_NOTES: [f32; 3] = [877.0, 1046.5, 698.5];
const NOTE_S: f32 = 0.045;

/// Where the softening starts, in Hz.
const SOFT_HZ: f32 = 8_000.0;

const TAU: f32 = std::f32::consts::TAU;

/// `sfx`, one of the opening's, as samples at `rate` scaled to i16's range.
pub(super) fn render(sfx: Sfx, rate: u32) -> Vec<f32> {
    let rate = rate.max(1) as f32;
    let retro = |mut v: Vec<f32>| {
        soften(&mut v, rate);
        soften(&mut v, rate);
        v.iter_mut().for_each(|s| *s *= 32_767.0);
        v
    };
    let boot = |mut v: Vec<f32>| {
        v.iter_mut().for_each(|s| *s *= BOOT_GAIN * 32_767.0);
        v
    };
    match sfx {
        Sfx::Key => retro(key(rate)),
        Sfx::Reading => retro(warble(rate, READING_S, &READING_NOTES, READING_LEVEL)),
        Sfx::BootRise => boot(rise(rate)),
        Sfx::BootStamp => boot(stamp(rate)),
        Sfx::BootSlide => boot(slide(rate)),
        Sfx::BootSeat => boot(seat(rate)),
        Sfx::Insert | Sfx::Eject => Vec::new(),
    }
}

fn samples(rate: f32, secs: f32) -> impl Iterator<Item = f32> {
    (0..(secs * rate) as usize).map(move |i| i as f32 / rate)
}

/// A square wave's sign at `phase`, in turns, `duty` of each turn high.
fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty {
        1.0
    } else {
        -1.0
    }
}

/// A 25% pulse at 440 Hz, which keeps every harmonic but the fourth's, dying in 45 ms.
fn key(rate: f32) -> Vec<f32> {
    samples(rate, KEY_S)
        .map(|t| KEY_LEVEL * pulse(440.0 * t, 0.25) * (t / 0.002).min(1.0) * (-t / 0.045).exp())
        .collect()
}

/// `notes` in turn, `NOTE_S` each, with a breath of noise, fading out over `secs`.
fn warble(rate: f32, secs: f32, notes: &[f32], level: f32) -> Vec<f32> {
    let mut phase = 0.0;
    let mut seed = 0x9204_u32;
    samples(rate, secs)
        .map(|t| {
            phase += notes[(t / NOTE_S) as usize % notes.len()] / rate;
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            let gate = (t / 0.004).min(1.0) * (1.0 - t / secs).max(0.0).powf(1.5);
            level * (pulse(phase, 0.5) * 0.85 + noise * 0.08) * gate
        })
        .collect()
}

/// One pole of low-pass at `SOFT_HZ`, run over `v` in place.
fn soften(v: &mut [f32], rate: f32) {
    let k = 1.0 - (-2.0 * std::f32::consts::PI * SOFT_HZ / rate).exp();
    let mut y = 0.0;
    for s in v.iter_mut() {
        y += k * (*s - y);
        *s = y;
    }
}

/// `x` along the line through `points`, held at the ends.
fn line(x: f32, points: &[(f32, f32)]) -> f32 {
    let (first, last) = (points[0], points[points.len() - 1]);
    if x <= first.0 {
        return first.1;
    }
    for w in points.windows(2) {
        if x <= w[1].0 {
            return w[0].1 + (w[1].1 - w[0].1) * (x - w[0].0) / (w[1].0 - w[0].0);
        }
    }
    last.1
}

/// A silent clip `secs` long.
fn clip(rate: f32, secs: f32) -> Vec<f32> {
    vec![0.0; (secs * rate) as usize]
}

/// `sound(t)` for every sample from `start` seconds in, for `secs`, added into `out`.
fn add(out: &mut [f32], rate: f32, start: f32, secs: f32, mut sound: impl FnMut(f32) -> f32) {
    let first = (start * rate) as usize;
    for (n, slot) in out
        .iter_mut()
        .skip(first)
        .take((secs * rate) as usize)
        .enumerate()
    {
        *slot += sound(n as f32 / rate);
    }
}

/// An FM note: a sine at `freq` whose pitch a sine at `ratio` times it bends, `index` deep at
/// the start and gone by four fifths of `dur`, under a 6 ms attack and an exponential decay,
/// `start` seconds in.
#[derive(Copy, Clone)]
struct Note {
    start: f32,
    freq: f32,
    dur: f32,
    level: f32,
    ratio: f32,
    index: f32,
}

fn fm(out: &mut [f32], rate: f32, note: Note) {
    let Note {
        start,
        freq,
        dur,
        level,
        ratio,
        index,
    } = note;
    let fmod = ratio * freq;
    let (deep, shallow) = (freq * index, freq * 0.02 + 1.0);
    add(out, rate, start, dur + 0.05, |t| {
        let env = if t < 0.006 {
            0.0001 + (level - 0.0001) * t / 0.006
        } else if t <= dur {
            level * (0.0001 / level).powf((t - 0.006) / (dur - 0.006))
        } else {
            0.0001 * (-(t - dur) * 80.0).exp()
        };
        let bend = if t < 0.8 * dur {
            deep * (shallow / deep).powf(t / (0.8 * dur))
        } else {
            shallow
        };
        let phase = TAU * freq * t - (bend / fmod) * ((TAU * fmod * t).cos() - 1.0);
        phase.sin() * env
    });
}

/// A tone that glides from `from` Hz to `to` over `dur`, dying away exponentially from `level`,
/// `start` seconds in: a square wave when `square`, else a sine.
#[derive(Copy, Clone)]
struct Glide {
    start: f32,
    square: bool,
    from: f32,
    to: f32,
    dur: f32,
    level: f32,
}

fn glide(out: &mut [f32], rate: f32, g: Glide) {
    let Glide {
        start,
        square,
        from,
        to,
        dur,
        level,
    } = g;
    let mut phase = 0.0f32;
    add(out, rate, start, dur + 0.02, |t| {
        let f = if t < dur {
            from * (to / from).powf(t / dur)
        } else {
            to
        };
        phase += TAU * f / rate;
        let wave = if square {
            phase.sin().signum()
        } else {
            phase.sin()
        };
        let env = if t < dur {
            level * (0.0001 / level).powf(t / dur)
        } else {
            0.0001
        };
        wave * env
    });
}

/// A triangle wave rising from 55 Hz to 170 Hz in three quarters of a second, swelling and
/// fading: the grid powering up.
fn rise(rate: f32) -> Vec<f32> {
    let mut out = clip(rate, BOOT_RISE_S);
    let mut phase = 0.0f32;
    add(&mut out, rate, 0.0, BOOT_RISE_S, |t| {
        let f = if t < 0.75 {
            55.0 * (170.0f32 / 55.0).powf(t / 0.75)
        } else {
            170.0
        };
        phase += TAU * f / rate;
        let tri = 2.0 / std::f32::consts::PI * phase.sin().asin();
        tri * line(
            t,
            &[(0.0, 0.0001), (0.65, 0.05), (0.99, 0.0001), (1.01, 0.0001)],
        )
    });
    out
}

/// A soft thump as SLOT's plate lands, and a note for each of its four letters, a rising C
/// major arpeggio, 88 ms apart.
fn stamp(rate: f32) -> Vec<f32> {
    let mut out = clip(rate, BOOT_STAMP_S);
    glide(
        &mut out,
        rate,
        Glide {
            start: 0.0,
            square: false,
            from: 120.0,
            to: 42.0,
            dur: 0.28,
            level: 0.3,
        },
    );
    for (i, freq) in [523.25, 659.25, 783.99, 1046.5].into_iter().enumerate() {
        let start = 0.088 * i as f32;
        fm(
            &mut out,
            rate,
            Note {
                start,
                freq,
                dur: 0.4,
                level: 0.14,
                ratio: 2.0,
                index: 1.5,
            },
        );
    }
    out
}

/// A sine sweeping from 240 Hz to 1.3 kHz as the B's piece speeds in.
fn slide(rate: f32) -> Vec<f32> {
    let mut out = clip(rate, BOOT_SLIDE_S);
    let mut phase = 0.0f32;
    add(&mut out, rate, 0.0, BOOT_SLIDE_S, |t| {
        let f = 240.0 * (1300.0f32 / 240.0).powf((t / 0.25).clamp(0.0, 1.0));
        phase += TAU * f / rate;
        phase.sin()
            * line(
                t,
                &[
                    (0.0, 0.0001),
                    (0.23, 0.055),
                    (0.255, 0.0001),
                    (0.26, 0.0001),
                ],
            )
    });
    out
}

/// The click as the tab bottoms out and a thunk with it, then a chord that blooms and rings,
/// and a high note a moment behind it.
fn seat(rate: f32) -> Vec<f32> {
    let mut out = clip(rate, BOOT_SEAT_S);
    glide(
        &mut out,
        rate,
        Glide {
            start: 0.0,
            square: true,
            from: 2400.0,
            to: 2400.0,
            dur: 0.03,
            level: 0.16,
        },
    );
    glide(
        &mut out,
        rate,
        Glide {
            start: 0.0,
            square: false,
            from: 170.0,
            to: 70.0,
            dur: 0.12,
            level: 0.4,
        },
    );
    for (start, freq, dur, level, ratio, index) in [
        (0.010, 1046.5, 1.4, 0.12, 2.0, 1.2),
        (0.010, 1568.0, 1.2, 0.08, 2.0, 1.0),
        (0.010, 2093.0, 0.9, 0.05, 3.0, 0.8),
        (0.170, 2637.0, 0.5, 0.05, 3.0, 1.0),
    ] {
        fm(
            &mut out,
            rate,
            Note {
                start,
                freq,
                dur,
                level,
                ratio,
                index,
            },
        );
    }
    out
}
