//! 测试支持：固定图、固定状态、固定时钟、内存里的假文件系统。
//! 由 T01 写完，后续任务不改。runtime 与 cli 的测试通过 `testkit` feature 使用。
//!
//! `Fixture` 把「一个 Work 的状态 + 图 + 假文件」捆在一起，提供与 CLI 同名的动作。
//! 它调用真实的 `decide`，所以 T06 之前所有动作都会 `todo!()`，这是预期的。

use std::collections::BTreeMap;

use crate::digest::Sha256Hex;
use crate::error::Result;
use crate::flow::{Graph, ResourceIndex, compile, parse_flow};
use crate::ids::{AttemptId, NodeId, WorkId, WorkName};
use crate::path::AbsPath;
use crate::work::{
    ArtifactRef, Command, Context, Decision, ObservedFile, Principal, Timestamp, WorkState,
    WorkbookRef, decide, input_paths_for, output_paths_for,
};
use crate::workbook::{HostRequire, Manifest, RequireKind, parse_manifest};

pub const ARTICLE_REVIEW_MANIFEST: &str =
    include_str!("../../../examples/article-review/workbook.toml");
pub const ARTICLE_REVIEW_FLOW: &str =
    include_str!("../../../examples/article-review/flows/default.toml");
pub const TWO_STEP_MANIFEST: &str = include_str!("../../../examples/two-step/workbook.toml");
pub const TWO_STEP_FLOW: &str = include_str!("../../../examples/two-step/flows/default.toml");
pub const GATED_RELEASE_MANIFEST: &str =
    include_str!("../../../examples/gated-release/workbook.toml");
pub const GATED_RELEASE_FLOW: &str =
    include_str!("../../../examples/gated-release/flows/default.toml");
pub const SPEC_DEV_MANIFEST: &str = include_str!("../../../workbooks/spec-dev/workbook.toml");
pub const SPEC_DEV_FLOW: &str = include_str!("../../../workbooks/spec-dev/flows/default.toml");

/// 每份样例目录里的文本文件（相对路径，内容）。用于假文件系统与 `ResourceIndex`。
pub fn example_files(name: &str) -> Vec<(&'static str, &'static str)> {
    match name {
        "two-step" => vec![
            ("workbook.toml", TWO_STEP_MANIFEST),
            ("flows/default.toml", TWO_STEP_FLOW),
            (
                "instructions/outline.md",
                include_str!("../../../examples/two-step/instructions/outline.md"),
            ),
            (
                "instructions/summary.md",
                include_str!("../../../examples/two-step/instructions/summary.md"),
            ),
        ],
        "article-review" => vec![
            ("workbook.toml", ARTICLE_REVIEW_MANIFEST),
            ("flows/default.toml", ARTICLE_REVIEW_FLOW),
            (
                "instructions/draft.md",
                include_str!("../../../examples/article-review/instructions/draft.md"),
            ),
            (
                "instructions/review.md",
                include_str!("../../../examples/article-review/instructions/review.md"),
            ),
            (
                "resources/review-checklist.md",
                include_str!("../../../examples/article-review/resources/review-checklist.md"),
            ),
        ],
        "gated-release" => vec![
            ("workbook.toml", GATED_RELEASE_MANIFEST),
            ("flows/default.toml", GATED_RELEASE_FLOW),
            (
                "instructions/notes.md",
                include_str!("../../../examples/gated-release/instructions/notes.md"),
            ),
            (
                "instructions/archive.md",
                include_str!("../../../examples/gated-release/instructions/archive.md"),
            ),
        ],
        "spec-dev" => vec![
            ("workbook.toml", SPEC_DEV_MANIFEST),
            ("flows/default.toml", SPEC_DEV_FLOW),
            (
                "instructions/spec.md",
                include_str!("../../../workbooks/spec-dev/instructions/spec.md"),
            ),
            (
                "instructions/plan.md",
                include_str!("../../../workbooks/spec-dev/instructions/plan.md"),
            ),
            (
                "instructions/plan-review.md",
                include_str!("../../../workbooks/spec-dev/instructions/plan-review.md"),
            ),
            (
                "instructions/scaffold.md",
                include_str!("../../../workbooks/spec-dev/instructions/scaffold.md"),
            ),
            (
                "instructions/implement.md",
                include_str!("../../../workbooks/spec-dev/instructions/implement.md"),
            ),
            (
                "instructions/verify.md",
                include_str!("../../../workbooks/spec-dev/instructions/verify.md"),
            ),
            (
                "instructions/fix.md",
                include_str!("../../../workbooks/spec-dev/instructions/fix.md"),
            ),
            (
                "instructions/escalate.md",
                include_str!("../../../workbooks/spec-dev/instructions/escalate.md"),
            ),
            (
                "instructions/review.md",
                include_str!("../../../workbooks/spec-dev/instructions/review.md"),
            ),
            (
                "instructions/deliver.md",
                include_str!("../../../workbooks/spec-dev/instructions/deliver.md"),
            ),
            (
                "resources/templates/spec.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/spec.md"),
            ),
            (
                "resources/templates/plan.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/plan.md"),
            ),
            (
                "resources/templates/tasks.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/tasks.md"),
            ),
            (
                "resources/templates/report.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/report.md"),
            ),
            (
                "resources/templates/scaffold.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/scaffold.md"),
            ),
            (
                "resources/checklists/spec-checklist.md",
                include_str!("../../../workbooks/spec-dev/resources/checklists/spec-checklist.md"),
            ),
            (
                "resources/checklists/task-rules.md",
                include_str!("../../../workbooks/spec-dev/resources/checklists/task-rules.md"),
            ),
            (
                "resources/checklists/review-checklist.md",
                include_str!(
                    "../../../workbooks/spec-dev/resources/checklists/review-checklist.md"
                ),
            ),
            (
                "resources/checklists/scaffold-rules.md",
                include_str!("../../../workbooks/spec-dev/resources/checklists/scaffold-rules.md"),
            ),
            (
                "resources/checklists/implementer-rules.md",
                include_str!(
                    "../../../workbooks/spec-dev/resources/checklists/implementer-rules.md"
                ),
            ),
        ],
        other => panic!("没有叫 {other} 的样例"),
    }
}

