//! Domain-neutral typed tag primitives.

use std::borrow::Borrow;

/// One grouped hierarchical tag path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tag<Atom> {
    atoms: Vec<Atom>,
}

impl<Atom> Tag<Atom> {
    /// Creates a tag from a grouped path of atoms.
    #[must_use]
    pub fn new<I>(atoms: I) -> Self
    where
        I: IntoIterator<Item = Atom>,
    {
        Self {
            atoms: atoms.into_iter().collect(),
        }
    }

    /// Returns this tag's atoms as one grouped path.
    #[must_use]
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}

impl<Atom> Tag<Atom>
where
    Atom: Eq,
{
    /// Returns true when this exact path contains `atom`.
    #[must_use]
    pub fn has_atom<Query>(&self, atom: &Query) -> bool
    where
        Atom: Borrow<Query>,
        Query: Eq + ?Sized,
    {
        self.atoms
            .iter()
            .any(|candidate| candidate.borrow() == atom)
    }

    /// Returns true when this tag starts with `prefix`.
    #[must_use]
    pub fn starts_with(&self, prefix: &Self) -> bool {
        self.atoms.starts_with(prefix.atoms())
    }

    /// Returns true when every atom appears inside this one grouped path.
    #[must_use]
    pub fn has_all_atoms<I, Query>(&self, atoms: I) -> bool
    where
        Atom: Borrow<Query>,
        I: IntoIterator,
        I::Item: Borrow<Query>,
        Query: Eq + ?Sized,
    {
        atoms.into_iter().all(|atom| self.has_atom(atom.borrow()))
    }
}

/// Query over exact tag paths.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TagSetQuery<Atom> {
    pub all: Vec<Tag<Atom>>,
    pub any: Vec<Tag<Atom>>,
    pub none: Vec<Tag<Atom>>,
}

/// A collection of grouped tag paths.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TagSet<Atom> {
    items: Vec<Tag<Atom>>,
}

impl<Atom> TagSet<Atom> {
    /// Creates a tag set from grouped tag paths.
    #[must_use]
    pub fn new<I>(items: I) -> Self
    where
        I: IntoIterator<Item = Tag<Atom>>,
    {
        Self {
            items: items.into_iter().collect(),
        }
    }

    /// Returns grouped tag paths in insertion order.
    #[must_use]
    pub fn items(&self) -> &[Tag<Atom>] {
        &self.items
    }
}

impl<Atom> TagSet<Atom>
where
    Atom: Eq,
{
    /// Returns true when the exact tag path exists.
    #[must_use]
    pub fn has(&self, tag: &Tag<Atom>) -> bool {
        self.items.iter().any(|existing| existing == tag)
    }

    /// Returns true when any grouped path contains `atom`.
    #[must_use]
    pub fn has_atom<Query>(&self, atom: &Query) -> bool
    where
        Atom: Borrow<Query>,
        Query: Eq + ?Sized,
    {
        self.items.iter().any(|tag| tag.has_atom(atom))
    }

    /// Returns true when any grouped path starts with `prefix`.
    #[must_use]
    pub fn has_prefix(&self, prefix: &Tag<Atom>) -> bool {
        self.items.iter().any(|tag| tag.starts_with(prefix))
    }

    /// Returns true when one grouped path contains all requested atoms.
    #[must_use]
    pub fn has_tag_with_all_atoms<I, Query>(&self, atoms: I) -> bool
    where
        Atom: Borrow<Query>,
        I: IntoIterator,
        I::Item: Borrow<Query>,
        Query: Eq + ?Sized,
    {
        let atoms: Vec<I::Item> = atoms.into_iter().collect();

        self.items
            .iter()
            .any(|tag| tag.has_all_atoms(atoms.iter().map(|atom| atom.borrow())))
    }

    /// Evaluates an exact-path query.
    #[must_use]
    pub fn matches(&self, query: &TagSetQuery<Atom>) -> bool {
        if query.none.iter().any(|tag| self.has(tag)) {
            return false;
        }
        if query.all.iter().any(|tag| !self.has(tag)) {
            return false;
        }
        query.any.is_empty() || query.any.iter().any(|tag| self.has(tag))
    }
}

/// Common behavior needed by ability and active-effect stores.
pub trait TagCollection: Clone {
    type Tag: Clone;

    fn has_tag(&self, tag: &Self::Tag) -> bool;
}

impl<Atom> TagCollection for TagSet<Atom>
where
    Atom: Clone + Eq,
{
    type Tag = Tag<Atom>;

    fn has_tag(&self, tag: &Self::Tag) -> bool {
        self.has(tag)
    }
}
