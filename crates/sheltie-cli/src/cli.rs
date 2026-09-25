//! 命令树。这是 `specs/contracts/protocol.md` §2 的机器形式，由骨架定死，填空任务不改。

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sheltie", version, about = "给协调者 agent 用的本地工作流引擎", long_about = None)]
pub struct Cli {
    /// 输出一行 JSON 响应封装，而不是给人读的文本。
    #[arg(long, global = true)]
    pub json: bool,

    /// 管理根。默认取 SHELTIE_HOME，再默认 ~/.sheltie。
    #[arg(long, global = true, value_name = "DIR")]
    pub home: Option<String>,

    /// 写操作的幂等 id。不给时引擎生成。
    #[arg(long, global = true, value_name = "UUID")]
    pub request_id: Option<String>,

    #[command(subcommand)]
    pub group: Group,
}

#[derive(Debug, Subcommand)]
pub enum Group {
    /// 管理 sheltie 自身。
    #[command(name = "self", subcommand)]
    SelfCmd(SelfCmd),
    /// 管理已装的 Workbook。
    #[command(subcommand)]
    Workbook(WorkbookCmd),
    /// 创建与查看 Work。
    #[command(subcommand)]
    Work(WorkCmd),
    /// 开始、提交、标记失败一次尝试。
    #[command(subcommand)]
    Attempt(AttemptCmd),
    /// 门槛批准。
    #[command(subcommand)]
    Gate(GateCmd),
}

#[derive(Debug, Subcommand)]
pub enum SelfCmd {
    /// 把当前二进制装到 ~/.sheltie/bin，建管理根。
    Install {
        /// 往 shell rc 文件追加 PATH。默认只打印提示。
        #[arg(long)]
        modify_path: bool,
    },
    /// 下载新版本，校验，原子替换。
    Update {
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,
    },
    /// 换回上一版本。
    Rollback,
    /// 删 bin/；--purge 才删整个管理根。
    Uninstall {
        #[arg(long)]
        purge: bool,
        /// --purge 时跳过确认。
        #[arg(long)]
        yes: bool,
    },
    /// 版本、平台、管理根、SCHEMA_VERSION。
    Version,
}

#[derive(Debug, Subcommand)]
pub enum WorkbookCmd {
    /// 校验并复制一个 Workbook 目录到管理根。
    Add { dir: String },
    /// 列出已装 Workbook。
    List,
    /// 打印 manifest、宿主资源声明与每个 Flow 的节点、边。
    Show {
        /// `<id>` 或 `<id>@<version>`。
        spec: String,
    },
    /// 删除一个已装版本。有未结束的 Work 引用时拒绝。
    Remove {
        /// 必须是 `<id>@<version>`。
        spec: String,
    },
    /// 重算目录摘要与记录对比。
    Verify {
        /// 省略时核对全部。
        spec: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum WorkCmd {
    /// 创建 Work。
    Start(StartArgs),
    /// 列出 Work。
    List,
    /// 打印状态卡。
    Status { work: String },
    /// 打印事实视图：每个节点到达、尝试、失败几次，平均耗时，从哪进来。
    Stats { work: String },
    /// 取消。
    Cancel { work: String },
}

#[derive(Debug, Args)]
pub struct StartArgs {
    /// `<id>` 或 `<id>@<version>`。
    #[arg(long)]
    pub workbook: String,
    #[arg(long)]
    pub flow: String,
    /// 省略时取 flow id。
    #[arg(long)]
    pub name: Option<String>,
    /// `k=v`；`v` 以 `@` 开头时读文件内容。可重复。
    #[arg(long = "input", value_name = "K=V")]
    pub inputs: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum AttemptCmd {
    /// 进入节点并开始一次尝试，返回任务书。
    Begin {
        work: String,
        #[arg(long)]
        node: String,
    },
    /// 提交尝试。
    Submit {
        work: String,
        #[arg(long)]
        attempt: String,
        /// 摘要文本，或 `@file`。
        #[arg(long)]
        summary: String,
    },
    /// 标记尝试失败。
    Fail {
        work: String,
        #[arg(long)]
        attempt: String,
        #[arg(long)]
        reason: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum GateCmd {
    /// 真人批准门槛。
    Approve {
        work: String,
        #[arg(long)]
        node: String,
    },
}

/// `k=v` 解析；`v` 以 `@` 开头读文件。返回 `(key, value)`。
#[allow(unused_variables)]
pub fn parse_input_arg(arg: &str) -> Result<(String, String), String> {
    todo!("T18")
}

/// `<id>@<version>` 解析。没有 `@` 时版本为 `None`。
#[allow(unused_variables)]
pub fn parse_workbook_spec(spec: &str) -> Result<(String, Option<String>), String> {
    todo!("T17")
}

/// `--summary` 与 `--reason` 的值：以 `@` 开头读文件，否则原样。
#[allow(unused_variables)]
pub fn read_text_arg(value: &str) -> Result<String, String> {
    todo!("T19")
}
