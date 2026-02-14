use crate::spf::{FInfo, SpfHeader, SpfVersion, SpfRegistry};
use crate::spf::types::encoding_from_name;
use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// SPF 文件读取器
pub struct SpfReader {
    mmap: Mmap,
    header: SpfHeader,
    version: SpfVersion,
    /// 文件名编码
    encoding: Option<&'static encoding_rs::Encoding>,
}

impl SpfReader {
    /// 打开 SPF 文件并映射到内存
    /// 编码自动从 SpfRegistry 获取（根据 file_id）
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
        let version: SpfVersion = bytemuck::pod_read_unaligned(&mmap[version_offset..len]);

        // 读取 SPF 头（版本号前 136 字节）
        let header_offset = version_offset - std::mem::size_of::<SpfHeader>();
        let header_end = header_offset + std::mem::size_of::<SpfHeader>();
        let header: SpfHeader = bytemuck::pod_read_unaligned(&mmap[header_offset..header_end]);

        // 从 registry 获取编码
        let encoding = SpfRegistry::find_by_file_id(header.file_id as u8)
            .map(|r| encoding_from_name(r.encoding));

        Ok(Self { mmap, header, version, encoding })
    }

    /// 获取 SPF 版本号
    pub fn version(&self) -> SpfVersion {
        self.version
    }

    /// 获取 SPF 文件头
    pub fn header(&self) -> &SpfHeader {
        &self.header
    }

    /// 获取文件名编码
    pub fn encoding(&self) -> Option<&'static encoding_rs::Encoding> {
        self.encoding
    }

    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.header.header_size as usize / std::mem::size_of::<FInfo>()
    }

    /// 获取总文件大小
    pub fn total_size(&self) -> usize {
        self.mmap.len()
    }

    /// 获取所有文件信息（FINFO 数组）
    pub fn file_infos(&self) -> Vec<FInfo> {
        let len = self.mmap.len();
        let header_size = std::mem::size_of::<SpfHeader>();
        let version_size = std::mem::size_of::<SpfVersion>();
        let finfo_size = std::mem::size_of::<FInfo>();

        let index_start = len - version_size - header_size - self.header.header_size as usize;
        let count = self.file_count();

        // 逐个读取以避免对齐问题
        let mut finfos = Vec::with_capacity(count);
        for i in 0..count {
            let offset = index_start + i * finfo_size;
            let finfo: FInfo = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + finfo_size]);
            finfos.push(finfo);
        }
        finfos
    }

    /// 获取指定文件的原始数据（零拷贝）
    pub fn get_file_data(&self, finfo: &FInfo) -> &[u8] {
        let start = finfo.offset as usize;
        let end = start + finfo.size as usize;
        &self.mmap[start..end]
    }

    /// 验证 SPF 文件完整性
    /// 返回 Ok(issues) 其中 issues 是发现的问题列表（空列表表示完全有效）
    pub fn verify(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        let finfos = self.file_infos();
        let total_len = self.mmap.len();

        // 计算数据区结束位置
        let header_size = std::mem::size_of::<SpfHeader>();
        let version_size = std::mem::size_of::<SpfVersion>();
        let data_area_end = total_len - version_size - header_size - self.header.header_size as usize;

        for (i, finfo) in finfos.iter().enumerate() {
            // 检查偏移量和大小
            let start = finfo.offset as usize;
            let size = finfo.size as usize;
            let end = start + size;

            // 检查偏移量是否为负数（通过 i32 检查）
            if finfo.offset < 0 {
                issues.push(format!("File #{} '{}': negative offset {}", i, finfo.file_name_str(), finfo.offset));
                continue;
            }

            // 检查大小是否为负数
            if finfo.size < 0 {
                issues.push(format!("File #{} '{}': negative size {}", i, finfo.file_name_str(), finfo.size));
                continue;
            }

            // 检查是否超出数据区
            if end > data_area_end {
                issues.push(format!("File #{} '{}': data range {}-{} exceeds data area (0-{})",
                    i, finfo.file_name_str(), start, end, data_area_end));
            }

            // 检查 RESID 的 FILE_ID 是否与 SPF 头一致
            if finfo.res_id.file_id() as i32 != self.header.file_id {
                issues.push(format!("File #{} '{}': RESID FILE_ID {} doesn't match SPF FILE_ID {}",
                    i, finfo.file_name_str(), finfo.res_id.file_id(), self.header.file_id));
            }

            // 检查文件名是否为空
            if finfo.file_name[0] == 0 {
                issues.push(format!("File #{}: empty file name", i));
            }
        }

        Ok(issues)
    }

    /// 解包所有文件到指定目录
    pub fn unpack(&self, output_dir: &Path) -> Result<()> {
        use std::fs;
        use std::io::Write;

        let finfos = self.file_infos();

        for finfo in finfos.iter() {
            let file_name = finfo.file_name_str_with_encoding(self.encoding);
            let output_path = output_dir.join(&file_name);

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
