#[path = "../application/gameplay/mod.rs"]
mod gameplay;

pub(crate) use gameplay::{components, mechanics};
pub use gameplay::{GamePlugin, Screen, systems};
