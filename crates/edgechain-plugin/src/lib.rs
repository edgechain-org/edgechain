pub mod error;
pub mod plugin;
pub mod registry;
pub mod manifest;

pub use error::PluginError;
pub use plugin::{EdgePlugin, EdgeRegistry};
pub use registry::PluginManager;
pub use manifest::PluginManifest;
