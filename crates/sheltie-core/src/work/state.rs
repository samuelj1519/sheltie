//! Work 的运行时状态。整份 `WorkState` 按一列 JSON 持久化（存储合同 §1.2）。

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Hex;
use crate::error::{Error, Result};
use crate::flow::{EdgeKind, Graph};
use crate::ids::{AttemptId, FlowId, NodeId, WorkId, WorkName, WorkbookId};
use crate::path::AbsPath;
use crate::text::Summary;

/// UTC 时间，秒精度，形如 `2026-09-24T03:00:00Z`（存储合同 §7）。由 runtime 传入。
///
/// 只能经 `parse` 或 `from_unix_secs` 构造，读取也走 `parse`：格式固定，所以字典序就是时间序，
/// `day()` 取前 10 字节就是日期。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    /// 校验 `YYYY-MM-DDTHH:MM:SSZ`：年 0000–9999，月日按公历（含闰年），时 00–23，分秒 00–59。
    /// 不收小数秒、时区偏移与闰秒。
    pub fn parse(value: &str) -> Result<Self> {
        let bad = |reason| Error::InvalidId {
            field: "timestamp".to_string(),
            value: value.to_string(),
            reason,
        };
        let b = value.as_bytes();
        let shape_ok = b.len() == 20
            && b.iter().enumerate().all(|(i, c)| match i {
                4 | 7 => *c == b'-',
                10 => *c == b'T',
                13 | 16 => *c == b':',
                19 => *c == b'Z',
                _ => c.is_ascii_digit(),
            });
        if !shape_ok {
            return Err(bad("不是 YYYY-MM-DDTHH:MM:SSZ"));
        }
        let num =
            |r: std::ops::Range<usize>| b[r].iter().fold(0u32, |n, c| n * 10 + u32::from(c - b'0'));
        let (y, mo, d) = (num(0..4), num(5..7), num(8..10));
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let month_days = match mo {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return Err(bad("月份不在 01–12")),
        };
        if d == 0 || d > month_days {
            return Err(bad("日期不存在"));
        }
        if num(11..13) > 23 || num(14..16) > 59 || num(17..19) > 59 {
            return Err(bad("时分秒越界"));
        }
        Ok(Self(value.to_string()))
    }

    /// Unix 秒数转成本类型。纯算法（Howard Hinnant 的 civil_from_days），不碰时钟。
    /// 9999 年以后饱和到 `9999-12-31T23:59:59Z`，保证结果仍是 `parse` 接受的格式。
    pub fn from_unix_secs(secs: u64) -> Self {
        let secs = secs.min(253_402_300_799);
        let z = (secs / 86_400) as i64 + 719_468;
        let rem = secs % 86_400;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        Self(format!(
            "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
            rem / 3600,
            (rem % 3600) / 60,
            rem % 60
        ))
    }

    /// 距 1970-01-01T00:00:00Z 的秒数，1970 年以前为负。`work stats` 的耗时由它相减得出。
    /// 构造时已校验，所以不会失败。
    pub fn unix_secs(&self) -> i64 {
        // `from_unix_secs` 的逆运算（Howard Hinnant 的 days_from_civil）；格式与日历由 `parse` 保证，直接取数。
        let b = self.0.as_bytes();
        let num =
            |r: std::ops::Range<usize>| b[r].iter().fold(0i64, |n, c| n * 10 + i64::from(c - b'0'));
        let (y, mo, d) = (num(0..4), num(5..7), num(8..10));
        let (h, mi, s) = (num(11..13), num(14..16), num(17..19));
        let (y, mp) = if mo > 2 { (y, mo - 3) } else { (y - 1, mo + 9) };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        (era * 146_097 + doe - 719_468) * 86_400 + h * 3600 + mi * 60 + s
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 取日期部分 `YYYY-MM-DD`，用于 `work_id`。
    pub fn day(&self) -> &str {
        &self.0[..10]
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 读取时也校验：库里的坏时间串报错（runtime 映射为 `STORE_CORRUPT`），不带进耗时计算。
impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

/// 操作者身份。MVP 是操作系统用户名。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Principal(pub String);

/// Work 绑定的 Workbook 版本与内容摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbookRef {
    pub id: WorkbookId,
    pub version: String,
    pub digest: Sha256Hex,
}

/// 一个按字节冻结的文件引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub path: AbsPath,
    pub sha256: Sha256Hex,
    pub bytes: u64,
}

