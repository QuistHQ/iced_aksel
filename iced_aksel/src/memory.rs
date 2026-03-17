use std::cell::{RefCell, RefMut};
use std::hash::Hash;

use aksel::Float;
use iced_core::{Border, Color, Layout, Point, Rectangle, renderer::Quad};
use iced_core::{keyboard, mouse};

use crate::interaction::{self, InteractionsCache};
use crate::{
    Action, LayerId, Quality, State,
    layer::Layer,
    render::{Backend, RenderCache},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverIdentity<AxisId, Tag: Hash + Eq + Clone> {
    Plot,
    Interaction(interaction::Id<Tag>),
    Axis(AxisId),
    // For the future, when we track wether the mouse left bound
    OutsideBounds,
}

#[derive(Debug, PartialEq)]
struct LayerIdentifier {
    id: Option<LayerId>,
    version: Option<u64>,
}

#[derive(Debug, PartialEq)]
pub struct CacheSignature {
    state_version: u64,
    layout_bounds: Rectangle,
    layers: Vec<LayerIdentifier>,
    force_redraw: bool,
}

impl CacheSignature {
    pub fn new<
        AxisId: Hash + Eq + Clone,
        Domain: Float,
        Message: Clone,
        Renderer: crate::Renderer,
        Tag: Hash + Eq + Clone,
        Theme,
    >(
        state: &State<AxisId, Domain, Theme>,
        layout: &Layout<'_>,
        layers: &[Layer<'_, AxisId, Domain, Message, Tag, Renderer, Theme>],
    ) -> Self {
        let mut force_redraw = false;
        Self {
            state_version: state.version(),
            layout_bounds: layout.bounds(),
            layers: layers
                .iter()
                .map(|l| {
                    // Make sure we force a redraw if no version is provided for any layer
                    if !force_redraw {
                        force_redraw = l.items.version().is_none();
                    }

                    LayerIdentifier {
                        id: l.items.id(),
                        version: l.items.version(),
                    }
                })
                .collect(),
            force_redraw,
        }
    }
}

/// Internal chart memory
pub struct Memory<AxisId, Message: Clone, Tag: Hash + Eq + Clone, Renderer: crate::Renderer> {
    pub action: Action<AxisId, Tag>,
    pub previous_click: Option<mouse::Click>,
    pub cache: Option<RefCell<RenderCache<Renderer>>>,
    pub debug_cache: Option<RefCell<RenderCache<Renderer>>>,
    pub last_signature: Option<CacheSignature>,
    pub interaction_cache: RefCell<InteractionsCache<Message, Tag>>,
    pub last_hovered_identity: HoverIdentity<AxisId, Tag>,
    pub partition_grid: Vec<Rectangle>,
    pub keyboard_modifiers: keyboard::Modifiers,
}

impl<AxisId, Tag: Hash + Eq + Clone, Message: Clone, Renderer: crate::Renderer>
    Memory<AxisId, Message, Tag, Renderer>
{
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            cache: None,
            debug_cache: None,
            interaction_cache: RefCell::new(InteractionsCache::new()),
            last_signature: None,
            last_hovered_identity: HoverIdentity::OutsideBounds,
            partition_grid: Vec::new(),
            keyboard_modifiers: keyboard::Modifiers::NONE,
        }
    }

    pub const fn update_modifiers(&mut self, modifiers: keyboard::Modifiers) {
        self.keyboard_modifiers = modifiers;
    }

    pub fn update_click(&mut self, position: Point, button: mouse::Button) -> mouse::Click {
        let click = mouse::Click::new(position, button, self.previous_click);
        self.previous_click = Some(click);
        click
    }

    pub fn update_partitions(&mut self, viewport: Rectangle) {
        // NOTE: This ensure dynamic sizing of the partition grid.
        // - 400×300 chart → 4×3 grid (12 cells)
        // - 800×600 chart → 8×6 grid (48 cells)
        // - 1920×1080 chart → 20×11 grid (220 cells)
        // - 2560×1440 chart → 26×15 grid (390 cells)
        const TARGET_CELL_SIZE: f32 = 100.0;

        self.partition_grid.clear();

        let grid_cols = (viewport.width / TARGET_CELL_SIZE).ceil().max(1.0) as usize;
        let grid_rows = (viewport.height / TARGET_CELL_SIZE).ceil().max(1.0) as usize;

        let cell_width = viewport.width / grid_cols as f32;
        let cell_height = viewport.height / grid_rows as f32;

        for row in 0..grid_rows {
            for col in 0..grid_cols {
                let x = (col as f32).mul_add(cell_width, viewport.x);
                let y = (row as f32).mul_add(cell_height, viewport.y);

                self.partition_grid.push(Rectangle {
                    x,
                    y,
                    width: cell_width,
                    height: cell_height,
                });
            }
        }
    }

    pub fn draw_partitions(&self, renderer: &mut Renderer, plot_bounds: Rectangle) {
        renderer.start_layer(plot_bounds);

        let border = Border {
            color: Color::from_rgba(0.3, 0.5, 0.8, 0.4),
            width: 1.0,
            radius: 0.0.into(),
        };

        for partition in &self.partition_grid {
            renderer.fill_quad(
                Quad {
                    bounds: *partition,
                    border,
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        }

        renderer.end_layer();
    }

    pub fn update(&mut self, signature: CacheSignature) {
        if signature.force_redraw || self.last_signature.as_ref() != Some(&signature) {
            // Update signature
            self.last_signature = Some(signature);

            // Clear render cache
            if let Some(cache) = self.cache.as_ref() {
                cache.borrow_mut().request_redraw();
            };

            // Clear debug cache
            if let Some(debug_cache) = self.debug_cache.as_ref() {
                debug_cache.borrow_mut().request_redraw();
            };

            // Clear interaction cache
            self.interaction_cache.borrow_mut().clear();
        }
    }

    pub fn cache_needs_redraw(&self) -> bool {
        self.cache
            .as_ref()
            .is_some_and(|cache| cache.borrow().needs_redraw())
    }

    pub fn make_sure_cache_is_initialized(&mut self, renderer: &Renderer, quality: Quality) {
        // Initialize main cache
        if let Some(cache) = &self.cache {
            cache.borrow_mut().set_quality(quality);
        } else {
            let mut cache = match renderer.preferred_backend() {
                Backend::Mesh => RenderCache::new_mesh(),
                Backend::Path => RenderCache::new_path(),
            };
            cache.set_quality(quality);
            self.cache = Some(RefCell::new(cache));
        }

        // Initialize debug cache
        if let Some(debug_cache) = &self.debug_cache {
            debug_cache.borrow_mut().set_quality(quality);
        } else {
            let mut debug_cache = match renderer.preferred_backend() {
                Backend::Mesh => RenderCache::new_mesh(),
                Backend::Path => RenderCache::new_path(),
            };
            debug_cache.set_quality(quality);
            self.debug_cache = Some(RefCell::new(debug_cache));
        }
    }

    /// Gets a mutable reference to the internal cache
    ///
    /// Panics if the cache isn't initialized
    pub fn get_cache_mut(&self) -> Option<RefMut<'_, RenderCache<Renderer>>> {
        self.cache.as_ref().map(|buf| buf.borrow_mut())
    }

    /// Gets a mutable reference to the internal cache
    ///
    /// Panics if the cache isn't initialized
    pub fn get_debug_cache_mut(&self) -> Option<RefMut<'_, RenderCache<Renderer>>> {
        self.debug_cache.as_ref().map(|buf| buf.borrow_mut())
    }
}
