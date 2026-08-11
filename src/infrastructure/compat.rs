// Shared between platforms
pub use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pub use wasmtimer::std::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
