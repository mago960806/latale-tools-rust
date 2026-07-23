use latale_tools::common::encoding_from_name;
use latale_tools::ldt::{
    export_to_csv, import_from_csv, FieldDef, FieldType, FieldValue, LdtReader, LdtWriter,
};
use latale_tools::spf::{SpfReader, SpfRegistry, SpfWriter};
use latale_tools::stg::{StageFile, StgReader, StgWriter};
use rusqlite::{params_from_iter, types::Value, Connection};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

type CommandResult<T> = Result<T, String>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpfFileEntry {
    name: String,
    size: usize,
    res_id: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpfInfo {
    path: String,
    version: i32,
    encrypted: bool,
    file_id: i32,
    registry_name: Option<String>,
    encoding: String,
    file_count: usize,
    total_size: usize,
    description: String,
    files: Vec<SpfFileEntry>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistryItem {
    file_id: u8,
    name: String,
    version: i32,
    encoding: String,
    include_dirs: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldInfo {
    name: String,
    field_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtInfo {
    path: String,
    database_id: i32,
    field_count: i32,
    row_count: i32,
    total_size: usize,
    fields: Vec<FieldInfo>,
    rows: Vec<LdtRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LdtRow {
    primary_key: i32,
    values: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StgInfo {
    path: String,
    stage_count: usize,
    group_count: usize,
    map_count: usize,
    total_size: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationResult {
    output_path: String,
    summary: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseResult {
    output_path: String,
    extracted_files: usize,
    imported_tables: usize,
    imported_rows: usize,
    skipped_rows: usize,
    failures: Vec<String>,
}

struct TableImportResult {
    imported_rows: usize,
    skipped_rows: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    operation: String,
    current: usize,
    total: usize,
    item: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenRequest {
    action: String,
    path: String,
}

#[tauri::command]
fn spf_info(path: String) -> CommandResult<SpfInfo> {
    let path_ref = Path::new(&path);
    let reader = SpfReader::open(path_ref).map_err(display_error)?;
    let registry = SpfRegistry::find_by_file_id(reader.header().file_id as u8);
    let encoding = registry
        .map(|item| item.encoding.to_owned())
        .or_else(|| reader.encoding().map(|encoding| encoding.name().to_owned()))
        .unwrap_or_else(|| "GBK".to_owned());
    let files = reader
        .file_infos()
        .into_iter()
        .map(|finfo| SpfFileEntry {
            name: finfo.file_name_str_with_encoding(reader.encoding()),
            size: finfo.size.max(0) as usize,
            res_id: finfo.res_id.0,
        })
        .collect();

    Ok(SpfInfo {
        path,
        version: reader.version(),
        encrypted: reader.is_encrypted(),
        file_id: reader.header().file_id,
        registry_name: registry.map(|item| item.name.to_owned()),
        encoding,
        file_count: reader.file_count(),
        total_size: reader.total_size(),
        description: reader.header().desc_str().to_owned(),
        files,
    })
}

#[tauri::command]
fn spf_verify(path: String) -> CommandResult<Vec<String>> {
    SpfReader::open(Path::new(&path))
        .and_then(|reader| reader.verify())
        .map_err(display_error)
}

#[tauri::command]
async fn spf_unpack(app: AppHandle, path: String) -> CommandResult<OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let input = PathBuf::from(&path);
        let reader = SpfReader::open(&input).map_err(display_error)?;
        let encrypted = reader.is_encrypted();
        let output = containing_directory(&input);
        let callback = |current: usize, total: usize, item: &str| {
            let _ = app.emit(
                "operation-progress",
                ProgressEvent {
                    operation: "SPF 解包".to_owned(),
                    current,
                    total,
                    item: item.to_owned(),
                },
            );
        };
        reader
            .unpack(&output, Some(&callback))
            .map_err(display_error)?;

        Ok(OperationResult {
            output_path: output.to_string_lossy().into_owned(),
            summary: if encrypted {
                format!("已自动解密并解包 {} 个文件", reader.file_count())
            } else {
                format!("已解包 {} 个文件", reader.file_count())
            },
        })
    })
    .await
    .map_err(display_error)?
}

#[tauri::command]
async fn spf_unpack_to_sqlite(
    app: AppHandle,
    path: String,
    encoding: String,
) -> CommandResult<DatabaseResult> {
    tauri::async_runtime::spawn_blocking(move || {
        unpack_spf_to_sqlite(
            Path::new(&path),
            &encoding,
            |operation, current, total, item| {
                let _ = app.emit(
                    "operation-progress",
                    ProgressEvent {
                        operation: operation.to_owned(),
                        current,
                        total,
                        item: item.to_owned(),
                    },
                );
            },
        )
    })
    .await
    .map_err(display_error)?
}

#[tauri::command]
fn spf_registry() -> Vec<RegistryItem> {
    SpfRegistry::ALL
        .iter()
        .map(|item| RegistryItem {
            file_id: item.file_id,
            name: item.name.to_owned(),
            version: item.version,
            encoding: item.encoding.to_owned(),
            include_dirs: item
                .include_dirs
                .iter()
                .map(|path| (*path).to_owned())
                .collect(),
        })
        .collect()
}

#[tauri::command]
async fn spf_pack(
    app: AppHandle,
    spf_name: String,
    data_dir: String,
    output_path: String,
    version: i32,
    encoding: String,
    encrypted: bool,
) -> CommandResult<OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let registry = SpfRegistry::find_by_name(&spf_name)
            .ok_or_else(|| format!("未知的 SPF 资源类型: {spf_name}"))?;
        let mut writer = SpfWriter::new(registry.file_id, version, &encoding);
        writer.set_encrypted(encrypted);

        for include_dir in registry.include_dirs {
            writer
                .add_from_dir(Path::new(&data_dir), include_dir)
                .map_err(display_error)?;
        }

        let output = PathBuf::from(&output_path);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(display_error)?;
        }

        let callback = |current: usize, total: usize, item: &str| {
            let _ = app.emit(
                "operation-progress",
                ProgressEvent {
                    operation: "SPF 打包".to_owned(),
                    current,
                    total,
                    item: item.to_owned(),
                },
            );
        };
        writer
            .write(&output, Some(&callback))
            .map_err(display_error)?;

        Ok(OperationResult {
            output_path,
            summary: format!(
                "已生成 {}，共 {} 个文件{}",
                output.file_name().unwrap_or_default().to_string_lossy(),
                writer.file_count(),
                if encrypted { "（加密）" } else { "" }
            ),
        })
    })
    .await
    .map_err(display_error)?
}

#[tauri::command]
fn ldt_info(path: String, encoding: String) -> CommandResult<LdtInfo> {
    let reader =
        LdtReader::open(Path::new(&path), encoding_from_name(&encoding)).map_err(display_error)?;
    let fields = reader
        .field_defs()
        .into_iter()
        .map(|field| FieldInfo {
            name: field.name,
            field_type: field.field_type.csv_type_name().to_owned(),
        })
        .collect();
    let rows = reader
        .read_rows()
        .map_err(display_error)?
        .into_iter()
        .map(|row| LdtRow {
            primary_key: row.primary_key,
            values: row
                .values
                .into_iter()
                .map(|value| value.to_csv_string())
                .collect(),
        })
        .collect();

    Ok(LdtInfo {
        path,
        database_id: reader.db_id(),
        field_count: reader.field_count(),
        row_count: reader.row_count(),
        total_size: reader.total_size(),
        fields,
        rows,
    })
}

#[tauri::command]
async fn ldt_convert(input_path: String, encoding: String) -> CommandResult<OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let input = PathBuf::from(&input_path);
        let extension = extension_of(&input);
        let output = match extension.as_str() {
            "ldt" => input.with_extension("CSV"),
            "csv" => input.with_extension("LDT"),
            _ => return Err("请选择 LDT 或 CSV 文件".to_owned()),
        };
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(display_error)?;
        }
        let encoding_ref = encoding_from_name(&encoding);

        let summary = match extension.as_str() {
            "ldt" => {
                let reader = LdtReader::open(&input, encoding_ref).map_err(display_error)?;
                let fields = reader.field_defs();
                let rows = reader.read_rows().map_err(display_error)?;
                let mut file = File::create(&output).map_err(display_error)?;
                export_to_csv(&mut file, &fields, &rows).map_err(display_error)?;
                format!("LDT → CSV：{} 个字段，{} 行", fields.len(), rows.len())
            }
            "csv" => {
                let mut file = File::open(&input).map_err(display_error)?;
                let (database_id, fields, rows) =
                    import_from_csv(&mut file).map_err(display_error)?;
                let mut writer = LdtWriter::new(database_id, encoding_ref);
                writer.set_field_defs(&fields).set_rows(&rows);
                writer.write(&output).map_err(display_error)?;
                format!("CSV → LDT：{} 个字段，{} 行", fields.len(), rows.len())
            }
            _ => return Err("请选择 LDT 或 CSV 文件".to_owned()),
        };

        Ok(OperationResult {
            output_path: output.to_string_lossy().into_owned(),
            summary,
        })
    })
    .await
    .map_err(display_error)?
}

