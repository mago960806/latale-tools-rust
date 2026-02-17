//! LDT type definitions
//!
//! This module defines the core types for LaTale database (LDT) files.

use bytemuck::{Pod, Zeroable};

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of fields in an LDT table
pub const MAX_FIELDS: usize = 128;

/// Size of each field name in bytes
pub const FIELD_NAME_SIZE: usize = 64;

/// Total size of the LDT header in bytes
/// Layout: db_id(4) + num_fields(4) + num_rows(4) + field_names(128*64=8192) + field_types(128*4=512) = 8716
pub const HEADER_SIZE: usize = 8716;

// === File footer constants ===

/// Footer marker string ("END")
pub const FOOTER_MARKER: &[u8; 3] = b"END";

/// Footer padding size (61 bytes of spaces)
pub const FOOTER_PADDING_SIZE: usize = 61;

/// Padding byte value (space character)
pub const PADDING_BYTE: u8 = 0x20;

/// Null terminator byte
pub const NULL_TERMINATOR: u8 = 0x00;

// === CSV related constants ===

/// CSV header for the ID column with type annotation
pub const CSV_ID_COLUMN_HEADER: &str = "ID:int32";

/// Separator between field name and type in CSV headers
pub const CSV_TYPE_SEPARATOR: char = ':';

/// Default database ID value
pub const DEFAULT_DB_ID: i32 = 0;

// === File extension constants ===

/// LDT file extension (lowercase)
pub const LDT_EXTENSION: &str = "ldt";

/// CSV file extension (lowercase)
pub const CSV_EXTENSION: &str = "csv";

/// LDT output file extension (uppercase)
pub const LDT_OUTPUT_EXT: &str = ".LDT";

/// CSV output file extension
pub const CSV_OUTPUT_EXT: &str = ".csv";

// === Default path constants ===

/// Default input directory for LDT files
pub const DEFAULT_LDT_DIR: &str = "DATA/LDT";

/// Default output directory for CSV files
pub const DEFAULT_CSV_DIR: &str = "DATA/CSV";

// ============================================================================
// FieldType Enum
// ============================================================================

/// Field types supported by LDT files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FieldType {
    /// Invalid/unused field
    NA = 0,
    /// String (max 8192 bytes)
    String = 1,
    /// Boolean value
    TF = 2,
    /// 32-bit integer
    Num = 3,
    /// Float/percentage
    Per = 4,
    /// Foreign key reference
    FID = 5,
    /// Alias (max 4096 bytes)
    Alias = 6,
    /// 64-bit integer
    Num64 = 7,
}

impl FieldType {
    /// Convert from i32 to FieldType
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => FieldType::NA,
            1 => FieldType::String,
            2 => FieldType::TF,
            3 => FieldType::Num,
            4 => FieldType::Per,
            5 => FieldType::FID,
            6 => FieldType::Alias,
            7 => FieldType::Num64,
            _ => FieldType::NA,
        }
    }

    /// Get the CSV type name for this field type
    pub fn csv_type_name(&self) -> &'static str {
        match self {
            FieldType::NA => "na",
            FieldType::String => "string",
            FieldType::TF => "bool",
            FieldType::Num => "int32",
            FieldType::Per => "float32",
            FieldType::FID => "fid",
            FieldType::Alias => "alias",
            FieldType::Num64 => "int64",
        }
    }

    /// Parse FieldType from CSV type name
    pub fn from_csv_type_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "na" => Some(FieldType::NA),
            "string" => Some(FieldType::String),
            "bool" => Some(FieldType::TF),
            "int32" => Some(FieldType::Num),
            "float32" => Some(FieldType::Per),
            "fid" => Some(FieldType::FID),
            "alias" => Some(FieldType::Alias),
            "int64" => Some(FieldType::Num64),
            _ => None,
        }
    }

    /// Check if this field type uses variable-length storage
    pub fn is_variable_length(&self) -> bool {
        matches!(self, FieldType::String | FieldType::Alias | FieldType::FID)
    }
}

// ============================================================================
// FieldValue Enum
// ============================================================================

/// Value types corresponding to FieldType
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Invalid/empty value
    NA,
    /// String value
    String(String),
    /// Boolean value
    TF(bool),
    /// 32-bit integer
    Num(i32),
    /// Float value
    Per(f32),
    /// Foreign key reference (SPF ID, row ID)
    FID(i32, i64),
    /// Alias string
    Alias(String),
    /// 64-bit integer
    Num64(i64),
}

impl FieldValue {
    /// Get the field type of this value
    pub fn field_type(&self) -> FieldType {
        match self {
            FieldValue::NA => FieldType::NA,
            FieldValue::String(_) => FieldType::String,
            FieldValue::TF(_) => FieldType::TF,
            FieldValue::Num(_) => FieldType::Num,
            FieldValue::Per(_) => FieldType::Per,
            FieldValue::FID(_, _) => FieldType::FID,
            FieldValue::Alias(_) => FieldType::Alias,
            FieldValue::Num64(_) => FieldType::Num64,
        }
    }

