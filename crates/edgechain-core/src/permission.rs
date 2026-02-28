use serde::{Deserialize, Serialize};

/// Represents a permission required by a command or plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Allows the plugin to read from the device's local file system.
    FilesystemRead,
    /// Allows the plugin to write to the device's local file system.
    FilesystemWrite,
    /// Allows the plugin to make outbound network requests.
    NetworkAccess,
    /// Allows the plugin to read the device's contacts.
    ContactsRead,
    /// Allows the plugin to access the device's location.
    LocationAccess,
    /// Allows the plugin to access the device's camera.
    CameraAccess,
    /// A custom permission defined by a string.
    Custom(String),
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::FilesystemRead => "filesystem:read",
            Permission::FilesystemWrite => "filesystem:write",
            Permission::NetworkAccess => "network:access",
            Permission::ContactsRead => "contacts:read",
            Permission::LocationAccess => "location:access",
            Permission::CameraAccess => "camera:access",
            Permission::Custom(s) => s.as_str(),
        }
    }
}
