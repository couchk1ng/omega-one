//! The opening: the boot sequence (`boot_seq`) played over the shelf. A boot that lands on the
//! shelf fades the bootloader's logo out to black, then plays three seconds of grid, plate and
//! the B's click on it, fades that out and lifts the black off the shelf. A press cuts it
//! short: the sequence fades out at once, then the shelf comes in, and the press goes no
//! further. Once the shelf shows through, a press ends it there and is the shelf's.
//! When the boot left a game on the shelf because its start took slot-b down, the black stays
//! and the start notice comes up on it in place of the shelf (`start_notice`), in the
//! terminal's letters, which this module still draws and tells its facts for.

use slot_b_gfx::{out_h, out_w};
use slot_b_input::Action;
use slot_b_power::{baseos_release, BaseosRelease};
use slot_b_ui::{ease, Draw, TexId, OPENING_ADVANCE, OPENING_PITCH, OPENING_RISE};

use crate::app::{App, Phase};
use crate::boot_fade::{FADE_MS, STEP_MS};
use crate::boot_seq::{self, Art, Block};

/// How long the bootloader's logo takes to fade to black before the sequence starts, in ms:
/// the logo's own fade.
pub const LEAD_MS: f32 = FADE_MS;

/// How long the start notice's terminal takes to fade in, in ms.
pub const IN_MS: f32 = 500.0;

/// How long the sequence runs, from its start to the start of its fade out, in ms, and how
/// long it takes to fade out.
pub const SCRIPT_MS: f32 = boot_seq::HOLD_MS;
pub const OUT_MS: f32 = 300.0;

/// How long the black takes to lift off the shelf, in ms.
pub const REVEAL_MS: f32 = 500.0;

/// How long a press takes to fade the sequence out, and the shelf in after it.
pub const CUT_OUT_MS: f32 = 200.0;
pub const CUT_REVEAL_MS: f32 = 300.0;

/// The start notice's terminal's size, in letters and lines: the screen it was written for.
pub(crate) const COLS: u32 = 32;
pub(crate) const LINES: u32 = 22;

/// What the opening tells about the handheld.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// Its name, as BaseOS gives it.
    pub model: String,
    /// The memory fitted, in MB: what the kernel was given, rounded up to the chips' size.
    pub mem_mb: u32,
    /// The memory free to use as it boots, in MB.
    pub avail_mb: u32,
    /// Its battery's size, where it is known.
    pub battery_mah: Option<u32>,
}

impl Default for Facts {
    /// The RG35XX SP's, the handheld slot-b is made on, for a computer standing in for one.
    fn default() -> Self {
        Facts {
            model: "RG35XX SP".into(),
            mem_mb: 1024,
            avail_mb: 872,
            battery_mah: Some(3300),
        }
    }
}

impl Facts {
    /// This handheld's, from BaseOS and the kernel, each the RG35XX SP's where it cannot be
    /// read.
    pub fn read() -> Facts {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok();
        Facts::from_parts(baseos_release(), meminfo.as_deref())
    }

    /// The facts in `release`, BaseOS's description of the handheld, and `meminfo`, the
    /// kernel's /proc/meminfo.
    pub fn from_parts(release: Option<BaseosRelease>, meminfo: Option<&str>) -> Facts {
        let mut facts = Facts::default();
        if let Some(r) = release {
            if let Some(model) = r
                .model
                .clone()
                .or_else(|| r.target.as_ref().map(|t| t.to_uppercase()))
            {
                facts.model = model;
            }
            facts.battery_mah = r.target.as_deref().and_then(battery_mah);
        }
        let kb = |key: &str| {
            meminfo?
                .lines()
                .find_map(|l| l.strip_prefix(key))
                .and_then(|rest| rest.split_whitespace().next())
                .and_then(|n| n.parse::<u64>().ok())
        };
        if let Some(total) = kb("MemTotal:") {
            facts.mem_mb = fitted_mb(total);
        }
        if let Some(avail) = kb("MemAvailable:") {
            facts.avail_mb = (avail / 1024) as u32;
        }
        facts
    }
}

/// The memory chips' size, in MB, for the `kb` the kernel was given of them: it keeps some for
/// itself and the GPU, so it rounds up to the power of two.
fn fitted_mb(kb: u64) -> u32 {
    (kb.div_ceil(1024).max(1) as u32).next_power_of_two()
}

/// The battery fitted to each BaseOS build target, where its maker's figure has been checked.
fn battery_mah(target: &str) -> Option<u32> {
    match target {
        "rg35xxsp" => Some(3300),
        _ => None,
    }
}

