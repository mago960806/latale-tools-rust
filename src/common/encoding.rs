//! Encoding utilities for LaTale file formats
//!
//! Provides encoding detection and conversion for different regional versions.

/// Default encoding for LaTale files (Chinese version)
pub const DEFAULT_ENCODING: &str = "GBK";

/// Get encoding from name string
///
/// Supported encodings:
/// - `GBK` / `GB2312` / `GB18030` → GBK (Chinese, default)
/// - `BIG5` / `BIG-5` → BIG5 (Taiwan)
/// - `EUC-KR` / `EUCKR` / `KOREAN` → EUC-KR (Korean)
/// - `SHIFT_JIS` / `SHIFTJIS` / `SJIS` / `CP932` / `JAPANESE` → Shift-JIS (Japanese)
/// - `UTF-8` / `UTF8` → UTF-8
pub fn encoding_from_name(name: &str) -> &'static encoding_rs::Encoding {
    match name.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => encoding_rs::UTF_8,
        "BIG5" | "BIG-5" => encoding_rs::BIG5,
        "EUC-KR" | "EUCKR" | "KOREAN" => encoding_rs::EUC_KR,
        "GBK" | "GB2312" | "GB18030" => encoding_rs::GBK,
        "SHIFT_JIS" | "SHIFTJIS" | "SJIS" | "CP932" | "JAPANESE" => encoding_rs::SHIFT_JIS,
        _ => encoding_rs::GBK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_from_name_gbk() {
        assert_eq!(encoding_from_name("GBK"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("gbk"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("GB2312"), encoding_rs::GBK);
        assert_eq!(encoding_from_name("GB18030"), encoding_rs::GBK);
    }

    #[test]
    fn test_encoding_from_name_big5() {
        assert_eq!(encoding_from_name("BIG5"), encoding_rs::BIG5);
        assert_eq!(encoding_from_name("big5"), encoding_rs::BIG5);
        assert_eq!(encoding_from_name("BIG-5"), encoding_rs::BIG5);
    }

    #[test]
    fn test_encoding_from_name_euc_kr() {
        assert_eq!(encoding_from_name("EUC-KR"), encoding_rs::EUC_KR);
        assert_eq!(encoding_from_name("euckr"), encoding_rs::EUC_KR);
        assert_eq!(encoding_from_name("KOREAN"), encoding_rs::EUC_KR);
    }

    #[test]
    fn test_encoding_from_name_shift_jis() {
        assert_eq!(encoding_from_name("SHIFT_JIS"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("sjis"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("CP932"), encoding_rs::SHIFT_JIS);
        assert_eq!(encoding_from_name("JAPANESE"), encoding_rs::SHIFT_JIS);
    }

    #[test]
    fn test_encoding_from_name_utf8() {
        assert_eq!(encoding_from_name("UTF-8"), encoding_rs::UTF_8);
        assert_eq!(encoding_from_name("utf8"), encoding_rs::UTF_8);
    }

    #[test]
    fn test_encoding_from_name_unknown_defaults_to_gbk() {
        assert_eq!(encoding_from_name("unknown"), encoding_rs::GBK);
        assert_eq!(encoding_from_name(""), encoding_rs::GBK);
    }
}
