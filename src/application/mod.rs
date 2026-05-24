// Application layer: use cases, repository traits, DTOs.
// Depends on domain only; no DB drivers or HTTP framework imports.

// DTOs compile on all targets — server functions reference them from the client side.
pub mod dto;

// Repository traits, services, and IO parsers are SSR-only (no DB on the client).
#[cfg(feature = "ssr")]
pub mod io;
#[cfg(feature = "ssr")]
pub mod repositories;
#[cfg(feature = "ssr")]
pub mod services;