/// 节点第 `n` 次到达。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Occurrence {
    pub node: NodeId,
    pub n: u32,
}

impl std::fmt::Display for Occurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.node, self.n)
    }
}

/// Attempt 只有执行事实。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Running,
    Succeeded,
    Failed,
}

impl AttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// 一次执行尝试。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub id: AttemptId,
    pub status: AttemptStatus,
    /// 从哪个 Occurrence 经哪种边到达；入口为 `None`。重试沿用第一次的值。
    pub entered_from: Option<(Occurrence, EdgeKind)>,
    /// 开工时冻结。`None` 只出现在 `required = false` 且上游尚无产出。
    pub inputs: BTreeMap<String, Option<ArtifactRef>>,
    /// 提交时封存。`Running` 时为空。
    pub outputs: BTreeMap<String, ArtifactRef>,
    pub summary: Option<Summary>,
    pub fail_reason: Option<Summary>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
}

impl Attempt {
    pub fn occurrence(&self) -> Occurrence {
        Occurrence {
            node: self.id.node.clone(),
            n: self.id.occurrence,
        }
    }
}

/// 门槛批准记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub node: NodeId,
    pub occurrence: u32,
    pub by: Principal,
    pub at: Timestamp,
}

/// Work 为什么受阻。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockedReason {
    Gate,
    RetriesExhausted,
    NoLegalEdge,
}

impl BlockedReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gate => "gate",
            Self::RetriesExhausted => "retries_exhausted",
            Self::NoLegalEdge => "no_legal_edge",
        }
    }
}

/// Work 状态。没有 `Failed`：重试耗尽后唯一出路是取消。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    tag = "kind",
    content = "reason",
    deny_unknown_fields
)]
pub enum WorkStatus {
    Active,
    Blocked(BlockedReason),
    Succeeded,
    Cancelled,
}

impl WorkStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Cancelled)
    }

    /// 数据库 `status` 列用的字面量。
    pub fn column(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Blocked(_) => "blocked",
            Self::Succeeded => "succeeded",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(reason) => write!(f, "blocked({})", reason.as_str()),
            other => f.write_str(other.column()),
        }
    }
}

