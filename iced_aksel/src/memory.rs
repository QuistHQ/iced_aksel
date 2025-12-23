use std::cell::RefCell;

use super::{Action, render};
use ab_glyph::FontVec;
use iced_core::{Font, mouse};
use iced_graphics::text::{
    cosmic_text::{FontSystem, fontdb::Query},
    font_system,
};

/// Internal chart memory
pub struct Memory<AxisId> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub tessellators: RefCell<render::Tessellator>,
    pub plot_font_bytes: FontVec,
    pub plot_font: Font,
    pub axis_font_bytes: FontVec,
    pub axis_font: Font,
}

impl<AxisId> Memory<AxisId> {
    pub fn new(plot_font: Font, axis_font: Font) -> Self {
        let mut lock = font_system().write().expect("Failed to read font_system");
        let system = lock.raw();

        let Some((plot_font_bytes, plot_font)) = update_font(system, plot_font) else {
            panic!("Font not found in system: {plot_font:?}");
        };
        let Some((axis_font_bytes, axis_font)) = update_font(system, axis_font) else {
            panic!("Font not found in system: {axis_font:?}");
        };

        Self {
            action: Action::default(),
            previous_click: None,
            tessellators: RefCell::new(render::Tessellator::default()),
            plot_font_bytes,
            plot_font,
            axis_font_bytes,
            axis_font,
        }
    }

    pub fn update_fonts(&mut self, axis_font: Font, plot_font: Font) {
        let mut lock = font_system().write().expect("Failed to read font_system");
        let system = lock.raw();

        if self.plot_font != plot_font {
            let Some((layout, font)) = update_font(system, plot_font) else {
                panic!("Font not found in system: {plot_font:?}");
            };

            self.plot_font = font;
            self.plot_font_bytes = layout;
            self.tessellators.borrow_mut().clear_glyph_cache();
        }

        if self.axis_font != axis_font {
            let Some((layout, font)) = update_font(system, axis_font) else {
                panic!("Font not found in system: {axis_font:?}");
            };

            self.axis_font = font;
            self.axis_font_bytes = layout;
        }

        drop(lock)
    }
}

fn update_font(system: &mut FontSystem, font: Font) -> Option<(FontVec, Font)> {
    let attrs = iced_graphics::text::to_attributes(font);
    let id = system
        .db()
        .query(&Query {
            families: &[attrs.family],
            weight: attrs.weight,
            stretch: attrs.stretch,
            style: attrs.style,
        })
        // TODO: Consider changing this - It shouldnt be necessary but fixes a crash for now
        // Looks like it has some trouble finding the right font for Dennis'pc atleast
        .or(Some(system.db().faces().next()?.id))
        .expect("Font not found");
    let bytes = system.get_font(id, attrs.weight).expect("Font not found");

    FontVec::try_from_vec(bytes.data().to_vec())
        .ok()
        .zip(Some(font))
}
