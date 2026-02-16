use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::ldt::{export_to_csv, import_from_csv, LdtReader, LdtWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;

// 格式化字节数为人类可读格式
fn format_size(size: usize) -> String {
    if size >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if size >= 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    }
}

// 格式化耗时：小于1秒显示毫秒，大于等于1秒显示秒
fn format_duration(millis: u128) -> String {
    if millis < 1000 {
        format!("{} ms", millis)
    } else {
        format!("{:.2} s", millis as f64 / 1000.0)
    }
}

// 打印分隔线
fn print_separator() {
    println!("{}", "-".repeat(60));
}

/// 打印分节标题
fn print_section_header<T: std::fmt::Display>(title: &str, extra: T) {
    println!();
    println!("[{}]", title);
    println!(" {}", extra);
    print_separator();
}

#[derive(Parser)]
#[command(name = "latale-ldt")]
#[command(about = "LaTale LDT 数据库工具", version)]
#[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示 LDT 文件信息
    Info {
        /// LDT 文件路径
        ldt_file: PathBuf,
        /// 显示前 N 行数据
        #[arg(short, long, default_value = "5")]
        rows: usize,
    },

    /// 双向转换：LDT ↔ CSV（支持单文件和目录批量）
    Convert {
        /// 输入文件或目录（默认 DATA/LDT）
        input: Option<PathBuf>,
        /// 输出路径
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { ldt_file, rows } => {
            cmd_info(&ldt_file, rows)?;
        }
        Commands::Convert { input, output } => {
            let input = input.as_deref();
            cmd_convert(input, output.as_deref())?;
        }
    }

    Ok(())
}

fn cmd_info(ldt_file: &std::path::Path, preview_rows: usize) -> Result<()> {
    let reader = LdtReader::open(ldt_file)
        .with_context(|| format!("无法打开 LDT 文件: {}", ldt_file.display()))?;

    let field_defs = reader.field_defs();

    // 提取数据库名称（从文件名）
    let db_name = ldt_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    print_section_header("文件信息", ldt_file.display());

    println!("数据库名称:  {}", db_name);
    println!("数据库 ID:   {}", reader.db_id());
    println!("字段数量:    {}", reader.field_count());
    println!("数据行数:    {}", reader.row_count());
    println!("文件大小:    {}", format_size(reader.total_size()));

    // 显示字段定义
    println!();
    println!("[字段定义]");
    print_separator();

    for (i, def) in field_defs.iter().enumerate() {
        println!(
            "  [{:2}] {:<24} {:<10}",
            i,
            def.name,
            def.field_type.csv_type_name()
        );
    }

    // 读取并显示部分数据
    if preview_rows > 0 && reader.row_count() > 0 {
        println!();
        println!(
            "[数据预览] 前 {} 行:",
            preview_rows.min(reader.row_count() as usize)
        );
        print_separator();

        let rows = reader.read_rows().context("读取数据行失败")?;

        for (i, row) in rows.iter().take(preview_rows).enumerate() {
            print!("  [{:3}] PK={}: ", i + 1, row.primary_key);
            for (j, value) in row.values.iter().enumerate().take(5) {
                if j > 0 {
                    print!(", ");
                }
                let s = value.to_csv_string();
                // Use char-based truncation to avoid UTF-8 boundary issues
                let truncated: String = s.chars().take(20).collect();
                if truncated.len() < s.len() {
                    print!("{}...", truncated);
                } else {
                    print!("{}", s);
                }
            }
            if row.values.len() > 5 {
                println!(" ... ({} more)", row.values.len() - 5);
            } else {
                println!();
            }
        }

        if rows.len() > preview_rows {
            println!("  ... ({} more rows)", rows.len() - preview_rows);
        }
    }

    println!();

    Ok(())
}

fn cmd_convert(input: Option<&std::path::Path>, output: Option<&std::path::Path>) -> Result<()> {
    let input = input.unwrap_or_else(|| std::path::Path::new("DATA/LDT"));

    if !input.exists() {
        bail!("输入路径不存在: {}", input.display());
    }

    if input.is_file() {
        convert_single_file(input, output)
    } else if input.is_dir() {
        convert_directory(input, output)
    } else {
        bail!("输入路径不是文件或目录: {}", input.display())
    }
}

