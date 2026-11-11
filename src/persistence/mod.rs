#[cfg(not(target_arch = "wasm32"))]
#[path = "../infrastructure/persistence/desktop.rs"]
pub mod desktop;
#[cfg(target_arch = "wasm32")]
#[path = "../infrastructure/persistence/web.rs"]
pub mod web;

#[path = "../infrastructure/persistence/models.rs"]
mod models;
#[path = "../infrastructure/persistence/sanitize.rs"]
mod sanitize;

pub use models::*;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct PersistentData(pub SaveData);

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn load() -> SaveData {
    match platform_load() {
        Ok(mut data) => {
            if data.schema_version > CURRENT_SCHEMA_VERSION {
                bevy::log::warn!(
                    "Save data schema {} is newer than this build supports ({}); keeping the save read-only",
                    data.schema_version,
                    CURRENT_SCHEMA_VERSION
                );
                return SaveData { save_allowed: false, ..SaveData::default() };
            }
            data.save_allowed = true;
            data.schema_version = CURRENT_SCHEMA_VERSION;
            sanitize::normalize_loaded_data(&mut data);
            data
        }
        Err(error) => {
            bevy::log::warn!(
                "Could not load player data; using defaults and leaving the original save untouched: {error}"
            );
            SaveData { save_allowed: false, ..SaveData::default() }
        }
    }
}

pub fn save(data: &SaveData) {
    if !data.save_allowed {
        bevy::log::warn!(
            "Player data is read-only because the existing save could not be safely loaded; refusing to overwrite it"
        );
        return;
    }
    if let Err(error) = platform_save(data) { bevy::log::error!("Could not save player data: {error}"); }
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_load() -> Result<SaveData, String> { desktop::load() }
#[cfg(target_arch = "wasm32")]
fn platform_load() -> Result<SaveData, String> { web::load() }
#[cfg(not(target_arch = "wasm32"))]
fn platform_save(data: &SaveData) -> Result<(), String> { desktop::save(data) }
#[cfg(target_arch = "wasm32")]
fn platform_save(data: &SaveData) -> Result<(), String> { web::save(data) }
