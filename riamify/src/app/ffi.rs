#![allow(clippy::missing_safety_doc)]

use super::*;

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppCreate(err: *mut *mut i8) -> *mut RiamifyApp {
    use std::ffi::CString;

    if !err.is_null() {
        *err = std::ptr::null_mut();
    }

    match RiamifyApp::create_app() {
        Ok(app) => Box::leak(Box::new(app)),
        Err(e) => {
            let e = format!("{}", e);
            if !err.is_null() {
                let e = CString::new(e).unwrap().into_raw();
                *err = e;
            }

            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppSpeak(
    app: *mut RiamifyApp,
    message: *const i8,
    error: *mut *mut i8,
) {
    use std::ffi::{CStr, CString};
    let app = &*app;
    if let Err(e) = app.speak(CStr::from_ptr(message).to_str().unwrap_or("")) {
        let e = format!("{}", e);
        if !error.is_null() {
            let e = CString::new(e).unwrap().into_raw();
            *error = e;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppGetPresetsLength(app: *mut RiamifyApp) -> usize {
    let app = &*app;
    app.presets.len()
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppGetPresetName(app: *mut RiamifyApp, id: i32) -> &'static str {
    let app = &*app;
    app.presets.name(id as usize)
}

// TODO: エラー処理を返す
#[no_mangle]
pub unsafe extern "C" fn RiamifyAppUpdatePresets(app: *mut RiamifyApp) -> bool {
    let (settings, presets) = if let Ok(p) = load_presets() {
        p
    } else {
        // TODO: expose error
        return false;
    };

    let app = &mut *app;

    let s_changed = settings != app.settings;
    let p_changed = presets != app.presets;

    if s_changed {
        app.settings = settings;

        let mut dictionary = Dictionary::default();
        // 設定の辞書を読み込む
        let patterns: Vec<_> = app
            .settings
            .dictionary
            .iter()
            .map(|w| Pattern {
                priority: 0,
                from: &w.word,
                to: &w.replace,
            })
            .collect();
        dictionary.load_custom(&*patterns);

        for f in &app.settings.external_dictionary {
            if dictionary.load(f).is_err() {
                eprintln!("TODO: failed to read dictionary");
                return false;
            }
        }

        dictionary.build_ac();

        app.dictionary = dictionary;
    }

    if p_changed {
        app.presets = presets;
    }

    s_changed || p_changed
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppSetPreset(app: *mut RiamifyApp, id: i32) {
    let app = &mut *app;
    if id < (app.presets.len() as i32) {
        app.preset = id;
    }
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppSetSpeed(app: *mut RiamifyApp, speed: i32) {
    let app = &mut *app;
    app.speed = speed;
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppRelease(app: *mut RiamifyApp) {
    drop(Box::from_raw(app));
}

#[repr(C)]
pub struct Waveform {
    pub waveform: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppGenerateWaveform(
    app: *mut RiamifyApp,
    message: *const i8,
    error: *mut *mut i8,
) -> Waveform {
    use std::ffi::{CStr, CString};

    let app = &*app;
    let message = CStr::from_ptr(message).to_str().unwrap_or("");

    // TODO: speed
    app.generate_waveform(message).unwrap_or_else(|e| {
        let e = format!("{}", e);
        if !error.is_null() {
            let e = CString::new(e).unwrap().into_raw();
            *error = e;
        }

        Waveform {
            waveform: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppGenerateWaveformIntoFile(
    app: *mut RiamifyApp,
    message: *const i8,
    error: *mut *mut i8,
) -> *mut i8 {
    use std::ffi::{CStr, CString};

    let app = &*app;
    let message = CStr::from_ptr(message).to_str().unwrap_or("");

    // TODO: speed
    app.write_waveform(message)
        .map(|uri| {
            CString::new(uri)
                .expect("failed to create CString.")
                .into_raw()
        })
        .unwrap_or_else(|e| {
            let e = format!("{}", e);
            if !error.is_null() {
                let e = CString::new(e).expect("failed to emit error").into_raw();
                *error = e;
            }

            std::ptr::null_mut()
        })
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppReleaseWaveform(waveform: Waveform) {
    let Waveform {
        waveform,
        len,
        capacity,
    } = waveform;

    if !waveform.is_null() {
        Vec::from_raw_parts(waveform, len, capacity);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RiamifyAppGetYomi(app: *mut RiamifyApp, msg: *const i8) -> *const i8 {
    use crate::yomi;
    use std::ffi::{CStr, CString};

    let msg = CStr::from_ptr(msg).to_string_lossy().to_lowercase();
    let app = &*app;

    let msg = &app.dictionary.try_replace(msg);

    CString::new(yomi::get_aquestalk_yomi(
        msg,
        app.settings.bouyomi.unwrap_or_default(),
    ))
    .map(|s| s.into_raw())
    .unwrap_or(std::ptr::null_mut())
}
