use std::cell::RefCell;

use super::{Action, render};
use ab_glyph::FontVec;
use iced_core::{Font, mouse};
use iced_graphics::text::{
    cosmic_text::{FontSystem, fallback::FontFallbackIter, fontdb::ID},
    font_system,
};

pub struct CachedFont {
    pub font: Font,
    pub bytes: FontVec,
    pub id: ID,
}

/// Internal chart memory
pub struct Memory<AxisId> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub tessellators: RefCell<render::Tessellator>,
    pub plot_font: CachedFont,
    pub axis_font: CachedFont,
}

impl<AxisId> Memory<AxisId> {
    pub fn new(plot_font: Font, axis_font: Font) -> Self {
        let mut lock = font_system().write().expect("Failed to read font_system");
        let system = lock.raw();

        let Some(plot_font) = update_font(system, plot_font) else {
            panic!("Font not found in system: {plot_font:?}");
        };
        let Some(axis_font) = update_font(system, axis_font) else {
            panic!("Font not found in system: {axis_font:?}");
        };

        drop(lock);

        Self {
            action: Action::default(),
            previous_click: None,
            tessellators: RefCell::new(render::Tessellator::default()),
            plot_font,
            axis_font,
        }
    }

    pub fn update_fonts(&mut self, axis_font: Font, plot_font: Font) {
        let mut lock = font_system().write().expect("Failed to read font_system");
        let system = lock.raw();

        #[allow(clippy::useless_let_if_seq)]
        let mut changed = false;

        if self.plot_font.font != plot_font {
            let Some(font) = update_font(system, plot_font) else {
                panic!("Font not found in system: {plot_font:?}");
            };

            self.plot_font = font;
            changed = true;
        }

        if self.axis_font.font != axis_font {
            let Some(font) = update_font(system, axis_font) else {
                panic!("Font not found in system: {axis_font:?}");
            };

            self.axis_font = font;
            changed = true;
        }

        if changed {
            self.tessellators.borrow_mut().clear_glyph_cache();
        }

        drop(lock)
    }
}

fn update_font(system: &mut FontSystem, font: Font) -> Option<CachedFont> {
    let attrs = iced_graphics::text::to_attributes(font);
    let fonts = system.get_font_matches(&attrs);
    let families = [&attrs.family];
    let scripts = []; // We don't support scripts
    let mut iter = FontFallbackIter::new(system, &fonts, &families, &scripts, "", attrs.weight);
    let bytes = iter.next().expect("No default font found");
    let id = bytes.id();

    Some(CachedFont {
        font,
        bytes: FontVec::try_from_vec(bytes.data().to_vec()).ok()?,
        id,
    })
}