/// 转换单个文件
fn convert_single_file(input: &Path, output: Option<&Path>) -> Result<()> {
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match input_ext.as_str() {
        "ldt" => {
            // LDT → CSV
            let output_path = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    // 默认输出到 DATA/CSV/{name}.csv
                    let name = input.file_stem().unwrap_or_default();
                    PathBuf::from("DATA/CSV").join(format!("{}.csv", name.to_string_lossy()))
                });

            // 如果输出路径是目录，则在其中创建文件
            let output_path = if output_path.extension().is_none() {
                let name = input.file_stem().unwrap_or_default();
                output_path.join(format!("{}.csv", name.to_string_lossy()))
            } else {
                output_path
            };

            convert_ldt_to_csv(input, &output_path)?;
        }

        "csv" => {
            // CSV → LDT
            let output_path = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    // 默认输出到 DATA/LDT/{name}.LDT
                    let name = input.file_stem().unwrap_or_default();
                    PathBuf::from("DATA/LDT").join(format!("{}.LDT", name.to_string_lossy()))
                });

            // 如果输出路径是目录，则在其中创建文件
            let output_path = if output_path.extension().is_none() {
                let name = input.file_stem().unwrap_or_default();
                output_path.join(format!("{}.LDT", name.to_string_lossy()))
            } else {
                output_path
            };

            convert_csv_to_ldt(input, &output_path)?;
        }

        _ => {
            bail!(
                "不支持的文件格式: {}，请使用 .LDT 或 .csv 文件",
                input_ext
            );
        }
    }

    Ok(())
}

/// 转换目录
fn convert_directory(input: &Path, output: Option<&Path>) -> Result<()> {
    // 统计文件类型
    let (ldt_files, csv_files) = count_files_by_type(input)?;

    // 混合类型报错
    if ldt_files.len() > 0 && csv_files.len() > 0 {
        bail!(
            "目录中同时存在 .LDT 和 .csv 文件，请分开处理。\n\
             找到 {} 个 .LDT 文件和 {} 个 .csv 文件",
            ldt_files.len(),
            csv_files.len()
        );
    }

    if ldt_files.is_empty() && csv_files.is_empty() {
        bail!("目录中没有 .LDT 或 .csv 文件: {}", input.display());
    }

    // 确定转换方向和默认输出
    let (files, default_output_dir, direction) = if !ldt_files.is_empty() {
        (ldt_files, "DATA/CSV", "LDT → CSV")
    } else {
        (csv_files, "DATA/LDT", "CSV → LDT")
    };

    let output_dir = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(default_output_dir));

    // 输入输出相同报错
    let input_canonical = std::fs::canonicalize(input).ok();
    let output_canonical = if output_dir.exists() {
        std::fs::canonicalize(&output_dir).ok()
    } else {
        None
    };

    if let (Some(inp), Some(outp)) = (&input_canonical, &output_canonical) {
        if inp == outp {
            bail!("输入目录和输出目录相同: {}", input.display());
        }
    }

    // 创建输出目录
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("无法创建输出目录: {}", output_dir.display()))?;

    print_section_header("批量转换", direction);
    println!("输入目录:    {}", input.display());
    println!("输出目录:    {}", output_dir.display());
    println!("文件数量:    {}", files.len());
    println!();

    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, file) in files.iter().enumerate() {
        let file_name = file.file_name().unwrap_or_default().to_string_lossy();
        print!("  [{}/{}] {} ... ", i + 1, files.len(), file_name);

        match convert_single_file_silent(file, &output_dir) {
            Ok(_) => {
                println!("完成");
                success_count += 1;
            }
            Err(e) => {
                println!("失败: {}", e);
                error_count += 1;
            }
        }
    }

    let elapsed = start.elapsed().as_millis();
    println!();
    println!(
        "[完成] 成功: {}, 失败: {}, 耗时 {}",
        success_count,
        error_count,
        format_duration(elapsed)
    );
    println!();

    Ok(())
}

/// 静默转换单个文件（用于批量转换）
fn convert_single_file_silent(input: &Path, output_dir: &Path) -> Result<()> {
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let name = input.file_stem().unwrap_or_default();

    match input_ext.as_str() {
        "ldt" => {
            let output_path = output_dir.join(format!("{}.csv", name.to_string_lossy()));
            convert_ldt_to_csv_internal(input, &output_path)
        }
        "csv" => {
            let output_path = output_dir.join(format!("{}.LDT", name.to_string_lossy()));
            convert_csv_to_ldt_internal(input, &output_path)
        }
        _ => bail!("不支持的文件格式: {}", input_ext),
    }
}

