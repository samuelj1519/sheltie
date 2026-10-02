//! Work 目录的单一布局函数（架构 §5、存储合同目录布局）。
//!
//! 新 Store 只有一种布局：`Occurrence` 与 `number` 是两个真实维度，目录标签
//! `occurrence-001` / `attempt-000` 只是零补齐的浏览形式；引擎文件（`brief.md`、
//! `engine/stats.json`）在 Attempt 目录根部，worker 输出在 `outputs/` 之下，
//! 两套命名空间不比较（workbook 合同 §3.2）。持久 caller 的切换归 C002-T07；
//! 本模块先以纯函数与独立测试固定目标形状，不引入 LegacyV1 或布局推断。

use crate::ids::{AttemptId, WorkId};
use crate::path::{AbsPath, RelPath};

/// Work 根目录 `works/<work_id>/`。
pub fn work_dir(root: &AbsPath, work_id: &WorkId) -> AbsPath {
    root.join_segment("works").join_segment(work_id.as_str())
}

/// 状态卡 `works/<work_id>/status-card.md`。当前状态的投影，每次写操作后重写。
pub fn status_card_path(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("status-card.md")
}

/// 冻结副本目录 `works/<work_id>/workbook/`。
pub fn workbook_copy_dir(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("workbook")
}

/// 起始输入目录 `works/<work_id>/start-inputs/`。
pub fn start_inputs_dir(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("start-inputs")
}

/// 起始输入文件 `works/<work_id>/start-inputs/<key>`。`key` 已由
/// `start.<key>` 的 ID 字符规则保证是单个安全段。
pub fn start_input_path(work_dir: &AbsPath, key: &str) -> AbsPath {
    start_inputs_dir(work_dir).join_segment(key)
}

/// Attempt 目录 `works/<work_id>/attempts/<node>/occurrence-<NNN>/attempt-<NNN>/`。
/// 标签零补齐三位；`AttemptId = node#n.number` 的含义不变。
pub fn attempt_dir(work_dir: &AbsPath, attempt: &AttemptId) -> AbsPath {
    work_dir
        .join_segment("attempts")
        .join_segment(attempt.node.as_str())
        .join_segment(&format!("occurrence-{:03}", attempt.occurrence))
        .join_segment(&format!("attempt-{:03}", attempt.number))
}

/// 任务书 `brief.md`，Attempt 目录根部（引擎命名空间）。
pub fn brief_path(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir.join_segment("brief.md")
}

/// `engine.stats` 输入文件 `engine/stats.json`，与 worker 输出分目录。
pub fn engine_stats_path(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir
        .join_segment("engine")
        .join_segment("stats.json")
}

/// worker 输出目录 `outputs/`。
pub fn outputs_dir(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir.join_segment("outputs")
}

/// 声明输出的落点 `outputs/<declared-path>`。`declared` 已由 Flow 的输出路径
/// 规则（可移植字符集、无祖先冲突）保证安全。
pub fn output_path(attempt_dir: &AbsPath, declared: &RelPath) -> AbsPath {
    outputs_dir(attempt_dir).join(declared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{NodeId, WorkId, WorkName};

    fn dir(s: &str) -> AbsPath {
        AbsPath::new(s).unwrap()
    }

    fn attempt(node: &str, occurrence: u32, number: u32) -> AttemptId {
        AttemptId::new(NodeId::new(node).unwrap(), occurrence, number)
    }

    fn rel(s: &str) -> RelPath {
        RelPath::new(s).unwrap()
    }

    // 期望值按架构 §5 的目录布局手写，不调用被测函数拼接。
    const ROOT: &str = "/h";
    const WORK: &str = "/h/works/2026-09-24-001-t";

    // Task: C002-T03
    #[test]
    fn layout_derives_work_paths_from_root_and_work_id() {
        let id = WorkId::new("2026-09-24", 1, &WorkName::normalize("t").unwrap()).unwrap();
        assert_eq!(work_dir(&dir(ROOT), &id).as_str(), WORK);
        assert_eq!(
            status_card_path(&dir(WORK)).as_str(),
            "/h/works/2026-09-24-001-t/status-card.md"
        );
        assert_eq!(
            workbook_copy_dir(&dir(WORK)).as_str(),
            "/h/works/2026-09-24-001-t/workbook"
        );
        assert_eq!(
            start_inputs_dir(&dir(WORK)).as_str(),
            "/h/works/2026-09-24-001-t/start-inputs"
        );
        assert_eq!(
            start_input_path(&dir(WORK), "topic").as_str(),
            "/h/works/2026-09-24-001-t/start-inputs/topic"
        );
    }

    // Task: C002-T03
    #[test]
    fn attempt_dir_keeps_two_dimensions_with_padded_labels() {
        // draft#2.1 的映射稳定：occurrence-002 / attempt-001。
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("draft", 2, 1)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-002/attempt-001"
        );
        // 首次到达首次执行是 occurrence-001 / attempt-000。
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("review", 1, 0)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/review/occurrence-001/attempt-000"
        );
        // 两位数仍然只补到至少三位，不截断。
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("n", 12, 3)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/n/occurrence-012/attempt-003"
        );
    }

    // Task: C002-T03
    #[test]
    fn engine_files_and_worker_outputs_are_separate_namespaces() {
        let adir = attempt_dir(&dir(WORK), &attempt("draft", 1, 0));
        // 引擎文件在 Attempt 根部，worker 输出在 outputs/ 之下。
        assert_eq!(
            brief_path(&adir).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/brief.md"
        );
        assert_eq!(
            engine_stats_path(&adir).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/engine/stats.json"
        );
        assert_eq!(
            outputs_dir(&adir).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs"
        );
    }

    // Task: C002-T03
    #[test]
    fn worker_stats_and_brief_do_not_collide_with_engine_files() {
        let adir = attempt_dir(&dir(WORK), &attempt("draft", 1, 0));
        // 引擎的 stats.json 在 engine/ 下；worker 声明 stats.json 落在 outputs/ 下，路径不同。
        let worker_stats = output_path(&adir, &rel("stats.json"));
        assert_ne!(worker_stats, engine_stats_path(&adir));
        assert_eq!(
            worker_stats.as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs/stats.json"
        );
        // outputs/brief.md 与 Attempt 根的 brief.md 不同路径，声明合法且互不覆盖。
        let worker_brief = output_path(&adir, &rel("outputs/brief.md"));
        assert_ne!(worker_brief, brief_path(&adir));
        assert_eq!(
            worker_brief.as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs/outputs/brief.md"
        );
    }

    // Task: C002-T03
    #[test]
    fn nested_declared_output_keeps_its_subdirectories() {
        let adir = attempt_dir(&dir(WORK), &attempt("draft", 1, 0));
        assert_eq!(
            output_path(&adir, &rel("notes/sub/x.md")).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs/notes/sub/x.md"
        );
    }
}
