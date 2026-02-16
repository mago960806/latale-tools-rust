use crate::spf::{FInfo, ResId, SpfHeader, SpfVersion, DESC_SIZE};
use crate::spf::types::encoding_from_name;
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// SPF 文件写入器
pub struct SpfWriter {
    file_id: u8,
    version: SpfVersion,
    desc: [u8; DESC_SIZE],
    /// 文件名编码
    encoding: &'static encoding_rs::Encoding,
    /// 文件数据，使用 Vec 保持插入顺序
    files: Vec<(String, Vec<u8>)>,
}

impl SpfWriter {
    /// 创建新的 SPF 写入器
    /// - file_id: SPF 文件 ID (0-255)
    /// - version: 版本号
    /// - encoding: 文件名编码（如 "GBK"、"EUC-KR"、"UTF-8"）
    pub fn new(file_id: u8, version: SpfVersion, encoding: &str) -> Self {
        Self {
            file_id,
            version,
            desc: [0u8; DESC_SIZE],
            encoding: encoding_from_name(encoding),
            files: Vec::new(),
        }
    }

    /// 设置描述信息
    pub fn set_desc(&mut self, desc: &str) {
        let bytes = desc.as_bytes();
        let len = bytes.len().min(DESC_SIZE - 1);
        self.desc[..len].copy_from_slice(&bytes[..len]);
    }

    /// 添加文件（文件名保持 UTF-8，按插入顺序存储）
    pub fn add_file(&mut self, name: String, data: Vec<u8>) {
        self.files.push((name, data));
    }

    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// 获取所有文件名（按插入顺序）
    pub fn file_names(&self) -> impl Iterator<Item = &String> {
        self.files.iter().map(|(name, _)| name)
    }

    /// 从目录扫描并添加所有文件（不递归子目录）
    /// prefix 是 SPF 内部路径前缀（如 "DATA/ANITABLE"）
    pub fn add_from_dir(&mut self, data_dir: &Path, prefix: &str) -> Result<()> {
        let prefix_path = data_dir.join(prefix);
        if !prefix_path.exists() {
            bail!("Directory not found: {}", prefix_path.display());
        }

        // 收集当前目录下的所有文件
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let entries = std::fs::read_dir(&prefix_path)
            .with_context(|| format!("Failed to read directory: {}", prefix_path.display()))?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let relative = path.strip_prefix(data_dir)
                    .context("Failed to strip prefix")?;

                let name = relative.to_string_lossy().replace('\\', "/");
                let data = std::fs::read(&path)
                    .with_context(|| format!("Failed to read: {}", path.display()))?;

                files.push((name, data));
            }
        }

        // 按文件名排序
        files.sort_by(|a, b| a.0.cmp(&b.0));

        // 添加到 writer
        for (name, data) in files {
            self.add_file(name, data);
        }

        Ok(())
    }

    /// 写入 SPF 文件
    /// callback: 可选回调函数 (current, total, filename)，用于显示进度
    pub fn write(&self, output_path: &Path, callback: super::ProgressCallback) -> Result<()> {
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create: {}", output_path.display()))?;
        let mut writer = BufWriter::new(file);

        let finfo_size = std::mem::size_of::<FInfo>();
        let file_count = self.files.len();
        let header_size = (file_count * finfo_size) as i32;

        // 1. 写入文件数据区，同时计算偏移量
        let mut finfos: Vec<FInfo> = Vec::with_capacity(file_count);
        let mut current_offset: i32 = 0;

        for (i, (name, data)) in self.files.iter().enumerate() {
            // 将 UTF-8 文件名编码为目标编码
            let (name_encoded, _, _) = self.encoding.encode(name);

            // 构建文件名数组
            let mut file_name = [0u8; 128];
            let len = name_encoded.len().min(127);
            if name_encoded.len() > 127 {
                eprintln!("[警告] 文件名 '{}' 超过 127 字节，已被截断", name);
            }
            file_name[..len].copy_from_slice(&name_encoded[..len]);

            // 构建 FINFO
            // INSTANCE_ID 从 1 开始，与原始 SPF 格式一致
            let finfo = FInfo {
                file_name,
                offset: current_offset,
                size: data.len() as i32,
                res_id: ResId::new(self.file_id, (finfos.len() + 1) as u32),
            };
            finfos.push(finfo);

            // 写入数据
            writer.write_all(data)
                .context("Failed to write file data")?;
            current_offset += data.len() as i32;

            // 调用回调
            if let Some(cb) = callback {
                cb(i + 1, file_count, name);
            }
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
            .context("Failed to write header")?;

        // 4. 写入版本号
        writer.write_all(&self.version.to_le_bytes())
            .context("Failed to write version")?;

        writer.flush()
            .context("Failed to flush")?;

        Ok(())
    }
}