/// The sequence's art as the frontend uploaded it, a texture for each piece.
#[derive(Copy, Clone)]
pub struct OpeningArt {
    pub grid: TexId,
    pub left: TexId,
    pub letters: TexId,
    pub piece: TexId,
    pub seam: TexId,
    pub shadow: TexId,
    pub glyph: TexId,
    pub glint: TexId,
}

impl OpeningArt {
    /// Every texture the art is made of.
    pub(crate) fn textures(&self) -> [TexId; 8] {
        [
            self.grid,
            self.left,
            self.letters,
            self.piece,
            self.seam,
            self.shadow,
            self.glyph,
            self.glint,
        ]
    }

    fn tex(&self, art: Art) -> TexId {
        match art {
            Art::Grid => self.grid,
            Art::Left => self.left,
            Art::Letters => self.letters,
            Art::Piece => self.piece,
            Art::Seam => self.seam,
            Art::Shadow => self.shadow,
            Art::Glyph => self.glyph,
            Art::Glint => self.glint,
        }
    }
}

/// The opening while it plays.
pub(crate) struct Opening {
    /// How long it has run, in ms from its first frame; None before that.
    elapsed: Option<f32>,
    /// When a press cut it short, and how brightly the sequence was drawn then.
    cut: Option<(f32, f32)>,
    /// The art the frontend has uploaded for it, once it has.
    art: Option<OpeningArt>,
}

impl Opening {
    fn new() -> Opening {
        Opening {
            elapsed: None,
            cut: None,
            art: None,
        }
    }

    fn elapsed(&self) -> f32 {
        self.elapsed.unwrap_or(0.0)
    }

    /// How far into the sequence it is, in ms from its start: still once a press has cut it
    /// short. Negative while the bootloader's logo fades.
    fn script_ms(&self) -> f32 {
        self.cut.map_or(self.elapsed(), |(at, _)| at) - LEAD_MS
    }

    /// How brightly the sequence is drawn over the black.
    fn ink(&self) -> f32 {
        if let Some((at, level)) = self.cut {
            return level * (1.0 - ease(((self.elapsed() - at) / CUT_OUT_MS).clamp(0.0, 1.0)));
        }
        let ms = self.script_ms();
        if ms < 0.0 {
            0.0
        } else if ms < SCRIPT_MS {
            1.0
        } else {
            1.0 - ((ms - SCRIPT_MS) / OUT_MS).min(1.0)
        }
    }

    /// When the black starts to lift off the shelf, in ms from the first frame, and how long
    /// it takes.
    fn reveal(&self) -> (f32, f32) {
        match self.cut {
            Some((at, _)) => (at + CUT_OUT_MS, CUT_REVEAL_MS),
            None => (LEAD_MS + SCRIPT_MS + OUT_MS, REVEAL_MS),
        }
    }

    /// How black the screen is under the sequence.
    fn veil(&self) -> f32 {
        let (from, takes) = self.reveal();
        1.0 - ease(((self.elapsed() - from) / takes).clamp(0.0, 1.0))
    }

    /// Whether the shelf has started to show through.
    fn revealing(&self) -> bool {
        self.elapsed() > self.reveal().0
    }

    fn over(&self) -> bool {
        self.veil() <= 0.0
    }
}

/// Where the terminal's first letter stands on a `w` by `h` screen: the terminal in the middle
/// of it, on the screen pixels' even grid. The top of its capitals, not of the line's face.
pub fn terminal_origin(w: u32, h: u32) -> (u32, u32) {
    let x = w.saturating_sub(COLS * OPENING_ADVANCE) / 2;
    let y = h.saturating_sub(LINES * OPENING_PITCH) / 2;
    (x & !1, y & !1)
}

fn origin() -> (f32, f32) {
    let (x, y) = terminal_origin(out_w(), out_h());
    (x as f32, y as f32)
}

impl App {
    /// Plays the opening over the shelf from the next frame, telling `facts`. It is only
    /// played while the boot is still on the shelf it landed on, as the boot logo's fade is.
    /// The start notice names the handheld as it does, whether it plays or not.
    pub fn start_opening(&mut self, facts: &Facts) {
        self.tell_start_notice(facts);
        if self.boot_fades() {
            self.opening = Some(Opening::new());
        }
    }

    /// Whether the opening is playing.
    pub fn opening_shown(&self) -> bool {
        self.opening.is_some()
    }

    /// How black the opening has the screen, and how brightly its sequence is drawn, while it
    /// plays.
    pub fn opening_levels(&self) -> Option<(f32, f32)> {
        self.opening.as_ref().map(|o| (o.veil(), o.ink()))
    }

    /// Whether the opening is waiting for the art it draws, so the frontend should upload it.
    pub fn opening_wants_art(&self) -> bool {
        self.opening.as_ref().is_some_and(|o| o.art.is_none())
    }

