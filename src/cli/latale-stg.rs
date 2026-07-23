use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::common::{encoding_from_name, GB, KB, MB, MILLIS_PER_SECOND, SEPARATOR_WIDTH};
use latale_tools::stg::{
    StageFile, StgReader, StgWriter, DEFAULT_STG_INPUT, JSON_EXTENSION, JSON_OUTPUT_EXT,
    STG_EXTENSION, STG_OUTPUT_EXT,
};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn format_size(size: usize) -> String {
    if size >= GB as usize {
        format!("{:.2} GB", size as f64 / GB)
    } else if size >= MB as usize {
        format!("{:.2} MB", size as f64 / MB)
    } else if size >= KB as usize {
        format!("{:.2} KB", size as f64 / KB)
    } else {
        format!("{} B", size)
    }
}

fn format_duration(millis: u128) -> String {
    if millis < MILLIS_PER_SECOND {
        format!("{} ms", millis)
    } else {
        format!("{:.2} s", millis as f64 / MILLIS_PER_SECOND as f64)
    }
}

fn print_separator() {
    println!("{}", "-".repeat(SEPARATOR_WIDTH));
}

fn print_section_header<T: std::fmt::Display>(title: &str, extra: T) {
    println!();
    println!("[{}]", title);
    println!(" {}", extra);
    print_separator();
}

#[derive(Parser)]
#[command(name = "latale-stg")]
#[command(about = "LaTale STG 场景数据工具", version)]
#[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示 STG 文件信息和数据
    Info {
        /// STG 文件路径
        #[arg(default_value = DEFAULT_STG_INPUT)]
        stg_file: PathBuf,
        /// 只显示前 N 个 Stage（默认显示全部）
        #[arg(short, long)]
        stages: Option<usize>,
        /// 每个 Stage 只显示前 N 个 MapGroup（默认显示全部）
        #[arg(long)]
        groups: Option<usize>,
        /// 每个 MapGroup 只显示前 N 个 MapInfo（默认显示全部）
        #[arg(long)]
        maps: Option<usize>,
        /// 字符串编码 (GBK, BIG5, EUC-KR, SHIFT_JIS, UTF-8)
        #[arg(long, default_value = "GBK")]
        encoding: String,
    },

    /// 双向转换：STG ↔ JSON
    Convert {
        /// 输入文件（默认 cn/STAGENEW.STG）
        #[arg(default_value = DEFAULT_STG_INPUT)]
        input: PathBuf,
        /// 输出文件或目录
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 字符串编码 (GBK, BIG5, EUC-KR, SHIFT_JIS, UTF-8)
        #[arg(long, default_value = "GBK")]
        encoding: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info {
            stg_file,
            stages,
            groups,
            maps,
            encoding,
        } => cmd_info(&stg_file, stages, groups, maps, &encoding)?,
        Commands::Convert {
            input,
            output,
            encoding,
        } => cmd_convert(&input, output.as_deref(), &encoding)?,
    }

    Ok(())
}

fn cmd_info(
    stg_file: &Path,
    stage_limit: Option<usize>,
    group_limit: Option<usize>,
    map_limit: Option<usize>,
    encoding_name: &str,
) -> Result<()> {
    let encoding = encoding_from_name(encoding_name);
    let reader = StgReader::open(stg_file, encoding)
        .with_context(|| format!("无法打开 STG 文件: {}", stg_file.display()))?;
    let stage_file = reader.read().context("解析 STG 文件失败")?;

    print_section_header("文件信息", stg_file.display());
    println!("Stage 数量:   {}", stage_file.stage_count());
    println!("Group 数量:   {}", stage_file.group_count());
    println!("Map 数量:     {}", stage_file.map_count());
    println!("文件大小:     {}", format_size(reader.total_size()));
    println!("文件编码:     {}", encoding_name);

    print_stage_data(&stage_file, stage_limit, group_limit, map_limit);
    println!();

    Ok(())
}

fn cmd_convert(input: &Path, output: Option<&Path>, encoding_name: &str) -> Result<()> {
    if !input.exists() {
        bail!("输入路径不存在: {}", input.display());
    }
    if !input.is_file() {
        bail!("输入路径不是文件: {}", input.display());
    }

    let input_ext = input
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    match input_ext.as_str() {
        ext if ext == STG_EXTENSION => convert_stg_to_json(input, output, encoding_name),
        ext if ext == JSON_EXTENSION => convert_json_to_stg(input, output, encoding_name),
        _ => bail!(
            "不支持的文件格式: {}，请使用 {} 或 {} 文件",
            input_ext,
            STG_OUTPUT_EXT,
            JSON_OUTPUT_EXT
        ),
    }
}

