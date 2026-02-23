pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub mod plugins;

pub use error::PluginError;
pub use manifest::PluginManifest;
pub use plugin::{EdgePlugin, EdgeRegistry};
pub use registry::PluginManager;
pub use plugins::{CrmPlugin, InventoryPlugin, FieldServicePlugin};
