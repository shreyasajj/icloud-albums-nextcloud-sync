pub mod auth;
pub mod client;
pub mod osconfig;

pub use auth::AuthManager;
pub use client::{Album, ICloudClient, Photo};
pub use osconfig::MacDeviceConfig;
