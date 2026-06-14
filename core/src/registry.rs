//! Domain-neutral registry helpers for compiled mechanics definitions.

/// A registry entry addressable by a stable key.
pub trait RegistryEntry {
    fn key(&self) -> &str;
}

/// A registry entry that can materialize a compiled definition.
pub trait DefinitionRegistryEntry: RegistryEntry {
    type Definition;

    fn build_definition(&self) -> Self::Definition;
}

/// Deterministic registry over caller-owned entry records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Registry<'a, Entry> {
    entries: &'a [Entry],
}

impl<'a, Entry> Registry<'a, Entry> {
    /// Creates a registry from entries ordered by the caller.
    #[must_use]
    pub const fn new(entries: &'a [Entry]) -> Self {
        Self { entries }
    }

    /// Returns all registered entries in caller-owned order.
    #[must_use]
    pub const fn entries(&self) -> &'a [Entry] {
        self.entries
    }

    /// Finds the first entry accepted by `predicate`.
    pub fn lookup<F>(&self, mut predicate: F) -> Option<&'a Entry>
    where
        F: FnMut(&Entry) -> bool,
    {
        self.entries.iter().find(|entry| predicate(*entry))
    }
}

impl<'a, Entry> Registry<'a, Entry>
where
    Entry: RegistryEntry,
{
    /// Finds an entry by stable key.
    #[must_use]
    pub fn lookup_key(&self, key: &str) -> Option<&'a Entry> {
        self.lookup(|entry| entry.key() == key)
    }
}

impl<'a, Entry> Registry<'a, Entry>
where
    Entry: DefinitionRegistryEntry,
{
    /// Builds a definition from an entry found by stable key.
    pub fn definition(&self, key: &str) -> Option<Entry::Definition> {
        self.lookup_key(key)
            .map(DefinitionRegistryEntry::build_definition)
    }
}
