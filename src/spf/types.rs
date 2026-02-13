//! SPF 文件格式数据结构定义

use std::fmt;
use bytemuck::{Pod, Zeroable};

/// FILE_ID 到 SPF 文件名的映射
///
/// 通过逆向分析确认的映射关系
pub static FILE_ID_TO_NAME: &[(&str, u8, u32, u32)] = &[
    // (文件名, FILE_ID, 版本号, 文件数)
    ("AJJIYA.SPF", 1, 2022091501, 21668),
    ("ROWID.SPF", 3, 0, 341),
    ("JINSSAGA.SPF", 4, 2022100601, 11499),
    ("MAKO1298.SPF", 5, 2022091501, 1478),
    ("METALGENI.SPF", 6, 2022091501, 7596),
    ("DALBONG.SPF", 7, 2022091501, 373),
    ("RYUMS.SPF", 8, 2022091501, 13392),
    ("BANX.SPF", 9, 2022100601, 1593),
    ("BARY.SPF", 10, 0, 2233),
    ("ZENNE.SPF", 11, 2022092701, 1787),
    ("CLAIRE.SPF", 12, 2022091501, 44),
    ("CVOICE.SPF", 13, 2022091501, 90),
    ("HOSHIM.SPF", 14, 0, 0),
];

/// 根据 FILE_ID 获取 SPF 文件名
pub fn get_spf_name_by_id(file_id: u8) -> Option<&'static str> {
    FILE_ID_TO_NAME
        .iter()
        .find(|(_, id, _, _)| *id == file_id)
        .map(|(name, _, _, _)| *name)
}

/// 根据 SPF 文件名获取 FILE_ID
pub fn get_file_id_by_name(name: &str) -> Option<u8> {
    let name_upper = name.to_uppercase();
    FILE_ID_TO_NAME
        .iter()
        .find(|(spf_name, _, _, _)| spf_name.to_uppercase() == name_upper)
        .map(|(_, id, _, _)| *id)
}

/// SPF 文件信息结构 (136字节)
///
/// 布局:
/// ```text
/// Offset  Size    Field
/// ------  ------- -----
/// 0       128     szFileName (C字符串，以\0结尾)
/// 128     4       iOffset (文件数据偏移)
/// 132     4       iSize (文件大小)
/// 136     4       ResID (资源ID: 高8位=FILE_ID, 低24位=INSTANCE_ID)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FInfo {
    /// 文件名 (C字符串，最多127字符 + \0)
    pub szFileName: [u8; 128],
    /// 文件数据在SPF中的偏移量
    pub iOffset: u32,
    /// 文件大小（字节）
    pub iSize: u32,
    /// 资源ID (高8位=FILE_ID, 低24位=INSTANCE_ID)
    pub ResID: u32,
}

impl Default for FInfo {
    fn default() -> Self {
        FInfo {
            szFileName: [0; 128],
            iOffset: 0,
            iSize: 0,
            ResID: 0,
        }
    }
}

impl FInfo {
    /// 获取文件名 (去除 \0 结尾)
    pub fn filename(&self) -> Result<String, std::string::FromUtf8Error> {
        let null_pos = self.szFileName.iter().position(|&b| b == 0).unwrap_or(128);
        String::from_utf8(self.szFileName[..null_pos].to_vec())
    }

    /// 设置文件名
    pub fn set_filename(&mut self, name: &str) -> Result<(), crate::Error> {
        if name.len() >= 128 {
            return Err(crate::Error::InvalidFilename(format!(
                "Filename too long: {} (max 127)",
                name
            )));
        }
        self.szFileName.fill(0);
        self.szFileName[..name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }
}

impl fmt::Display for FInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 复制字段到局部变量以避免对齐问题
        let offset = u32::from_le_bytes(self.iOffset.to_le_bytes());
        let size = u32::from_le_bytes(self.iSize.to_le_bytes());
        let resid = u32::from_le_bytes(self.ResID.to_le_bytes());

        match self.filename() {
            Ok(name) => write!(f, "{} (offset={}, size={}, ResID={:#x})", name, offset, size, resid),
            Err(_) => write!(f, "<invalid> (offset={}, size={}, ResID={:#x})", offset, size, resid),
        }
    }
}