#[tauri::command]
fn stg_info(path: String, encoding: String) -> CommandResult<StgInfo> {
    let reader =
        StgReader::open(Path::new(&path), encoding_from_name(&encoding)).map_err(display_error)?;
    let total_size = reader.total_size();
    let stage = reader.read().map_err(display_error)?;
    Ok(StgInfo {
        path,
        stage_count: stage.stage_count(),
        group_count: stage.group_count(),
        map_count: stage.map_count(),
        total_size,
    })
}

#[tauri::command]
async fn stg_convert(
    input_path: String,
    output_path: String,
    encoding: String,
) -> CommandResult<OperationResult> {
    tauri::async_runtime::spawn_blocking(move || {
        let input = PathBuf::from(&input_path);
        let output = PathBuf::from(&output_path);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).map_err(display_error)?;
        }

        let extension = extension_of(&input);
        let encoding_ref = encoding_from_name(&encoding);
        let (stage, direction) = match extension.as_str() {
            "stg" => {
                let reader = StgReader::open(&input, encoding_ref).map_err(display_error)?;
                let stage = reader.read().map_err(display_error)?;
                let file = File::create(&output).map_err(display_error)?;
                serde_json::to_writer_pretty(file, &stage).map_err(display_error)?;
                (stage, "STG → JSON")
            }
            "json" => {
                let file = File::open(&input).map_err(display_error)?;
                let stage: StageFile = serde_json::from_reader(file).map_err(display_error)?;
                StgWriter::new(stage.clone(), encoding_ref)
                    .write(&output)
                    .map_err(display_error)?;
                (stage, "JSON → STG")
            }
            _ => return Err("请选择 STG 或 JSON 文件".to_owned()),
        };

        Ok(OperationResult {
            output_path,
            summary: format!(
                "{direction}：{} Stage / {} Group / {} Map",
                stage.stage_count(),
                stage.group_count(),
                stage.map_count()
            ),
        })
    })
    .await
    .map_err(display_error)?
}

