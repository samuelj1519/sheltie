//! Canonical Work directory layout (architecture §5 and storage contract).
//!
//! The new Store has one layout; Occurrence and number are separate dimensions. Directory labels
//! occurrence-001 / attempt-000 are zero-padded display forms. Engine files (brief.md,
//! engine/stats.json) are at the Attempt root; worker outputs are beneath outputs/.
//! The namespaces do not collide (workbook contract §3.2); C002-T07 migrates persistent callers.
//! Pure functions and independent tests fix the target shape without LegacyV1 or inferred layouts.

use crate::ids::{AttemptId, WorkId};
use crate::path::{AbsPath, RelPath};

/// Work root: works/<work_id>/.
pub fn work_dir(root: &AbsPath, work_id: &WorkId) -> AbsPath {
    root.join_segment("works").join_segment(work_id.as_str())
}

/// Status-card projection: works/<work_id>/status-card.md, rewritten after every write.
pub fn status_card_path(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("status-card.md")
}

/// Frozen copy: works/<work_id>/workbook/.
pub fn workbook_copy_dir(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("workbook")
}

/// Start-input directory: works/<work_id>/start-inputs/.
pub fn start_inputs_dir(work_dir: &AbsPath) -> AbsPath {
    work_dir.join_segment("start-inputs")
}

/// Start input: works/<work_id>/start-inputs/<key>. The
/// start.<key> ID rules guarantee one safe segment.
pub fn start_input_path(work_dir: &AbsPath, key: &str) -> AbsPath {
    start_inputs_dir(work_dir).join_segment(key)
}

/// Attempt directory: works/<work_id>/attempts/<node>/occurrence-<NNN>/attempt-<NNN>/.
/// Labels are padded to three digits; AttemptId = node#n.number retains its meaning.
pub fn attempt_dir(work_dir: &AbsPath, attempt: &AttemptId) -> AbsPath {
    work_dir
        .join_segment("attempts")
        .join_segment(attempt.node.as_str())
        .join_segment(&format!("occurrence-{:03}", attempt.occurrence))
        .join_segment(&format!("attempt-{:03}", attempt.number))
}

/// Brief: brief.md at the Attempt root, in the engine namespace.
pub fn brief_path(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir.join_segment("brief.md")
}

/// engine.stats input: engine/stats.json, separate from worker outputs.
pub fn engine_stats_path(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir
        .join_segment("engine")
        .join_segment("stats.json")
}

/// Worker output directory: outputs/.
pub fn outputs_dir(attempt_dir: &AbsPath) -> AbsPath {
    attempt_dir.join_segment("outputs")
}

/// Declared output: outputs/<declared-path>. Flow path rules
/// (portable characters and no ancestor conflicts) guarantee declared path safety.
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

    // Handwritten architecture §5 layout expectations, without calling the function under test.
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
        // Stable draft#2.1 mapping: occurrence-002 / attempt-001.
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("draft", 2, 1)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-002/attempt-001"
        );
        // First visit and execution: occurrence-001 / attempt-000.
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("review", 1, 0)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/review/occurrence-001/attempt-000"
        );
        // Pad two-digit values to at least three digits, without truncation.
        assert_eq!(
            attempt_dir(&dir(WORK), &attempt("n", 12, 3)).as_str(),
            "/h/works/2026-09-24-001-t/attempts/n/occurrence-012/attempt-003"
        );
    }

    // Task: C002-T03
    #[test]
    fn engine_files_and_worker_outputs_are_separate_namespaces() {
        let adir = attempt_dir(&dir(WORK), &attempt("draft", 1, 0));
        // Engine files remain at the Attempt root, worker outputs beneath outputs/.
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
        // Engine stats.json is beneath engine/; worker stats.json is beneath outputs/, a distinct path.
        let worker_stats = output_path(&adir, &rel("stats.json"));
        assert_ne!(worker_stats, engine_stats_path(&adir));
        assert_eq!(
            worker_stats.as_str(),
            "/h/works/2026-09-24-001-t/attempts/draft/occurrence-001/attempt-000/outputs/stats.json"
        );
        // outputs/brief.md differs from brief.md at the Attempt root; both are valid without overwriting each other.
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