/// LDT → CSV 转换（内部实现，不打印输出）
fn convert_ldt_to_csv_internal(input: &Path, output_path: &Path) -> Result<()> {
    // 读取 LDT
    let reader = LdtReader::open(input)
        .with_context(|| format!("无法打开 LDT 文件: {}", input.display()))?;

    let db_id = reader.db_id();
    let field_defs = reader.field_defs();
    let rows = reader.read_rows().context("读取数据行失败")?;

    // 提取数据库名称
    let db_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    // 写入 CSV
    let mut file = std::fs::File::create(output_path)
        .with_context(|| format!("无法创建 CSV 文件: {}", output_path.display()))?;
    export_to_csv(&mut file, db_id, &field_defs, &rows, db_name)
        .context("写入 CSV 文件失败")?;

    Ok(())
}

/// LDT → CSV 转换（带打印输出）
fn convert_ldt_to_csv(input: &Path, output_path: &Path) -> Result<()> {
    let start = Instant::now();

    // 读取 LDT 获取信息用于打印
    let reader = LdtReader::open(input)
        .with_context(|| format!("无法打开 LDT 文件: {}", input.display()))?;

    let db_id = reader.db_id();
    let field_defs = reader.field_defs();
    let rows = reader.read_rows().context("读取数据行失败")?;

    // 提取数据库名称
    let db_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    // 写入 CSV
    let mut file = std::fs::File::create(output_path)
        .with_context(|| format!("无法创建 CSV 文件: {}", output_path.display()))?;
    export_to_csv(&mut file, db_id, &field_defs, &rows, db_name)
        .context("写入 CSV 文件失败")?;

    let elapsed = start.elapsed().as_millis();

    // 打印信息
    print_section_header("转换", "LDT → CSV");
    println!("输入文件:    {}", input.display());
    println!("输出文件:    {}", output_path.display());
    println!("数据库 ID:   {}", db_id);
    println!("字段数量:    {}", field_defs.len());
    println!("数据行数:    {}", rows.len());
    println!();
    println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
    println!();

    Ok(())
}

/// CSV → LDT 转换（内部实现，不打印输出）
fn convert_csv_to_ldt_internal(input: &Path, output_path: &Path) -> Result<()> {
    // 读取 CSV
    let mut file = std::fs::File::open(input)
        .with_context(|| format!("无法打开 CSV 文件: {}", input.display()))?;
    let (db_id, field_defs, rows) =
        import_from_csv(&mut file).context("读取 CSV 文件失败")?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    // 写入 LDT
    let mut writer = LdtWriter::new(db_id);
    writer.set_field_defs(field_defs);
    writer.set_rows(rows);
    writer.write(output_path).context("写入 LDT 文件失败")?;

    Ok(())
}

/// CSV → LDT 转换（带打印输出）
fn convert_csv_to_ldt(input: &Path, output_path: &Path) -> Result<()> {
    let start = Instant::now();

    // 读取 CSV
    let mut file = std::fs::File::open(input)
        .with_context(|| format!("无法打开 CSV 文件: {}", input.display()))?;
    let (db_id, field_defs, rows) =
        import_from_csv(&mut file).context("读取 CSV 文件失败")?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    // 写入 LDT
    let mut writer = LdtWriter::new(db_id);
    writer.set_field_defs(field_defs.clone());
    writer.set_rows(rows.clone());
    writer.write(output_path).context("写入 LDT 文件失败")?;

    let elapsed = start.elapsed().as_millis();

    // 打印信息
    print_section_header("转换", "CSV → LDT");
    println!("输入文件:    {}", input.display());
    println!("输出文件:    {}", output_path.display());
    println!("数据库 ID:   {}", db_id);
    println!("字段数量:    {}", field_defs.len());
    println!("数据行数:    {}", rows.len());
    println!();
    println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
    println!();

    Ok(())
}

/// 统计目录中各类型文件数量
fn count_files_by_type(dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut ldt_files = Vec::new();
    let mut csv_files = Vec::new();

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("无法读取目录: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.context("读取目录条目失败")?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "ldt" => ldt_files.push(path),
            "csv" => csv_files.push(path),
            _ => {}
        }
    }

    // 按文件名排序
    ldt_files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(&b.file_name().unwrap_or_default())
    });
    csv_files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(&b.file_name().unwrap_or_default())
    });

    Ok((ldt_files, csv_files))
}
