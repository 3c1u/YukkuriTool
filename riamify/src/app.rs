// TODO 長過ぎ。リファクタリングしよ？

use crate::dictionary::{Dictionary, Pattern};
use crate::ffi::AquesTalk;
use crate::settings::Settings;

use serde::{Deserialize, Serialize};

pub mod ffi;
use ffi::Waveform;

use crate::settings::ResamplingQuality;

#[derive(Default)]
pub struct RiamifyApp {
    talkCache: Vec<AquesTalk>,
    dictionary: Dictionary,
    settings: Settings,
    presets: Presets,
    speed: i32,
    preset: i32,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Presets {
    presets: Vec<Preset>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Preset {
    name: String,
    speed: i32,
    pitch: i32,
    slot: usize,
    #[serde(default)]
    delay: i32,
    // プリセットの説明（後方互換性のためオプション）
    #[serde(default)]
    description: String,
}

impl Presets {
    pub fn open<P: AsRef<std::path::Path>>(filename: P) -> crate::Result<Self> {
        Ok(serde_yaml::from_slice(&*std::fs::read(filename)?)?)
    }

    pub fn len(&self) -> usize {
        self.presets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.presets.is_empty()
    }

    pub fn name(&self, id: usize) -> &str {
        self.presets.get(id).map(|p| &*p.name).unwrap_or("")
    }
}

// メチャキモい　リファクタリングしたい
impl RiamifyApp {
    fn create_app() -> crate::Result<Self> {
        if cfg!(target_os = "macos") {
            use std::env;
            let mut path = env::current_exe().unwrap();
            path.pop();
            path.pop();
            path.pop();
            path.pop();
            env::set_current_dir(path).unwrap();
        }

        let (settings, presets) = load_presets()?;
        let mut dictionary = Dictionary::default();

        // 設定の辞書を読み込む
        let patterns: Vec<_> = settings
            .dictionary
            .iter()
            .map(|w| Pattern {
                priority: 0,
                from: &w.word,
                to: &w.replace,
            })
            .collect();
        dictionary.load_custom(&*patterns);

        for f in &settings.external_dictionary {
            dictionary.load(f)?;
        }

        dictionary.build_ac();

        Ok(RiamifyApp {
            talkCache: settings.load_dlls()?,
            settings,
            presets,
            dictionary,
            speed: 100,
            ..Default::default()
        })
    }
    fn speak(&self, message: &str) -> crate::Result<()> {
        crate::player::play_wav(self.generate_from_preset(message)?);
        Ok(())
    }

    fn raw_convert(&self, waveform: &[f32]) -> Vec<f32> {
        let preset = &self.presets.presets[self.preset as usize];
        let pitch = preset.pitch;

        let mut w = waveform.to_vec();

        let total_delay = (self.settings.delay.unwrap_or(22050) + preset.delay) as f64;
        let new_sampling_rate = 80.0 * (pitch as f64);
        let total_delay_samples = (total_delay / 44100.0 * new_sampling_rate) as usize;

        w.resize(w.len() + total_delay_samples, 0.0);

        w
    }

    fn do_upconvert(
        &self,
        do_oversample: bool,
        resampling_quality: ResamplingQuality,
        waveform: &[f32],
    ) -> Vec<f32> {
        let preset = &self.presets.presets[self.preset as usize];
        let pitch = preset.pitch;

        if resampling_quality == ResamplingQuality::Raw {
            return self.raw_convert(waveform);
        }

        let mut w = vec![];

        let waveform: &[f32] = if do_oversample {
            for &s in waveform {
                w.push(s);
                w.push(s);
            }
            &w
        } else {
            waveform
        };

        let sample_rate = if do_oversample { 16000.0 } else { 8000.0 };

        let src_ratio = (44100.0 / sample_rate) * 100.0 / (pitch as f64);

        // 空白時間もついでに足しておく
        let total_delay_samples = (self.settings.delay.unwrap_or(22050) + preset.delay) as isize;
        let mut waveform_out = vec![
            0.0;
            ((waveform.len() as f64 * src_ratio) as isize + total_delay_samples).max(0)
                as usize
        ];

        // リサンプルする
        use libsamplerate::{src_simple, SRC_DATA};

        let mut src_data = SRC_DATA {
            data_in: waveform.as_ptr(),
            data_out: waveform_out.as_mut_ptr(),
            input_frames: waveform.len() as _,
            output_frames: waveform_out.len() as _,
            input_frames_used: 0,
            output_frames_gen: 0,
            end_of_input: 0,
            src_ratio,
        };

        unsafe {
            src_simple(
                &mut src_data,
                match resampling_quality {
                    ResamplingQuality::Fastest => 2,
                    ResamplingQuality::Medium => 1,
                    ResamplingQuality::High => 0,
                    _ => unreachable!(),
                },
                1,
            );
        }

        waveform_out
    }

    fn generate_from_preset(&self, message: &str) -> crate::Result<Vec<u8>> {
        use hound::WavReader;

        let preset = &self.presets.presets[self.preset as usize];
        let pitch = preset.pitch;

        let waveform: Vec<_> =
            self.talkCache[preset.slot].speak(message, preset.speed * self.speed / 100)?;

        let mut waveform = WavReader::new(&*waveform)
            .map_err(|_| crate::Error::Custom(".WAVファイルの読み込みに失敗しました".into()))?;

        let waveform: Result<Vec<_>, _> = waveform
            .samples::<i16>()
            .map(|v| v.map(|v| (v as f32) / (std::i16::MAX as f32)))
            .collect();
        let waveform = waveform
            .map_err(|_| crate::Error::Custom(".WAVファイルの生成に失敗しました".into()))?;

        // 無理矢理アップコンバート
        let do_oversample = self.settings.oversample.unwrap_or_default();
        let resampling_quality = self.settings.resampling_quality.unwrap_or_default();

        let waveform_out = self.do_upconvert(do_oversample, resampling_quality, &waveform);

        let spec = if resampling_quality == ResamplingQuality::Raw {
            hound::WavSpec {
                channels: 1,
                sample_rate: (80.0 * (pitch as f64)) as u32,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            }
        } else {
            hound::WavSpec {
                channels: 1,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            }
        };

        let mut waveform_res = std::io::Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut waveform_res, spec)
            .map_err(|_| crate::Error::Custom(".WAVファイルの書き込みに失敗しました".into()))?;

        // -0.3 [dB] をピークにする
        let wf_gain =
            10.0f32.powf(-0.3 / 20.0) / waveform_out.iter().fold(0.0f32, |acc, x| acc.max(x.abs()));

        for v in waveform_out {
            writer
                .write_sample((wf_gain * v * (std::i16::MAX as f32)) as i16)
                .map_err(|_| crate::Error::Custom(".WAVファイルの書き込みに失敗しました".into()))?;
        }

        writer
            .finalize()
            .map_err(|_| crate::Error::Custom(".WAVファイルの書き込みに失敗しました".into()))?;

        Ok(waveform_res.into_inner())
    }

    fn generate_waveform(&self, message: &str) -> crate::Result<Waveform> {
        let mut w = self.generate_from_preset(message)?;

        let w_raw = w.as_mut_ptr();
        let len = w.len();
        let capacity = w.capacity();

        std::mem::forget(w);

        Ok(Waveform {
            waveform: w_raw,
            len,
            capacity,
        })
    }

    fn write_waveform(&self, message: &str) -> crate::Result<String> {
        use chrono::Utc;
        use std::path::PathBuf;
        use url::Url;

        let w = self.generate_from_preset(message)?;

        let mut filename = PathBuf::new();
        filename.push(&self.settings.export);
        filename.push(format!(
            "{}.wav",
            Utc::now().format(&self.settings.format).to_string()
        ));

        std::fs::write(&filename, w)?;

        Ok(Url::from_file_path(filename.canonicalize()?)
            .map_err(|_| crate::Error::InternalError)?
            .into_string())
    }
}

/// 設定の読み込みとかをする奴。
/// リファクタリングしたい．
fn load_presets() -> crate::Result<(Settings, Presets)> {
    use crate::Error;

    let settings = Settings::open("settings.yml").or_else(|err| {
        use std::fs::File;
        use std::io::prelude::*;

        let settings = include_str!("settings.yml");

        // .YAMLのパースでのエラーは、エラー箇所をログに出すが、
        // ファイルの上書きはしない
        if let Error::YamlError(err) = err {
            crate::ffi::alert(&format!("failed to parse YAML: {}.", err));
        } else {
            let f = File::create("settings.yml");
            if let Ok(mut f) = f {
                f.write_all(settings.as_bytes())
                    .expect("failed to write settings.yml");
            } else {
                eprintln!("failed to open settings.yml; reading the default settings instead.");
            }
        }

        serde_yaml::from_str(settings)
    })?;

    let presets = Presets::open("presets.yml").or_else(|err| {
        use std::fs::File;
        use std::io::prelude::*;

        let presets = include_str!("presets.yml");

        // .YAMLのパースでのエラーは、エラー箇所をログに出すが、
        // ファイルの上書きはしない
        if let Error::YamlError(err) = err {
            eprintln!("failed to parse YAML: {}", err);
            eprintln!("loading the default presets instead.");
        } else {
            let f = File::create("presets.yml");
            if let Ok(mut f) = f {
                f.write_all(presets.as_bytes())
                    .expect("failed to write presets.yml");
            } else {
                eprintln!("failed to open presets.yml; reading the default settings instead.");
            }
        }

        serde_yaml::from_str(presets)
    })?;

    Ok((settings, presets))
}
