//! Versioned update contracts shared by the runtime, launcher and asset builder.
pub mod archive;
pub mod download;
pub mod install;
pub mod installer;
pub mod instructions;
pub mod manager;
pub mod manager_app;
pub mod manager_release;
pub mod manifests;
pub mod storage;
pub mod transaction;

mod ownership;
mod progress;
