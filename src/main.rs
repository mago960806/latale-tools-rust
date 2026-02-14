use clap::Parser;

#[derive(Parser)]
#[command(name = "latale-spf")]
#[command(about = "LaTale SPF resource pack/unpack tool")]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("latale-spf - LaTale SPF resource tool");
    Ok(())
}
