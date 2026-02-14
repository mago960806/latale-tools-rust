use crate::spf::{FInfo, ResId, SpfHeader, SPF_VERSION, DESC_SIZE};
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// SPF 文件写入器
pub struct SpfWriter {
    file_id: u8,
    desc: [u8; 32],
    /// 文件数据，使用 BTreeMap 保证按文件名排序
    files: BTreeMap<String, Vec<u8>>,
}

impl SpfWriter {
    /// 创建新的 SPF 写入器
    pub fn new(file_id: u8) -> Self {
        Self {
            file_id,
            desc: [0u8; DESC_SIZE],
            files: BTreeMap::new(),
        }
    }

    /// 设置描述信息
    pub fn set_desc(&mut self, desc: &str) {
        let bytes = desc.as_bytes();
        let len = bytes.len().min(DESC_SIZE - 1);
        self.desc[..len].copy_from_slice(&bytes[..len]);
    }

    /// 添加文件
    pub fn add_file(&mut self, name: String, data: Vec<u8>) {
        self.files.insert(name, data);
    }

    /// 从目录扫描并添加所有文件
    /// prefix 是 SPF 内部路径前缀（如 "CHAR/HOSHIM"）
    pub fn add_from_dir(&mut self, data_dir: &Path, prefix: &str) -> Result<()> {
        use walkdir::WalkDir;

        let prefix_path = data_dir.join(prefix);
        if !prefix_path.exists() {
            bail!("Directory not found: {}", prefix_path.display());
        }

        for entry in WalkDir::new(&prefix_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let full_path = entry.path();
            let relative = full_path.strip_prefix(data_dir)
                .context("Failed to strip prefix")?;

            let name = relative.to_string_lossy().replace('\\', "/");
            let data = std::fs::read(full_path)
                .with_context(|| format!("Failed to read: {}", full_path.display()))?;

            self.add_file(name, data);
        }

        Ok(())
    }

    /// 写入 SPF 文件
    pub fn write(&self, output_path: &Path) -> Result<()> {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create: {}", output_path.display()))?;
        let mut writer = BufWriter::new(file);

        let finfo_size = std::mem::size_of::<FInfo>();
        let file_count = self.files.len();
        let header_size = (file_count * finfo_size) as i32;

        // 1. 写入文件数据区，同时计算偏移量
        let mut finfos: Vec<FInfo> = Vec::with_capacity(file_count);
        let mut current_offset: i32 = 0;

        for (name, data) in &self.files {
            // 构建文件名数组
            let mut file_name = [0u8; 128];
            let name_bytes = name.as_bytes();
            let len = name_bytes.len().min(127);
            file_name[..len].copy_from_slice(&name_bytes[..len]);

            // 构建 FINFO
            let finfo = FInfo {
                file_name,
                offset: current_offset,
                size: data.len() as i32,
                res_id: ResId::new(self.file_id, finfos.len() as u32),
            };
            finfos.push(finfo);

            // 写入数据
            writer.write_all(data)
                .context("Failed to write file data")?;
            current_offset += data.len() as i32;
        }

        // 2. 写入 FINFO 索引表
        for finfo in &finfos {
            let bytes: &[u8] = bytemuck::bytes_of(finfo);
            writer.write_all(bytes)
                .context("Failed to write FINFO")?;
        }

        // 3. 写入 SPF 头
        let header = SpfHeader {
            header_size,
            file_id: self.file_id as i32,
            desc: self.desc,
        };
        let header_bytes: &[u8] = bytemuck::bytes_of(&header);
        writer.write_all(header_bytes)
            .context("Failed to write SPF header")?;

        // 4. 写入版本号
        let version_bytes: &[u8] = bytemuck::bytes_of(&SPF_VERSION);
        writer.write_all(version_bytes)
            .context("Failed to write version")?;

        writer.flush().context("Failed to flush")?;

        Ok(())
    }
}
