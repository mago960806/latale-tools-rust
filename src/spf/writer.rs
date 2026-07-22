use crate::common::encoding_from_name;
use crate::spf::{
    crypto, FInfo, ResId, SpfHeader, SpfVersion, DESC_SIZE, FINFO_SIZE, SPF_ENCRYPTED_FLAG,
};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// SPF 文件写入器
pub struct SpfWriter {
    file_id: u8,
    version: SpfVersion,
    encrypted: bool,
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
            encrypted: false,
            desc: [0u8; DESC_SIZE],
            encoding: encoding_from_name(encoding),
            files: Vec::new(),
        }
    }

    /// 设置是否写出新版 ChaCha20 加密 SPF；默认不加密。
    pub fn set_encrypted(&mut self, encrypted: bool) {
        self.encrypted = encrypted;
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
                let relative = path
                    .strip_prefix(data_dir)
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
        if self.version < 0 {
            bail!("SPF version must be non-negative: {}", self.version);
        }

        let file = File::create(output_path)
            .with_context(|| format!("Failed to create: {}", output_path.display()))?;
        let mut writer = BufWriter::new(file);

        let finfo_size = std::mem::size_of::<FInfo>();
        let file_count = self.files.len();
        let header_size = (file_count * finfo_size) as i32;
        let raw_version = self.version as u32
            | if self.encrypted {
                SPF_ENCRYPTED_FLAG
            } else {
                0
            };

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
            if self.encrypted {
                let mut encrypted_data = data.clone();
                crypto::crypt_resource(&mut encrypted_data, finfo.offset, finfo.size, raw_version);
                writer
                    .write_all(&encrypted_data)
                    .context("Failed to write encrypted file data")?;
            } else {
                writer
                    .write_all(data)
                    .context("Failed to write file data")?;
            }
            current_offset += data.len() as i32;

            // 调用回调
            if let Some(cb) = callback {
                cb(i + 1, file_count, name);
            }
        }

        // 2. 写入 FINFO 索引表
        for finfo in &finfos {
            if self.encrypted {
                let mut bytes = [0u8; FINFO_SIZE];
                bytes.copy_from_slice(bytemuck::bytes_of(finfo));
                crypto::crypt_finfo(&mut bytes);
                writer
                    .write_all(&bytes)
                    .context("Failed to write encrypted FINFO")?;
            } else {
                writer
                    .write_all(bytemuck::bytes_of(finfo))
                    .context("Failed to write FINFO")?;
            }
        }

        // 3. 写入 SPF 头
        let header = SpfHeader {
            header_size,
            file_id: self.file_id as i32,
            desc: self.desc,
        };
        let header_bytes: &[u8] = bytemuck::bytes_of(&header);
        writer
            .write_all(header_bytes)
            .context("Failed to write header")?;

        // 4. 写入版本号
        writer
            .write_all(&raw_version.to_le_bytes())
            .context("Failed to write version")?;

        writer.flush().context("Failed to flush")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spf::SpfReader;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reader_writer_roundtrip_supports_plain_and_encrypted_spf() -> Result<()> {
        for encrypted in [false, true] {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after UNIX epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "latale-spf-roundtrip-{}-{unique}.SPF",
                if encrypted { "encrypted" } else { "plain" }
            ));

            let mut writer = SpfWriter::new(3, 2_026_071_602, "GBK");
            writer.set_encrypted(encrypted);
            writer.add_file("DATA/LDT/FIRST.LDT".to_owned(), b"first resource".to_vec());
            writer.add_file("DATA/LDT/SECOND.LDT".to_owned(), (0..=255).collect());
            writer.write(&path, None)?;

            let reader = SpfReader::open(&path)?;
            assert_eq!(reader.version(), 2_026_071_602);
            assert_eq!(reader.is_encrypted(), encrypted);
            assert!(reader.verify()?.is_empty());

            let finfos = reader.file_infos();
            assert_eq!(finfos.len(), 2);
            assert_eq!(reader.get_file_data(&finfos[0]).as_ref(), b"first resource");
            assert_eq!(
                reader.get_file_data(&finfos[1]).as_ref(),
                &(0..=255).collect::<Vec<_>>()
            );

            std::fs::remove_file(path)?;
        }

        Ok(())
    }
}
