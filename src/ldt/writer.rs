//! LDT file writer implementation
//!
//! This module provides functionality to write LDT database files.

use crate::ldt::{FieldDef, FieldValue, FieldType, Row, MAX_FIELDS, FIELD_NAME_SIZE};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// LDT file writer
pub struct LdtWriter {
    db_id: i32,
    field_defs: Vec<FieldDef>,
    rows: Vec<Row>,
}

impl LdtWriter {
    /// Create a new LDT writer with the given database ID
    pub fn new(db_id: i32) -> Self {
        Self {
            db_id,
            field_defs: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Add a field definition
    pub fn add_field(&mut self, name: &str, field_type: FieldType) -> &mut Self {
        if self.field_defs.len() < MAX_FIELDS {
            self.field_defs.push(FieldDef {
                name: name.to_string(),
                field_type,
            });
        }
        self
    }

    /// Set field definitions
    pub fn set_field_defs(&mut self, field_defs: Vec<FieldDef>) -> &mut Self {
        self.field_defs = field_defs;
        self
    }

    /// Add a row
    pub fn add_row(&mut self, row: Row) -> &mut Self {
        self.rows.push(row);
        self
    }

    /// Set all rows
    pub fn set_rows(&mut self, rows: Vec<Row>) -> &mut Self {
        self.rows = rows;
        self
    }

    /// Get the number of fields
    pub fn field_count(&self) -> usize {
        self.field_defs.len()
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Write the LDT file to the given path
    pub fn write(&self, path: &Path) -> Result<()> {
        if self.field_defs.is_empty() {
            bail!("Cannot write LDT file with no fields");
        }
        if self.field_defs.len() > MAX_FIELDS {
            bail!("Too many fields: {} (max: {})", self.field_defs.len(), MAX_FIELDS);
        }

        let file = File::create(path)
            .with_context(|| format!("Failed to create LDT file: {}", path.display()))?;
        let mut writer = BufWriter::new(file);

        // Write header
        self.write_header(&mut writer)?;

        // Write rows
        for row in &self.rows {
            self.write_row(&mut writer, row)?;
        }

        // Write footer: "END" + 61 spaces (64 bytes total)
        writer.write_all(b"END")?;
        writer.write_all(&[0x20u8; 61])?;

        writer.flush()
            .with_context(|| format!("Failed to flush LDT file: {}", path.display()))?;

        Ok(())
    }

    /// Write the LDT header
    fn write_header<W: Write>(&self, writer: &mut W) -> Result<()> {
        let num_fields = self.field_defs.len() as i32;
        let num_rows = self.rows.len() as i32;

        // Write db_id (4 bytes)
        writer.write_all(&self.db_id.to_le_bytes())?;

        // Write num_fields (4 bytes)
        writer.write_all(&num_fields.to_le_bytes())?;

        // Write num_rows (4 bytes)
        writer.write_all(&num_rows.to_le_bytes())?;

        // Write field names (128 * 64 bytes)
        // Format: string + null(0x00) + space padding(0x20) to 64 bytes
        // Empty fields: all zeros (0x00)
        for i in 0..MAX_FIELDS {
            if i < self.field_defs.len() {
                let mut name_bytes = [0x20u8; FIELD_NAME_SIZE]; // Fill with spaces
                let name = self.field_defs[i].name.as_bytes();
                let copy_len = name.len().min(FIELD_NAME_SIZE - 1);
                name_bytes[..copy_len].copy_from_slice(&name[..copy_len]);
                name_bytes[copy_len] = 0x00; // Null terminator
                writer.write_all(&name_bytes)?;
            } else {
                // Empty field: all zeros
                writer.write_all(&[0u8; FIELD_NAME_SIZE])?;
            }
        }

        // Write field types (128 * 4 bytes)
        for i in 0..MAX_FIELDS {
            let type_value = if i < self.field_defs.len() {
                self.field_defs[i].field_type as i32
            } else {
                FieldType::NA as i32
            };
            writer.write_all(&type_value.to_le_bytes())?;
        }

        Ok(())
    }

    /// Write a single row
    fn write_row<W: Write>(&self, writer: &mut W, row: &Row) -> Result<()> {
        // Write primary key (4 bytes)
        let pk = row.primary_key as i32;
        writer.write_all(&pk.to_le_bytes())?;

        // Write field values
        for (i, value) in row.values.iter().enumerate() {
            let expected_type = if i < self.field_defs.len() {
                self.field_defs[i].field_type
            } else {
                FieldType::NA
            };
            self.write_field_value(writer, value, expected_type)?;
        }

        Ok(())
    }

    /// Write a single field value
    fn write_field_value<W: Write>(&self, writer: &mut W, value: &FieldValue, _expected_type: FieldType) -> Result<()> {
        match value {
            FieldValue::NA => {
                // No bytes written for NA type
            }

            FieldValue::Num(n) => {
                writer.write_all(&n.to_le_bytes())?;
            }

            FieldValue::Per(f) => {
                writer.write_all(&f.to_le_bytes())?;
            }

            FieldValue::TF(b) => {
                let v: i32 = if *b { 1 } else { 0 };
                writer.write_all(&v.to_le_bytes())?;
            }

            FieldValue::Num64(n) => {
                writer.write_all(&n.to_le_bytes())?;
            }

            FieldValue::String(s) => {
                // Encode as GBK (LaTale uses GBK encoding for strings)
                let (bytes, _, _) = encoding_rs::GBK.encode(&s);
                let bytes = bytes.as_ref();
                // Write length + content (no terminator)
                writer.write_all(&(bytes.len() as u16).to_le_bytes())?;
                writer.write_all(bytes)?;
            }

            FieldValue::Alias(s) => {
                // Encode as GBK
                let (bytes, _, _) = encoding_rs::GBK.encode(&s);
                let bytes = bytes.as_ref();
                // Write length + content (no terminator)
                writer.write_all(&(bytes.len() as u16).to_le_bytes())?;
                writer.write_all(bytes)?;
            }

            FieldValue::FID(spf_id, row_id) => {
                let s = format!("{},{}", spf_id, row_id);
                let (bytes, _, _) = encoding_rs::GBK.encode(&s);
                let bytes = bytes.as_ref();
                // Write length + content (no terminator)
                writer.write_all(&(bytes.len() as u16).to_le_bytes())?;
                writer.write_all(bytes)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_basic() {
        let mut writer = LdtWriter::new(1);
        writer.add_field("ID", FieldType::Num64);
        writer.add_field("Name", FieldType::String);
        writer.add_field("Price", FieldType::Num);
        writer.add_field("Rate", FieldType::Per);
        writer.add_field("Enabled", FieldType::TF);

        writer.add_row(Row {
            primary_key: 1,
            values: vec![
                FieldValue::Num64(1),
                FieldValue::String("Test Item".to_string()),
                FieldValue::Num(100),
                FieldValue::Per(1.5),
                FieldValue::TF(true),
            ],
        });

        assert_eq!(writer.field_count(), 5);
        assert_eq!(writer.row_count(), 1);
    }
}
