use clap::Parser;
use sheltie_core::ids::WorkId;
use sheltie_export::Error;
use sheltie_export::output::Report;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "取得明确最终成果的可编辑新副本",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(long, value_name = "BINARY", value_parser = absolute_path)]
    sheltie: PathBuf,
    #[arg(long, value_name = "DIR", value_parser = absolute_path)]
    home: PathBuf,
    #[arg(long, value_name = "FULL_WORK_ID")]
    work: String,
    #[arg(long, value_name = "PARENT", value_parser = absolute_path)]
    to: PathBuf,
    #[arg(long)]
    json: bool,
}

fn absolute_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err("必须提供绝对路径".into())
    }
}

fn write_report(report: Report, json: bool) -> i32 {
    let mut stdout = std::io::stdout().lock();
    if let Err(error) = report
        .write(json, &mut stdout)
        .and_then(|()| stdout.flush())
    {
        eprintln!("写入导出响应失败，请核查已建立的副本或暂存：{error}");
        return 1;
    }
    report.exit_code()
}

fn run() -> i32 {
    let args: Vec<_> = std::env::args_os().collect();
    let json_requested = args.iter().any(|arg| arg == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) if error.exit_code() == 0 => {
            return match error.print() {
                Ok(()) => 0,
                Err(error) => {
                    eprintln!("写入命令帮助或版本失败：{error}");
                    1
                }
            };
        }
        Err(error) => {
            return write_report(
                Report::failure(
                    None,
                    None,
                    None,
                    Error::Rejected {
                        code: "INVALID_ARGUMENT",
                        message: error.to_string(),
                    },
                ),
                json_requested,
            );
        }
    };
    let work = match WorkId::parse(&cli.work) {
        Ok(work) => work,
        Err(error) => {
            return write_report(
                Report::failure(
                    None,
                    None,
                    None,
                    Error::Rejected {
                        code: "INVALID_ARGUMENT",
                        message: error.to_string(),
                    },
                ),
                cli.json,
            );
        }
    };
    write_report(
        sheltie_export::export::copy(&cli.sheltie, &cli.home, &work, &cli.to),
        cli.json,
    )
}

fn main() {
    std::process::exit(run());
}
