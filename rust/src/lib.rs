//! Bounded SVG kernels, client management and independent-update backend.
pub mod helper_svg;
pub mod xml;

pub mod client_management;
pub mod update;

#[doc(hidden)]
pub use {libc, serde_json};
