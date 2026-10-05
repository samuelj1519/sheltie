//! Test support: fixed graphs, state, clock, and an in-memory filesystem.
//! Runtime and CLI tests use the `testkit` feature.
//!
//! `Fixture` bundles Work state, graph, and fake files, exposing actions with CLI names.
//! Calls the real `decide`, exercising the production path.

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
use crate::workbook::{Manifest, parse_manifest};

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

/// Example-directory text files (relative path, content), for the fake filesystem and `ResourceIndex`.
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
                "instructions/retro.md",
                include_str!("../../../workbooks/spec-dev/instructions/retro.md"),
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
                "resources/templates/lessons.md",
                include_str!("../../../workbooks/spec-dev/resources/templates/lessons.md"),
            ),
            (
                "resources/checklists/approval-rules.md",
                include_str!("../../../workbooks/spec-dev/resources/checklists/approval-rules.md"),
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
            (
                "resources/checklists/retro-rules.md",
                include_str!("../../../workbooks/spec-dev/resources/checklists/retro-rules.md"),
            ),
        ],
        other => panic!("No example named {other}"),
    }
}

/// Build a `ResourceIndex` from example files.
pub fn resources_of(name: &str) -> ResourceIndex {
    let mut idx = ResourceIndex::default();
    for (path, content) in example_files(name) {
        idx = idx.with_utf8(path, content.len() as u64);
    }
    idx
}

pub fn article_review_manifest() -> Manifest {
    parse_manifest(ARTICLE_REVIEW_MANIFEST)
        .unwrap_or_else(|e| panic!("Example manifest should be valid: {e}"))
}

pub fn article_review_resources() -> ResourceIndex {
    resources_of("article-review")
}

/// Fixed clock: the current time for every test.
pub fn now() -> Timestamp {
    Timestamp::parse("2026-09-24T03:00:00Z")
        .unwrap_or_else(|e| panic!("Fixed clock should be valid: {e}"))
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

/// Test root: core does not access the real filesystem; this is only a string prefix.
pub const TEST_ROOT: &str = "/tmp/sheltie-test";

/// proptest: `n` nodes `n0..n{n-1}`, with edges as index pairs; out-of-range indices trigger rule 2.
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
    parse_flow(&text).unwrap_or_else(|e| panic!("Generated Flow should parse: {e}"))
}

/// Complete fixture for one Work.
pub struct Fixture {
    pub graph: Graph,
    pub manifest: Manifest,
    pub instructions: BTreeMap<NodeId, String>,
    state: Option<WorkState>,
    /// Fake filesystem: absolute path to bytes.
    files: BTreeMap<AbsPath, Vec<u8>>,
    work_dir: AbsPath,
}

impl Fixture {
    /// Construct from real declarations and in-memory files, retaining parse, compile, and real decide.
    pub fn from_texts(manifest_text: &str, flow_text: &str, files: &[(&str, &str)]) -> Self {
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

    /// Construct from embedded examples without filesystem access.
    pub fn from_example(name: &str) -> Self {
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

    /// article-review with `review.max_visits = 1`.
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
            "schema = \"workbook/v1\"\nid = \"single\"\nversion = \"1.0.0\"\nname = \"Single node\"\nflows = [\"flows/default.toml\"]\n{manifest_extra}"
        );
        let flow = format!(
            "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"Only node\"\nexecutor = \"agent\"\ninstruction = {{ text = \"Do this task.\" }}\n{node_extra}\n"
        );
        let mut fx = Self::from_texts(&manifest, &flow, &[]);
        fx.start(&[]).unwrap_or_else(|e| panic!("{e}"));
        fx
    }

