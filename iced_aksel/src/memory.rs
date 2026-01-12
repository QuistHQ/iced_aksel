use std::cell::RefCell;

use super::{Action, render};
use iced_core::mouse;

/// Internal chart memory
pub struct Memory<AxisId> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub tessellators: RefCell<render::Tessellator>,
}

impl<AxisId> Memory<AxisId> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            tessellators: RefCell::new(render::Tessellator::default()),
        }
    }
}
