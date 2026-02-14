/// SPF 文件注册信息
#[derive(Debug, Clone, Copy)]
pub struct SpfRegistry {
    /// SPF 文件 ID
    pub file_id: u8,
    /// SPF 名称（不含扩展名）
    pub name: &'static str,
    /// 内部路径前缀
    pub path_prefix: &'static str,
}

impl SpfRegistry {
    /// 所有 SPF 映射表
    /// 基于 docs/08_spf_resource_system.md 中的 FILE_ID 对照表
    pub const ALL: &'static [SpfRegistry] = &[
        SpfRegistry { file_id: 0, name: "TESTPACK", path_prefix: "TEST" },
        SpfRegistry { file_id: 2, name: "HOSHIM", path_prefix: "CHAR/HOSHIM" },
        SpfRegistry { file_id: 3, name: "ROWID", path_prefix: "CHAR/ROWID" },
        SpfRegistry { file_id: 5, name: "MAKO1298", path_prefix: "CHAR/MAKO1298" },
        SpfRegistry { file_id: 6, name: "METALGENI", path_prefix: "CHAR/METALGENI" },
        SpfRegistry { file_id: 7, name: "DALBONG", path_prefix: "CHAR/DALBONG" },
        SpfRegistry { file_id: 8, name: "RYUMS", path_prefix: "CHAR/RYUMS" },
        SpfRegistry { file_id: 9, name: "BANX", path_prefix: "CHAR/BANX" },
        SpfRegistry { file_id: 10, name: "BARY", path_prefix: "CHAR/BARY" },
        SpfRegistry { file_id: 12, name: "CLAIRE", path_prefix: "CHAR/CLAIRE" },
        SpfRegistry { file_id: 13, name: "CVOICE", path_prefix: "CVOICE" },
        // 更多 SPF 可根据需要添加
    ];

    /// 根据 SPF 名称查找注册信息
    pub fn find_by_name(name: &str) -> Option<&'static SpfRegistry> {
        // 去除可能的扩展名
        let name = name.trim_end_matches(".SPF").trim_end_matches(".spf");
        Self::ALL.iter().find(|r| r.name.eq_ignore_ascii_case(name))
    }

    /// 根据 FILE_ID 查找注册信息
    pub fn find_by_file_id(file_id: u8) -> Option<&'static SpfRegistry> {
        Self::ALL.iter().find(|r| r.file_id == file_id)
    }
}
