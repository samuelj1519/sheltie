use clap::Parser;

#[derive(Parser)]
#[command(
    version,
    about = "取得明确最终成果的可编辑新副本",
    arg_required_else_help = true
)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
