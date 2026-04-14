//! STG type definitions.

use serde::{Deserialize, Serialize};

pub const STG_EXTENSION: &str = "stg";
pub const JSON_EXTENSION: &str = "json";
pub const STG_OUTPUT_EXT: &str = ".STG";
pub const JSON_OUTPUT_EXT: &str = ".JSON";
pub const DEFAULT_STG_INPUT: &str = "cn/STAGENEW.STG";
pub const FIXED_STRING_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLink {
    Null,
    Horizontal,
    Vertical,
    Unknown(i32),
}

impl MapLink {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Null,
            1 => Self::Horizontal,
            2 => Self::Vertical,
            value => Self::Unknown(value),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Null => "MAP_LINK_NULL",
            Self::Horizontal => "MAP_LINK_HORIZONTAL",
            Self::Vertical => "MAP_LINK_VERTICAL",
            Self::Unknown(_) => "UNKNOWN",
        }
    }
}

pub const GROUP_TYPE_FIELD: i32 = 1 << 0;
pub const GROUP_TYPE_MINIMAP: i32 = 1 << 1;
pub const GROUP_TYPE_EVENT: i32 = 1 << 2;
pub const GROUP_TYPE_MARKET: i32 = 1 << 3;
pub const GROUP_TYPE_EXP: i32 = 1 << 4;
pub const GROUP_TYPE_PVP: i32 = 1 << 5;
pub const GROUP_TYPE_CASH: i32 = 1 << 7;
pub const GROUP_TYPE_INDUN: i32 = 1 << 8;
pub const GROUP_TYPE_REVIVE: i32 = 1 << 9;
pub const GROUP_TYPE_SUMMON: i32 = 1 << 10;

pub const GROUP_TYPE_FLAGS: &[(i32, &str)] = &[
    (GROUP_TYPE_FIELD, "GROUP_TYPE_FIELD"),
    (GROUP_TYPE_MINIMAP, "GROUP_TYPE_MINIMAP"),
    (GROUP_TYPE_EVENT, "GROUP_TYPE_EVENT"),
    (GROUP_TYPE_MARKET, "GROUP_TYPE_MARKET"),
    (GROUP_TYPE_EXP, "GROUP_TYPE_EXP"),
    (GROUP_TYPE_PVP, "GROUP_TYPE_PVP"),
    (GROUP_TYPE_CASH, "GROUP_TYPE_CASH"),
    (GROUP_TYPE_INDUN, "GROUP_TYPE_INDUN"),
    (GROUP_TYPE_REVIVE, "GROUP_TYPE_REVIVE"),
    (GROUP_TYPE_SUMMON, "GROUP_TYPE_SUMMON"),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageFile {
    #[serde(rename = "StageCount")]
    pub stage_count: i32,
    #[serde(rename = "StageList")]
    pub stage_list: Vec<Stage>,
}

impl StageFile {
    pub fn stage_count(&self) -> usize {
        self.stage_list.len()
    }

    pub fn group_count(&self) -> usize {
        self.stage_list
            .iter()
            .map(|stage| stage.group_list.len())
            .sum()
    }

    pub fn map_count(&self) -> usize {
        self.stage_list
            .iter()
            .flat_map(|stage| &stage.group_list)
            .map(|group| group.map_list.len())
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stage {
    #[serde(rename = "StageID")]
    pub stage_id: i32,
    #[serde(rename = "SyncRegionWidth")]
    pub sync_region_width: i32,
    #[serde(rename = "SyncRegionHeight")]
    pub sync_region_height: i32,
    #[serde(rename = "StageName")]
    pub stage_name: String,
    #[serde(rename = "PaletteFile")]
    pub palette_file: String,
    #[serde(rename = "UnknownStageValue")]
    pub unknown_stage_value: i32,
    #[serde(rename = "GroupCount")]
    pub group_count: i32,
    #[serde(rename = "GroupList")]
    pub group_list: Vec<MapGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapGroup {
    #[serde(rename = "GroupID")]
    pub group_id: i32,
    #[serde(rename = "MapLink")]
    pub map_link: i32,
    #[serde(rename = "BGID")]
    pub bg_id: i32,
    #[serde(rename = "Type")]
    pub group_type: i32,
    #[serde(rename = "GroupName")]
    pub group_name: String,
    #[serde(rename = "BGFile")]
    pub bg_file: String,
    #[serde(rename = "BGMFile")]
    pub bgm_file: String,
    #[serde(rename = "SoundEffectType")]
    pub sound_effect_type: i32,
    #[serde(rename = "MiniMapIconID")]
    pub mini_map_icon_id: i32,
    #[serde(rename = "MiniMapResID")]
    pub mini_map_res_id: i32,
    #[serde(rename = "Gravity")]
    pub gravity: i32,
    #[serde(rename = "MaxDropSpeed")]
    pub max_drop_speed: i32,
    #[serde(rename = "VelocityX")]
    pub velocity_x: i32,
    #[serde(rename = "JumpSpeed")]
    pub jump_speed: i32,
    #[serde(rename = "UpDownVelocity")]
    pub up_down_velocity: i32,
    #[serde(rename = "HangingVelocity")]
    pub hanging_velocity: i32,
    #[serde(rename = "UnknownGroupFlag")]
    pub unknown_group_flag: i32,
    #[serde(rename = "MapCount")]
    pub map_count: i32,
    #[serde(rename = "MapList")]
    pub map_list: Vec<MapInfo>,
}

impl MapGroup {
    pub fn map_link_kind(&self) -> MapLink {
        MapLink::from_i32(self.map_link)
    }

    pub fn group_type_names(&self) -> Vec<&'static str> {
        GROUP_TYPE_FLAGS
            .iter()
            .filter_map(|(flag, name)| {
                if self.group_type & *flag != 0 {
                    Some(*name)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn group_type_unknown_bits(&self) -> i32 {
        let known_bits = GROUP_TYPE_FLAGS
            .iter()
            .fold(0, |bits, (flag, _)| bits | *flag);
        self.group_type & !known_bits
    }

    pub fn gravity_value(&self) -> f32 {
        fixed_point_to_float(self.gravity)
    }

    pub fn max_drop_speed_value(&self) -> f32 {
        fixed_point_to_float(self.max_drop_speed)
    }

    pub fn velocity_x_value(&self) -> f32 {
        fixed_point_to_float(self.velocity_x)
    }

    pub fn jump_speed_value(&self) -> f32 {
        fixed_point_to_float(self.jump_speed)
    }

    pub fn up_down_velocity_value(&self) -> f32 {
        fixed_point_to_float(self.up_down_velocity)
    }

    pub fn hanging_velocity_value(&self) -> f32 {
        fixed_point_to_float(self.hanging_velocity)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapInfo {
    #[serde(rename = "BGIndex")]
    pub bg_index: i32,
    #[serde(rename = "MapName")]
    pub map_name: String,
    #[serde(rename = "FormFile")]
    pub form_file: String,
    #[serde(rename = "AttributeFile")]
    pub attribute_file: String,
    #[serde(rename = "MiniMapFile")]
    pub mini_map_file: String,
}

fn fixed_point_to_float(value: i32) -> f32 {
    value as f32 * 0.001
}
