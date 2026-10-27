//! wasm-bindgen boundary for the browser host.

use ibe_sim::{Command, EngineError, Game, Upgrade};
use wasm_bindgen::prelude::*;

/// Thin host boundary. The browser owns the clock and the storage;
/// everything else lives in `ibe-sim`.
#[wasm_bindgen]
pub struct IbeGame {
    game: Game,
}

#[wasm_bindgen]
impl IbeGame {
    /// Creates a new game from a browser-provided seed.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn new(seed: f64) -> Self {
        // JS seeds are non-negative integers.
        Self {
            game: Game::new(seed as u64),
        }
    }

    /// Advances the simulation by wall-clock milliseconds since the last call.
    pub fn tick(&mut self, elapsed_ms: f64) {
        self.game.tick(elapsed_ms / 1000.0);
    }

    /// Returns the flat view model as a JavaScript value.
    pub fn view(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.game.view()).unwrap_or(JsValue::NULL)
    }

    /// Applies a string command and returns an error message, or `undefined`.
    pub fn dispatch(&mut self, input: &str) -> Option<String> {
        match parse_command(input) {
            Ok(command) => self
                .game
                .dispatch(command)
                .err()
                .map(|error| error.to_string()),
            Err(error) => Some(error.to_string()),
        }
    }

    /// Serializes the current state to JSON.
    pub fn save(&mut self, unix_seconds: f64) -> String {
        self.game.save(unix_seconds)
    }

    /// Loads a save and applies offline progress.
    ///
    /// # Errors
    /// Returns a JS error when the save is invalid or its version is unsupported.
    #[wasm_bindgen(js_name = loadSave)]
    pub fn load_save(json: &str, unix_seconds: f64) -> Result<IbeGame, JsValue> {
        Game::load(json, unix_seconds)
            .map(|game| Self { game })
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }
}

fn parse_command(input: &str) -> Result<Command, EngineError> {
    let (verb, argument) = match input.split_once(':') {
        Some((verb, argument)) => (verb.trim(), Some(argument.trim())),
        None => (input.trim(), None),
    };

    match verb {
        "buy_account" => Ok(Command::BuyAccount),
        "buy_slot" => Ok(Command::BuySlot),
        "sell" => Ok(Command::Sell),
        "click" => Ok(Command::Click),
        "upgrade" => match argument.unwrap_or("") {
            "script" => Ok(Command::Upgrade(Upgrade::Script)),
            "proxy" => Ok(Command::Upgrade(Upgrade::Proxy)),
            "humanizer" => Ok(Command::Upgrade(Upgrade::Humanizer)),
            other => Err(EngineError::UnknownUpgrade(other.to_string())),
        },
        "pause" => Ok(Command::TogglePause(parse_account_id(argument)?)),
        "burn" => Ok(Command::Burn(parse_account_id(argument)?)),
        "aggression" => Ok(Command::SetAggression(parse_aggression(argument)?)),
        "face_sweep" => Ok(Command::FaceSweep),
        "prestige" => Ok(Command::Prestige),
        "wait" => Ok(Command::Wait(parse_wait(argument)?)),
        other => Err(EngineError::UnknownCommand(other.to_string())),
    }
}

fn parse_account_id(argument: Option<&str>) -> Result<u32, EngineError> {
    let value =
        argument.ok_or_else(|| EngineError::InvalidArgument("Account id required".to_string()))?;
    value
        .trim_start_matches("#A")
        .trim_start_matches("#a")
        .parse()
        .map_err(|_| EngineError::InvalidArgument("Account id must be a number".to_string()))
}

fn parse_aggression(argument: Option<&str>) -> Result<f64, EngineError> {
    argument
        .unwrap_or("1")
        .parse()
        .map_err(|_| EngineError::InvalidArgument("Aggression must be a number".to_string()))
}

fn parse_wait(argument: Option<&str>) -> Result<f64, EngineError> {
    argument
        .unwrap_or("0")
        .parse()
        .map_err(|_| EngineError::InvalidArgument("wait needs seconds".to_string()))
}

#[cfg(test)]
mod tests {
    use super::parse_command;
    use ibe_sim::{Command, Upgrade};

    #[test]
    fn parses_core_commands() {
        assert_eq!(parse_command("buy_account").unwrap(), Command::BuyAccount);
        assert_eq!(
            parse_command("upgrade:script").unwrap(),
            Command::Upgrade(Upgrade::Script)
        );
        assert_eq!(
            parse_command("pause:#A03").unwrap(),
            Command::TogglePause(3)
        );
        assert_eq!(
            parse_command("aggression:1.4").unwrap(),
            Command::SetAggression(1.4)
        );
    }

    #[test]
    fn rejects_invalid_commands() {
        assert!(parse_command("nope").is_err());
        assert!(parse_command("upgrade:nope").is_err());
        assert!(parse_command("pause:#Axx").is_err());
    }
}