    /// Started single node with skill:company-api and a matching manifest declaration.
    pub fn with_requires() -> Self {
        Self::single_node(
            "[[requires]]\nkind = \"skill\"\nname = \"company-api\"\n",
            "requires = [\"skill:company-api\"]\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// Started single node with required must and optional maybe outputs.
    pub fn with_optional_output() -> Self {
        Self::single_node(
            "",
            "outputs = [{ name = \"must\", path = \"must.md\" }, { name = \"maybe\", path = \"maybe.md\", required = false }]",
        )
    }

    /// Started single gated terminal node without outgoing edges.
    pub fn single_gated_terminal() -> Self {
        Self::single_node(
            "",
            "gate = true\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// Started single node with input stats from engine.stats.
    pub fn with_engine_stats_input() -> Self {
        Self::single_node(
            "",
            "inputs = [{ name = \"stats\", from = \"engine.stats\" }]\noutputs = [{ name = \"out\", path = \"out.md\" }]",
        )
    }

    /// `first -> second` (main), `first -> side` (branch), `side -> second` (main);
    /// second requires side.out; started with first completed, so begin("second") must report INPUT_UNAVAILABLE.
    pub fn two_step_with_required_input_but_edge_before_success() -> Self {
        let manifest = "schema = \"workbook/v1\"\nid = \"diamond\"\nversion = \"1.0.0\"\nname = \"Diamond\"\nflows = [\"flows/default.toml\"]\n";
        let flow = r#"
schema = "flow/v1"
id = "default"
entry = "first"

[[nodes]]
id = "first"
title = "First"
executor = "agent"
instruction = { text = "Do the first task." }
outputs = [{ name = "out", path = "out.md" }]

[[nodes]]
id = "side"
title = "Side"
executor = "agent"
instruction = { text = "Do the side task." }
outputs = [{ name = "out", path = "out.md" }]

[[nodes]]
id = "second"
title = "Second"
executor = "agent"
instruction = { text = "Do the second task." }
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

    // ── Actions ─────────────────────────────────────────────────

    /// article-review default start input: `topic = "hello"`.
    pub fn started(self) -> Self {
        self.started_with(&[("topic", "hello")])
    }

    pub fn started_with(mut self, inputs: &[(&str, &str)]) -> Self {
        self.start(inputs).unwrap_or_else(|e| panic!("start：{e}"));
        self
    }

    pub fn state(&self) -> &WorkState {
        self.state
            .as_ref()
            .unwrap_or_else(|| panic!("Not started yet"))
    }

    fn apply(&mut self, cmd: Command) -> Result<Decision> {
        let d = decide(self.state.as_ref(), &self.graph, &cmd, &ctx())?;
        self.state = Some(d.state.clone());
        Ok(d)
    }

    fn observe(&self, path: &AbsPath) -> Option<ObservedFile> {
        self.files.get(path).map(|bytes| {
            ObservedFile::new(path.clone(), Sha256Hex::of_bytes(bytes), bytes.len() as u64)
        })
    }

    pub fn start(&mut self, inputs: &[(&str, &str)]) -> Result<Decision> {
        let mut refs = BTreeMap::new();
        for (key, value) in inputs {
            let path = crate::work::layout::start_input_path(&self.work_dir, key);
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

    /// Write all declared outputs, ten bytes each, then submit.
    pub fn submit_ok(&mut self, attempt: &str, summary: &str) -> Result<Decision> {
        let id = AttemptId::parse(attempt)?;
        let names: Vec<String> = output_paths_for(self.state(), &self.graph, &id)?
            .into_keys()
            .collect();
        let spec: Vec<(&str, u64)> = names.iter().map(|n| (n.as_str(), 10)).collect();
        self.submit_with(&id, summary, &spec)
    }

    /// Write only the specified outputs (name, byte length); treat others as missing, then submit.
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

    pub fn replace(&mut self, attempt: &str, reason: &str) -> Result<Decision> {
        let attempt = AttemptId::parse(attempt)?;
        let paths = crate::work::replacement_input_paths_for(self.state(), &self.graph, &attempt)?;
        let observed_inputs = paths
            .into_iter()
            .map(|(name, path)| (name, path.and_then(|path| self.observe(&path))))
            .collect();
        let instruction_text = self
            .instructions
            .get(&attempt.node)
            .cloned()
            .unwrap_or_default();
        self.apply(Command::ReplaceAttempt {
            attempt,
            reason: reason.to_string(),
            observed_inputs,
            instruction_text,
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

    /// Change a submitted output's bytes to simulate artifact tampering.
    pub fn tamper_output(&mut self, attempt: &str, output: &str) {
        let id = AttemptId::parse(attempt).unwrap_or_else(|e| panic!("{e}"));
        let path = self
            .state()
            .attempt(&id)
            .and_then(|a| a.outputs.get(output))
            .map(|r| r.path.clone())
            .unwrap_or_else(|| panic!("{attempt} has no output {output}"));
        self.files.insert(path, b"tampered".to_vec());
    }

    /// article-review: run until review succeeds with a negative conclusion.
    /// Complete draft if no Attempt exists; if draft already succeeded after the previous loop, begin review directly.
    pub fn run_to_review_done_not_passing(&mut self) {
        if self.state().latest_attempt_of_current().is_none() {
            let draft_n = self
                .state()
                .visits_of(&NodeId::new("draft").unwrap_or_else(|e| panic!("{e}")));
            self.begin("draft").unwrap_or_else(|e| panic!("{e}"));
            self.submit_ok(&format!("draft#{draft_n}.0"), "Draft")
                .unwrap_or_else(|e| panic!("{e}"));
        }
        self.begin("review").unwrap_or_else(|e| panic!("{e}"));
        let review_n = self.state().current.n;
        self.submit_ok(
            &format!("review#{review_n}.0"),
            "Rejected. Paragraph two lacks evidence; see review.md line 12.",
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }

    /// spec-dev: spec -> plan -> plan-review, with human decision `word`.
    pub fn run_spec_dev_to_plan_review_returning(&mut self, word: &str) {
        self.begin("spec").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("spec#1.0", "Specification ready")
            .unwrap_or_else(|e| panic!("{e}"));
        self.begin("plan").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("plan#1.0", "Plan ready")
            .unwrap_or_else(|e| panic!("{e}"));
        self.begin("plan-review").unwrap_or_else(|e| panic!("{e}"));
        self.submit_ok("plan-review#1.0", word)
            .unwrap_or_else(|e| panic!("{e}"));
    }
}
