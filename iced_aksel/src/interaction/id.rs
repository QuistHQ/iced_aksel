use std::{
    borrow,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// An identifier for an interaction.
///
/// # Examples
///
/// ```rust
/// use iced_aksel::interaction::Id;
///
/// // Create a static identifier
/// let id = Id::new("my-button");
///
/// // Create a unique identifier
/// let unique_id = Id::unique();
///
/// // Create an identifier with a tag
/// let tagged_id = id.with_tag(42);
/// assert_eq!(tagged_id.tag(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<Tag = ()> {
    identifier: Identifier,
    tag: Option<Tag>,
}

impl<Tag: Hash + Eq + Clone> Id<Tag> {
    /// Creates a new custom [`Id`]
    pub const fn new(id: &'static str) -> Self {
        Self {
            identifier: Identifier::Custom(borrow::Cow::Borrowed(id)),
            tag: None,
        }
    }

    /// Creates a new **unique** id.
    ///
    /// This is gauranteed to be unique
    pub fn unique() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            identifier: Identifier::Unique(id),
            tag: None,
        }
    }

    /// Appends a tag to the [`Id`]
    ///
    /// This is useful for having a single "parent" id, that is tagged with a "sub-id" to support
    /// granular interactions within the same general area
    pub fn with_tag(&self, tag: Tag) -> Self {
        Self {
            identifier: self.identifier.clone(),
            tag: Some(tag),
        }
    }

    /// Returns the parent [`Identifier`]
    pub const fn parent(&self) -> &Identifier {
        &self.identifier
    }

    /// Returns the tag of the [`Id`], if any
    pub fn tag(&self) -> Option<Tag> {
        self.tag.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    Unique(usize),
    Custom(borrow::Cow<'static, str>),
}
