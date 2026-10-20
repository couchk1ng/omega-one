//! The lines of a terminal, each made again when what it says changes: a key typed, a count
//! run up, for the start notice; and the boot sequence's art, uploaded once. Once either is over
//! its textures shrink to a pixel and give their memory back.

use slot_b_gfx::{out_h, out_w, Compositor, TexId};
use slot_b_ui::{boot_art, opening_line, UndoFace};

use crate::app::App;
use crate::frontend::faces::upload_nearest;
use crate::opening::OpeningArt;

/// A terminal's rows in the opening's lettering, as uploaded.
#[derive(Default)]
pub(crate) struct TerminalRows {
    /// Each row's texture and what it says.
    rows: Vec<(Option<TexId>, String)>,
}

impl TerminalRows {
    /// Makes each of `lines` again whose row said something else, and hands its face to `set`.
    pub(crate) fn sync(
        &mut self,
        compositor: &mut Compositor,
        lines: Vec<(usize, String)>,
        mut set: impl FnMut(usize, TexId, u32, u32),
    ) {
        for (row, text) in lines {
            if self.rows.len() <= row {
                self.rows.resize_with(row + 1, Default::default);
            }
            let (tex, said) = &mut self.rows[row];
            if *said == text {
                continue;
            }
            let face = opening_line(&text);
            let id = upload_nearest(compositor, tex, &face);
            *said = text;
            set(row, id, face.w, face.h);
        }
    }

    /// Shrinks every row's texture to a pixel, giving its memory back.
    pub(crate) fn free(&mut self, compositor: &mut Compositor) {
        for (tex, _) in self.rows.drain(..) {
            if let Some(tex) = tex {
                compositor.update_texture(tex, 1, 1, &[0; 4]);
            }
        }
    }
}

/// The textures of the boot sequence's art: made when the opening first wants them, and shrunk
/// to a pixel each, giving their memory back, once it is over.
#[derive(Default)]
pub(crate) struct OpeningTextures {
    made: Option<OpeningArt>,
    given_back: bool,
}

impl OpeningTextures {
    fn upload(compositor: &mut Compositor) -> OpeningArt {
        let art = boot_art(out_w(), out_h());
        let mut make =
            |face: &UndoFace| compositor.create_texture_nearest(face.w, face.h, &face.rgba);
        OpeningArt {
            grid: make(&art.grid),
            left: make(&art.left.face),
            letters: make(&art.letters.face),
            piece: make(&art.piece.face),
            seam: make(&art.seam.face),
            shadow: make(&art.shadow.face),
            glyph: make(&art.glyph.face),
            glint: make(&art.glint.face),
        }
    }

    fn free(&mut self, compositor: &mut Compositor) {
        let Some(art) = self.made.filter(|_| !self.given_back) else {
            return;
        };
        for tex in art.textures() {
            compositor.update_texture(tex, 1, 1, &[0; 4]);
        }
        self.given_back = true;
    }
}

pub(crate) fn sync_opening(
    app: &mut App,
    compositor: &mut Compositor,
    state: &mut OpeningTextures,
) {
    if !app.opening_shown() {
        return state.free(compositor);
    }
    if app.opening_wants_art() && !state.given_back {
        let art = *state
            .made
            .get_or_insert_with(|| OpeningTextures::upload(compositor));
        app.set_opening_art(art);
    }
}
