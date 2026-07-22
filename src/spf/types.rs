use bytemuck::{Pod, Zeroable};

// ============================================================================
// Constants
// ============================================================================

/// Size of file name field in FInfo struct
pub const FILE_NAME_SIZE: usize = 128;

/// Maximum length of file name (excluding null terminator)
pub const FILE_NAME_MAX_LEN: usize = 127;

/// Total size of FInfo struct in bytes
pub const FINFO_SIZE: usize = 140;

/// Total size of SpfHeader struct in bytes
pub const SPF_HEADER_SIZE: usize = 136;

/// Bit shift for FILE_ID in ResId
pub const RESID_FILE_ID_SHIFT: u32 = 24;

/// Mask for INSTANCE_ID in ResId (low 24 bits)
pub const INSTANCE_ID_MASK: u32 = 0x00FF_FFFF;

/// Starting value for INSTANCE_ID
pub const INSTANCE_ID_START: u32 = 1;

/// SPF file extension
pub const SPF_EXTENSION: &str = ".SPF";

/// Encryption flag stored in the highest bit of the on-disk SPF version
pub const SPF_ENCRYPTED_FLAG: u32 = 0x8000_0000;

// ============================================================================
// Types
// ============================================================================

/// SPF 版本号（文件末尾 4 字节）
pub type SpfVersion = i32;

/// RESID：32 位资源 ID
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, Pod)]
#[repr(transparent)]
pub struct ResId(pub u32);

impl ResId {
    /// 从 FILE_ID 和 INSTANCE_ID 创建 ResId
    pub fn new(file_id: u8, instance_id: u32) -> Self {
        Self(((file_id as u32) << RESID_FILE_ID_SHIFT) | (instance_id & INSTANCE_ID_MASK))
    }

    /// 获取 FILE_ID（高 8 位）
    pub fn file_id(self) -> u8 {
        (self.0 >> RESID_FILE_ID_SHIFT) as u8
    }

    /// 获取 INSTANCE_ID（低 24 位）
    pub fn instance_id(self) -> u32 {
        self.0 & INSTANCE_ID_MASK
    }
}

impl std::fmt::Debug for ResId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResId")
            .field("file_id", &self.file_id())
            .field("instance_id", &self.instance_id())
            .finish()
    }
}

/// 文件信息结构（140 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FInfo {
    /// 文件名（C 字符串，128 字节）
    pub file_name: [u8; FILE_NAME_SIZE],
    /// 文件在 SPF 中的偏移量
    pub offset: i32,
    /// 文件大小（字节）
    pub size: i32,
    /// 资源 ID
    pub res_id: ResId,
}

const _: () = assert!(std::mem::size_of::<FInfo>() == FINFO_SIZE);

impl FInfo {
    /// 获取文件名原始字节（去除尾部的 null 字符）
    pub fn file_name_bytes(&self) -> &[u8] {
        let end = self
            .file_name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(FILE_NAME_SIZE);
        &self.file_name[..end]
    }

    /// 获取文件名字符串，默认使用 GBK 解码
    pub fn file_name_str(&self) -> String {
        self.file_name_str_with_encoding(None)
    }

    /// 获取文件名字符串，使用指定编码或默认 GBK
    pub fn file_name_str_with_encoding(
        &self,
        encoding: Option<&'static encoding_rs::Encoding>,
    ) -> String {
        let bytes = self.file_name_bytes();
        let enc = encoding.unwrap_or(encoding_rs::GBK);
        let (s, _, _) = enc.decode(bytes);
        s.into_owned()
    }
}

/// SPF 文件头（136 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpfHeader {
    /// 索引表总大小（所有 FInfo 的字节总和）
    pub header_size: i32,
    /// SPF 文件 ID
    pub file_id: i32,
    /// 描述信息
    pub desc: [u8; DESC_SIZE],
}

const _: () = assert!(std::mem::size_of::<SpfHeader>() == SPF_HEADER_SIZE);

impl SpfHeader {
    /// 获取描述字符串
    pub fn desc_str(&self) -> &str {
        let end = self.desc.iter().position(|&b| b == 0).unwrap_or(DESC_SIZE);
        std::str::from_utf8(&self.desc[..end]).unwrap_or("")
    }
}

/// 描述信息长度
pub const DESC_SIZE: usize = 128;
