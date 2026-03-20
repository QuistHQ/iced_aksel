use std::{
    borrow,
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// An identifier for an interaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<Tag = ()> {
    identifier: Identifier,
    tag: Option<Tag>,
}

impl<Tag: Hash + Eq + Clone> Id<Tag> {
    pub const fn new(id: &'static str) -> Self {
        Self {
            identifier: Identifier::Custom(borrow::Cow::Borrowed(id)),
            tag: None,
        }
    }

    pub fn unique() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            identifier: Identifier::Unique(id),
            tag: None,
        }
    }

    pub fn with_tag(&self, tag: Tag) -> Self {
        Self {
            identifier: self.identifier.clone(),
            tag: Some(tag),
        }
    }

    pub const fn parent(&self) -> &Identifier {
        &self.identifier
    }

    pub fn tag(&self) -> Option<Tag> {
        self.tag.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    Unique(usize),
    Custom(borrow::Cow<'static, str>),
}
