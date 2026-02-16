use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::ldt::{export_to_csv, import_from_csv, LdtReader, LdtWriter};
use std::path::PathBuf;
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

    /// 双向转换：LDT ↔ CSV
    Convert {
        /// 输入文件（.LDT 或 .csv）
        input: PathBuf,
        /// 输出文件（默认根据输入文件类型自动确定）
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
            cmd_convert(&input, output.as_deref())?;
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
        println!("  [{:2}] {:<24} {:<10}", i, def.name, def.field_type.csv_type_name());
    }

    // 读取并显示部分数据
    if preview_rows > 0 && reader.row_count() > 0 {
        println!();
        println!("[数据预览] 前 {} 行:", preview_rows.min(reader.row_count() as usize));
        print_separator();

        let rows = reader.read_rows()
            .context("读取数据行失败")?;

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

fn cmd_convert(input: &std::path::Path, output: Option<&std::path::Path>) -> Result<()> {
    let input_ext = input.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match input_ext.as_str() {
        "ldt" => {
            // LDT → CSV
            let output_path = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| input.with_extension("csv"));

            print_section_header("转换", format!("LDT → CSV"));
            println!("输入文件:    {}", input.display());
            println!("输出文件:    {}", output_path.display());

            let start = Instant::now();

            // 读取 LDT
            let reader = LdtReader::open(input)
                .with_context(|| format!("无法打开 LDT 文件: {}", input.display()))?;

            let db_id = reader.db_id();
            let field_defs = reader.field_defs();
            let rows = reader.read_rows()
                .context("读取数据行失败")?;

            // 提取数据库名称
            let db_name = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");

            println!("数据库 ID:   {}", db_id);
            println!("字段数量:    {}", field_defs.len());
            println!("数据行数:    {}", rows.len());

            // 写入 CSV
            let mut file = std::fs::File::create(&output_path)
                .with_context(|| format!("无法创建 CSV 文件: {}", output_path.display()))?;
            export_to_csv(&mut file, db_id, &field_defs, &rows, db_name)
                .context("写入 CSV 文件失败")?;

            let elapsed = start.elapsed().as_millis();
            println!();
            println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
        }

        "csv" => {
            // CSV → LDT
            let output_path = output
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    let stem = input.file_stem().unwrap_or_default();
                    input.parent().unwrap_or(std::path::Path::new("."))
                        .join(format!("{}.LDT", stem.to_string_lossy()))
                });

            print_section_header("转换", format!("CSV → LDT"));
            println!("输入文件:    {}", input.display());
            println!("输出文件:    {}", output_path.display());

            let start = Instant::now();

            // 读取 CSV
            let mut file = std::fs::File::open(input)
                .with_context(|| format!("无法打开 CSV 文件: {}", input.display()))?;
            let (db_id, field_defs, rows) = import_from_csv(&mut file)
                .context("读取 CSV 文件失败")?;

            println!("数据库 ID:   {}", db_id);
            println!("字段数量:    {}", field_defs.len());
            println!("数据行数:    {}", rows.len());

            // 写入 LDT
            let mut writer = LdtWriter::new(db_id);
            writer.set_field_defs(field_defs);
            writer.set_rows(rows);
            writer.write(&output_path)
                .context("写入 LDT 文件失败")?;

            let elapsed = start.elapsed().as_millis();
            println!();
            println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
        }

        _ => {
            bail!("不支持的文件格式: {}，请使用 .LDT 或 .csv 文件", input_ext);
        }
    }

    println!();

    Ok(())
}
