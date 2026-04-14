//! STG file writer implementation.

use crate::stg::{StageFile, FIXED_STRING_SIZE};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct StgWriter {
    stage_file: StageFile,
    encoding: &'static encoding_rs::Encoding,
}

impl StgWriter {
    pub fn new(stage_file: StageFile, encoding: &'static encoding_rs::Encoding) -> Self {
        Self {
            stage_file,
            encoding,
        }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let file = File::create(path)
            .with_context(|| format!("Failed to create STG file: {}", path.display()))?;
        let mut writer = BufWriter::new(file);

        self.validate_count(
            self.stage_file.stage_count,
            self.stage_file.stage_list.len(),
            "StageCount",
        )?;
        self.write_i32(&mut writer, self.stage_file.stage_count, "StageCount")?;

        for stage in &self.stage_file.stage_list {
            self.validate_count(stage.group_count, stage.group_list.len(), "GroupCount")?;

            self.write_i32(&mut writer, stage.stage_id, "stage_id")?;
            self.write_i32(&mut writer, stage.sync_region_width, "sync_region_width")?;
            self.write_i32(&mut writer, stage.sync_region_height, "sync_region_height")?;
            self.write_fixed_string(&mut writer, &stage.stage_name, "stage_name")?;
            self.write_fixed_string(&mut writer, &stage.palette_file, "palette_file")?;
            self.write_i32(
                &mut writer,
                stage.unknown_stage_value,
                "unknown_stage_value",
            )?;
            self.write_i32(&mut writer, stage.group_count, "group_count")?;

            for group in &stage.group_list {
                self.validate_count(group.map_count, group.map_list.len(), "MapCount")?;

                self.write_i32(&mut writer, group.group_id, "group_id")?;
                self.write_i32(&mut writer, group.map_link, "map_link")?;
                self.write_i32(&mut writer, group.bg_id, "bg_id")?;
                self.write_i32(&mut writer, group.group_type, "group_type")?;
                self.write_fixed_string(&mut writer, &group.group_name, "group_name")?;
                self.write_fixed_string(&mut writer, &group.bg_file, "bg_file")?;
                self.write_fixed_string(&mut writer, &group.bgm_file, "bgm_file")?;
                self.write_i32(&mut writer, group.sound_effect_type, "sound_effect_type")?;
                self.write_i32(&mut writer, group.mini_map_icon_id, "mini_map_icon_id")?;
                self.write_i32(&mut writer, group.mini_map_res_id, "mini_map_res_id")?;
                self.write_i32(&mut writer, group.gravity, "gravity")?;
                self.write_i32(&mut writer, group.max_drop_speed, "max_drop_speed")?;
                self.write_i32(&mut writer, group.velocity_x, "velocity_x")?;
                self.write_i32(&mut writer, group.jump_speed, "jump_speed")?;
                self.write_i32(&mut writer, group.up_down_velocity, "up_down_velocity")?;
                self.write_i32(&mut writer, group.hanging_velocity, "hanging_velocity")?;
                self.write_i32(&mut writer, group.unknown_group_flag, "unknown_group_flag")?;
                self.write_i32(&mut writer, group.map_count, "map_count")?;

                for map in &group.map_list {
                    self.write_i32(&mut writer, map.bg_index, "bg_index")?;
                    self.write_fixed_string(&mut writer, &map.map_name, "map_name")?;
                    self.write_fixed_string(&mut writer, &map.form_file, "form_file")?;
                    self.write_fixed_string(&mut writer, &map.attribute_file, "attribute_file")?;
                    self.write_fixed_string(&mut writer, &map.mini_map_file, "mini_map_file")?;
                }
            }
        }

        writer
            .flush()
            .with_context(|| format!("Failed to flush STG file: {}", path.display()))?;

        Ok(())
    }

    fn validate_count(&self, stored: i32, actual: usize, field: &str) -> Result<()> {
        let actual = i32::try_from(actual)
            .map_err(|_| anyhow::anyhow!("{} actual length is out of i32 range", field))?;
        if stored != actual {
            bail!(
                "{} mismatch: field value {}, actual {}",
                field,
                stored,
                actual
            );
        }
        Ok(())
    }

    fn write_i32<W, T>(&self, writer: &mut W, value: T, field: &str) -> Result<()>
    where
        W: Write,
        T: TryInto<i32> + Copy,
    {
        let value = value
            .try_into()
            .map_err(|_| anyhow::anyhow!("{} value is out of i32 range", field))?;
        writer.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_fixed_string<W: Write>(&self, writer: &mut W, value: &str, field: &str) -> Result<()> {
        let (encoded, _, had_errors) = self.encoding.encode(value);
        if had_errors {
            bail!("{} cannot be encoded with {}", field, self.encoding.name());
        }

        let encoded = encoded.as_ref();
        if encoded.len() >= FIXED_STRING_SIZE {
            bail!(
                "{} is too long after {} encoding: {} bytes (max {})",
                field,
                self.encoding.name(),
                encoded.len(),
                FIXED_STRING_SIZE - 1
            );
        }

        let mut bytes = [0u8; FIXED_STRING_SIZE];
        bytes[..encoded.len()].copy_from_slice(encoded);
        writer.write_all(&bytes)?;
        Ok(())
    }
}