fn unpack_spf_to_sqlite<F>(
    input: &Path,
    encoding: &str,
    progress: F,
) -> CommandResult<DatabaseResult>
where
    F: Fn(&str, usize, usize, &str),
{
    let reader = SpfReader::open(input).map_err(display_error)?;
    let rowid = SpfRegistry::find_by_name("ROWID").expect("ROWID registry must exist");
    if reader.header().file_id as u8 != rowid.file_id {
        return Err("只有 ROWID.SPF 可以生成 LDT 数据库".to_owned());
    }

    let output_dir = containing_directory(input);
    let ldt_files: Vec<(String, PathBuf)> = reader
        .file_infos()
        .into_iter()
        .filter_map(|finfo| {
            let file_name = finfo.file_name_str_with_encoding(reader.encoding());
            (extension_of(Path::new(&file_name)) == "ldt")
                .then(|| (file_name.clone(), output_dir.join(file_name)))
        })
        .collect();

    if ldt_files.is_empty() {
        return Err("ROWID.SPF 中没有找到 LDT 文件".to_owned());
    }

    let unpack_callback = |current: usize, total: usize, item: &str| {
        progress("SPF 解包", current, total, item);
    };
    reader
        .unpack(&output_dir, Some(&unpack_callback))
        .map_err(display_error)?;

    let database_path = output_dir.join("latale.db");
    let mut connection = Connection::open(&database_path).map_err(display_error)?;
    let total = ldt_files.len();
    let mut imported_tables = 0;
    let mut imported_rows = 0;
    let mut skipped_rows = 0;
    let mut failures = Vec::new();

    for (index, (file_name, path)) in ldt_files.iter().enumerate() {
        progress("生成数据库", index + 1, total, file_name);
        match import_ldt_table(&mut connection, path, encoding) {
            Ok(result) => {
                imported_tables += 1;
                imported_rows += result.imported_rows;
                skipped_rows += result.skipped_rows;
            }
            Err(error) => failures.push(format!("{file_name}: {error}")),
        }
    }

    Ok(DatabaseResult {
        output_path: database_path.to_string_lossy().into_owned(),
        extracted_files: reader.file_count(),
        imported_tables,
        imported_rows,
        skipped_rows,
        failures,
    })
}

