use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Snes,
    Ps1,
    Genesis,
    MegaDrive,
}

impl Platform {
    pub const ALL: [Platform; 7] = [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Snes,
        Platform::Ps1,
        Platform::Genesis,
        Platform::MegaDrive,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Snes => "Super Nintendo",
            Platform::Ps1 => "PlayStation",
            Platform::Genesis => "Genesis",
            Platform::MegaDrive => "Mega Drive",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Snes => "SNES",
            Platform::Ps1 => "PS1",
            Platform::Genesis => "Genesis",
            Platform::MegaDrive => "MD",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Snes => &["sfc", "smc"],
            Platform::Ps1 => &["cue", "chd", "pbp", "m3u"],
            Platform::Genesis | Platform::MegaDrive => &["md", "gen", "smd", "bin"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Snes => (256, 224),
            Platform::Ps1 => (320, 240),
            Platform::Genesis | Platform::MegaDrive => (320, 224),
        }
    }

    /// Whether the frontend keeps L1/R1 while a game of this system plays, for its picture
    /// size: only the Game Boys, whose games have no shoulder buttons.
    pub fn owns_shoulders(self) -> bool {
        matches!(self, Platform::Gb | Platform::Gbc)
    }

    /// Whether this is a Sega Mega Drive or Genesis: the two folders are one machine, kept
    /// apart so each has its own games and labels.
    pub fn sega_16bit(self) -> bool {
        matches!(self, Platform::Genesis | Platform::MegaDrive)
    }

    /// Whether held X/Y are turbo A/B: on the handhelds, whose games have no X or Y.
    pub fn turbo_xy(self) -> bool {
        matches!(self, Platform::Gba | Platform::Gb | Platform::Gbc)
    }

    /// Whether the core's colour correction for a handheld's screen applies.
    pub fn colour_correction(self) -> bool {
        matches!(self, Platform::Gba | Platform::Gb | Platform::Gbc)
    }

    /// Whether held L2 rewinds: not on the PlayStation, whose games use L2 and whose states
    /// are too big for the rewind buffer.
    pub fn rewinds(self) -> bool {
        self != Platform::Ps1
    }

    /// Whether L2/R2 belong to the game rather than to rewind and fast-forward.
    pub fn game_owns_l2_r2(self) -> bool {
        self == Platform::Ps1
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}