/// 一个 Work 的全部状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkState {
    pub work_id: WorkId,
    pub name: WorkName,
    pub workbook: WorkbookRef,
    pub flow: FlowId,
    /// `~/.sheltie/works/<work_id>`。所有 Attempt 目录与冻结副本都在它下面。
    pub work_dir: AbsPath,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub status: WorkStatus,
    /// 当前所在 Occurrence。终态后保留最后一个。
    pub current: Occurrence,
    pub visits: BTreeMap<NodeId, u32>,
    pub attempts: Vec<Attempt>,
    pub approvals: Vec<Approval>,
    /// 累计受阻事实（GF-29）：gate 提交成功、重试耗尽、`no_legal_edge` 发生各 +1，
    /// 由状态转换在发生时记录，取消后不减少。schema 1 的旧 state 没有该字段，
    /// 读取按「缺字段即拒绝」处理，不用默认值猜历史。
    pub blocked_count: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl WorkState {
    /// 核对持久状态内部必须相符的事实。Store 负责把字段位置映射为 STORE_CORRUPT。
    pub fn validate_persisted(&self) -> std::result::Result<(), String> {
        if self.work_id.as_str().get(15..) != Some(self.name.as_str()) {
            return Err("work_id 与 name 不一致".to_string());
        }
        if self.work_id.as_str().get(..10) != Some(self.created_at.day()) {
            return Err("work_id 日期与 created_at 不一致".to_string());
        }
        if self.current.n == 0 || self.visits.get(&self.current.node) != Some(&self.current.n) {
            return Err("current 与 visits 不一致".to_string());
        }
        if self.visits.values().any(|count| *count == 0) {
            return Err("visits 含零次到达".to_string());
        }

        let mut seen = BTreeSet::new();
        let mut running = None;
        for (index, attempt) in self.attempts.iter().enumerate() {
            let id = &attempt.id;
            if id.occurrence == 0
                || self
                    .visits
                    .get(&id.node)
                    .is_none_or(|count| id.occurrence > *count)
            {
                return Err(format!("attempts[{index}].id 与 visits 不一致"));
            }
            if !seen.insert(id) {
                return Err(format!("attempts[{index}].id 重复"));
            }
            let valid = match attempt.status {
                AttemptStatus::Running => {
                    if running.replace(attempt.occurrence()).is_some() {
                        return Err("存在多个 running Attempt".to_string());
                    }
                    attempt.ended_at.is_none()
                        && attempt.summary.is_none()
                        && attempt.fail_reason.is_none()
                        && attempt.outputs.is_empty()
                }
                AttemptStatus::Succeeded => {
                    attempt.ended_at.is_some()
                        && attempt.summary.is_some()
                        && attempt.fail_reason.is_none()
                }
                AttemptStatus::Failed => {
                    attempt.ended_at.is_some()
                        && attempt.summary.is_none()
                        && attempt.fail_reason.is_some()
                        && attempt.outputs.is_empty()
                }
            };
            if !valid {
                return Err(format!("attempts[{index}] 的 status/结果字段不一致"));
            }
        }
        if running
            .as_ref()
            .is_some_and(|occurrence| occurrence != &self.current)
        {
            return Err("running Attempt 与 current 不一致".to_string());
        }
        if running.is_some()
            && self.status != WorkStatus::Active
            && self.status != WorkStatus::Cancelled
        {
            return Err("running Attempt 与 Work status 不一致".to_string());
        }

        for (index, approval) in self.approvals.iter().enumerate() {
            if !self.attempts.iter().any(|attempt| {
                attempt.id.node == approval.node
                    && attempt.id.occurrence == approval.occurrence
                    && attempt.status == AttemptStatus::Succeeded
            }) {
                return Err(format!("approvals[{index}] 没有对应的成功 Attempt"));
            }
        }

        let latest = self
            .latest_attempt_of_current()
            .map(|attempt| attempt.status);
        let expected = match self.status {
            WorkStatus::Blocked(BlockedReason::Gate | BlockedReason::NoLegalEdge)
            | WorkStatus::Succeeded => Some(AttemptStatus::Succeeded),
            WorkStatus::Blocked(BlockedReason::RetriesExhausted) => Some(AttemptStatus::Failed),
            WorkStatus::Active | WorkStatus::Cancelled => None,
        };
        if expected.is_some_and(|status| latest != Some(status)) {
            return Err("Work status 与当前 Attempt 不一致".to_string());
        }
        if self.status == WorkStatus::Blocked(BlockedReason::Gate)
            && self.approvals.iter().any(|approval| {
                approval.node == self.current.node && approval.occurrence == self.current.n
            })
        {
            return Err("当前 gate 已批准却仍受阻".to_string());
        }
        // Each blocking event belongs to an ended Attempt or a Gate Approval.
        // These are necessary bounds, not a reconstruction of the stored total.
        let maximum = self.attempts.len().saturating_add(self.approvals.len());
        let minimum = self
            .approvals
            .len()
            .saturating_add(usize::from(matches!(self.status, WorkStatus::Blocked(_))));
        let count = u64::from(self.blocked_count);
        if count < minimum as u64 || count > maximum as u64 {
            return Err("blocked_count 与 Attempt/Approval 受阻事实不一致".to_string());
        }
        Ok(())
    }

    /// 冻结图提供门槛定义；持久状态只能引用已有批准事实，不能推断或补造批准。
    pub fn validate_gate_facts(&self, graph: &Graph) -> std::result::Result<(), String> {
        let approved = |node: &NodeId, occurrence: u32| {
            self.approvals
                .iter()
                .any(|approval| &approval.node == node && approval.occurrence == occurrence)
        };
        for approval in &self.approvals {
            if !graph.node(&approval.node).is_some_and(|node| node.gate()) {
                return Err(format!(
                    "{}#{} 的批准没有对应门槛",
                    approval.node, approval.occurrence
                ));
            }
        }
        let current = graph
            .node(&self.current.node)
            .ok_or_else(|| format!("当前Occurrence {} 不在冻结图中", self.current))?;
        if self.status == WorkStatus::Blocked(BlockedReason::Gate) && !current.gate() {
            return Err(format!("当前非门槛 {} 不能是Gate受阻", self.current));
        }
        for attempt in &self.attempts {
            let occurrence = attempt.occurrence();
            let has_left_or_can_leave = occurrence != self.current
                || matches!(
                    self.status,
                    WorkStatus::Active
                        | WorkStatus::Succeeded
                        | WorkStatus::Blocked(BlockedReason::NoLegalEdge)
                );
            if attempt.status == AttemptStatus::Succeeded
                && graph.node(&attempt.id.node).is_some_and(|node| node.gate())
                && has_left_or_can_leave
                && !approved(&occurrence.node, occurrence.n)
            {
                return Err(format!("已离开或可离开的门槛 {occurrence} 缺少批准"));
            }
        }
        Ok(())
    }

    /// 冻结副本目录 `work_dir/workbook`。
    pub fn workbook_dir(&self) -> AbsPath {
        self.work_dir.join_segment("workbook")
    }

    /// Attempt 目录：单一 `WorkLayout`（架构 §5）。
    /// `attempts/<node>/occurrence-<NNN>/attempt-<NNN>/`。
    pub fn attempt_dir(&self, id: &AttemptId) -> AbsPath {
        crate::work::layout::attempt_dir(&self.work_dir, id)
    }

    /// 状态卡路径 `work_dir/status-card.md`（当前投影）。
    pub fn status_card_path(&self) -> AbsPath {
        crate::work::layout::status_card_path(&self.work_dir)
    }

    pub fn attempt(&self, id: &AttemptId) -> Option<&Attempt> {
        self.attempts.iter().find(|a| &a.id == id)
    }

    pub fn attempt_mut(&mut self, id: &AttemptId) -> Option<&mut Attempt> {
        self.attempts.iter_mut().find(|a| &a.id == id)
    }

    /// 当前 Occurrence 的最新 Attempt。
    pub fn latest_attempt_of_current(&self) -> Option<&Attempt> {
        self.attempts
            .iter()
            .rev()
            .find(|a| a.occurrence() == self.current)
    }

    /// 某节点最近一次 `Succeeded` 的 Attempt。
    pub fn latest_succeeded_of(&self, node: &NodeId) -> Option<&Attempt> {
        self.attempts
            .iter()
            .rev()
            .find(|a| &a.id.node == node && a.status == AttemptStatus::Succeeded)
    }

    pub fn visits_of(&self, node: &NodeId) -> u32 {
        self.visits.get(node).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    // Task: T01
    #[test]
    fn timestamp_parse_accepts_utc_second_precision() {
        for ok in [
            "2026-09-24T03:00:00Z",
            "2024-02-29T23:59:59Z",
            "2000-02-29T00:00:00Z",
            "0000-01-01T00:00:00Z",
            "9999-12-31T23:59:59Z",
        ] {
            let ts = Timestamp::parse(ok).unwrap();
            assert_eq!(ts.as_str(), ok);
            assert_eq!(ts.to_string(), ok);
        }
        assert_eq!(
            Timestamp::parse("2026-09-24T03:00:00Z").unwrap().day(),
            "2026-09-24"
        );
    }

    // Task: T01
    #[test]
    fn timestamp_parse_rejects_offsets_fractions_and_impossible_dates() {
        for bad in [
            "2026-01-01T00:00:00+08:00",
            "2026-09-24T03:04:05.678Z",
            "2026-09-24T03:00:00z",
            "2026-09-24 03:00:00Z",
            "2026-09-24T03:00:00",
            "2026-9-24T03:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-00-01T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-02-29T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2026-01-00T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:60:00Z",
            "2026-12-31T23:59:60Z",
            "２026-01-01T00:00:00Z",
            "",
        ] {
            assert!(Timestamp::parse(bad).is_err(), "{bad:?} 应当报错");
        }
    }

    // Task: T01
    #[test]
    fn timestamp_deserialize_validates_and_serialize_is_plain_string() {
        let ts: Timestamp = serde_json::from_str("\"2026-09-24T03:00:00Z\"").unwrap();
        assert_eq!(
            serde_json::to_string(&ts).unwrap(),
            "\"2026-09-24T03:00:00Z\""
        );
        assert!(serde_json::from_str::<Timestamp>("\"2026-09-24T03:00:00+08:00\"").is_err());
        assert!(serde_json::from_str::<Timestamp>("\"2026-02-30T00:00:00Z\"").is_err());
    }

    // Task: T01
    #[test]
    fn timestamp_from_unix_secs_matches_known_dates() {
        // 期望值由 Python datetime 独立算出。
        for (secs, want) in [
            (0, "1970-01-01T00:00:00Z"),
            // 1970-03-01 是穷举可达定义域找到的第一个判定日：yoe 的三个世纪修正项
            // （/1460、/36524、/146096）被动任何一处，这天起算错一年。
            (5_097_600, "1970-03-01T00:00:00Z"),
            (1_790_218_800, "2026-09-24T03:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (4_107_542_399, "2100-02-28T23:59:59Z"),
            // 远年：yoe 的世纪修正项（/1460、/36524、/146096）只在这些年份区段起作用。
            (7_272_419_445, "2200-06-15T12:30:45Z"),
            (10_413_792_000, "2300-01-01T00:00:00Z"),
            // era 末日的 doe = 146096：末项修正 /146096 只在这一天取值 1。
            (13_569_465_599, "2399-12-31T23:59:59Z"),
            (13_574_563_200, "2400-02-29T00:00:00Z"),
            (16_756_761_599, "2500-12-31T23:59:59Z"),
            (64_076_576_523, "4000-07-04T01:02:03Z"),
            (253_386_360_000, "9999-06-30T12:00:00Z"),
            (253_402_300_799, "9999-12-31T23:59:59Z"),
            // 越过 9999 年饱和，不产出五位年份。
            (253_402_300_800, "9999-12-31T23:59:59Z"),
            (u64::MAX, "9999-12-31T23:59:59Z"),
        ] {
            let ts = Timestamp::from_unix_secs(secs);
            assert_eq!(ts.as_str(), want);
            assert_eq!(Timestamp::parse(want).unwrap(), ts);
        }
    }

    // Task: T10
    #[test]
    fn timestamp_unix_secs_matches_independent_calendar_math() {
        // 期望值由 Python datetime 独立算出；1970 年以前为负。
        for (s, want) in [
            ("1970-01-01T00:00:00Z", 0),
            ("2026-09-24T03:04:05Z", 1_790_219_045),
            ("2000-02-29T00:00:00Z", 951_782_400),
            ("2100-03-01T00:00:00Z", 4_107_542_400),
            ("1969-12-31T23:59:59Z", -1),
            ("1900-03-01T00:00:00Z", -2_203_891_200),
            ("0000-01-01T00:00:00Z", -62_167_219_200),
            // 远年：era/yoe 修正项只在这些区段起作用。
            ("2200-06-15T12:30:45Z", 7_272_419_445),
            ("2400-02-29T00:00:00Z", 13_574_563_200),
            ("4000-07-04T01:02:03Z", 64_076_576_523),
            ("9999-06-30T12:00:00Z", 253_386_360_000),
            ("9999-12-31T23:59:59Z", 253_402_300_799),
        ] {
            assert_eq!(Timestamp::parse(s).unwrap().unix_secs(), want, "{s}");
        }
    }

    // Task: C002-T39
    #[test]
    fn work_status_json_keeps_all_variants_and_rejects_unknown_fields_in_any_position() {
        for (valid, expected) in [
            (r#"{"kind":"active"}"#, WorkStatus::Active),
            (
                r#"{"kind":"blocked","reason":"gate"}"#,
                WorkStatus::Blocked(BlockedReason::Gate),
            ),
            (
                r#"{"kind":"blocked","reason":"retries_exhausted"}"#,
                WorkStatus::Blocked(BlockedReason::RetriesExhausted),
            ),
            (
                r#"{"kind":"blocked","reason":"no_legal_edge"}"#,
                WorkStatus::Blocked(BlockedReason::NoLegalEdge),
            ),
            (r#"{"kind":"succeeded"}"#, WorkStatus::Succeeded),
            (r#"{"kind":"cancelled"}"#, WorkStatus::Cancelled),
        ] {
            assert_eq!(serde_json::from_str::<WorkStatus>(valid).unwrap(), expected);
            assert_eq!(serde_json::to_string(&expected).unwrap(), valid);
            let before = format!(r#"{{"unexpected":true,{}"#, &valid[1..]);
            let after = format!(r#"{},"unexpected":true}}"#, &valid[..valid.len() - 1]);
            for invalid in [before, after] {
                assert!(
                    serde_json::from_str::<WorkStatus>(&invalid).is_err(),
                    "{invalid}"
                );
            }
        }
        assert_eq!(
            serde_json::from_str::<WorkStatus>(r#"{"reason":"gate","kind":"blocked"}"#).unwrap(),
            WorkStatus::Blocked(BlockedReason::Gate)
        );
        assert!(
            serde_json::from_str::<WorkStatus>(
                r#"{"kind":"blocked","unexpected":true,"reason":"gate"}"#
            )
            .is_err()
        );
    }
}