/// 用样例文件建 `ResourceIndex`。
pub fn resources_of(name: &str) -> ResourceIndex {
    let mut idx = ResourceIndex::default();
    for (path, content) in example_files(name) {
        idx = idx.with_utf8(path, content.len() as u64);
    }
    idx
}

pub fn article_review_manifest() -> Manifest {
    parse_manifest(ARTICLE_REVIEW_MANIFEST).unwrap_or_else(|e| panic!("样例 manifest 应合法：{e}"))
}

pub fn article_review_resources() -> ResourceIndex {
    resources_of("article-review")
}

/// 固定时钟：所有测试里的「现在」。
pub fn now() -> Timestamp {
    Timestamp("2026-09-24T03:00:00Z".to_string())
}

pub fn principal() -> Principal {
    Principal("tester".to_string())
}

pub fn ctx() -> Context {
    Context {
        now: now(),
        principal: principal(),
    }
}

pub fn occ(node: &str, n: u32) -> crate::work::Occurrence {
    crate::work::Occurrence {
        node: NodeId::new(node).unwrap_or_else(|e| panic!("{e}")),
        n,
    }
}

/// 测试根目录。core 不碰真实文件系统，这只是个字符串前缀。
pub const TEST_ROOT: &str = "/tmp/sheltie-test";

/// proptest 用：`n` 个节点 `n0..n{n-1}`，边按下标对给出（越界下标指向不存在的节点，用于触发规则 2）。
pub fn random_flow(n: usize, edges: &[(usize, usize)]) -> crate::flow::FlowDef {
    let mut text = String::from("schema = \"flow/v1\"\nid = \"rand\"\nentry = \"n0\"\n");
    for i in 0..n {
        text.push_str(&format!(
            "[[nodes]]\nid = \"n{i}\"\ntitle = \"N{i}\"\nexecutor = \"agent\"\ninstruction = {{ text = \"do {i}\" }}\n"
        ));
    }
    for (from, to) in edges {
        text.push_str(&format!(
            "[[edges]]\nfrom = \"n{from}\"\nto = \"n{to}\"\nkind = \"main\"\n"
        ));
    }
    parse_flow(&text).unwrap_or_else(|e| panic!("随机 Flow 应能解析：{e}"))
}

