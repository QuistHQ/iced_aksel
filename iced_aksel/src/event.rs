use iced_core::{keyboard, mouse};

#[derive(Debug, Clone, Copy)]
pub struct MoveEvent<P> {
    pub position: P,
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

#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent<P> {
    pub position: P,
    pub delta: mouse::ScrollDelta,
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

#[derive(Debug, Clone, Copy)]
pub struct DragEvent<D> {
    pub delta: D,
    pub button_held: mouse::Button,
    pub click_kind: mouse::click::Kind,
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

#[derive(Debug, Clone, Copy)]
pub struct PressEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: mouse::click::Kind,
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

    pub fn is_single_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Single
    }

    pub fn is_double_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Double
    }

    pub fn is_triple_click(&self) -> bool {
        self.click_kind == mouse::click::Kind::Triple
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReleaseEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: Option<mouse::click::Kind>,
    pub modifiers: keyboard::Modifiers,
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

    pub fn is_single_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Single)
    }

    pub fn is_double_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Double)
    }

    pub fn is_triple_click(&self) -> bool {
        self.click_kind
            .is_some_and(|kind| kind == mouse::click::Kind::Triple)
    }
}

pub enum Handler<Message, Args> {
    Direct(Message),
    Closure(Box<dyn Fn(Args) -> Option<Message> + 'static>),
}

impl<Message: Clone, Args> Handler<Message, Args> {
    pub fn run(&self, args: Args) -> Option<Message> {
        match self {
            Self::Direct(m) => Some(m.clone()),
            Self::Closure(f) => f(args),
        }
    }
}

// Marker traits to circumvent rust overlapping traits.
pub struct FnMarker;
pub struct DirectMarker;

pub trait IntoHandler<Message, Args, Marker> {
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
