use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use latale_tools::spf::{SpfReader, SpfRegistry, SpfWriter};
use std::path::PathBuf;

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

// 打印分隔线
fn print_separator() {
    println!("{}", "-".repeat(60));
}

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF 资源打包/解包工具", version)]
#[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示 SPF 文件信息
    Info {
        /// SPF 文件路径
        spf_file: PathBuf,
        /// 显示详细文件列表
        #[arg(short, long)]
        list: bool,
    },

    /// 验证 SPF 文件完整性
    Verify {
        /// SPF 文件路径
        spf_file: PathBuf,
    },

    /// 解包 SPF 文件到目录
    Unpack {
        /// SPF 文件路径
        spf_file: PathBuf,
        /// 输出目录（默认为当前目录）
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 文件名编码（默认从注册表获取）
        #[arg(long)]
        encoding: Option<String>,
        /// 仅模拟运行，不实际写入文件
        #[arg(long)]
        dry_run: bool,
    },

    /// 打包目录为 SPF 文件
    Pack {
        /// SPF 文件名（用于确定 FILE_ID 和源路径）
        spf_name: String,
        /// 输出文件路径（默认为当前目录下的 {name}.SPF）
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 源数据根目录（默认为当前目录）
        #[arg(long)]
        data_dir: Option<PathBuf>,
        /// 版本号（默认从注册表获取）
        #[arg(long)]
        version: Option<i32>,
        /// 文件名编码（默认从注册表获取）
        #[arg(long)]
        encoding: Option<String>,
        /// 仅模拟运行，不实际写入文件
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { spf_file, list } => {
            cmd_info(&spf_file, list)?;
        }
        Commands::Verify { spf_file } => {
            cmd_verify(&spf_file)?;
        }
        Commands::Unpack { spf_file, output, encoding, dry_run } => {
            cmd_unpack(&spf_file, output.as_deref(), encoding.as_deref(), dry_run)?;
        }
        Commands::Pack { spf_name, output, data_dir, version, encoding, dry_run } => {
            cmd_pack(&spf_name, output.as_deref(), data_dir.as_deref(), version, encoding.as_deref(), dry_run)?;
        }
    }

    Ok(())
}

fn cmd_info(spf_file: &std::path::Path, list: bool) -> Result<()> {
    let reader = SpfReader::open(spf_file)
        .with_context(|| format!("无法打开 SPF 文件: {}", spf_file.display()))?;

    let header = reader.header();
    let registry = SpfRegistry::find_by_file_id(header.file_id as u8);
    let encoding = registry.map(|r| r.encoding);

    println!();
    println!("[文件信息] {}", spf_file.display());
    print_separator();

    println!("版本号:      {}", reader.version());
    println!("文件编号:    {} (0x{:02X})", header.file_id, header.file_id);

    if let Some(reg) = registry {
        println!("注册名称:    {}", reg.name);
        println!("文件名编码:  {}", reg.encoding);
        println!("包含目录:    {}", reg.include_dirs.join(", "));
    }

    let desc = header.desc_str();
    if !desc.is_empty() {
        println!("描述:        {}", desc);
    }

    println!("文件数量:    {}", reader.file_count());
    println!("索引表大小:  {} ({} 字节)", format_size(header.header_size as usize), header.header_size);
    println!("文件总大小:  {}", format_size(reader.total_size()));

    if list {
        let finfos = reader.file_infos();
        println!();
        println!("[文件列表] 共 {} 个:", finfos.len());
        print_separator();

        for (i, finfo) in finfos.iter().enumerate() {
            println!("  [{:5}] {:<48} {:>8}  RESID=0x{:08X} (file_id={}, instance_id={})",
                i + 1,
                finfo.file_name_str_with_encoding(encoding),
                format_size(finfo.size as usize),
                finfo.res_id.0,
                finfo.res_id.file_id(),
                finfo.res_id.instance_id()
            );
        }
    }

    println!();

    Ok(())
}

