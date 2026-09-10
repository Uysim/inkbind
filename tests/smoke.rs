//! End-to-end smoke tests for the public crate surface.

use inkbind::{Error, VERSION};

#[test]
fn version_matches_cargo_package_version() {
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    assert!(!VERSION.is_empty());
}

#[test]
fn error_type_is_thread_safe_and_static() {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<Error>();
}

#[test]
fn error_implements_std_error() {
    fn assert_std_error<T: std::error::Error>() {}
    assert_std_error::<Error>();
}
