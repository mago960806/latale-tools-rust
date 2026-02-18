//! LDT file reader implementation
//!
//! This module provides zero-copy reading of LDT database files using memory mapping.

use crate::ldt::{FieldDef, FieldType, FieldValue, LdtHeader, Row, HEADER_SIZE, MAX_FIELDS};
use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// LDT file reader with memory-mapped access
pub struct LdtReader {
    mmap: Mmap,
    header: LdtHeader,
    encoding: &'static encoding_rs::Encoding,
}

impl LdtReader {
    /// Open an LDT file and map it to memory
    pub fn open(path: &Path, encoding: &'static encoding_rs::Encoding) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open LDT file: {}", path.display()))?;

        // SAFETY: File mapping is safe, we only read
        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap LDT file: {}", path.display()))?;

        let len = mmap.len();
        if len < HEADER_SIZE {
            bail!(
                "LDT file too small: {} bytes (minimum: {})",
                len,
                HEADER_SIZE
            );
        }

        // Read header from the beginning
        let header: LdtHeader = bytemuck::pod_read_unaligned(&mmap[0..HEADER_SIZE]);

        // Validate header
        if header.num_fields < 0 || header.num_fields > MAX_FIELDS as i32 {
            bail!("Invalid field count: {}", header.num_fields);
        }
        if header.num_rows < 0 {
            bail!("Invalid row count: {}", header.num_rows);
        }

        Ok(Self { mmap, header, encoding })
    }

    /// Get the encoding used by this reader
    pub fn encoding(&self) -> &'static encoding_rs::Encoding {
        self.encoding
    }

    /// Get the LDT header
    pub fn header(&self) -> &LdtHeader {
        &self.header
    }

    /// Get database ID
    pub fn db_id(&self) -> i32 {
        self.header.db_id
    }

    /// Get number of fields
    pub fn field_count(&self) -> i32 {
        self.header.num_fields
    }

    /// Get number of rows
    pub fn row_count(&self) -> i32 {
        self.header.num_rows
    }

    /// Get field definitions
    pub fn field_defs(&self) -> Vec<FieldDef> {
        self.header.field_defs()
    }

    /// Read all rows from the LDT file
    pub fn read_rows(&self) -> Result<Vec<Row>> {
        let num_fields = self.header.num_fields as usize;
        let num_rows = self.header.num_rows as usize;

        if num_fields == 0 || num_rows == 0 {
            return Ok(Vec::new());
        }

        let mut rows = Vec::with_capacity(num_rows);
        let mut offset = HEADER_SIZE;

        for _ in 0..num_rows {
            let row = self.read_row(offset, num_fields)?;
            offset = row.1;
            rows.push(row.0);
        }

        Ok(rows)
    }

    /// Read a single row starting at the given offset
    /// Returns (Row, next_offset)
    fn read_row(&self, offset: usize, num_fields: usize) -> Result<(Row, usize)> {
        let mut pos = offset;

        // Read primary key (4 bytes, stored as i64 in Row struct)
        if pos + 4 > self.mmap.len() {
            bail!(
                "Unexpected end of file while reading primary key at offset {}",
                pos
            );
        }
        let primary_key: i32 = bytemuck::pod_read_unaligned(&self.mmap[pos..pos + 4]);
        pos += 4;

        // Read field values
        let mut values = Vec::with_capacity(num_fields);
        for i in 0..num_fields {
            let field_type = self.header.field_type(i);
            let (value, new_pos) = self.read_field_value(pos, field_type)?;
            values.push(value);
            pos = new_pos;
        }

        Ok((
            Row {
                primary_key,
                values,
            },
            pos,
        ))
    }

    /// Read a single field value starting at the given offset
    /// Returns (FieldValue, next_offset)
    fn read_field_value(
        &self,
        offset: usize,
        field_type: FieldType,
    ) -> Result<(FieldValue, usize)> {
        match field_type {
            FieldType::NA => Ok((FieldValue::NA, offset)),

            FieldType::Num => {
                if offset + 4 > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading Num at offset {}",
                        offset
                    );
                }
                let value: i32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::Num(value), offset + 4))
            }

            FieldType::Per => {
                if offset + 4 > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading Per at offset {}",
                        offset
                    );
                }
                let value: f32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::Per(value), offset + 4))
            }

            FieldType::TF => {
                if offset + 4 > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading TF at offset {}",
                        offset
                    );
                }
                let value: i32 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 4]);
                Ok((FieldValue::TF(value != 0), offset + 4))
            }

            FieldType::Num64 => {
                if offset + 8 > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading Num64 at offset {}",
                        offset
                    );
                }
                let value: i64 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 8]);
                Ok((FieldValue::Num64(value), offset + 8))
            }

            FieldType::String | FieldType::Alias | FieldType::FID => {
                // Variable-length fields: 2-byte length prefix + data (no terminator)
                if offset + 2 > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading string length at offset {}",
                        offset
                    );
                }
                let len: u16 = bytemuck::pod_read_unaligned(&self.mmap[offset..offset + 2]);
                let len = len as usize;
                let data_start = offset + 2;

                if data_start + len > self.mmap.len() {
                    bail!(
                        "Unexpected end of file while reading string data at offset {}",
                        data_start
                    );
                }

                let data = &self.mmap[data_start..data_start + len];
                // Decode using the configured encoding
                let (s, _, _) = self.encoding.decode(data);
                let s = s.into_owned();
                let new_offset = data_start + len;

                match field_type {
                    FieldType::String => Ok((FieldValue::String(s), new_offset)),
                    FieldType::Alias => Ok((FieldValue::Alias(s), new_offset)),
                    FieldType::FID => {
                        // FID format: "{spf_id},{row_id}"
                        let parts: Vec<&str> = s.split(',').collect();
                        if parts.len() == 2 {
                            let spf_id: i32 = parts[0].parse().unwrap_or_else(|_| {
                                eprintln!("Warning: Failed to parse FID spf_id: {}", parts[0]);
                                0
                            });
                            let row_id: i64 = parts[1].parse().unwrap_or_else(|_| {
                                eprintln!("Warning: Failed to parse FID row_id: {}", parts[1]);
                                0
                            });
                            Ok((FieldValue::FID(spf_id, row_id), new_offset))
                        } else {
                            Ok((FieldValue::FID(0, 0), new_offset))
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Get total file size
    pub fn total_size(&self) -> usize {
        self.mmap.len()
    }
}