    /// Convert to CSV string representation
    pub fn to_csv_string(&self) -> String {
        match self {
            FieldValue::NA => String::new(),
            FieldValue::String(s) => s.clone(),
            FieldValue::TF(b) => b.to_string(),
            FieldValue::Num(n) => n.to_string(),
            FieldValue::Per(f) => f.to_string(),
            FieldValue::FID(spf_id, row_id) => format!("{},{}", spf_id, row_id),
            FieldValue::Alias(s) => s.clone(),
            FieldValue::Num64(n) => n.to_string(),
        }
    }

    /// Parse from CSV string with the given field type
    pub fn from_csv_string(s: &str, ty: FieldType) -> Self {
        if s.is_empty() {
            // Return type-appropriate "empty" value based on field type
            return match ty {
                FieldType::NA => FieldValue::NA,
                FieldType::String => FieldValue::String(String::new()),
                FieldType::TF => FieldValue::TF(false),
                FieldType::Num => FieldValue::Num(0),
                FieldType::Per => FieldValue::Per(0.0),
                FieldType::FID => FieldValue::FID(0, 0),
                FieldType::Alias => FieldValue::Alias(String::new()),
                FieldType::Num64 => FieldValue::Num64(0),
            };
        }

        match ty {
            FieldType::NA => FieldValue::NA,
            FieldType::String => FieldValue::String(s.to_string()),
            FieldType::TF => {
                let lower = s.to_lowercase();
                FieldValue::TF(lower == "true" || lower == "1" || lower == "yes")
            }
            FieldType::Num => FieldValue::Num(s.parse().unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse Num value: {}", s);
                0
            })),
            FieldType::Per => FieldValue::Per(s.parse().unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse Per value: {}", s);
                0.0
            })),
            FieldType::FID => {
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
                    FieldValue::FID(spf_id, row_id)
                } else {
                    FieldValue::FID(0, 0)
                }
            }
            FieldType::Alias => FieldValue::Alias(s.to_string()),
            FieldType::Num64 => FieldValue::Num64(s.parse().unwrap_or_else(|_| {
                eprintln!("Warning: Failed to parse Num64 value: {}", s);
                0
            })),
        }
    }
}

// ============================================================================
// FieldDef Struct
// ============================================================================

/// Field definition with name and type
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
}

// ============================================================================
// LdtHeader Struct
// ============================================================================

/// LDT file header (8716 bytes)
#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LdtHeader {
    /// Database ID
    pub db_id: i32,
    /// Number of fields in the table
    pub num_fields: i32,
    /// Number of rows in the table
    pub num_rows: i32,
    /// Field names (128 fields * 64 bytes each = 8192 bytes)
    pub field_names: [[u8; FIELD_NAME_SIZE]; MAX_FIELDS],
    /// Field types (128 fields * 4 bytes each = 512 bytes)
    pub field_types: [i32; MAX_FIELDS],
}

impl LdtHeader {
    /// Get the name of a field at the given index
    pub fn field_name(&self, index: usize) -> String {
        if index >= MAX_FIELDS {
            return String::new();
        }

        let name_bytes = &self.field_names[index];
        let end = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(FIELD_NAME_SIZE);
        String::from_utf8_lossy(&name_bytes[..end]).into_owned()
    }

    /// Get the type of a field at the given index
    pub fn field_type(&self, index: usize) -> FieldType {
        if index >= MAX_FIELDS {
            return FieldType::NA;
        }
        FieldType::from_i32(self.field_types[index])
    }

    /// Get all field definitions for this header
    pub fn field_defs(&self) -> Vec<FieldDef> {
        let num = self.num_fields as usize;
        let mut defs = Vec::with_capacity(num);

        for i in 0..num {
            defs.push(FieldDef {
                name: self.field_name(i),
                field_type: self.field_type(i),
            });
        }

        defs
    }
}

// Compile-time size check
const _: () = assert!(std::mem::size_of::<LdtHeader>() == HEADER_SIZE);

// ============================================================================
// Row Struct
// ============================================================================

