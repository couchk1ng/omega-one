//! The shelf: the carts in a row over the background and the bar across the top. Each is a
//! layer of its own as the shelf slides away to a menu and back. The bar across the bottom, with
//! the shelf's system in its pill, is drawn over every screen (bar.rs).

use slot_b_gfx::out_w;
use slot_b_ui::{ease, rest_y, shelf_box, stage_y, sway_reach, Draw, TopBar, SPRITE_PX, TOP_BAR_H};

use crate::app::App;
use crate::nav::Offsets;

const CORE_PICKER_RECEDE: f32 = 0.26;

/// How far clear of the cart, and of the bar across the top, the title caption stands, in pixels.
const TITLE_GAP: f32 = 8.0;
const CORE_PICKER_DIM: f32 = 0.614;

impl App {
    /// The shelf, its carts moved `off.carts_dx` across and `off.carts_dy` down, its top bar
    /// `off.bar_dy` down.
    pub(crate) fn draw_shelf(&self, off: Offsets, out: &mut Vec<Draw>) {
        let carts = out.len();
        match (self.core_picker_shown(), self.selected_stem()) {
            (Some(picker), Some(stem)) => {
                let open = ease(picker.openness(self.now()));
                let dim = 1.0 + (CORE_PICKER_DIM - 1.0) * open;
                self.shelf()
                    .draw_row(Some(stem), 0.0, CORE_PICKER_RECEDE * open, dim, out);
                shift(&mut out[carts..], off.carts_dx, off.carts_dy);
            }
            _ => {
                self.shelf()
                    .draw_row(None, self.shelf_shake(), 0.0, 1.0, out);
                self.draw_shelf_title(out);
                shift(&mut out[carts..], off.carts_dx, off.carts_dy);
            }
        }
        let bar = out.len();
        self.draw_top_bar(out);
        shift(&mut out[bar..], 0.0, off.bar_dy);
    }

    /// The selected game's name in a pill above it, while titles are on. It stands clear of the
    /// cart as it floats and sways, centred across the screen, and never higher than the bar
    /// across the top. It goes with the carts as the shelf slides.
    pub(crate) fn draw_shelf_title(&self, out: &mut Vec<Draw>) {
        let (true, Some(tex)) = (self.state.titles, self.shelf_title.face) else {
            return;
        };
        let Some((platform, _)) = self.shelves.get(self.shelf_at) else {
            return;
        };
        let (w, h) = shelf_box(*platform);
        let pill_h = (slot_b_ui::kit::action::BAR * SPRITE_PX) as f32;
        let above_cart = rest_y(h) - sway_reach(w, h) - TITLE_GAP - pill_h;
        let y = above_cart.max(stage_y() + TOP_BAR_H + TITLE_GAP);
        let art_w = self.shelf_title.w;
        let x = (out_w() / SPRITE_PX / 2).saturating_sub(art_w / 2) * SPRITE_PX;
        out.push(Draw::Tex {
            x: x as f32,
            y,
            w: (art_w * SPRITE_PX) as f32,
            h: pill_h,
            tex,
            alpha: self.title_alpha(),
        });
    }

    /// The bar across the top, in the chrome's colours, its faces baked in its ink.
    pub(crate) fn draw_top_bar(&self, out: &mut Vec<Draw>) {
        let r = self.top_readouts;
        TopBar {
            colours: self.chrome_colours(),
            links: self.links(),
            faces: self.link_faces.as_ref(),
            clock: r.clock,
            battery: self.battery,
            percent: r.percent,
            bolt: r.bolt,
        }
        .draw(out);
    }
}

/// Moves each of `drawn` `dx` across and `dy` down.
fn shift(drawn: &mut [Draw], dx: f32, dy: f32) {
    for d in drawn {
        *d = d.shifted(dx, dy);
    }
}
