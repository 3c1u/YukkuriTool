use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::{Path, PathBuf};

use crate::ffi::AquesTalk;
use crate::Result;

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub(crate) prefix: PathBuf,
    pub(crate) voices: Vec<Slot>,
    pub(crate) dictionary: Vec<Word>,
    pub(crate) export: PathBuf,
    pub(crate) format: String,
    pub(crate) external_dictionary: Vec<PathBuf>,
    // 以下はオプション（後方互換性のため）
    pub(crate) delay: Option<i32>,
    pub(crate) oversample: Option<bool>,
    pub(crate) bouyomi: Option<bool>,
    pub(crate) resampling_quality: Option<ResamplingQuality>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResamplingQuality {
    Fastest,
    Medium,
    High,
    Raw,
}

impl Default for ResamplingQuality {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub replace: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotType {
    ExternalDll,
    Internal,
}

impl Default for SlotType {
    fn default() -> Self {
        Self::ExternalDll
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    #[serde(default)]
    path: PathBuf,
    name: String,
    description: String,
    #[serde(default)]
    slot_type: SlotType,
}

impl Settings {
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Ok(serde_yaml::from_slice(&*std::fs::read(filename)?)?)
    }

    pub fn load_dlls(&self) -> Result<Vec<AquesTalk>> {
        let mut voices = vec![];

        for voice in &self.voices {
            let mut p = self.prefix.clone();
            p.push(&voice.path);
            voices.push(AquesTalk::open(p)?);
        }

        Ok(voices)
    }
}
