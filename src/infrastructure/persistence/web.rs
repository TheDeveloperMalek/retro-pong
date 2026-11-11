use super::SaveData;
use wasm_bindgen::JsValue;

const SAVE_KEY: &str = "retro-pong.save.v1";

fn storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or_else(|| "browser window is unavailable".to_string())?
        .local_storage()
        .map_err(js_error)?
        .ok_or_else(|| "browser LocalStorage is unavailable".to_string())
}

fn js_error(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| "browser storage error".to_string())
}

pub fn load() -> Result<SaveData, String> {
    let Some(raw) = storage()?.get_item(SAVE_KEY).map_err(js_error)? else {
        return Ok(SaveData::default());
    };
    serde_json::from_str(&raw).map_err(|error| format!("browser save could not be parsed: {error}"))
}

pub fn save(data: &SaveData) -> Result<(), String> {
    let json = serde_json::to_string(data).map_err(|error| error.to_string())?;
    storage()?.set_item(SAVE_KEY, &json).map_err(js_error)
}
