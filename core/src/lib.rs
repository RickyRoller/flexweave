#![deny(unsafe_code)]
#![doc = "Domain-agnostic mechanics primitives for consumer runtimes."]
#![doc = ""]
#![doc = "This phase-one placeholder reserves the public Rust package path while"]
#![doc = "the complete Core source is prepared for this repository."]

/// Human-readable name for the Core surface reserved by this crate.
pub const CORE_SURFACE: &str = "Flexweave Core";

/// Returns the reserved Core surface name.
#[must_use]
pub const fn core_surface() -> &'static str {
    CORE_SURFACE
}

#[cfg(test)]
mod tests {
    use super::{CORE_SURFACE, core_surface};

    #[test]
    fn exposes_reserved_core_surface_name() {
        assert_eq!(core_surface(), CORE_SURFACE);
    }
}
