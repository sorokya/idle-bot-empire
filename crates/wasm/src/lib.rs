use ibe_sim::Game;
use wasm_bindgen::prelude::*;

/// Thin host boundary. The browser owns the clock and the storage;
/// everything else lives in `ibe-sim`.
#[wasm_bindgen]
pub struct IbeGame {
    game: Game,
}

#[wasm_bindgen]
impl IbeGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: f64) -> IbeGame {
        IbeGame {
            game: Game::new(seed as u64),
        }
    }

    /// Advance by wall-clock milliseconds since the last call.
    pub fn tick(&mut self, elapsed_ms: f64) {
        self.game.tick(elapsed_ms / 1000.0);
    }

    pub fn view(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.game.view()).unwrap_or(JsValue::NULL)
    }

    /// Returns an error message for the log, or null on success.
    pub fn dispatch(&mut self, cmd: &str) -> Option<String> {
        self.game.dispatch(cmd).err()
    }

    pub fn save(&mut self, unix_secs: f64) -> String {
        self.game.save(unix_secs)
    }

    #[wasm_bindgen(js_name = loadSave)]
    pub fn load_save(json: &str, unix_secs: f64) -> Result<IbeGame, JsValue> {
        Game::load(json, unix_secs)
            .map(|game| IbeGame { game })
            .map_err(|e| JsValue::from_str(&e))
    }
}
