use iced_core::{keyboard, mouse};

/// An event omitted for `on_move` handlers
#[derive(Debug, Clone, Copy)]
pub struct MoveEvent<P> {
    /// Normalized position of the cursor
    pub position: P,
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl<P> MoveEvent<P> {
    pub(crate) const fn new(position: P, modifiers: keyboard::Modifiers) -> Self {
        Self {
            position,
            modifiers,
        }
    }
}

/// An event omitted for `on_enter` handlers
#[derive(Debug, Clone, Copy)]
pub struct EnterEvent {
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl EnterEvent {
    pub(crate) const fn new(modifiers: keyboard::Modifiers) -> Self {
        Self { modifiers }
    }
}

/// An event omitted for `on_exit` handlers
#[derive(Debug, Clone, Copy)]
pub struct ExitEvent {
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl ExitEvent {
    pub(crate) const fn new(modifiers: keyboard::Modifiers) -> Self {
        Self { modifiers }
    }
}

/// An event omitted for `on_scroll` handlers
#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent<P> {
    /// Normalized position of the cursor
    pub position: P,
    /// Delta of the scroll
    pub delta: mouse::ScrollDelta,
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl<P> ScrollEvent<P> {
    pub(crate) const fn new(
        position: P,
        delta: mouse::ScrollDelta,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            delta,
            modifiers,
        }
    }
}

/// Normalized drag delta for panning operations.
///
/// Values are in the range 0.0-1.0 and can be passed directly to axis `pan` methods.
///
/// # Example
///
/// ```rust
/// use iced_aksel::plot::DragDelta;
///
/// let delta = Delta { x: 0.1, y: 0.05 };
/// // Use with state.pan_axes(..., delta.x, delta.y)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Delta {
    /// Normalized horizontal drag distance (0.0-1.0).
    pub x: f32,
    /// Normalized vertical drag distance (0.0-1.0).
    pub y: f32,
}

/// An event omitted for `on_drag` handlers
#[derive(Debug, Clone, Copy)]
pub struct DragEvent<D> {
    /// Normalized delta of the drag
    pub delta: D,
    /// Button held during drag
    pub button_held: mouse::Button,
    /// Click kind before starting the drag
    pub click_kind: mouse::click::Kind,
    /// Current keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl<D> DragEvent<D> {
    pub(crate) const fn new(
        delta: D,
        button_held: mouse::Button,
        click_kind: mouse::click::Kind,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            delta,
            button_held,
            click_kind,
            modifiers,
        }
    }
}

/// An event omitted for `on_press` events
#[derive(Debug, Clone, Copy)]
pub struct PressEvent<P> {
    /// Normalized position of the cursor
    pub position: P,
    /// Button pressed
    pub button: mouse::Button,
    /// Click kind
    pub click_kind: mouse::click::Kind,
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
}

impl<P> PressEvent<P> {
    pub(crate) const fn new(
        position: P,
        button: mouse::Button,
        click_kind: mouse::click::Kind,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
        }
    }

    /// Returns true if the `click_kind` was a `Kind::Single`
    pub fn is_single_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Single
    }

    /// Returns true if the `click_kind` was a `Kind::Double`
    pub fn is_double_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Double
    }

    /// Returns true if the `click_kind` was a `Kind::Triple`
    pub fn is_triple_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Triple
    }
}

/// An event omitted for `on_release` handlers
#[derive(Debug, Clone, Copy)]
pub struct ReleaseEvent<P> {
    /// Normalized position of the cursor
    pub position: P,
    /// Button released
    pub button: mouse::Button,
    /// Click kind before releasing
    ///
    /// Will be None if the click originated from outside the Chart
    pub click_kind: Option<mouse::click::Kind>,
    /// Keyboard modifiers
    pub modifiers: keyboard::Modifiers,
    /// Wether the user was dragging the cursor before releasing
    pub was_dragging: bool,
}

impl<P> ReleaseEvent<P> {
    pub(crate) const fn new(
        position: P,
        button: mouse::Button,
        click_kind: Option<mouse::click::Kind>,
        modifiers: keyboard::Modifiers,
        was_dragging: bool,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
            was_dragging,
        }
    }

    /// Returns true if the `click_kind` was a `Some(Kind::Single)`
    pub fn is_single_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Single)
    }

    /// Returns true if the `click_kind` was a `Some(Kind::Double)`
    pub fn is_double_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Double)
    }

    /// Returns true if the `click_kind` was a `Some(Kind::Triple)`
    pub fn is_triple_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Triple)
    }
}

/// A handler that can either be a direct `Message` or a closure with arguments
pub enum Handler<Message, Args> {
    /// A direct message value
    Direct(Message),
    /// A closure, returning an optional message
    Closure(Box<dyn Fn(Args) -> Option<Message> + 'static>),
}

impl<Message: Clone, Args> Handler<Message, Args> {
    pub(crate) fn run(&self, args: Args) -> Option<Message> {
        match self {
            Self::Direct(m) => Some(m.clone()),
            Self::Closure(f) => f(args),
        }
    }
}

// Marker struct to circumvent rust overlapping traits.
/// Fn marker for closures
pub struct FnMarker;
/// Direct marker for messages
pub struct DirectMarker;

/// Converts direct messages or closures into a [`Handler`]
pub trait IntoHandler<Message, Args, Marker> {
    /// Convert `self` into a [`Handler`]
    fn into_handler(self) -> Handler<Message, Args>;
}

impl<Message, Args> IntoHandler<Message, Args, DirectMarker> for Message
where
    Message: Clone + 'static,
{
    fn into_handler(self) -> Handler<Message, Args> {
        Handler::Direct(self)
    }
}

/// Implmements `on_<action>` and `on_<action>_with` methods for event handlers
///
/// The field name for the handler is assumed to be `on_<action>` with a type of [`Option<Handler<Message, Args>>`](Handler).
macro_rules! impl_handlers {
    (
        $(
            $(#[$doc:meta])*
            $action:ident : $Args:ty
        );+ $(;)?
    ) => {
        paste::paste! {
            $(
                $(#[$doc])*
                pub fn [<on_ $action>]<H, Marker>(mut self, h: H) -> Self
                where
                    H: crate::event::IntoHandler<Message, $Args, Marker>,
                {
                    self.[<on_ $action>] = Some(h.into_handler());
                    self
                }
            )+
        }
    };
}

pub(crate) use impl_handlers;

macro_rules! impl_into_handler_for_fn {
    // 0 args
    () => {
        impl<Message, F, R> IntoHandler<Message, (), FnMarker> for F
        where
            F: Fn() -> R + 'static,
            R: Into<Option<Message>>,
            Message: 'static,
        {
            fn into_handler(self) -> Handler<Message, ()> {
                Handler::Closure(Box::new(move |()| self().into()))
            }
        }
    };

    // N args (stored as a tuple Args = (A, B, ...))
    ($($A:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        impl<Message, F, R, $($A),+> IntoHandler<Message, ($($A,)+), FnMarker> for F
        where
            F: Fn($($A),+) -> R + 'static,
            R: Into<Option<Message>>,
            Message: 'static,
        {
            fn into_handler(self) -> Handler<Message, ($($A,)+)> {
                Handler::Closure(Box::new(move |args: ($($A,)+)| {
                    let ($($A,)+) = args;
                    self($($A),+).into()
                }))
            }
        }
    };
}

impl_into_handler_for_fn!();
impl_into_handler_for_fn!(A);
impl_into_handler_for_fn!(A, B);