fn cmd_verify(spf_file: &std::path::Path) -> Result<()> {
    println!();
    println!("[验证文件] {}", spf_file.display());
    print_separator();

    let reader = SpfReader::open(spf_file)
        .with_context(|| format!("无法打开 SPF 文件: {}", spf_file.display()))?;

    let header = reader.header();
    let registry = SpfRegistry::find_by_file_id(header.file_id as u8);

    if let Some(reg) = registry {
        println!("注册名称:    {}", reg.name);
        println!("文件名编码:  {}", reg.encoding);
    }
    println!("文件数量:    {}", reader.file_count());

    let result = reader.verify();

    match result {
        Ok(issues) => {
            if issues.is_empty() {
                println!("[通过] 文件完整无损");
            } else {
                println!("[警告] 发现 {} 个问题:", issues.len());
                for issue in &issues {
                    eprintln!("  - {}", issue);
                }
                println!("[失败] 验证未通过");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("[错误] 验证出错: {}", e);
            std::process::exit(1);
        }
    }

    println!();

    Ok(())
}

fn cmd_unpack(
    spf_file: &std::path::Path,
    output: Option<&std::path::Path>,
    encoding: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| std::path::Path::new("."));

    println!();
    println!("[解包] {}", spf_file.display());
    print_separator();

    let reader = SpfReader::open(spf_file)
        .with_context(|| format!("无法打开 SPF 文件: {}", spf_file.display()))?;

    let header = reader.header();

    // 尝试从 registry 获取默认编码
    let registry = SpfRegistry::find_by_file_id(header.file_id as u8);
    let encoding = encoding.or(registry.map(|r| r.encoding));

    println!("版本号:      {}", reader.version());
    println!("文件编号:    {}", header.file_id);
    if let Some(reg) = registry {
        println!("注册名称:    {}", reg.name);
    }
    if let Some(enc) = encoding {
        println!("文件名编码:  {}", enc);
    }
    println!("文件数量:    {}", reader.file_count());
    println!("输出目录:    {}", output_dir.display());

    if dry_run {
        println!();
        println!("[模拟运行] 将解包以下 {} 个文件:", reader.file_count());
        print_separator();

        let finfos = reader.file_infos();
        for finfo in finfos {
            println!("  - {} ({})", finfo.file_name_str_with_encoding(encoding), format_size(finfo.size as usize));
        }
        println!();
        println!("[提示] 模拟运行，未实际写入文件");
    } else {
        println!();
        println!("[执行] 正在解包...");
        println!();

        reader.unpack_with_encoding(output_dir, encoding, true)
            .context("解包失败")?;

        println!();
        println!("[完成] 共解包 {} 个文件", reader.file_count());
    }

    println!();

    Ok(())
}

fn cmd_pack(
    spf_name: &str,
    output: Option<&std::path::Path>,
    data_dir: Option<&std::path::Path>,
    version: Option<i32>,
    encoding: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let registry = SpfRegistry::find_by_name(spf_name)
        .with_context(|| format!("未知的 SPF 名称: {}", spf_name))?;

    let data_dir = data_dir.unwrap_or_else(|| std::path::Path::new("."));
    let encoding = encoding.unwrap_or(registry.encoding);

    // 输出路径：有 -o 则直接使用，否则默认为当前目录下的 {name}.SPF
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::Path::new(".").join(format!("{}.SPF", registry.name)));

    println!();
    println!("[打包] {}.SPF", registry.name);
    print_separator();
    println!("文件编号:    {} (0x{:02X})", registry.file_id, registry.file_id);
    println!("版本号:      {}", version.unwrap_or(registry.version));
    println!("文件名编码:  {}", encoding);
    println!("包含目录:    {}", registry.include_dirs.join(", "));

    let mut writer = SpfWriter::with_version(registry.file_id, version.unwrap_or(registry.version));

    for dir in registry.include_dirs {
        println!("源目录:      {}/{}", data_dir.display(), dir);
        writer.add_from_dir(data_dir, dir, Some(encoding), false)
            .context(format!("读取源文件失败: {}/{}", data_dir.display(), dir))?;
    }

    println!("文件数量:    {}", writer.file_count());
    println!("输出文件:    {}", output_path.display());

    if dry_run {
        println!();
        println!("[模拟运行] 将打包以下 {} 个文件:", writer.file_count());
        print_separator();

        for name in writer.file_names() {
            println!("  - {}", name);
        }
        println!();
        println!("[提示] 模拟运行，未实际写入文件");
    } else {
        println!();
        println!("[执行] 正在打包...");
        println!();

        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .context(format!("创建目录失败: {}", parent.display()))?;
            }
        }

        writer.write(&output_path)
            .context("写入 SPF 文件失败")?;

        println!();
        println!("[完成] {}", output_path.display());
    }

    println!();

    Ok(())
}
