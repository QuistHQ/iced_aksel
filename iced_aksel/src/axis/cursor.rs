use crate::style::CursorStyle;
use iced_core::{Color, Font, Pixels};

/// The result returned from a cursor renderer function.
#[derive(Debug, Clone)]
pub struct CursorResult {
    pub(crate) label: Option<String>,
    pub(crate) style: Option<CursorStyle>,
}

impl CursorResult {
    /// Start with a base style (usually derived from the Theme).
    pub fn empty() -> Self {
        Self {
            label: None,
            style: None,
        }
    }

    /// Set the label text.
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
    
    /// Set the style of the cursor badge
    pub fn style(mut self, style: CursorStyle) -> Self {
        self.style = Some(style);
        self
    }
}