fn import_ldt_table(
    connection: &mut Connection,
    path: &Path,
    encoding: &str,
) -> CommandResult<TableImportResult> {
    let reader = LdtReader::open(path, encoding_from_name(encoding)).map_err(display_error)?;
    let fields = reader.field_defs();
    let rows = reader.read_rows().map_err(display_error)?;
    let table_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| format!("无法确定数据表名称: {}", path.display()))?;
    let column_names = sqlite_column_names(&fields);

    let mut definitions = vec![format!("{} INTEGER", quote_identifier("ID"))];
    definitions.extend(fields.iter().zip(&column_names).map(|(field, name)| {
        format!(
            "{} {}",
            quote_identifier(name),
            sqlite_type(field.field_type)
        )
    }));

    let quoted_table = quote_identifier(table_name);
    let transaction = connection.transaction().map_err(display_error)?;
    transaction
        .execute_batch(&format!(
            "DROP TABLE IF EXISTS {quoted_table}; CREATE TABLE {quoted_table} ({});",
            definitions.join(", ")
        ))
        .map_err(display_error)?;

    let placeholders = vec!["?"; fields.len() + 1].join(", ");
    let insert_sql = format!("INSERT INTO {quoted_table} VALUES ({placeholders})");
    let mut statement = transaction.prepare(&insert_sql).map_err(display_error)?;
    let mut imported_rows = 0;
    let mut skipped_rows = 0;

    for row in rows {
        if row.primary_key == 0 {
            skipped_rows += 1;
            continue;
        }

        let mut values = Vec::with_capacity(row.values.len() + 1);
        values.push(Value::Integer(i64::from(row.primary_key)));
        values.extend(row.values.into_iter().map(sqlite_value));
        statement
            .execute(params_from_iter(values.iter()))
            .map_err(display_error)?;
        imported_rows += 1;
    }

    drop(statement);
    transaction.commit().map_err(display_error)?;
    Ok(TableImportResult {
        imported_rows,
        skipped_rows,
    })
}

fn sqlite_column_names(fields: &[FieldDef]) -> Vec<String> {
    let mut used = HashSet::from(["id".to_owned()]);
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let trimmed = field.name.trim_start_matches('_').replace('\0', "");
            let mut base = if trimmed.is_empty() {
                format!("FIELD_{}", index + 1)
            } else if trimmed.eq_ignore_ascii_case("ID") {
                "ID1".to_owned()
            } else {
                trimmed
            };
            let original = base.clone();
            let mut suffix = 2;
            while !used.insert(base.to_lowercase()) {
                base = format!("{original}_{suffix}");
                suffix += 1;
            }
            base
        })
        .collect()
}