/// A row in an LDT table
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Primary key (row ID, stored as i32 in LDT file)
    pub primary_key: i32,
    /// Field values
    pub values: Vec<FieldValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_from_i32() {
        assert_eq!(FieldType::from_i32(0), FieldType::NA);
        assert_eq!(FieldType::from_i32(1), FieldType::String);
        assert_eq!(FieldType::from_i32(2), FieldType::TF);
        assert_eq!(FieldType::from_i32(3), FieldType::Num);
        assert_eq!(FieldType::from_i32(4), FieldType::Per);
        assert_eq!(FieldType::from_i32(5), FieldType::FID);
        assert_eq!(FieldType::from_i32(6), FieldType::Alias);
        assert_eq!(FieldType::from_i32(7), FieldType::Num64);
        assert_eq!(FieldType::from_i32(-1), FieldType::NA);
        assert_eq!(FieldType::from_i32(100), FieldType::NA);
    }

    #[test]
    fn test_field_type_csv_type_name() {
        assert_eq!(FieldType::NA.csv_type_name(), "na");
        assert_eq!(FieldType::String.csv_type_name(), "string");
        assert_eq!(FieldType::TF.csv_type_name(), "bool");
        assert_eq!(FieldType::Num.csv_type_name(), "int32");
        assert_eq!(FieldType::Per.csv_type_name(), "float32");
        assert_eq!(FieldType::FID.csv_type_name(), "fid");
        assert_eq!(FieldType::Alias.csv_type_name(), "alias");
        assert_eq!(FieldType::Num64.csv_type_name(), "int64");
    }

    #[test]
    fn test_field_type_from_csv_type_name() {
        assert_eq!(FieldType::from_csv_type_name("na"), Some(FieldType::NA));
        assert_eq!(
            FieldType::from_csv_type_name("STRING"),
            Some(FieldType::String)
        );
        assert_eq!(FieldType::from_csv_type_name("Bool"), Some(FieldType::TF));
        assert_eq!(FieldType::from_csv_type_name("invalid"), None);
    }

    #[test]
    fn test_field_type_is_variable_length() {
        assert!(!FieldType::NA.is_variable_length());
        assert!(FieldType::String.is_variable_length());
        assert!(!FieldType::TF.is_variable_length());
        assert!(!FieldType::Num.is_variable_length());
        assert!(!FieldType::Per.is_variable_length());
        assert!(FieldType::FID.is_variable_length());
        assert!(FieldType::Alias.is_variable_length());
        assert!(!FieldType::Num64.is_variable_length());
    }

    #[test]
    fn test_field_value_field_type() {
        assert_eq!(FieldValue::NA.field_type(), FieldType::NA);
        assert_eq!(
            FieldValue::String("test".into()).field_type(),
            FieldType::String
        );
        assert_eq!(FieldValue::TF(true).field_type(), FieldType::TF);
        assert_eq!(FieldValue::Num(42).field_type(), FieldType::Num);
        assert_eq!(FieldValue::Per(3.14).field_type(), FieldType::Per);
        assert_eq!(FieldValue::FID(1, 100).field_type(), FieldType::FID);
        assert_eq!(
            FieldValue::Alias("alias".into()).field_type(),
            FieldType::Alias
        );
        assert_eq!(FieldValue::Num64(9999999999).field_type(), FieldType::Num64);
    }

    #[test]
    fn test_field_value_to_csv_string() {
        assert_eq!(FieldValue::NA.to_csv_string(), "");
        assert_eq!(FieldValue::String("hello".into()).to_csv_string(), "hello");
        assert_eq!(FieldValue::TF(true).to_csv_string(), "true");
        assert_eq!(FieldValue::TF(false).to_csv_string(), "false");
        assert_eq!(FieldValue::Num(42).to_csv_string(), "42");
        assert_eq!(FieldValue::FID(1, 100).to_csv_string(), "1,100");
        assert_eq!(FieldValue::Num64(9999999999).to_csv_string(), "9999999999");
    }

    #[test]
    fn test_field_value_from_csv_string() {
        assert_eq!(
            FieldValue::from_csv_string("", FieldType::NA),
            FieldValue::NA
        );
        assert_eq!(
            FieldValue::from_csv_string("test", FieldType::String),
            FieldValue::String("test".into())
        );
        assert_eq!(
            FieldValue::from_csv_string("true", FieldType::TF),
            FieldValue::TF(true)
        );
        assert_eq!(
            FieldValue::from_csv_string("false", FieldType::TF),
            FieldValue::TF(false)
        );
        assert_eq!(
            FieldValue::from_csv_string("42", FieldType::Num),
            FieldValue::Num(42)
        );
        assert_eq!(
            FieldValue::from_csv_string("3.14", FieldType::Per),
            FieldValue::Per(3.14)
        );
        assert_eq!(
            FieldValue::from_csv_string("1,100", FieldType::FID),
            FieldValue::FID(1, 100)
        );
        assert_eq!(
            FieldValue::from_csv_string("alias_val", FieldType::Alias),
            FieldValue::Alias("alias_val".into())
        );
        assert_eq!(
            FieldValue::from_csv_string("9999999999", FieldType::Num64),
            FieldValue::Num64(9999999999)
        );
    }

    #[test]
    fn test_ldt_header_size() {
        assert_eq!(std::mem::size_of::<LdtHeader>(), HEADER_SIZE);
    }
}