/// SPF 文件头结构 (40字节)
///
/// 布局:
/// ```text
/// Offset  Size    Field
/// ------  ------- -----
/// 0       4       iHeaderSize (索引表总大小，字节数)
/// 4       4       iFileID (SPF文件ID)
/// 8       32      szDesc (描述信息)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FSPFHeader {
    /// 索引表总大小（字节数）= 文件数量 * 136 (FINFO大小)
    pub iHeaderSize: u32,
    /// SPF 文件 ID (对应 FILE_ID)
    pub iFileID: u32,
    /// 描述信息 (32字节)
    pub szDesc: [u8; 32],
}

impl Default for FSPFHeader {
    fn default() -> Self {
        FSPFHeader {
            iHeaderSize: 0,
            iFileID: 0,
            szDesc: [0; 32],
        }
    }
}

impl FSPFHeader {
    /// 获取文件数量
    pub fn file_count(&self) -> u32 {
        self.iHeaderSize / layout::FINFO_SIZE
    }

    /// 获取描述信息 (去除 \0 结尾)
    pub fn description(&self) -> Result<String, std::string::FromUtf8Error> {
        let null_pos = self.szDesc.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8(self.szDesc[..null_pos].to_vec())
    }

    /// 设置描述信息
    pub fn set_description(&mut self, desc: &str) -> Result<(), crate::Error> {
        if desc.len() >= 32 {
            return Err(crate::Error::InvalidFilename(format!(
                "Description too long: {} (max 31)",
                desc
            )));
        }
        self.szDesc.fill(0);
        self.szDesc[..desc.len()].copy_from_slice(desc.as_bytes());
        Ok(())
    }
}

/// SPF 文件布局常量
pub mod layout {
    /// FINFO 结构体大小（字节）
    /// szFileName[128] + iOffset[4] + iSize[4] + ResID[4] = 140
    pub const FINFO_SIZE: u32 = 140;

    /// SPF 文件头大小（字节）
    pub const HEADER_SIZE: u32 = 40;

    /// 填充区域大小（字节）
    pub const PADDING_SIZE: u32 = 96;

    /// 版本号大小（字节）
    pub const VERSION_SIZE: u32 = 4;

    /// 从文件末尾到 SPF 头的偏移（版本号4字节 + SPF头40字节 + 填充96字节 = 140）
    /// 注意：实际测试发现 SPF头位置是 file_size - 140
    pub const HEADER_FROM_END: u64 = 140;

    /// 版本号从文件末尾的偏移
    pub const VERSION_FROM_END: u64 = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finfo_default() {
        let finfo = FInfo::default();
        // 通过字节复制来避免packed结构体的未对齐引用
        let offset = u32::from_le_bytes(finfo.iOffset.to_le_bytes());
        let size = u32::from_le_bytes(finfo.iSize.to_le_bytes());
        let resid = u32::from_le_bytes(finfo.ResID.to_le_bytes());
        assert_eq!(offset, 0);
        assert_eq!(size, 0);
        assert_eq!(resid, 0);
    }

    #[test]
    fn test_finfo_filename() {
        let mut finfo = FInfo::default();
        finfo.set_filename("test.txt").unwrap();
        assert_eq!(finfo.filename().unwrap(), "test.txt");
    }

    #[test]
    fn test_finfo_filename_too_long() {
        let mut finfo = FInfo::default();
        let long_name = "a".repeat(128);
        assert!(finfo.set_filename(&long_name).is_err());
    }

    #[test]
    fn test_fspfheader_default() {
        let header = FSPFHeader::default();
        let header_size = u32::from_le_bytes(header.iHeaderSize.to_le_bytes());
        let file_id = u32::from_le_bytes(header.iFileID.to_le_bytes());
        assert_eq!(header_size, 0);
        assert_eq!(file_id, 0);
    }

    #[test]
    fn test_fspfheader_file_count() {
        let mut header = FSPFHeader::default();
        let size: u32 = layout::FINFO_SIZE * 10; // 10个文件
        header.iHeaderSize = size;
        assert_eq!(header.file_count(), 10);
    }

    #[test]
    fn test_get_spf_name_by_id() {
        assert_eq!(get_spf_name_by_id(1), Some("AJJIYA.SPF"));
        assert_eq!(get_spf_name_by_id(5), Some("MAKO1298.SPF"));
        assert_eq!(get_spf_name_by_id(99), None);
    }

    #[test]
    fn test_get_file_id_by_name() {
        assert_eq!(get_file_id_by_name("AJJIYA.SPF"), Some(1));
        assert_eq!(get_file_id_by_name("ajjiya.spf"), Some(1)); // 不区分大小写
        assert_eq!(get_file_id_by_name("UNKNOWN.SPF"), None);
    }
}
