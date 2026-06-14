use flexweave::{CORE_SURFACE, core_surface};

#[test]
fn placeholder_crate_exports_core_surface() {
    assert_eq!(core_surface(), "Flexweave Core");
    assert_eq!(CORE_SURFACE, core_surface());
}