/// 一个 Work 的完整测试夹具。
pub struct Fixture {
    pub graph: Graph,
    pub manifest: Manifest,
    pub instructions: BTreeMap<NodeId, String>,
    state: Option<WorkState>,
    /// 假文件系统：绝对路径 → 字节。
    files: BTreeMap<AbsPath, Vec<u8>>,
    work_dir: AbsPath,
}

impl Fixture {
    fn from_texts(manifest_text: &str, flow_text: &str, files: &[(&str, &str)]) -> Self {
        let manifest = parse_manifest(manifest_text).unwrap_or_else(|e| panic!("manifest：{e}"));
        let def = parse_flow(flow_text).unwrap_or_else(|e| panic!("flow：{e}"));
        let mut res = ResourceIndex::default();
        for (p, c) in files {
            res = res.with_utf8(p, c.len() as u64);
        }
        let graph = compile(&def, &manifest, &res).unwrap_or_else(|e| panic!("compile：{e}"));
        let work_dir = AbsPath::new(format!("{TEST_ROOT}/works/2026-09-24-001-t"))
            .unwrap_or_else(|e| panic!("{e}"));
        let mut fs = BTreeMap::new();
        let wb_dir = work_dir.join_segment("workbook");
        for (p, c) in files {
            let rel = crate::path::RelPath::new(*p).unwrap_or_else(|e| panic!("{e}"));
            fs.insert(wb_dir.join(&rel), c.as_bytes().to_vec());
        }
        let mut instructions = BTreeMap::new();
        for node in graph.nodes() {
            let text = match &node.instruction {
                crate::flow::Instruction::Text(t) => t.clone(),
                crate::flow::Instruction::File(rel) => files
                    .iter()
                    .find(|(p, _)| *p == rel.as_str())
                    .map(|(_, c)| (*c).to_string())
                    .unwrap_or_default(),
            };
            instructions.insert(node.id.clone(), text);
        }
        Self {
            graph,
            manifest,
            instructions,
            state: None,
            files: fs,
            work_dir,
        }
    }

    fn from_example(name: &str) -> Self {
        let files = example_files(name);
        let manifest = files
            .iter()
            .find(|(p, _)| *p == "workbook.toml")
            .map(|(_, c)| *c)
            .unwrap_or_default();
        let flow = files
            .iter()
            .find(|(p, _)| *p == "flows/default.toml")
            .map(|(_, c)| *c)
            .unwrap_or_default();
        Self::from_texts(manifest, flow, &files)
    }

    pub fn article_review() -> Self {
        Self::from_example("article-review")
    }

    pub fn two_step() -> Self {
        Self::from_example("two-step")
    }

    pub fn gated_release() -> Self {
        Self::from_example("gated-release")
    }

    pub fn spec_dev() -> Self {
        Self::from_example("spec-dev")
    }

    /// article-review，但 `review.max_visits = 1`。
    pub fn article_review_with_review_max_visits_1() -> Self {
        let files = example_files("article-review");
        let flow = ARTICLE_REVIEW_FLOW.replacen(
            "max_visits = 3\n\n[[nodes]]\nid = \"publish\"",
            "max_visits = 1\n\n[[nodes]]\nid = \"publish\"",
            1,
        );
        Self::from_texts(ARTICLE_REVIEW_MANIFEST, &flow, &files)
    }

