//! STG file reader implementation.

use crate::stg::{MapGroup, MapInfo, Stage, StageFile, FIXED_STRING_SIZE};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

const MAX_STAGE_COUNT: i32 = 100_000;
const MAX_GROUP_COUNT: i32 = 100_000;
const MAX_MAP_COUNT: i32 = 100_000;

pub struct StgReader {
    data: Vec<u8>,
    encoding: &'static encoding_rs::Encoding,
}

impl StgReader {
    pub fn open(path: &Path, encoding: &'static encoding_rs::Encoding) -> Result<Self> {
        let data = fs::read(path)
            .with_context(|| format!("Failed to read STG file: {}", path.display()))?;
        Ok(Self { data, encoding })
    }

    pub fn read(&self) -> Result<StageFile> {
        let mut cursor = Cursor::new(&self.data, self.encoding);
        let stage_count = cursor.read_count("stage_count", MAX_STAGE_COUNT)?;
        let mut stages = Vec::with_capacity(stage_count);

        for stage_index in 0..stage_count {
            stages.push(cursor.read_stage(stage_index)?);
        }

        if cursor.pos != self.data.len() {
            bail!(
                "STG parse did not reach EOF: offset {} of {} bytes",
                cursor.pos,
                self.data.len()
            );
        }

        Ok(StageFile {
            stage_count: stage_count as i32,
            stage_list: stages,
        })
    }

    pub fn total_size(&self) -> usize {
        self.data.len()
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    encoding: &'static encoding_rs::Encoding,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8], encoding: &'static encoding_rs::Encoding) -> Self {
        Self {
            data,
            pos: 0,
            encoding,
        }
    }

    fn read_stage(&mut self, stage_index: usize) -> Result<Stage> {
        let stage_id = self.read_i32("stage_id")?;
        let sync_region_width = self.read_i32("sync_region_width")?;
        let sync_region_height = self.read_i32("sync_region_height")?;
        let stage_name = self.read_fixed_string("stage_name")?;
        let palette_file = self.read_fixed_string("palette_file")?;
        let unknown_stage_value = self.read_i32("unknown_stage_value")?;
        let group_count = self.read_count("group_count", MAX_GROUP_COUNT)?;

        if sync_region_width <= 0 || sync_region_height <= 0 {
            bail!(
                "Invalid sync region size at stage index {}: {}x{}",
                stage_index,
                sync_region_width,
                sync_region_height
            );
        }

        let mut groups = Vec::with_capacity(group_count);
        for group_index in 0..group_count {
            groups.push(self.read_group(stage_index, group_index)?);
        }

        Ok(Stage {
            stage_id,
            sync_region_width,
            sync_region_height,
            stage_name,
            palette_file,
            unknown_stage_value,
            group_count: group_count as i32,
            group_list: groups,
        })
    }

    fn read_group(&mut self, stage_index: usize, group_index: usize) -> Result<MapGroup> {
        let group_id = self.read_i32("group_id")?;
        let map_link = self.read_i32("map_link")?;
        let bg_id = self.read_i32("bg_id")?;
        let group_type = self.read_i32("group_type")?;
        let group_name = self.read_fixed_string("group_name")?;
        let bg_file = self.read_fixed_string("bg_file")?;
        let bgm_file = self.read_fixed_string("bgm_file")?;
        let sound_effect_type = self.read_i32("sound_effect_type")?;
        let mini_map_icon_id = self.read_i32("mini_map_icon_id")?;
        let mini_map_res_id = self.read_i32("mini_map_res_id")?;
        let gravity = self.read_i32("gravity")?;
        let max_drop_speed = self.read_i32("max_drop_speed")?;
        let velocity_x = self.read_i32("velocity_x")?;
        let jump_speed = self.read_i32("jump_speed")?;
        let up_down_velocity = self.read_i32("up_down_velocity")?;
        let hanging_velocity = self.read_i32("hanging_velocity")?;
        let unknown_group_flag = self.read_i32("unknown_group_flag")?;
        let map_count = self.read_count("map_count", MAX_MAP_COUNT)?;

        if !matches!(map_link, 0..=2) {
            bail!(
                "Invalid map_link at stage index {}, group index {}: {}",
                stage_index,
                group_index,
                map_link
            );
        }

        let mut maps = Vec::with_capacity(map_count);
        for map_index in 0..map_count {
            maps.push(self.read_map(stage_index, group_index, map_index)?);
        }

        Ok(MapGroup {
            group_id,
            map_link,
            bg_id,
            group_type,
            group_name,
            bg_file,
            bgm_file,
            sound_effect_type,
            mini_map_icon_id,
            mini_map_res_id,
            gravity,
            max_drop_speed,
            velocity_x,
            jump_speed,
            up_down_velocity,
            hanging_velocity,
            unknown_group_flag,
            map_count: map_count as i32,
            map_list: maps,
        })
    }

    fn read_map(
        &mut self,
        _stage_index: usize,
        _group_index: usize,
        _map_index: usize,
    ) -> Result<MapInfo> {
        let bg_index = self.read_i32("bg_index")?;
        let map_name = self.read_fixed_string("map_name")?;
        let form_file = self.read_fixed_string("form_file")?;
        let attribute_file = self.read_fixed_string("attribute_file")?;
        let mini_map_file = self.read_fixed_string("mini_map_file")?;

        Ok(MapInfo {
            bg_index,
            map_name,
            form_file,
            attribute_file,
            mini_map_file,
        })
    }

    fn read_count(&mut self, field: &str, max: i32) -> Result<usize> {
        let value = self.read_i32(field)?;
        if value < 0 || value > max {
            bail!("Invalid {}: {}", field, value);
        }
        Ok(value as usize)
    }

    fn read_i32(&mut self, field: &str) -> Result<i32> {
        if self.pos + 4 > self.data.len() {
            bail!(
                "Unexpected EOF while reading {} at offset {}",
                field,
                self.pos
            );
        }
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4]
            .try_into()
            .expect("slice length checked");
        self.pos += 4;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_fixed_string(&mut self, field: &str) -> Result<String> {
        if self.pos + FIXED_STRING_SIZE > self.data.len() {
            bail!(
                "Unexpected EOF while reading {} at offset {}",
                field,
                self.pos
            );
        }

        let start = self.pos;
        let bytes = &self.data[start..start + FIXED_STRING_SIZE];
        self.pos += FIXED_STRING_SIZE;

        let content_len = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(FIXED_STRING_SIZE);

        let content = &bytes[..content_len];
        let (decoded, _, had_errors) = self.encoding.decode(content);
        if had_errors {
            bail!(
                "{} at offset {} failed to decode with {}",
                field,
                start,
                self.encoding.name()
            );
        }

        Ok(decoded.into_owned())
    }
}