fn convert_stg_to_json(input: &Path, output: Option<&Path>, encoding_name: &str) -> Result<()> {
    let start = Instant::now();
    let encoding = encoding_from_name(encoding_name);
    let output_path = output_path(input, output, JSON_OUTPUT_EXT);

    let reader = StgReader::open(input, encoding)
        .with_context(|| format!("无法打开 STG 文件: {}", input.display()))?;
    let stage_file = reader.read().context("解析 STG 文件失败")?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    let file = File::create(&output_path)
        .with_context(|| format!("无法创建 JSON 文件: {}", output_path.display()))?;
    serde_json::to_writer_pretty(file, &stage_file).context("写入 JSON 文件失败")?;

    let elapsed = start.elapsed().as_millis();
    print_section_header("转换", "STG → JSON");
    println!("输入文件:     {}", input.display());
    println!("输出文件:     {}", output_path.display());
    println!("Stage 数量:   {}", stage_file.stage_count());
    println!("Group 数量:   {}", stage_file.group_count());
    println!("Map 数量:     {}", stage_file.map_count());
    println!("文件编码:     {}", encoding_name);
    println!();
    println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
    println!();

    Ok(())
}

fn convert_json_to_stg(input: &Path, output: Option<&Path>, encoding_name: &str) -> Result<()> {
    let start = Instant::now();
    let encoding = encoding_from_name(encoding_name);
    let output_path = output_path(input, output, STG_OUTPUT_EXT);

    let file =
        File::open(input).with_context(|| format!("无法打开 JSON 文件: {}", input.display()))?;
    let stage_file: StageFile = serde_json::from_reader(file).context("读取 JSON 文件失败")?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
    }

    StgWriter::new(stage_file.clone(), encoding)
        .write(&output_path)
        .context("写入 STG 文件失败")?;

    let elapsed = start.elapsed().as_millis();
    print_section_header("转换", "JSON → STG");
    println!("输入文件:     {}", input.display());
    println!("输出文件:     {}", output_path.display());
    println!("Stage 数量:   {}", stage_file.stage_count());
    println!("Group 数量:   {}", stage_file.group_count());
    println!("Map 数量:     {}", stage_file.map_count());
    println!("文件编码:     {}", encoding_name);
    println!();
    println!("[完成] 转换完成，耗时 {}", format_duration(elapsed));
    println!();

    Ok(())
}

fn output_path(input: &Path, output: Option<&Path>, output_ext: &str) -> PathBuf {
    let name = input.file_stem().unwrap_or_default().to_string_lossy();
    match output {
        Some(path) if path.extension().is_none() => path.join(format!("{}{}", name, output_ext)),
        Some(path) => path.to_path_buf(),
        None => input.with_file_name(format!("{}{}", name, output_ext)),
    }
}

fn print_stage_data(
    stage_file: &StageFile,
    stage_limit: Option<usize>,
    group_limit: Option<usize>,
    map_limit: Option<usize>,
) {
    println!();
    let stage_take = stage_limit.unwrap_or_else(|| stage_file.stage_count());
    println!(
        "[Stage 数据] 显示 {} / {}:",
        stage_take.min(stage_file.stage_count()),
        stage_file.stage_count()
    );
    print_separator();

    for (stage_index, stage) in stage_file.stage_list.iter().take(stage_take).enumerate() {
        println!(
            "  Stage[{}] ID={} \"{}\"",
            stage_index,
            stage.stage_id,
            display_name(&stage.stage_name, &stage.palette_file)
        );

        let group_take = group_limit.unwrap_or(stage.group_list.len());
        for (group_index, group) in stage.group_list.iter().take(group_take).enumerate() {
            println!(
                "    ├─ Group[{}] ID={} \"{}\"",
                group_index,
                group.group_id,
                display_name(&group.group_name, &group.bg_file)
            );

            let map_take = map_limit.unwrap_or(group.map_list.len());
            for (map_index, map) in group.map_list.iter().take(map_take).enumerate() {
                println!(
                    "    │  └─ Map[{}] BGIndex={} \"{}\"",
                    map_index,
                    map.bg_index,
                    display_name(&map.map_name, &map.form_file)
                );
            }

            if group.map_list.len() > map_take {
                println!(
                    "            ... ({} more maps)",
                    group.map_list.len() - map_take
                );
            }
        }

        if stage.group_list.len() > group_take {
            println!(
                "        ... ({} more groups)",
                stage.group_list.len() - group_take
            );
        }
    }

    if stage_file.stage_count() > stage_take {
        println!(
            "  ... ({} more stages)",
            stage_file.stage_count() - stage_take
        );
    }
}

fn display_name<'a>(name: &'a str, fallback: &'a str) -> &'a str {
    let name = name.trim();
    if name.is_empty() {
        fallback
    } else {
        name
    }
}