fn sqlite_type(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::TF | FieldType::Num | FieldType::Num64 => "INTEGER",
        FieldType::Per => "REAL",
        FieldType::NA | FieldType::String | FieldType::FID | FieldType::Alias => "TEXT",
    }
}

fn sqlite_value(value: FieldValue) -> Value {
    match value {
        FieldValue::NA => Value::Null,
        FieldValue::String(value) | FieldValue::Alias(value) => Value::Text(value),
        FieldValue::TF(value) => Value::Integer(i64::from(value)),
        FieldValue::Num(value) => Value::Integer(i64::from(value)),
        FieldValue::Per(value) => Value::Real(f64::from(value)),
        FieldValue::FID(spf_id, row_id) => Value::Text(format!("{spf_id},{row_id}")),
        FieldValue::Num64(value) => Value::Integer(value),
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[tauri::command]
fn launch_requests() -> Vec<OpenRequest> {
    parse_open_requests(&std::env::args().collect::<Vec<_>>())
}

fn parse_open_requests(args: &[String]) -> Vec<OpenRequest> {
    let mut requests = Vec::new();
    let mut action = "open".to_owned();
    let mut index = 1;

    while index < args.len() {
        if args[index] == "--action" && index + 1 < args.len() {
            action = args[index + 1].clone();
            index += 2;
            continue;
        }

        let path = Path::new(&args[index]);
        if matches!(
            extension_of(path).as_str(),
            "spf" | "ldt" | "stg" | "csv" | "json"
        ) {
            requests.push(OpenRequest {
                action: action.clone(),
                path: args[index].clone(),
            });
            action = "open".to_owned();
        }
        index += 1;
    }

    requests
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn containing_directory(path: &Path) -> PathBuf {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let requests = parse_open_requests(&args);
            if !requests.is_empty() {
                let _ = app.emit("open-request", requests);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            spf_info,
            spf_verify,
            spf_unpack,
            spf_unpack_to_sqlite,
            spf_registry,
            spf_pack,
            ldt_info,
            ldt_convert,
            stg_info,
            stg_convert,
            launch_requests,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LaTale Tools");
}

#[cfg(test)]
mod tests {
    use super::*;
    use latale_tools::ldt::Row;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("latale-gui-{label}-{unique}"))
    }

    #[test]
    fn open_request_parser_keeps_context_menu_action() {
        let args = vec![
            "latale-tools.exe".to_owned(),
            "--action".to_owned(),
            "verify".to_owned(),
            r"C:\Game\ROWID.SPF".to_owned(),
        ];

        let requests = parse_open_requests(&args);
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].action, "verify");
        assert_eq!(requests[0].path, r"C:\Game\ROWID.SPF");
    }

    #[test]
    fn spf_commands_read_encrypted_writer_output() {
        let directory = unique_temp_dir("spf");
        std::fs::create_dir_all(&directory).expect("test directory should be created");
        let path = directory.join("ROWID.SPF");

        let mut writer = SpfWriter::new(3, 2_026_071_602, "GBK");
        writer.set_encrypted(true);
        writer.add_file("DATA/LDT/GUI_TEST.LDT".to_owned(), b"gui test".to_vec());
        writer
            .write(&path, None)
            .expect("test SPF should be written");

        let info = spf_info(path.to_string_lossy().into_owned()).expect("SPF info should load");
        assert!(info.encrypted);
        assert_eq!(info.version, 2_026_071_602);
        assert_eq!(info.file_count, 1);
        assert_eq!(info.files[0].name, "DATA/LDT/GUI_TEST.LDT");
        assert!(spf_verify(path.to_string_lossy().into_owned())
            .expect("SPF verification should run")
            .is_empty());
        assert_eq!(containing_directory(&path), directory);

        std::fs::remove_file(path).expect("test SPF should be removed");
        std::fs::remove_dir(directory).expect("test directory should be removed");
    }

    #[test]
    fn ldt_info_returns_all_rows_and_convert_uses_input_directory() {
        let directory = unique_temp_dir("ldt");
        std::fs::create_dir_all(&directory).expect("test directory should be created");
        let input = directory.join("GUI_TEST.LDT");

        let fields = vec![
            FieldDef {
                name: "Name".to_owned(),
                field_type: FieldType::String,
            },
            FieldDef {
                name: "Value".to_owned(),
                field_type: FieldType::Num,
            },
        ];
        let rows = vec![
            Row {
                primary_key: 1,
                values: vec![FieldValue::String("Alpha".to_owned()), FieldValue::Num(10)],
            },
            Row {
                primary_key: 2,
                values: vec![FieldValue::String("Beta".to_owned()), FieldValue::Num(20)],
            },
        ];
        let mut writer = LdtWriter::new(7, encoding_from_name("GBK"));
        writer.set_field_defs(&fields).set_rows(&rows);
        writer.write(&input).expect("test LDT should be written");

        let info = ldt_info(input.to_string_lossy().into_owned(), "GBK".to_owned())
            .expect("LDT info should load");
        assert_eq!(info.rows.len(), 2);
        assert_eq!(info.rows[0].primary_key, 1);
        assert_eq!(info.rows[0].values, ["Alpha", "10"]);

        let result = tauri::async_runtime::block_on(ldt_convert(
            input.to_string_lossy().into_owned(),
            "GBK".to_owned(),
        ))
        .expect("LDT conversion should succeed");
        let output = directory.join("GUI_TEST.CSV");
        assert_eq!(PathBuf::from(result.output_path), output);
        assert!(output.is_file());

        std::fs::remove_file(input).expect("test LDT should be removed");
        std::fs::remove_file(output).expect("test CSV should be removed");
        std::fs::remove_dir(directory).expect("test directory should be removed");
    }

    #[test]
    fn encrypted_rowid_spf_unpacks_and_builds_sqlite() {
        let directory = unique_temp_dir("sqlite");
        std::fs::create_dir_all(&directory).expect("test directory should be created");
        let source_ldt = directory.join("SOURCE.LDT");
        let spf_path = directory.join("ROWID.SPF");

        let fields = vec![
            FieldDef {
                name: "__Name".to_owned(),
                field_type: FieldType::String,
            },
            FieldDef {
                name: "ID".to_owned(),
                field_type: FieldType::Num,
            },
            FieldDef {
                name: "Enabled".to_owned(),
                field_type: FieldType::TF,
            },
            FieldDef {
                name: "Ratio".to_owned(),
                field_type: FieldType::Per,
            },
            FieldDef {
                name: "Link".to_owned(),
                field_type: FieldType::FID,
            },
            FieldDef {
                name: "Alias".to_owned(),
                field_type: FieldType::Alias,
            },
            FieldDef {
                name: "Large".to_owned(),
                field_type: FieldType::Num64,
            },
        ];
        let rows = vec![
            Row {
                primary_key: 0,
                values: vec![
                    FieldValue::String("placeholder".to_owned()),
                    FieldValue::Num(0),
                    FieldValue::TF(false),
                    FieldValue::Per(0.0),
                    FieldValue::FID(0, 0),
                    FieldValue::Alias(String::new()),
                    FieldValue::Num64(0),
                ],
            },
            Row {
                primary_key: 7,
                values: vec![
                    FieldValue::String("测试".to_owned()),
                    FieldValue::Num(77),
                    FieldValue::TF(true),
                    FieldValue::Per(1.25),
                    FieldValue::FID(3, 42),
                    FieldValue::Alias("item_alias".to_owned()),
                    FieldValue::Num64(9_000_000_001),
                ],
            },
        ];
        let mut ldt_writer = LdtWriter::new(9, encoding_from_name("GBK"));
        ldt_writer.set_field_defs(&fields).set_rows(&rows);
        ldt_writer
            .write(&source_ldt)
            .expect("test LDT should be written");

        let mut spf_writer = SpfWriter::new(3, 2_026_072_301, "GBK");
        spf_writer.set_encrypted(true);
        spf_writer.add_file(
            "DATA/LDT/SQL_TEST.LDT".to_owned(),
            std::fs::read(&source_ldt).expect("test LDT should be readable"),
        );
        spf_writer
            .write(&spf_path, None)
            .expect("test SPF should be written");

        let result = unpack_spf_to_sqlite(&spf_path, "GBK", |_, _, _, _| {})
            .expect("SPF should unpack and export to SQLite");
        assert_eq!(result.extracted_files, 1);
        assert_eq!(result.imported_tables, 1);
        assert_eq!(result.imported_rows, 1);
        assert_eq!(result.skipped_rows, 1);
        assert!(result.failures.is_empty());
        assert!(directory.join("DATA/LDT/SQL_TEST.LDT").is_file());

        let connection =
            Connection::open(directory.join("latale.db")).expect("generated database should open");
        let column_names: Vec<String> = connection
            .prepare("PRAGMA table_info(\"SQL_TEST\")")
            .expect("table metadata query should prepare")
            .query_map([], |row| row.get(1))
            .expect("table metadata query should run")
            .collect::<Result<_, _>>()
            .expect("column names should decode");
        assert_eq!(
            column_names,
            ["ID", "Name", "ID1", "Enabled", "Ratio", "Link", "Alias", "Large"]
        );

        let stored = connection
            .query_row(
                "SELECT ID, Name, ID1, Enabled, Ratio, Link, Alias, Large FROM \"SQL_TEST\"",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .expect("exported row should be queryable");
        assert_eq!(stored.0, 7);
        assert_eq!(stored.1, "测试");
        assert_eq!(stored.2, 77);
        assert_eq!(stored.3, 1);
        assert!((stored.4 - 1.25).abs() < f64::EPSILON);
        assert_eq!(stored.5, "3,42");
        assert_eq!(stored.6, "item_alias");
        assert_eq!(stored.7, 9_000_000_001);
        drop(connection);

        std::fs::remove_file(source_ldt).expect("source LDT should be removed");
        std::fs::remove_file(spf_path).expect("test SPF should be removed");
        std::fs::remove_file(directory.join("latale.db")).expect("test database should be removed");
        std::fs::remove_file(directory.join("DATA/LDT/SQL_TEST.LDT"))
            .expect("extracted LDT should be removed");
        std::fs::remove_dir(directory.join("DATA/LDT"))
            .expect("test LDT directory should be removed");
        std::fs::remove_dir(directory.join("DATA")).expect("test DATA directory should be removed");
        std::fs::remove_dir(directory).expect("test directory should be removed");
    }

    #[test]
    #[ignore = "requires LATALE_REAL_ROWID and writes the extracted DATA directory"]
    fn real_rowid_spf_unpacks_and_builds_sqlite() {
        let source = PathBuf::from(
            std::env::var("LATALE_REAL_ROWID")
                .expect("set LATALE_REAL_ROWID to a real ROWID SPF file"),
        );
        let encoding =
            std::env::var("LATALE_REAL_ROWID_ENCODING").unwrap_or_else(|_| "GBK".to_owned());
        let directory = unique_temp_dir("real-rowid");
        std::fs::create_dir_all(&directory).expect("test directory should be created");
        let spf_path = directory.join("ROWID.SPF");
        std::fs::hard_link(&source, &spf_path).expect("real ROWID should be linked into test dir");

        let result = unpack_spf_to_sqlite(&spf_path, &encoding, |_, _, _, _| {})
            .expect("real ROWID should unpack and export to SQLite");
        assert!(result.extracted_files > 0);
        assert!(result.imported_tables > 0);
        assert!(result.imported_rows > 0);
        assert!(
            result.failures.is_empty(),
            "some LDT files failed: {}",
            result.failures.join("; ")
        );

        let connection =
            Connection::open(&result.output_path).expect("generated database should open");
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .expect("generated tables should be queryable");
        assert_eq!(table_count as usize, result.imported_tables);
        println!(
            "real ROWID: {} files, {} tables, {} rows, {} skipped",
            result.extracted_files,
            result.imported_tables,
            result.imported_rows,
            result.skipped_rows
        );
        drop(connection);

        std::fs::remove_dir_all(&directory).expect("real fixture output should be removed");
    }
}
