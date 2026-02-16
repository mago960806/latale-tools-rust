//! CSV conversion for LDT files
//!
//! This module provides import/export functionality between LDT and CSV formats.

use crate::ldt::{
    FieldDef, FieldType, FieldValue, Row, CSV_ID_COLUMN_HEADER, CSV_TYPE_SEPARATOR, DEFAULT_DB_ID,
};
use anyhow::{Context, Result};
use csv::{Reader, Writer};
use std::io::{Read, Write};

/// Export LDT data to CSV format
pub fn export_to_csv<W: Write>(
    writer: &mut W,
    field_defs: &[FieldDef],
    rows: &[Row],
) -> Result<()> {
    let mut csv_writer = Writer::from_writer(writer);

    // Write CSV header with type annotations
    // First column is always the primary key (int64)
    let mut headers = vec![CSV_ID_COLUMN_HEADER.to_string()];
    for f in field_defs.iter() {
        headers.push(format!("{}{}{}", f.name, CSV_TYPE_SEPARATOR, f.field_type.csv_type_name()));
    }
    csv_writer
        .write_record(&headers)
        .context("Failed to write CSV header")?;

    // Write rows
    for row in rows {
        let mut record = vec![row.primary_key.to_string()];
        for v in row.values.iter() {
            record.push(v.to_csv_string());
        }
        csv_writer
            .write_record(&record)
            .context("Failed to write CSV record")?;
    }

    csv_writer.flush().context("Failed to flush CSV writer")?;
    Ok(())
}

/// Import LDT data from CSV format
/// Returns (db_id, field_definitions, rows)
/// Note: field_definitions does NOT include the ID (primary key) column
/// db_id is always 0 (default)
pub fn import_from_csv<R: Read>(reader: &mut R) -> Result<(i32, Vec<FieldDef>, Vec<Row>)> {
    let mut csv_reader = Reader::from_reader(reader);

    // Parse field definitions from header
    let headers = csv_reader
        .headers()
        .context("Failed to read CSV headers")?;

    let mut all_field_defs = Vec::with_capacity(headers.len());
    for header in headers.iter() {
        // Format: "fieldname:typename" or just "fieldname"
        // Note: Don't trim - preserve spaces in field names, and warn if type has unexpected spaces
        let parts: Vec<&str> = header.split(CSV_TYPE_SEPARATOR).collect();
        let name = parts[0].to_string();

        let field_type = if parts.len() > 1 {
            let type_str = parts[1];
            if type_str != type_str.trim() {
                eprintln!("Warning: Type name '{}' has leading/trailing spaces in header: {}", type_str, header);
            }
            FieldType::from_csv_type_name(type_str).unwrap_or(FieldType::NA)
        } else {
            FieldType::NA
        };

        all_field_defs.push(FieldDef { name, field_type });
    }

    // Skip the first column (ID/primary key) - it's stored separately in LDT
    let field_defs: Vec<FieldDef> = all_field_defs.into_iter().skip(1).collect();

    // Parse rows
    let mut rows = Vec::new();
    for result in csv_reader.records() {
        let record = result.context("Failed to read CSV record")?;

        if record.is_empty() {
            continue;
        }

        // First column is the primary key
        let primary_key: i32 = record
            .get(0)
            .context("Missing primary key column")?
            .parse()
            .context("Invalid primary key value")?;

        // Parse remaining columns as field values
        let mut values = Vec::with_capacity(field_defs.len());
        for (i, col) in record.iter().skip(1).enumerate() {
            let field_type = field_defs
                .get(i)
                .map(|d| d.field_type)
                .unwrap_or(FieldType::NA);
            values.push(FieldValue::from_csv_string(col, field_type));
        }

        rows.push(Row { primary_key, values });
    }

    // db_id is always 0 (default)
    Ok((DEFAULT_DB_ID, field_defs, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let field_defs = vec![
            FieldDef {
                name: "ID".to_string(),
                field_type: FieldType::Num64,
            },
            FieldDef {
                name: "Name".to_string(),
                field_type: FieldType::String,
            },
            FieldDef {
                name: "Price".to_string(),
                field_type: FieldType::Num,
            },
        ];

        let rows = vec![
            Row {
                primary_key: 1,
                values: vec![
                    FieldValue::Num64(1),
                    FieldValue::String("Test Item".to_string()),
                    FieldValue::Num(100),
                ],
            },
            Row {
                primary_key: 2,
                values: vec![
                    FieldValue::Num64(2),
                    FieldValue::String("Another, Item".to_string()),
                    FieldValue::Num(200),
                ],
            },
        ];

        // Export to CSV
        let mut csv_output = Vec::new();
        export_to_csv(&mut csv_output, &field_defs, &rows).unwrap();

        // Import from CSV
        let csv_string = String::from_utf8(csv_output).unwrap();
        let (db_id, imported_defs, imported_rows) =
            import_from_csv(&mut csv_string.as_bytes()).unwrap();

        assert_eq!(db_id, DEFAULT_DB_ID); // db_id is always DEFAULT_DB_ID
        assert_eq!(imported_defs.len(), 3);
        assert_eq!(imported_rows.len(), 2);
        assert_eq!(imported_rows[0].primary_key, 1);
        assert_eq!(imported_rows[1].primary_key, 2);
    }

    #[test]
    fn test_export_handles_special_characters() {
        let field_defs = vec![FieldDef {
            name: "Text".to_string(),
            field_type: FieldType::String,
        }];

        let rows = vec![Row {
            primary_key: 1,
            values: vec![FieldValue::String(
                "Line1\nLine2,\"quoted\",".to_string(),
            )],
        }];

        // Export to CSV
        let mut csv_output = Vec::new();
        export_to_csv(&mut csv_output, &field_defs, &rows).unwrap();

        // Import from CSV
        let csv_string = String::from_utf8(csv_output).unwrap();
        let (_db_id, _imported_defs, imported_rows) =
            import_from_csv(&mut csv_string.as_bytes()).unwrap();

        assert_eq!(imported_rows.len(), 1);
        assert_eq!(
            imported_rows[0].values[0],
            FieldValue::String("Line1\nLine2,\"quoted\",".to_string())
        );
    }

    #[test]
    fn test_export_handles_empty_values() {
        let field_defs = vec![
            FieldDef {
                name: "Name".to_string(),
                field_type: FieldType::String,
            },
            FieldDef {
                name: "Count".to_string(),
                field_type: FieldType::Num,
            },
        ];

        let rows = vec![Row {
            primary_key: 1,
            values: vec![FieldValue::NA, FieldValue::Num(0)],
        }];

        // Export to CSV
        let mut csv_output = Vec::new();
        export_to_csv(&mut csv_output, &field_defs, &rows).unwrap();

        // Import from CSV
        let csv_string = String::from_utf8(csv_output).unwrap();
        let (_db_id, _imported_defs, imported_rows) =
            import_from_csv(&mut csv_string.as_bytes()).unwrap();

        assert_eq!(imported_rows.len(), 1);
        // NA values are exported as empty strings and imported back as NA
        assert_eq!(imported_rows[0].values[0], FieldValue::NA);
        assert_eq!(imported_rows[0].values[1], FieldValue::Num(0));
    }
}
