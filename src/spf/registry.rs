/// SPF 文件注册信息
#[derive(Debug, Clone, Copy)]
pub struct SpfRegistry {
    /// SPF 文件 ID
    pub file_id: u8,
    /// SPF 名称（不含扩展名）
    pub name: &'static str,
    /// 版本号
    pub version: i32,
    /// 文件名编码（如 "GBK", "EUC-KR", "BIG5"）
    pub encoding: &'static str,
    /// 包含的目录列表（用于打包时匹配，保持原始顺序）
    pub include_dirs: &'static [&'static str],
}

impl SpfRegistry {
    /// 所有 SPF 映射表
    pub const ALL: &'static [SpfRegistry] = &[
        SpfRegistry { file_id: 1, name: "AJJIYA", version: 2022091501, encoding: "EUC-KR", include_dirs: &["DATA/ANITABLE"] },
        SpfRegistry { file_id: 2, name: "HOSHIM", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/FX"] },
        SpfRegistry { file_id: 3, name: "ROWID", version: 0, encoding: "GBK", include_dirs: &["DATA/LDT"] },
        SpfRegistry { file_id: 4, name: "JINSSAGA", version: 2022100601, encoding: "GBK", include_dirs: &["DATA/BGFORMAT"] },
        SpfRegistry { file_id: 5, name: "MAKO1298", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/BACKGROUND"] },
        SpfRegistry { file_id: 6, name: "METALGENI", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/SOUND"] },
        SpfRegistry { file_id: 7, name: "DALBONG", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/BGM"] },
        SpfRegistry { file_id: 8, name: "RYUMS", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/TERRAIN"] },
        SpfRegistry { file_id: 9, name: "BANX", version: 2022100601, encoding: "GBK", include_dirs: &["DATA/GLOBALRES", "DATA/GLOBALRES/CROP", "DATA/GLOBALRES/COLLECT_MOB", "DATA/GLOBALRES/WIN_HELP"] },
        SpfRegistry { file_id: 10, name: "BARY", version: 0, encoding: "GBK", include_dirs: &["DATA/CURSOR", "DATA/INTERFACE/CONCEPT", "DATA/INTERFACE/CONCEPT/THEME", "DATA/INTERFACE/GUIDE/BANNER", "DATA/INTERFACE/GUIDE/CONTENT", "DATA/INTERFACE/GUIDE/SPAC", "DATA/INTERFACE/GUIDE/WINHELP", "DATA/INTERFACE/UI", "DATA/INTERFACE/WORLDMAP", "DATA/LOADING", "DATA/LOBBY", "DATA/LOGGIN"] },
        SpfRegistry { file_id: 11, name: "ZENNE", version: 2022092701, encoding: "GBK", include_dirs: &["DATA/CHAR/CHARLAYER/40_WEAPON_001", "DATA/CHAR/CHARLAYER/41_WEAPON_015_NEW", "DATA/CHAR/CHARLAYER/20_HAIR_DEFAULT", "DATA/CHAR/CHARLAYER/21_HAIR_SPECIAL", "DATA/CHAR/MONSTER/004_PET", "DATA/CHAR/MONSTER/005_OBJ", "DATA/CHAR/CHARLAYER/50_FASHION-400", "DATA/CHAR/CHARLAYER/60_FASHION-600", "DATA/CHAR/CHARLAYER/80_FASHION-900", "DATA/CHAR/MONSTER/007_ITEM", "DATA/CHAR/MONSTER/999_PROLOGUE", "DATA/CHAR/CHARLAYER/31_BASICCLOTH_SPECIAL", "DATA/CHAR/CHARLAYER/51_FASHION-425", "DATA/CHAR/CHARLAYER/61_FASHION-625", "DATA/CHAR/CHARLAYER/99_EVENT", "DATA/CHAR/CHARLAYER/52_FASHION-450", "DATA/CHAR/CHARLAYER/62_FASHION-650", "DATA/CHAR/CHARLAYER/70_CHINA_700", "DATA/CHAR/CHARLAYER", "DATA/CHAR/CHARLAYER/00_BODY", "DATA/CHAR/CHARLAYER/53_FASHION-475", "DATA/CHAR/CHARLAYER/63_FASHION-675", "DATA/CHAR/CHARLAYER/54_FASHION-500", "DATA/CHAR/FILTERING/01_CHARACTER", "DATA/CHAR/CHARLAYER/42_WEAPON_SPECIAL", "DATA/CHAR/CHARLAYER/55_FASHION-525", "DATA/CHAR/CHARLAYER/10_FACE", "DATA/CHAR/CHARLAYER/56_FASHION-550", "DATA/CHAR/CHARLAYER/72_RENEWAL_750", "DATA/CHAR/MONSTER/006_TRAP", "DATA/CHAR/MONSTER/100_MONSTER_COLOSSEUM_HIGH_001", "DATA/CHAR/FILTERING/02_MONSTER", "DATA/CHAR/MONSTER/999_SYSTEM", "DATA/CHAR/MONSTER", "DATA/CHAR/FILTERING", "DATA/CHAR/MONSTER/001_MOB_001", "DATA/CHAR/MONSTER/001_MOB_002", "DATA/CHAR/MONSTER/001_MOB_003", "DATA/CHAR/CHARLAYER/30_BASICCLOTH", "DATA/CHAR/MONSTER/002_LAYERMOB_001", "DATA/CHAR/MONSTER/002_LAYERMOB_002", "DATA/CHAR/MONSTER/002_LAYERMOB_003", "DATA/CHAR/MONSTER/002_LAYERMOB_2016", "DATA/CHAR/MONSTER/002_LAYERMOB_2017", "DATA/CHAR/MONSTER/002_LAYERMOB_2018", "DATA/CHAR/MONSTER/002_LAYERMOB_2019", "DATA/CHAR/MONSTER/002_LAYERMOB_2020", "DATA/CHAR/MONSTER/002_LAYERMOB_2021", "DATA/CHAR/MONSTER/002_LAYERMOB_2022", "DATA/CHAR/MONSTER/003_NPC", "DATA/CHAR/MONSTER/008_EVENT"] },
        SpfRegistry { file_id: 12, name: "CLAIRE", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/PROLOGUE/UI", "DATA/LOGO", "DATA/PROLOGUE"] },
        SpfRegistry { file_id: 13, name: "CVOICE", version: 2022091501, encoding: "GBK", include_dirs: &["DATA/CP_IMAGE", "DATA/STORY/STORY1", "DATA/STORY/STORY2", "DATA/STORY/STORY3", "DATA/STORY/STORY4", "DATA/STORY/STORY5", "DATA/STORY/STORY6", "DATA/STORY/STORY7", "DATA/STORY/STORY8"] },
    ];

    /// 根据 SPF 名称查找注册信息
    pub fn find_by_name(name: &str) -> Option<&'static SpfRegistry> {
        let name = name.trim_end_matches(".SPF").trim_end_matches(".spf");
        Self::ALL.iter().find(|r| r.name.eq_ignore_ascii_case(name))
    }

    /// 根据 FILE_ID 查找注册信息
    pub fn find_by_file_id(file_id: u8) -> Option<&'static SpfRegistry> {
        Self::ALL.iter().find(|r| r.file_id == file_id)
    }
}
