use bytemuck::{Pod, Zeroable};

/// SPF 版本号（文件末尾 4 字节）
pub type SpfVersion = i32;

/// RESID：32 位资源 ID
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroable, Pod)]
#[repr(transparent)]
pub struct ResId(pub u32);

impl ResId {
    /// 从 FILE_ID 和 INSTANCE_ID 创建 ResId
    pub fn new(file_id: u8, instance_id: u32) -> Self {
        Self(((file_id as u32) << 24) | (instance_id & 0x00FF_FFFF))
    }

    /// 获取 FILE_ID（高 8 位）
    pub fn file_id(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// 获取 INSTANCE_ID（低 24 位）
    pub fn instance_id(self) -> u32 {
        self.0 & 0x00FF_FFFF
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
    pub file_name: [u8; 128],
    /// 文件在 SPF 中的偏移量
    pub offset: i32,
    /// 文件大小（字节）
    pub size: i32,
    /// 资源 ID
    pub res_id: ResId,
}

const _: () = assert!(std::mem::size_of::<FInfo>() == 140);

impl FInfo {
    /// 获取文件名字符串（去除尾部的 null 字符）
    pub fn file_name_str(&self) -> &str {
        let end = self.file_name.iter().position(|&b| b == 0).unwrap_or(128);
        // SAFETY: SPF 文件名应该是有效的 ASCII
        unsafe { std::str::from_utf8_unchecked(&self.file_name[..end]) }
    }
}

/// SPF 文件头（40 字节）
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpfHeader {
    /// 索引表总大小（所有 FInfo 的字节总和）
    pub header_size: i32,
    /// SPF 文件 ID
    pub file_id: i32,
    /// 描述信息
    pub desc: [u8; 32],
}

const _: () = assert!(std::mem::size_of::<SpfHeader>() == 40);

impl SpfHeader {
    /// 获取描述字符串
    pub fn desc_str(&self) -> &str {
        let end = self.desc.iter().position(|&b| b == 0).unwrap_or(32);
        unsafe { std::str::from_utf8_unchecked(&self.desc[..end]) }
    }
}

/// SPF 文件常量
pub const SPF_VERSION: SpfVersion = 0;
/// 最大文件名长度
pub const MAX_FILE_NAME: usize = 128;
/// 描述信息长度
pub const DESC_SIZE: usize = 32;
