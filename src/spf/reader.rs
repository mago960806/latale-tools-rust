use crate::spf::{FInfo, SpfHeader, SpfVersion, SPF_VERSION};
use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// SPF 文件读取器
pub struct SpfReader {
    mmap: Mmap,
    header: SpfHeader,
    version: SpfVersion,
}

impl SpfReader {
    /// 打开 SPF 文件并映射到内存
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open SPF file: {}", path.display()))?;

        // SAFETY: 文件映射是安全的，我们只读取不写入
        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap SPF file: {}", path.display()))?;

        let len = mmap.len();
        if len < std::mem::size_of::<SpfVersion>() + std::mem::size_of::<SpfHeader>() {
            bail!("SPF file too small: {} bytes", len);
        }

        // 从文件末尾读取版本号（最后 4 字节）
        let version_offset = len - std::mem::size_of::<SpfVersion>();
        let version: SpfVersion = bytemuck::pod_read_unaligned(&mmap[version_offset..]);

        // 读取 SPF 头（版本号前 40 字节）
        let header_offset = version_offset - std::mem::size_of::<SpfHeader>();
        let header: SpfHeader = bytemuck::pod_read_unaligned(&mmap[header_offset..]);

        // 验证版本号
        if version != SPF_VERSION {
            bail!("Unsupported SPF version: {} (expected {})", version, SPF_VERSION);
        }

        Ok(Self { mmap, header, version })
    }

    /// 获取 SPF 版本号
    pub fn version(&self) -> SpfVersion {
        self.version
    }

    /// 获取 SPF 文件头
    pub fn header(&self) -> &SpfHeader {
        &self.header
    }

    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.header.header_size as usize / std::mem::size_of::<FInfo>()
    }

    /// 获取所有文件信息（FINFO 数组）
    pub fn file_infos(&self) -> &[FInfo] {
        let len = self.mmap.len();
        let header_size = std::mem::size_of::<SpfHeader>();
        let version_size = std::mem::size_of::<SpfVersion>();
        let finfo_size = std::mem::size_of::<FInfo>();

        let index_start = len - version_size - header_size - self.header.header_size as usize;
        let count = self.file_count();

        // SAFETY: FInfo 是 Pod 类型，字节布局保证正确
        bytemuck::cast_slice(&self.mmap[index_start..index_start + count * finfo_size])
    }

    /// 获取指定文件的原始数据（零拷贝）
    pub fn get_file_data(&self, finfo: &FInfo) -> &[u8] {
        let start = finfo.offset as usize;
        let end = start + finfo.size as usize;
        &self.mmap[start..end]
    }

    /// 解包所有文件到指定目录
    pub fn unpack(&self, output_dir: &Path) -> Result<()> {
        use std::fs;
        use std::io::Write;

        let finfos = self.file_infos();

        for finfo in finfos {
            let file_name = finfo.file_name_str();
            let output_path = output_dir.join(file_name);

            // 创建父目录
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // 写入文件数据
            let data = self.get_file_data(finfo);
            let mut file = fs::File::create(&output_path)
                .with_context(|| format!("Failed to create file: {}", output_path.display()))?;
            file.write_all(data)
                .with_context(|| format!("Failed to write file: {}", output_path.display()))?;
        }

        Ok(())
    }
}