    fn single_node(manifest_extra: &str, node_extra: &str) -> Self {
        let manifest = format!(
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"单节点\"\nflows = [\"flows/default.toml\"]\n{manifest_extra}"
        );
        let flow = format!(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = {{ text = \"做这一件事。\" }}\n{node_extra}\n"
        );
        let mut fx = Self::from_texts(&manifest, &flow, &[]);
        fx.start(&[]).unwrap_or_else(|e| panic!("{e}"));
        fx
    }

    /// 单节点，声明 `requires = ["skill:company-api"]`，manifest 有对应声明。已 start。
    pub fn with_requires() -> Self {
        Self::single_node(
            "[[requires]]\nkind = \"skill\"\nname = \"company-api\"\n",
            "requires = [\"skill:company-api\"]\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// 单节点，输出 `must`（必需）与 `maybe`（可选）。已 start。
    pub fn with_optional_output() -> Self {
        Self::single_node(
            "",
            "outputs = [{ name = \"must\", path = \"must.md\" }, { name = \"maybe\", path = \"maybe.md\", required = false }]",
        )
    }

    /// 单节点、`gate = true`、无出边。已 start。
    pub fn single_gated_terminal() -> Self {
        Self::single_node(
            "",
            "gate = true\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// 单节点，输入 `stats` 来自 `engine.stats`。已 start。
    pub fn with_engine_stats_input() -> Self {
        Self::single_node(
            "",
            "inputs = [{ name = \"stats\", from = \"engine.stats\" }]\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// `first -> second`（main）、`first -> side`（branch）、`side -> second`（main）；
    /// `second` 必需输入 `side.out`。已 start 并完成 `first`，此时 `begin("second")` 应报 `INPUT_UNAVAILABLE`。
    pub fn two_step_with_required_input_but_edge_before_success() -> Self {
        let manifest = "schema = \"workbook/v1\"\nid = \"diamond\"\nversion = \"1.0.0\"\nname = \"菱形\"\nflows = [\"flows/default.toml\"]\n";
        let flow = r#"
schema = "flow/v1"
id = "default"
entry = "first"

[[nodes]]
id = "first"
title = "一"
executor = "agent"
instruction = { text = "做一。" }
outputs = [{ name = "out", path = "out.md" }]

[[nodes]]
id = "side"
title = "旁"
executor = "agent"
instruction = { text = "做旁。" }
outputs = [{ name = "out", path = "out.md" }]

[[nodes]]
id = "second"
title = "二"
executor = "agent"
instruction = { text = "做二。" }
inputs = [{ name = "side_out", from = "side.out" }]
outputs = [{ name = "out", path = "out.md" }]

[[edges]]
from = "first"
to = "second"
kind = "main"

[[edges]]
from = "first"
to = "side"
kind = "branch"

[[edges]]
from = "side"
to = "second"
kind = "main"
"#;
        let mut fx = Self::from_texts(manifest, flow, &[]);
        fx.start(&[]).unwrap_or_else(|e| panic!("{e}"));
        fx.begin("first").unwrap_or_else(|e| panic!("{e}"));
        fx.submit_ok("first#1.0", "ok")
            .unwrap_or_else(|e| panic!("{e}"));
        fx
    }

    // ── 动作 ─────────────────────────────────────────────────

    /// article-review 默认起始输入 `topic = "hello"`。
    pub fn started(self) -> Self {
        self.started_with(&[("topic", "hello")])
    }

    pub fn started_with(mut self, inputs: &[(&str, &str)]) -> Self {
        self.start(inputs).unwrap_or_else(|e| panic!("start：{e}"));
        self
    }

    pub fn state(&self) -> &WorkState {
        self.state.as_ref().unwrap_or_else(|| panic!("还没 start"))
    }

    fn apply(&mut self, cmd: Command) -> Result<Decision> {
        let d = decide(self.state.as_ref(), &self.graph, &cmd, &ctx())?;
        self.state = Some(d.state.clone());
        Ok(d)
    }

    fn observe(&self, path: &AbsPath) -> Option<ObservedFile> {
        self.files.get(path).map(|bytes| {
            ObservedFile::for_test(path.clone(), Sha256Hex::of_bytes(bytes), bytes.len() as u64)
        })
    }

    pub fn start(&mut self, inputs: &[(&str, &str)]) -> Result<Decision> {
        let mut refs = BTreeMap::new();
        for (key, value) in inputs {
            let path = self.work_dir.join_segment("inputs").join_segment(key);
            self.files.insert(path.clone(), value.as_bytes().to_vec());
            refs.insert(
                (*key).to_string(),
                ArtifactRef {
                    path,
                    sha256: Sha256Hex::of_bytes(value.as_bytes()),
                    bytes: value.len() as u64,
                },
            );
        }
        let digest = Sha256Hex::of_bytes(b"fixture-workbook");
        let cmd = Command::Start {
            work_id: WorkId::new("2026-09-24", 1, &WorkName::normalize("t")?)?,
            name: WorkName::normalize("t")?,
            workbook: WorkbookRef {
                id: self.manifest.id.clone(),
                version: self.manifest.version.clone(),
                digest,
            },
            flow: crate::ids::FlowId::new("default")?,
            work_dir: self.work_dir.clone(),
            inputs: refs,
        };
        self.apply(cmd)
    }

    pub fn begin(&mut self, node: &str) -> Result<Decision> {
        let node = NodeId::new(node)?;
        let paths = input_paths_for(self.state(), &self.graph, &node)?;
        let observed = paths
            .into_iter()
            .map(|(name, p)| (name, p.and_then(|p| self.observe(&p))))
            .collect();
        let instruction_text = self.instructions.get(&node).cloned().unwrap_or_default();
        self.apply(Command::BeginAttempt {
            node,
            observed_inputs: observed,
            instruction_text,
        })
    }

    /// 写出全部声明输出（每个 10 字节），然后提交。
    pub fn submit_ok(&mut self, attempt: &str, summary: &str) -> Result<Decision> {
        let id = AttemptId::parse(attempt)?;
        let names: Vec<String> = output_paths_for(self.state(), &self.graph, &id)?
            .into_keys()
            .collect();
        let spec: Vec<(&str, u64)> = names.iter().map(|n| (n.as_str(), 10)).collect();
        self.submit_with(&id, summary, &spec)
    }

    /// 只写出给定的输出（名字，字节数），其余视为缺失，然后提交。
    pub fn submit_with(
        &mut self,
        attempt: &AttemptId,
        summary: &str,
        outputs: &[(&str, u64)],
    ) -> Result<Decision> {
        let paths = output_paths_for(self.state(), &self.graph, attempt)?;
        let mut observed = BTreeMap::new();
        for (name, path) in paths {
            match outputs.iter().find(|(n, _)| *n == name) {
                Some((_, size)) => {
                    let bytes = vec![b'x'; *size as usize];
                    self.files.insert(path.clone(), bytes);
                    observed.insert(name, self.observe(&path));
                }
                None => {
                    observed.insert(name, None);
                }
            }
        }
        self.apply(Command::SubmitAttempt {
            attempt: attempt.clone(),
            summary: summary.to_string(),
            observed_outputs: observed,
        })
    }

    pub fn fail(&mut self, attempt: &str, reason: &str) -> Result<Decision> {
        self.apply(Command::FailAttempt {
            attempt: AttemptId::parse(attempt)?,
            reason: reason.to_string(),
        })
    }

    pub fn approve(&mut self, node: &str) -> Result<Decision> {
        self.apply(Command::ApproveGate {
            node: NodeId::new(node)?,
        })
    }

    pub fn cancel(&mut self) -> Result<Decision> {
        self.apply(Command::Cancel)
    }

    /// 改掉某个已提交输出的字节，模拟有人动了产物。
    pub fn tamper_output(&mut self, attempt: &str, output: &str) {
        let id = AttemptId::parse(attempt).unwrap_or_else(|e| panic!("{e}"));
        let path = self
            .state()
            .attempt(&id)
            .and_then(|a| a.outputs.get(output))
            .map(|r| r.path.clone())
            .unwrap_or_else(|| panic!("{attempt} 没有输出 {output}"));
        self.files.insert(path, b"tampered".to_vec());
    }

    /// article-review：从「current 是 draft 且无 Attempt」跑到 review 成功、结论不通过。
    pub fn run_to_review_done_not_passing(&mut self) {
        let draft_n = self
            .state()
            .visits_of(&NodeId::new("draft").unwrap_or_else(|e| panic!("{e}")));
        self.begin("draft").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok(&format!("draft#{draft_n}.0"), "初稿")
            .unwrap_or_else(|e| panic!("{e}"));
        self.begin("review").unwrap_or_else(|e| panic!("{e}"));
        let review_n = self.state().current.n;
        self.submit_ok(
            &format!("review#{review_n}.0"),
            "不通过。第二段论据不足，见 review.md 第 12 行。",
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }

    /// spec-dev：spec → plan → plan-review，人审结论为 `word`。
    pub fn run_spec_dev_to_plan_review_returning(&mut self, word: &str) {
        self.begin("spec").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("spec#1.0", "规格好了")
            .unwrap_or_else(|e| panic!("{e}"));
        self.begin("plan").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("plan#1.0", "方案好了")
            .unwrap_or_else(|e| panic!("{e}"));
        self.begin("plan-review").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("plan-review#1.0", word)
            .unwrap_or_else(|e| panic!("{e}"));
    }
}

/// 给 `with_requires` 之类的场景补一条 manifest 声明。
pub fn require(kind: RequireKind, name: &str) -> HostRequire {
    HostRequire {
        kind,
        name: name.to_string(),
        version: None,
        digest: None,
        source: None,
    }
}
