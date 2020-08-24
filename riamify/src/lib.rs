#![allow(non_snake_case)]

mod ffi;
pub use ffi::AquesTalk;

pub mod dictionary;
pub mod player;
pub mod settings;
pub mod yomi;

pub mod app;
pub use app::*;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/Oエラーです：{}", _0)]
    IoError(std::io::Error),
    #[fail(display = "YAMLのパースに失敗しました：{}", _0)]
    YamlError(serde_yaml::Error),
    #[fail(display = "DLLの読み込みに失敗しました")]
    DllError,
    #[fail(display = "その他のエラーです")]
    Other,
    #[fail(display = "メモリが不足しています")]
    OutOfMemory,
    #[fail(display = "処理できない記号を含んでいます")]
    UndefinedSymbol,
    #[fail(display = "negaive prosodyエラーです")]
    NegativeProsody,
    #[fail(display = "内部エラーで寸")]
    InternalError,
    #[fail(display = "不正なタグです")]
    InvalidTag,
    #[fail(display = "タグが長すぎます")]
    TooLongTag,
    #[fail(display = "不正な数字です")]
    InvalidTagNumber,
    #[fail(display = "データがありません")]
    NoData,
    #[fail(display = "長すぎます")]
    TooLong,
    #[fail(display = "フレーズが長すぎます")]
    TooLongPhrase,
    #[fail(display = "オーバーフローしました")]
    Overflow,
    #[fail(display = "ヒープメモリが不足しています")]
    LowHeapMemory,
    #[fail(display = "{}", _0)]
    Custom(String),
}

impl From<std::io::Error> for crate::Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_yaml::Error> for crate::Error {
    fn from(e: serde_yaml::Error) -> Self {
        Self::YamlError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn RiamifyReleasePointer(p: *mut i8) {
    use std::ffi::CString;

    drop(CString::from_raw(p));
}
