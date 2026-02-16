//! CSV conversion for LDT files
//!
//! This module provides import/export functionality between LDT and CSV formats.

use crate::ldt::{FieldDef, FieldType, FieldValue, Row};
use anyhow::{bail, Context, Result};
use std::io::{Read, Write};

/// CSV header comment format:
/// # database: <name>
/// # rows: <count>

/// Export LDT data to CSV format
pub fn export_to_csv<W: Write>(
    writer: &mut W,
    _db_id: i32,
    field_defs: &[FieldDef],
    rows: &[Row],
    _name: &str,
) -> Result<()> {
    // Write CSV header with type annotations
    // First column is always the primary key (int64)
    let mut headers = vec!["ID:int64".to_string()];
    for f in field_defs.iter() {
        headers.push(format!("{}:{}", f.name, f.field_type.csv_type_name()));
    }
    writeln!(writer, "{}", headers.join(","))?;

    // Write rows
    for row in rows {
        // Prepend primary key as first column
        let mut line_values = vec![row.primary_key.to_string()];
        for v in row.values.iter() {
            let s = v.to_csv_string();
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                line_values.push(format!("\"{}\"", s.replace('"', "\"\"")));
            } else {
                line_values.push(s);
            }
        }
        writeln!(writer, "{}", line_values.join(","))?;
    }

    Ok(())
}

/// Import LDT data from CSV format
/// Returns (db_id, field_definitions, rows)
/// Note: field_definitions does NOT include the ID (primary key) column
/// db_id is always 0 (default)
pub fn import_from_csv<R: Read>(reader: &mut R) -> Result<(i32, Vec<FieldDef>, Vec<Row>)> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        bail!("CSV file is empty");
    }

    // Skip any comment lines at the beginning
    let mut line_idx = 0;
    while line_idx < lines.len() && lines[line_idx].trim().starts_with('#') {
        line_idx += 1;
    }

    if line_idx >= lines.len() {
        bail!("No CSV header found");
    }

    // Parse field definitions from header
    let header_line = lines[line_idx];
    let all_field_defs = parse_csv_header(header_line)?;

    // Skip the first column (ID/primary key) - it's stored separately in LDT
    let field_defs: Vec<FieldDef> = all_field_defs.into_iter().skip(1).collect();
    line_idx += 1;

    // Parse rows
    let mut rows = Vec::new();
    while line_idx < lines.len() {
        let line = lines[line_idx].trim();
        if line.is_empty() || line.starts_with('#') {
            line_idx += 1;
            continue;
        }

        let row = parse_csv_row(line, &field_defs)?;
        rows.push(row);
        line_idx += 1;
    }

    // db_id is always 0 (default)
    Ok((0, field_defs, rows))
}

/// Parse CSV header line to extract field definitions
fn parse_csv_header(line: &str) -> Result<Vec<FieldDef>> {
    let columns = parse_csv_line(line)?;
    let mut field_defs = Vec::with_capacity(columns.len());

    for col in &columns {
        // Format: "fieldname:typename" or just "fieldname"
        let parts: Vec<&str> = col.split(':').collect();
        let name = parts[0].trim().to_string();

        let field_type = if parts.len() > 1 {
            FieldType::from_csv_type_name(parts[1].trim())
                .ok_or_else(|| anyhow::anyhow!("Unknown field type: {}", parts[1]))?
        } else {
            FieldType::NA
        };

        field_defs.push(FieldDef { name, field_type });
    }

    Ok(field_defs)
}

/// Parse a CSV data row
fn parse_csv_row(line: &str, field_defs: &[FieldDef]) -> Result<Row> {
    let columns: Vec<String> = parse_csv_line(line)?;

    if columns.is_empty() {
        bail!("Empty CSV row");
    }

    // First column is the primary key
    let primary_key: i64 = columns[0].parse()
        .with_context(|| format!("Invalid primary key: {}", columns[0]))?;

    // Parse remaining columns as field values
    // field_defs now contains only the actual fields (not the ID column)
    let mut values = Vec::with_capacity(field_defs.len());

    for (i, col) in columns.iter().skip(1).enumerate() {
        let field_type = if i < field_defs.len() {
            field_defs[i].field_type
        } else {
            FieldType::NA
        };
        values.push(FieldValue::from_csv_string(col, field_type));
    }

    Ok(Row { primary_key, values })
}

/// Parse a CSV line respecting quoted strings
fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    let mut columns = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if in_quotes {
            if c == '"' {
                // Check for escaped quote
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    current.push('"');
                    i += 2;
                    continue;
                } else {
                    in_quotes = false;
                    i += 1;
                    continue;
                }
            } else {
                current.push(c);
            }
        } else {
            if c == '"' {
                in_quotes = true;
            } else if c == ',' {
                columns.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(c);
            }
        }
        i += 1;
    }

    // Add the last column
    columns.push(current.trim().to_string());

    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_line_simple() {
        let line = "1,hello,world,42";
        let cols = parse_csv_line(line).unwrap();
        assert_eq!(cols, vec!["1", "hello", "world", "42"]);
    }

    #[test]
    fn test_parse_csv_line_quoted() {
        let line = "1,\"hello, world\",42";
        let cols = parse_csv_line(line).unwrap();
        assert_eq!(cols, vec!["1", "hello, world", "42"]);
    }

    #[test]
    fn test_parse_csv_line_escaped_quotes() {
        let line = "1,\"say \"\"hi\"\"\",42";
        let cols = parse_csv_line(line).unwrap();
        assert_eq!(cols, vec!["1", "say \"hi\"", "42"]);
    }

    #[test]
    fn test_export_import_roundtrip() {
        let field_defs = vec![
            FieldDef { name: "ID".to_string(), field_type: FieldType::Num64 },
            FieldDef { name: "Name".to_string(), field_type: FieldType::String },
            FieldDef { name: "Price".to_string(), field_type: FieldType::Num },
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
        export_to_csv(&mut csv_output, 0, &field_defs, &rows, "TEST").unwrap();

        // Import from CSV
        let csv_string = String::from_utf8(csv_output).unwrap();
        let (db_id, imported_defs, imported_rows) =
            import_from_csv(&mut csv_string.as_bytes()).unwrap();

        assert_eq!(db_id, 0);  // db_id is always 0
        assert_eq!(imported_defs.len(), 3);
        assert_eq!(imported_rows.len(), 2);
        assert_eq!(imported_rows[0].primary_key, 1);
        assert_eq!(imported_rows[1].primary_key, 2);
    }
}
