use latale_tools::common::encoding_from_name;
use latale_tools::ldt::{export_to_csv, import_from_csv, LdtReader, LdtWriter};
use latale_tools::spf::{SpfReader, SpfRegistry, SpfWriter};
use latale_tools::stg::{StageFile, StgReader, StgWriter};
use serde::Serialize;
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
    use latale_tools::ldt::{FieldDef, FieldType, FieldValue, Row};
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
            "latale-tools-gui.exe".to_owned(),
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
}
