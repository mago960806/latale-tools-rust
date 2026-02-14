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
    /// 获取文件名原始字节（去除尾部的 null 字符）
    pub fn file_name_bytes(&self) -> &[u8] {
        let end = self.file_name.iter().position(|&b| b == 0).unwrap_or(128);
        &self.file_name[..end]
    }

    /// 获取文件名字符串，默认使用 GBK 解码
    pub fn file_name_str(&self) -> String {
        self.file_name_str_with_encoding(None)
    }

    /// 获取文件名字符串，使用指定编码或默认 GBK
    pub fn file_name_str_with_encoding(&self, encoding: Option<&'static encoding_rs::Encoding>) -> String {
        let bytes = self.file_name_bytes();
        let enc = encoding.unwrap_or(encoding_rs::GBK);
        let (s, _, _) = enc.decode(bytes);
        s.into_owned()
    }
}

/// 根据编码名称获取编码
pub fn encoding_from_name(name: &str) -> &'static encoding_rs::Encoding {
    match name.to_uppercase().as_str() {
        "UTF-8" | "UTF8" => encoding_rs::UTF_8,
        "BIG5" | "BIG-5" => encoding_rs::BIG5,
        "EUC-KR" | "EUCKR" | "KOREAN" => encoding_rs::EUC_KR,
        "GBK" | "GB2312" | "GB18030" => encoding_rs::GBK,
        "SHIFT_JIS" | "SHIFTJIS" | "SJIS" | "CP932" | "JAPANESE" => encoding_rs::SHIFT_JIS,
        _ => encoding_rs::UTF_8,
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
    pub desc: [u8; 128],
}

const _: () = assert!(std::mem::size_of::<SpfHeader>() == 136);

impl SpfHeader {
    /// 获取描述字符串
    pub fn desc_str(&self) -> &str {
        let end = self.desc.iter().position(|&b| b == 0).unwrap_or(128);
        std::str::from_utf8(&self.desc[..end]).unwrap_or("")
    }
}

/// 描述信息长度
pub const DESC_SIZE: usize = 128;
