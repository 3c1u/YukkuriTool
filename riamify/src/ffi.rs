#![allow(unused)]

use crate::{Error, Result};
use encoding_rs::SHIFT_JIS;
use libloading as lib;
use std::path::Path;

type Synthesize<'a> = lib::Symbol<
    'a,
    unsafe extern "stdcall" fn(message: *const u8, speed: i32, total_bytes: *mut i32) -> *mut u8,
>;

extern "C" {
    #[allow(improper_ctypes)]
    fn ___RiamifyShowAlert(message: &str);
}

pub fn alert(message: &str) {
    unsafe {
        ___RiamifyShowAlert(message);
    }
}

#[allow(unused)]
type FreeWave<'a> = lib::Symbol<'a, unsafe extern "stdcall" fn(waveform: *mut u8)>;

/// AquesTalk instance.
#[derive(Debug)]
pub struct AquesTalk {
    #[cfg(windows)]
    lib: Option<lib::Library>,
    internal: Option<i32>,
}

impl AquesTalk {
    /// Loads an AquesTalk instance from a DLL.
    #[cfg(windows)]
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<AquesTalk> {
        let lib = lib::Library::new(filename.as_ref().canonicalize()?)?;

        // check if symbols are available.
        unsafe {
            let _synthesize: Synthesize = lib.get(b"AquesTalk_Synthe")?;
            let _free_wave: FreeWave = lib.get(b"AquesTalk_FreeWave")?;
        }

        Ok(AquesTalk {
            lib: Some(lib),
            internal: None,
        })
    }

    pub fn internal_slot(voice_id: i32) -> AquesTalk {
        AquesTalk {
            #[cfg(windows)]
            lib: None,
            internal: Some(voice_id),
        }
    }

    #[cfg(not(windows))]
    pub fn open<P: AsRef<Path>>(_: P) -> Result<AquesTalk> {
        Ok(AquesTalk { internal: None })
    }

    #[allow(unused)]
    fn speak_internal(&self, message: &str, speed: i32) -> Result<Vec<u8>> {
        use std::ffi::CString;

        extern "C" {
            fn AquesTalk_Synthe_Utf8(
                message: *const i8,
                speed: i32,
                total_bytes: *mut i32,
            ) -> *mut u8;
            fn AquesTalk_FreeWave(waveform: *mut u8);
        }

        let message = CString::new(message).unwrap();

        let result = unsafe {
            let mut total_bytes: i32 = 0;
            let ptr = AquesTalk_Synthe_Utf8(message.as_ptr(), speed, &mut total_bytes);

            if ptr.is_null() {
                return match total_bytes {
                    101 => Err(Error::OutOfMemory),
                    102 | 105 => Err(Error::UndefinedSymbol),
                    103 => Err(Error::NegativeProsody),
                    106 => Err(Error::InvalidTag),
                    107 => Err(Error::TooLongTag),
                    108 => Err(Error::InvalidTagNumber),
                    111 => Err(Error::NoData),
                    200 => Err(Error::TooLong),
                    201 => Err(Error::TooLongPhrase),
                    202 | 204 => Err(Error::Overflow),
                    203 => Err(Error::LowHeapMemory),
                    _ => Err(Error::Other),
                };
            }

            std::slice::from_raw_parts_mut(ptr, total_bytes as usize)
        };

        let buf = result.to_owned();

        unsafe {
            AquesTalk_FreeWave(result.as_mut_ptr());
        }

        Ok(buf)
    }

    #[cfg(not(windows))]
    pub fn speak(&self, message: &str, speed: i32) -> Result<Vec<u8>> {
        self.speak_internal(message, speed)
    }

    /// Let speak.
    #[cfg(windows)]
    pub fn speak(&self, message: &str, speed: i32) -> Result<Vec<u8>> {
        let (message, _, _) = SHIFT_JIS.encode(message);
        let message = {
            let mut message_vec: Vec<u8> = message.into_owned();
            if message_vec.last().copied() != Some(0) {
                // should be null-terminated
                message_vec.push(0);
            }

            message_vec
        };

        let synthesize: Synthesize =
            unsafe { self.lib.as_ref().unwrap().get(b"AquesTalk_Synthe")? };
        let free_wave: FreeWave = unsafe { self.lib.as_ref().unwrap().get(b"AquesTalk_FreeWave")? };

        let result = unsafe {
            let mut total_bytes: i32 = 0;
            let ptr = synthesize(message.as_ptr(), speed, &mut total_bytes);

            if ptr.is_null() {
                return match total_bytes {
                    101 => Err(Error::OutOfMemory),
                    102 | 105 => Err(Error::UndefinedSymbol),
                    103 => Err(Error::NegativeProsody),
                    106 => Err(Error::InvalidTag),
                    107 => Err(Error::TooLongTag),
                    108 => Err(Error::InvalidTagNumber),
                    111 => Err(Error::NoData),
                    200 => Err(Error::TooLong),
                    201 => Err(Error::TooLongPhrase),
                    202 | 204 => Err(Error::Overflow),
                    203 => Err(Error::LowHeapMemory),
                    _ => Err(Error::Other),
                };
            }

            std::slice::from_raw_parts_mut(ptr, total_bytes as usize)
        };

        let buf = result.to_owned();

        unsafe {
            free_wave(result.as_mut_ptr());
        }

        Ok(buf)
    }
}