    /// Hands the opening the textures of the art it draws.
    pub fn set_opening_art(&mut self, art: OpeningArt) {
        if let Some(o) = self.opening.as_mut() {
            o.art = Some(art);
        }
    }

    /// Whether the opening keeps the screen black, so nothing under it need be drawn.
    pub(crate) fn opening_covers(&self) -> bool {
        self.opening.as_ref().is_some_and(|o| o.veil() >= 1.0)
    }

    /// Whether the opening has put its terminal away and its black would start to lift off the
    /// shelf now.
    pub(crate) fn opening_lifting(&self) -> bool {
        self.opening.as_ref().is_some_and(Opening::revealing)
    }

    /// Whether the opening takes `action`. A press cuts the terminal short and goes no
    /// further; once the shelf shows through it ends the opening and is the shelf's. The lid
    /// and POWER end it at once and do what they do; brightness and volume work through it.
    pub(crate) fn opening_takes(&mut self, action: Action) -> bool {
        let Some(o) = &mut self.opening else {
            return false;
        };
        match action {
            Action::BrightnessUp
            | Action::BrightnessDown
            | Action::BlueLightUp
            | Action::BlueLightDown
            | Action::VolumeUp
            | Action::VolumeDown
            | Action::MuteToggle
            | Action::LidOpen => false,
            Action::LidClose
            | Action::PowerPress
            | Action::PowerTap
            | Action::PowerHold
            | Action::PowerOff => {
                self.opening = None;
                false
            }
            Action::GbaUp(_) | Action::RewindStop | Action::FfStop => !o.revealing(),
            _ if o.revealing() => {
                self.opening = None;
                false
            }
            _ => {
                if o.cut.is_none() {
                    o.cut = Some((o.elapsed(), o.ink()));
                }
                true
            }
        }
    }

    /// Moves the opening on by `dt` seconds, from its first frame, sounding what falls due,
    /// and ends it once the shelf has come in or gone.
    pub(crate) fn step_opening(&mut self, dt: f32) {
        let on_shelf = matches!(self.phase, Phase::Shelf);
        let Some(o) = &mut self.opening else {
            return;
        };
        let before = o.elapsed.map_or(f32::NEG_INFINITY, |_| o.script_ms());
        o.elapsed = Some(match o.elapsed {
            None => 0.0,
            Some(e) => e + (dt * 1000.0).min(STEP_MS),
        });
        if o.cut.is_none() {
            if let Some(sfx) = boot_seq::sound(before, o.script_ms()) {
                self.sfx = Some(sfx);
            }
        }
        if o.over() || !on_shelf {
            self.opening = None;
        }
    }

    /// The black over everything and the sequence on it, as far as each has faded.
    pub(crate) fn draw_opening(&self, out: &mut Vec<Draw>) {
        let Some(o) = &self.opening else {
            return;
        };
        let veil = o.veil();
        if veil <= 0.0 {
            return;
        }
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: out_w() as f32,
            h: out_h() as f32,
            colour: [0.0, 0.0, 0.0, veil],
        });
        let Some(art) = o.art else {
            return;
        };
        for block in boot_seq::scene(o.script_ms(), o.ink(), out_w(), out_h()) {
            out.push(match block {
                Block::Rect { x, y, w, h, colour } => Draw::Rect { x, y, w, h, colour },
                Block::Art {
                    art: piece,
                    x,
                    y,
                    w,
                    h,
                    colour,
                    clip,
                } => Draw::Spun {
                    x,
                    y,
                    w,
                    h,
                    tex: art.tex(piece),
                    colour,
                    turn: 0.0,
                    yaw: 0.0,
                    depth: 0.0,
                    clip,
                },
            });
        }
    }
}

/// A terminal over everything: the black under it, `veil` opaque, and its rows' `faces` on the
/// terminal's grid, as bright as `ink`.
pub(crate) fn draw_terminal(
    veil: f32,
    ink: f32,
    faces: &[Option<(TexId, u32, u32)>],
    out: &mut Vec<Draw>,
) {
    if veil <= 0.0 {
        return;
    }
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [0.0, 0.0, 0.0, veil],
    });
    if ink <= 0.0 {
        return;
    }
    let (x, y) = origin();
    for (row, face) in faces.iter().enumerate() {
        let Some((tex, w, h)) = *face else {
            continue;
        };
        out.push(Draw::Tex {
            x,
            y: y + (row as u32 * OPENING_PITCH) as f32 - OPENING_RISE as f32,
            w: w as f32,
            h: h as f32,
            tex,
            alpha: ink,
        });
    }
}
