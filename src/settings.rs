use serde::{Deserialize, Serialize};

use crate::utils;
use std::fs;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub ocr_lang: String,
    pub tra_lang: String,
    pub tra_provider: String,
}

impl Settings {
    pub fn ocr_lang(&self) -> &str {
        &self.ocr_lang
    }

    pub fn tra_lang(&self) -> &str {
        &self.tra_lang
    }

    pub fn tra_provider(&self) -> &str {
        if self.tra_provider.is_empty() {
            "google"
        } else {
            &self.tra_provider
        }
    }

    pub fn set(&mut self, prop: &str, value: String) -> Result<(), anyhow::Error> {
        match prop {
            "tra-lang" => {
                self.tra_lang = value;
            }
            "tra-provider" => {
                self.tra_provider = value;
            }
            "ocr-lang" => {
                self.ocr_lang = value;
            }
            &_ => {}
        }
        if let Err(err) = self.update_json() {
            println!("Failed to update settings: {err:?}");
        }
        Ok(())
    }

    pub fn update_json(&self) -> Result<(), anyhow::Error> {
        if let Ok(path) = utils::settings_path() {
            let file = fs::File::create(path)?;
            serde_json::to_writer(file, self)?;
        }
        Ok(())
    }
}
