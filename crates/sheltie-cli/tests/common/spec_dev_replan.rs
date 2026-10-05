use super::*;

// Task: C002-T30
#[test]
fn replan_binds_the_exact_plan_and_tasks_copies_reviewed_by_the_human() {
    let env = Env::new();
    let project = Proj::init(&env, "t30-reviewed-inputs");
    let original = project.head();
    let work = start_spec_dev(&env, &project);
    let spec = env.begin(&work, "spec");
    submit_outputs(
        &env,
        &work,
        &spec,
        &[("spec", SPEC_MD)],
        "Specification complete",
    );
    let plan = env.begin(&work, "plan");
    let plan_bytes = plan_md(&original, "- Initial version\n");
    let tasks_bytes = tasks_md(&[("T01 Example", "src/example.py", "example_is_valid")]);
    submit_outputs(
        &env,
        &work,
        &plan,
        &[("plan", &plan_bytes), ("tasks", &tasks_bytes)],
        "Plan complete",
    );
    let review = env.begin(&work, "plan-review");
    assert!(review["data"]["outputs"].get("reviewed-plan").is_some());
    assert!(review["data"]["outputs"].get("reviewed-tasks").is_some());
    let decision = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "Split the tasks again.",
    )
    .replacen("Accepted", "Revise plan", 1);
    std::fs::write(output_file(&review, "decision"), &decision).unwrap();
    std::fs::write(output_file(&review, "reviewed-plan"), plan_bytes.as_bytes()).unwrap();
    let (rejected, code) = env.fail(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        review["data"]["attempt"].as_str().unwrap(),
        "--summary",
        "Reviewed tasks copy omitted",
    ]);
    assert_eq!(code, 1);
    assert_eq!(rejected["error"]["code"], "OUTPUT_MISSING");
    let submitted = submit_outputs(
        &env,
        &work,
        &review,
        &[("decision", &decision)],
        "Revise plan",
    );
    assert_eq!(
        std::fs::read(output_file(&review, "reviewed-plan")).unwrap(),
        plan_bytes.as_bytes()
    );
    assert_eq!(
        std::fs::read(output_file(&review, "reviewed-tasks")).unwrap(),
        tasks_bytes.as_bytes()
    );
    let replanner = env.follow_begin(&submitted, "plan");
    assert_eq!(
        input_path(&replanner, "previous_plan"),
        output_file(&review, "reviewed-plan")
    );
    assert_eq!(
        input_path(&replanner, "previous_tasks"),
        output_file(&review, "reviewed-tasks")
    );
    let brief = brief_text(&replanner);
    assert!(brief.contains(input_path(&replanner, "previous_plan").to_str().unwrap()));
    assert!(brief.contains(input_path(&replanner, "previous_tasks").to_str().unwrap()));
    assert!(replanner["data"]["inputs"]["previous_verification"].is_null());
}

struct ColdReplan {
    original: String,
    verified: Vec<String>,
    next_task: Option<String>,
    project: Proj,
}

const VERIFIED_HEADER: &str = "## Verified tasks\n\n| Task | Task baseline | Candidate commit | Approval source | Verification report | Raw evidence |\n| --- | --- | --- | --- | --- | --- |\n";

pub(super) fn verified_table(rows: &[String]) -> String {
    format!(
        "{VERIFIED_HEADER}{}",
        rows.iter()
            .map(|row| format!("{row}\n"))
            .collect::<String>()
    )
}

fn cold_field(text: &str, key: &str) -> Result<String, String> {
    text.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Missing field {key}"))
}

fn cold_original(text: &str) -> Result<String, String> {
    if let Ok(value) = cold_field(text, "Original baseline") {
        return Ok(value);
    }
    text.lines()
        .find(|line| line.starts_with("Original baseline ("))
        .and_then(|line| line.rsplit('`').nth(1))
        .filter(|hash| hash.len() == 40)
        .map(str::to_string)
        .ok_or_else(|| "Missing original baseline".to_string())
}

fn cold_rows(text: &str) -> Result<Vec<String>, String> {
    let section = text
        .split_once("## Verified tasks\n")
        .ok_or("Missing verified-task table")?
        .1;
    Ok(section
        .split("\n## ")
        .next()
        .unwrap()
        .lines()
        .filter(|line| {
            let Some(task) = line.split('|').nth(1).map(str::trim) else {
                return false;
            };
            task.len() == 3
                && task.starts_with('T')
                && task.as_bytes()[1..].iter().all(u8::is_ascii_digit)
        })
        .map(str::to_string)
        .collect())
}

fn cold_read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// A fresh worker starts only from CLI brief bindings, without baseline/completion-list parameters.
fn cold_replan(begun: &Value) -> Result<ColdReplan, String> {
    let inputs = begun["data"]["inputs"]
        .as_object()
        .ok_or("Missing brief input table")?;
    let path = |name: &str| -> Result<PathBuf, String> {
        inputs
            .get(name)
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| format!("Missing input {name}"))
    };
    let previous_plan = path("previous_plan")?;
    let previous_tasks = path("previous_tasks")?;
    let plan = cold_read(&previous_plan)?;
    let tasks = cold_read(&previous_tasks)?;
    let original = cold_original(&plan)?;
    if cold_original(&tasks)? != original {
        return Err("Reviewed plan/tasks disagree on original baseline".into());
    }
    let plan_rows = cold_lineage(
        &previous_plan,
        &original,
        &mut std::collections::BTreeSet::new(),
    )?;
    if cold_rows(&tasks)? != plan_rows {
        return Err("Reviewed plan/tasks disagree on verified prefix".into());
    }
    let decision_path = path("decision")?;
    let decision = cold_read(&decision_path)?;
    if cold_field(&decision, "Approved plan")? != sha256_file(&previous_plan) {
        let report = cold_read(&path("previous_verification")?)?;
        let approval = cold_read(Path::new(&cold_field(&report, "Approval source")?))?;
        if approval.lines().next() != Some("Continue")
            || cold_field(&approval, "Corrected approval")? != decision_path.to_string_lossy()
            || cold_field(&report, "Original approval source")? != decision_path.to_string_lossy()
            || decision.lines().next() != Some("Accepted")
            || cold_field(&approval, "Approved plan")? != sha256_file(&previous_plan)
            || sha256_file(Path::new(&cold_field(&report, "Verified plan")?))
                != sha256_file(&previous_plan)
        {
            return Err(
                "Reviewed plan does not match human version records and explicit correction".into(),
            );
        }
    }
    let project = Proj {
        root: PathBuf::from(cold_read(&path("project")?)?),
    };
    let verified =
        if let Some(report_path) = inputs.get("previous_verification").and_then(Value::as_str) {
            cold_lineage(
                Path::new(report_path),
                &original,
                &mut std::collections::BTreeSet::new(),
            )?
        } else {
            plan_rows.clone()
        };
    if !verified.starts_with(&plan_rows) {
        return Err("Latest verification table loses the reviewed-plan verified prefix".into());
    }
    for row in &verified {
        let columns = row
            .split('|')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if columns.len() != 6 {
            return Err("Verification row has an incomplete field count".into());
        }
        let report = cold_read(Path::new(columns[4]))?;
        let proof = cold_read(Path::new(columns[5]))?;
        let report_first = report.lines().next().unwrap_or_default();
        if !(report_first == "Accepted, all tasks complete"
            || report_first.starts_with("Accepted, next task T"))
        {
            return Err("A failed verification report cannot prove a verified task".into());
        }
        let approval = cold_read(Path::new(columns[3]))?;
        match approval.lines().next() {
            Some("Accepted") => {}
            Some("Continue") => {
                let prior_path = cold_field(&approval, "Corrected approval")?;
                let prior = cold_read(Path::new(&prior_path))?;
                if prior.lines().next() != Some("Accepted")
                    || cold_field(&report, "Original approval source")? != prior_path
                {
                    return Err(
                        "Human correction must reference the original accepted approval".into(),
                    );
                }
            }
            _ => {
                return Err("An unapproved historical version cannot prove a verified task".into());
            }
        }
        if cold_field(&report, "Approval source")? != columns[3] {
            return Err("Approval source differs from the cumulative table".into());
        }
        for stream in ["stdout", "stderr"] {
            let stream_path = cold_field(&proof, stream)?;
            std::fs::read(&stream_path).map_err(|error| format!("{stream_path}: {error}"))?;
            if sha256_file(Path::new(&stream_path))
                != cold_field(&proof, &format!("{stream}_sha256"))?
            {
                return Err("Raw-evidence output digest differs".into());
            }
        }

        if cold_original(&report)? != original
            || cold_field(&report, "Task")? != columns[0]
            || cold_field(&report, "Baseline")? != columns[1]
            || cold_field(&report, "Commit")? != columns[2]
            || cold_field(&proof, "Commit")? != columns[2]
            || cold_field(&proof, "Exit code")? != "0"
        {
            return Err("Historical verification row differs from raw evidence".into());
        }
        if project.git(&["merge-base", &original, columns[2]]) != original
            || project.git(&["merge-base", columns[1], columns[2]]) != columns[1]
            || !project
                .git(&["log", "-1", "--format=%B", columns[2]])
                .lines()
                .any(|line| line == format!("Task: {}", columns[0]))
        {
            return Err("Verified commit differs from project Git".into());
        }
        let expected_files = cold_field(&proof, "Files")?
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>();
        if project.diff_names(columns[1], columns[2]) != expected_files {
            return Err("Verified scope differs from the original file set".into());
        }
        let old_tasks = cold_read(Path::new(&cold_field(&report, "Verified tasks file")?))?;
        let heading = format!("## {} ", columns[0]);
        let card = old_tasks
            .split_once(&heading)
            .ok_or("Historical task card is missing")?
            .1
            .split("\n## ")
            .next()
            .unwrap();
        let allowed = card
            .lines()
            .find_map(|line| line.strip_prefix("- Allowed files: "))
            .ok_or("Historical task card lacks its file allowlist")?
            .split('、')
            .map(|item| item.trim().trim_matches('`'))
            .collect::<Vec<_>>();
        if expected_files
            .iter()
            .any(|file| !allowed.contains(&file.as_str()))
        {
            return Err("Historical verified scope exceeds the task-card allowlist".into());
        }
        let old_plan = cold_read(Path::new(&cold_field(&report, "Verified plan")?))?;
        let gate_command = cold_field(&proof, "Command")?;
        if !old_plan.lines().any(|line| line == gate_command) {
            return Err("Original gate command differs from the approved plan".into());
        }
        if check_approval(
            &approval,
            Path::new(&cold_field(&report, "Verified specification")?),
            Path::new(&cold_field(&report, "Verified plan")?),
        )
        .is_err()
        {
            return Err("Historical approval version differs".into());
        }
    }
    for name in ["previous_change", "previous_fix_change"] {
        let Some(record_path) = inputs.get(name).and_then(Value::as_str) else {
            continue;
        };
        let record = cold_read(Path::new(record_path))?;
        if cold_original(&record)? != original {
            return Err(format!("{name} resets the original baseline"));
        }
        let prefix = cold_lineage(
            Path::new(record_path),
            &original,
            &mut std::collections::BTreeSet::new(),
        )?;
        let source = cold_read(Path::new(&cold_field(&record, "Inheritance source")?))?;
        if cold_rows(&source)? != prefix || !verified.starts_with(&prefix) {
            return Err(format!("{name} loses or rewrites the inherited prefix"));
        }
        let candidate = cold_field(&record, "Commit")?;
        let latest = verified
            .last()
            .map(|row| row.split('|').nth(3).unwrap().trim());
        let stale = candidate != "none"
            && latest.is_some_and(|latest| {
                project.git(&["merge-base", &candidate, latest]) == candidate
            });
        if !stale && prefix != verified {
            return Err(format!(
                "{name} current unverified change loses prior completed rows"
            ));
        }
    }
    let completed = verified
        .iter()
        .map(|row| row.split('|').nth(1).unwrap().trim().to_string())
        .collect::<Vec<_>>();
    let next_task = tasks
        .lines()
        .filter_map(|line| line.strip_prefix("## T"))
        .map(|rest| format!("T{}", rest.split_whitespace().next().unwrap()))
        .find(|task| !completed.contains(task));
    Ok(ColdReplan {
        original,
        verified,
        next_task,
        project,
    })
}

fn prepare_cold_replan(env: &Env, mode: &str) -> (Value, Proj, String, String) {
    let project = Proj::init(env, &format!("t30-cold-{mode}"));
    let original = project.head();
    let work = start_spec_dev(env, &project);
    let correction = mode == "approval_correction";
    plan_and_approve_with_hash(
        env,
        &work,
        &project,
        &original,
        "Conditions: none.",
        correction.then_some("wrong-plan-hash"),
    );
    if correction {
        let scaffold = env.begin(&work, "scaffold");
        let prior = input_path(&scaffold, "decision");
        assert!(
            check_approval(
                &read(&prior),
                &input_path(&scaffold, "spec"),
                &input_path(&scaffold, "plan")
            )
            .is_err()
        );
        let stopped = submit_outputs(
            env,
            &work,
            &scaffold,
            &[("scaffold", "Blocked\nApproval digests differ\n")],
            "Approval invalidated",
        );
        let human = env.follow_begin(&stopped, "escalate");
        let decision = format!(
            "Continue\nCorrected approval: {}\nApproved specification: {}\nApproved plan: {}\nConditions: none.\n",
            input_path(&human, "approval").display(),
            sha256_file(&input_path(&human, "spec")),
            sha256_file(&input_path(&human, "plan"))
        );
        submit_outputs(
            env,
            &work,
            &human,
            &[("decision", &decision)],
            "Explicitly correct approved versions",
        );
    }
    let scaffold = scaffold_two_files(env, &work, &project);
    let implement = env.follow_begin(&scaffold, "implement");
    let (base, candidate) = commit_export_task(&project, "cold-fixture");
    let plan_source = input_path(&implement, "plan");
    let change = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_done(
            "T01",
            &base,
            &candidate,
            0,
            &["src/export.py", "tests/test_export.py"],
            ""
        ),
        plan_source.display(),
        verified_table(&[])
    );
    let change = if correction {
        format!(
            "{change}\nApproval correction source: {}\nOriginal approval source: {}\n",
            input_path(&implement, "escalation").display(),
            input_path(&implement, "decision").display()
        )
    } else {
        change
    };
    let result = submit_outputs(
        env,
        &work,
        &implement,
        &[("change", &change)],
        "Complete T01",
    );
    let verification = env.follow_begin(&result, "verify");
    if !correction {
        assert_t01_clean(&project, &base, &candidate, &verification);
    } else {
        assert_eq!(
            project.diff_names(&base, &candidate),
            ["src/export.py", "tests/test_export.py"]
        );
    }
    let approval_path = input_path(
        &verification,
        if correction { "escalation" } else { "decision" },
    );
    assert_eq!(
        check_approval(
            &read(&approval_path),
            &input_path(&verification, "spec"),
            &input_path(&verification, "plan")
        ),
        Ok(())
    );
    project.gate();
    let proof = env.dir.path().join("cold-proof-T01.txt");
    project.capture_gate(&base, &candidate, &proof);
    let row = format!(
        "| T01 | {base} | {candidate} | {} | {} | {} |",
        approval_path.display(),
        output_file(&verification, "report").display(),
        proof.display()
    );
    let report = format!(
        "{}\nCommit: {candidate}\nOriginal baseline: {original}\nCurrent change: {}\nInheritance source: {}\nVerified specification: {}\nVerified plan: {}\nVerified tasks file: {}\nApproval source: {}\n\n{}",
        verify_report(
            "Accepted, next task T02",
            "T01",
            0,
            &base,
            "- Independent scope/gate/approval checks passed"
        ),
        output_file(&implement, "change").display(),
        plan_source.display(),
        input_path(&verification, "spec").display(),
        input_path(&verification, "plan").display(),
        input_path(&verification, "tasks").display(),
        approval_path.display(),
        verified_table(std::slice::from_ref(&row))
    );
    let report = if correction {
        format!(
            "{report}\nOriginal approval source: {}\n",
            input_path(&verification, "decision").display()
        )
    } else {
        report
    };
    let result = submit_outputs(
        env,
        &work,
        &verification,
        &[("report", &report)],
        "Accepted, next task T02",
    );
    let pending = env.follow_begin(&result, "implement");
    let stuck = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_stuck("T02", "Task requires replanning and is not yet verified."),
        output_file(&verification, "report").display(),
        verified_table(&[row])
    );
    let result = submit_outputs(env, &work, &pending, &[("change", &stuck)], "Blocked T02");
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md(
                if mode == "continue" {
                    "Continue"
                } else {
                    "Revise plan"
                },
                "Preserve T01 and the original baseline; adjust only later T02.",
            ),
        )],
        "Revise plan",
    );
    let next_node = if mode == "continue" {
        "implement"
    } else {
        "plan"
    };
    let replanner = env.follow_begin(&result, next_node);
    (replanner, project, original, candidate)
}

// Negative cases test the independent verifier; cold-continuation cases below cover real CLI bindings/freezing.
fn file_replan_fixture(env: &Env, task_count: usize) -> Value {
    let project = Proj::init(env, "cold-file-project");
    let original = project.head();
    let directory = env.dir.path().join("cold-files");
    std::fs::create_dir(&directory).unwrap();
    let write = |name: &str, text: String| {
        let path = directory.join(name);
        std::fs::write(&path, text).unwrap();
        path
    };
    let spec = write("spec.md", SPEC_MD.to_string());
    let project_path = write("project.txt", project.root().display().to_string());
    let cards = [
        (
            "T01 Export",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 Encoding",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
        (
            "T03 Third item",
            "src/third.py、tests/test_third.py",
            "test_third",
        ),
        (
            "T04 Later item",
            "src/fourth.py、tests/test_fourth.py",
            "test_fourth",
        ),
    ];
    let mut rows = Vec::new();
    let mut previous_report: Option<PathBuf> = None;
    let (plan, tasks, decision) = loop {
        let index = rows.len();
        let source = previous_report
            .as_ref()
            .map_or_else(|| "none".to_string(), |path| path.display().to_string());
        let plan = write(
            &format!("plan-{index}.md"),
            format!(
                "{}\nInheritance source: {source}\n\n{}",
                plan_md(&original, ""),
                verified_table(&rows)
            ),
        );
        let tasks = write(
            &format!("tasks-{index}.md"),
            format!(
                "{}\nOriginal baseline: {original}\nInheritance source: {source}\n\n{}",
                tasks_md(&cards),
                verified_table(&rows)
            ),
        );
        let decision = write(
            &format!("decision-{index}.md"),
            approval_md(
                &sha256_file(&spec),
                &sha256_file(&plan),
                "Conditions: none.",
            ),
        );
        if index == task_count {
            break (plan, tasks, decision);
        }
        let task = format!("T{:02}", index + 1);
        let name = ["export", "encode", "third"][index];
        let base = project.head();
        std::fs::write(
            project.root().join(format!("src/{name}.py")),
            format!("def {name}():\n    return True\n"),
        )
        .unwrap();
        std::fs::write(
            project.root().join(format!("tests/test_{name}.py")),
            format!("def test_{name}():\n    assert True\n"),
        )
        .unwrap();
        let candidate = project.commit(&format!(
            "feat: {task}\n\nTask: {task}\nAgent: file-fixture"
        ));
        let proof = directory.join(format!("cold-proof-{task}.txt"));
        project.capture_gate(&base, &candidate, &proof);
        let change = write(
            &format!("change-{task}.md"),
            format!(
                "Complete\nOriginal baseline: {original}\nInheritance source: {}\nCommit: {candidate}\n\n{}",
                plan.display(),
                verified_table(&rows)
            ),
        );
        let report = directory.join(format!("report-{task}.md"));
        rows.push(format!(
            "| {task} | {base} | {candidate} | {} | {} | {} |",
            decision.display(),
            report.display(),
            proof.display()
        ));
        std::fs::write(&report, format!(
            "Accepted, all tasks complete\nTask: {task}\nBaseline: {base}\nCommit: {candidate}\nOriginal baseline: {original}\nCurrent change: {}\nInheritance source: {}\nVerified specification: {}\nVerified plan: {}\nVerified tasks file: {}\nApproval source: {}\n\n{}",
            change.display(), plan.display(), spec.display(), plan.display(), tasks.display(), decision.display(), verified_table(&rows)
        )).unwrap();
        previous_report = Some(report);
    };
    let change = write(
        "pending-change.md",
        format!(
            "Blocked\nOriginal baseline: {original}\nInheritance source: {}\nCommit: none\n\n{}",
            previous_report.as_ref().unwrap().display(),
            verified_table(&rows)
        ),
    );
    serde_json::json!({"data": {"inputs": {
        "project": project_path,
        "previous_plan": plan,
        "previous_tasks": tasks,
        "decision": decision,
        "previous_verification": previous_report,
        "previous_change": change,
    }}})
}

// Task: C002-T30
#[test]
fn fresh_replanner_recovers_verified_task_and_original_baseline_only_from_bound_files() {
    let env = Env::new();
    let (begun, project, expected_original, expected_task1) = prepare_cold_replan(&env, "legal");
    let basis = cold_replan(&begun).unwrap();
    assert_eq!(basis.original, expected_original);
    assert_eq!(basis.next_task.as_deref(), Some("T02"));
    assert_eq!(basis.project.head(), expected_task1);
    assert_eq!(basis.verified.len(), 1);
    assert!(basis.verified[0].contains(&expected_task1));
    assert!(basis.verified[0].contains("| T01 |"));
    let work = begun["data"]["inputs"]["project"].as_str().unwrap();
    assert!(Path::new(work).is_file());
    let work_id = env.ok(&["work", "list"])["data"][0]["work_id"]
        .as_str()
        .unwrap()
        .to_string();
    let plan = format!(
        "{}\nInheritance source: {}\n\n{}",
        plan_md(&basis.original, "- Rearrange T02 from bound evidence\n"),
        input_path(&begun, "previous_verification").display(),
        verified_table(&basis.verified)
    );
    let tasks = format!(
        "{}\nOriginal baseline: {}\nInheritance source: {}\n\n{}",
        tasks_md(&[
            (
                "T01 Export entry point",
                "src/export.py、tests/test_export.py",
                "test_export"
            ),
            (
                "T02 Encoding",
                "src/encode.py、tests/test_encode.py",
                "test_encode"
            )
        ]),
        basis.original,
        input_path(&begun, "previous_verification").display(),
        verified_table(&basis.verified)
    );
    let result = submit_outputs(
        &env,
        &work_id,
        &begun,
        &[("plan", &plan), ("tasks", &tasks)],
        "Replan while preserving historical facts",
    );
    let review = env.follow_begin(&result, "plan-review");
    let approval = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "Conditions: none.",
    );
    let result = submit_outputs(
        &env,
        &work_id,
        &review,
        &[("decision", &approval)],
        "Approve the replanned version",
    );
    let scaffold = env.follow_begin(&result, "scaffold");
    let result = submit_outputs(
        &env,
        &work_id,
        &scaffold,
        &[(
            "scaffold",
            &format!(
                "Complete\nCommit: {}\nRetain T01; T02 signatures and tests are unchanged\n",
                basis.project.head()
            ),
        )],
        "Preserve completed tasks",
    );
    let implement = env.follow_begin(&result, "implement");
    assert_eq!(
        cold_rows(&read(&input_path(&implement, "plan"))).unwrap(),
        basis.verified
    );
    let base = project.head();
    std::fs::write(
        project.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        project.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    project.gate();
    let candidate = project.commit("feat(encode): Encoding\n\nTask: T02\nAgent: cold-fixture");
    let change = format!(
        "{}\nOriginal baseline: {}\nInheritance source: {}\n\n{}",
        change_done(
            "T02",
            &base,
            &candidate,
            0,
            &["src/encode.py", "tests/test_encode.py"],
            ""
        ),
        basis.original,
        input_path(&implement, "plan").display(),
        verified_table(&basis.verified)
    );
    let result = submit_outputs(
        &env,
        &work_id,
        &implement,
        &[("change", &change)],
        "Complete T02",
    );
    let verification = env.follow_begin(&result, "verify");
    assert_eq!(
        project.diff_names(&base, &candidate),
        ["src/encode.py", "tests/test_encode.py"]
    );
    assert_eq!(
        check_approval(
            &read(&input_path(&verification, "decision")),
            &input_path(&verification, "spec"),
            &input_path(&verification, "plan")
        ),
        Ok(())
    );
    project.gate();
    let proof = env.dir.path().join("cold-proof-T02.txt");
    project.capture_gate(&base, &candidate, &proof);
    let mut rows = basis.verified.clone();
    rows.push(format!(
        "| T02 | {base} | {candidate} | {} | {} | {} |",
        input_path(&verification, "decision").display(),
        output_file(&verification, "report").display(),
        proof.display()
    ));
    let report = format!(
        "{}\nCommit: {candidate}\nOriginal baseline: {}\nCurrent change: {}\nInheritance source: {}\nVerified specification: {}\nVerified plan: {}\nVerified tasks file: {}\nApproval source: {}\n\n{}",
        verify_report(
            "Accepted, all tasks complete",
            "T02",
            0,
            &base,
            "- Independent verification passed"
        ),
        basis.original,
        output_file(&implement, "change").display(),
        input_path(&implement, "plan").display(),
        input_path(&verification, "spec").display(),
        input_path(&verification, "plan").display(),
        input_path(&verification, "tasks").display(),
        input_path(&verification, "decision").display(),
        verified_table(&rows)
    );
    let result = submit_outputs(
        &env,
        &work_id,
        &verification,
        &[("report", &report)],
        "Accepted, all tasks complete",
    );
    let final_review = env.follow_begin(&result, "review");
    assert_eq!(
        plan_baseline(&read(&input_path(&final_review, "plan"))),
        expected_original
    );
    assert_eq!(
        project.diff_names(&expected_original, &project.head()),
        [
            "src/encode.py",
            "src/export.py",
            "tests/test_encode.py",
            "tests/test_export.py"
        ]
    );
    assert_eq!(
        rows[0], basis.verified[0],
        "The cumulative prefix is preserved verbatim"
    );
}

// Task: C002-T30
#[test]
fn fresh_replanner_stops_on_reset_baseline_missing_fields_or_dropped_verified_task() {
    for (mutation, expected) in [
        (
            "reset_baseline",
            "previous_change resets the original baseline",
        ),
        ("missing_fields", "Missing original baseline"),
        (
            "drop_completed",
            "Record loses or rewrites the provenance-chain prefix",
        ),
        ("missing_evidence", "cold-proof-T01.txt"),
    ] {
        let env = Env::new();
        let begun = file_replan_fixture(&env, 1);
        let valid = cold_replan(&begun).unwrap();
        assert_eq!(valid.verified.len(), 1);
        let change = input_path(&begun, "previous_change");
        let text = read(&change);
        match mutation {
            "reset_baseline" => std::fs::write(
                &change,
                text.replace(&valid.original, &valid.project.head()),
            )
            .unwrap(),
            "missing_fields" => std::fs::write(
                &change,
                text.replace(&format!("Original baseline: {}\n", valid.original), ""),
            )
            .unwrap(),
            "drop_completed" => std::fs::write(
                &change,
                text.replace(&verified_table(&valid.verified), &verified_table(&[])),
            )
            .unwrap(),
            "missing_evidence" => {
                std::fs::remove_file(env.dir.path().join("cold-files/cold-proof-T01.txt")).unwrap()
            }
            _ => unreachable!(),
        }
        let error = match cold_replan(&begun) {
            Ok(_) => panic!("fresh worker must stop on {mutation} without producing a new plan"),
            Err(error) => error,
        };
        assert!(error.contains(expected), "{mutation}: {error}");
    }
}

fn cold_lineage(
    path: &Path,
    original: &str,
    active: &mut std::collections::BTreeSet<PathBuf>,
) -> Result<Vec<String>, String> {
    let identity = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !active.insert(identity.clone()) {
        return Err(format!("Inheritance cycle: {}", path.display()));
    }
    let result = (|| {
        let text = cold_read(path)?;
        if cold_original(&text)? != original {
            return Err("Provenance chain has inconsistent original baselines".into());
        }
        let rows = cold_rows(&text)?;
        let first = text.lines().next().unwrap_or_default();
        if matches!(
            first,
            "Rejected" | "Rejected, needs human" | "Accepted, all tasks complete"
        ) || first.starts_with("Accepted, next task T")
        {
            let change_path = cold_field(&text, "Current change")?;
            for key in [
                "Task",
                "Baseline",
                "Commit",
                "Verified specification",
                "Verified plan",
                "Verified tasks file",
                "Approval source",
            ] {
                cold_field(&text, key)?;
            }
            let change = cold_read(Path::new(&change_path))?;
            let prefix = cold_lineage(Path::new(&change_path), original, active)?;
            if cold_field(&text, "Inheritance source")?
                != cold_field(&change, "Inheritance source")?
            {
                return Err("verify inheritance source differs from the current change".into());
            }
            let passed = first == "Accepted, all tasks complete"
                || first.starts_with("Accepted, next task T");
            if !rows.starts_with(&prefix) || rows.len() != prefix.len() + usize::from(passed) {
                return Err(
                    "Historical verification loses or rewrites the cumulative prefix".into(),
                );
            }
            if passed {
                let last = rows
                    .last()
                    .unwrap()
                    .split('|')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .collect::<Vec<_>>();
                if last.len() != 6
                    || last[0] != cold_field(&text, "Task")?
                    || last[1] != cold_field(&text, "Baseline")?
                    || last[2] != cold_field(&text, "Commit")?
                    || Path::new(last[4]) != path
                {
                    return Err("New row differs from the accepted report".into());
                }
            }
        } else {
            let source = cold_field(&text, "Inheritance source")?;
            if source == "none" {
                if !rows.is_empty()
                    || !(text.starts_with("# Technical plan") || text.starts_with("# Tasks"))
                {
                    return Err(
                        "Only initial empty plan/tasks tables may start a provenance chain".into(),
                    );
                }
            } else if cold_lineage(Path::new(&source), original, active)? != rows {
                return Err("Record loses or rewrites the provenance-chain prefix".into());
            }
        }
        Ok(rows)
    })();
    active.remove(&identity);
    result
}

// Task: C002-T30
#[test]
fn implement_after_escalation_continue_keeps_latest_verified_prefix_even_when_plan_is_older() {
    let env = Env::new();
    let (begun, project, original, verified_commit) = prepare_cold_replan(&env, "continue");
    assert!(brief_text(&begun).contains("From: escalate"));
    let old_plan = cold_read(&input_path(&begun, "plan")).unwrap();
    assert!(cold_rows(&old_plan).unwrap().is_empty());
    let report_source = input_path(&begun, "report");
    let latest = cold_lineage(
        &report_source,
        &original,
        &mut std::collections::BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(latest.len(), 1);
    assert!(latest[0].contains(&verified_commit));
    let base = project.head();
    std::fs::write(
        project.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        project.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    project.gate();
    let candidate = project.commit("feat(encode): Continue T02\n\nTask: T02\nAgent: cold-fixture");
    let change = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_done(
            "T02",
            &base,
            &candidate,
            0,
            &["src/encode.py", "tests/test_encode.py"],
            "Continue while preserving T01; T02 candidate awaits verification."
        ),
        report_source.display(),
        verified_table(&latest)
    );
    let work_id = env.ok(&["work", "list"])["data"][0]["work_id"]
        .as_str()
        .unwrap()
        .to_string();
    let result = submit_outputs(
        &env,
        &work_id,
        &begun,
        &[("change", &change)],
        "Complete T02; preserve the prefix",
    );
    let verification = env.follow_begin(&result, "verify");
    let recorded = input_path(&verification, "change");
    assert_eq!(
        cold_lineage(&recorded, &original, &mut std::collections::BTreeSet::new()).unwrap(),
        latest
    );
    let omitted = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_stuck("T02", "Invalid empty table"),
        report_source.display(),
        verified_table(&[])
    );
    let bad = env.dir.path().join("omitted-prefix.md");
    std::fs::write(&bad, omitted).unwrap();
    assert!(
        cold_lineage(&bad, &original, &mut std::collections::BTreeSet::new())
            .unwrap_err()
            .contains("prefix")
    );
}

fn review_fixture_replan(env: &Env, work: &str, begun: &Value, extra_tasks: bool) -> Value {
    let original = cold_original(&read(&input_path(begun, "previous_plan"))).unwrap();
    let source = input_path(begun, "previous_verification");
    let rows = cold_rows(&read(&source)).unwrap();
    let plan = format!(
        "{}\nInheritance source: {}\n\n{}",
        plan_md(&original, "- Replan while preserving original facts\n"),
        source.display(),
        verified_table(&rows)
    );
    let mut cards = vec![
        (
            "T01 Export entry point",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 Encoding",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
    ];
    if extra_tasks {
        cards.extend([
            (
                "T03 Third item",
                "src/third.py、tests/test_third.py",
                "test_third",
            ),
            (
                "T04 Later item",
                "src/fourth.py、tests/test_fourth.py",
                "test_fourth",
            ),
        ]);
    }
    let tasks = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        tasks_md(&cards),
        source.display(),
        verified_table(&rows)
    );
    let result = submit_outputs(
        env,
        work,
        begun,
        &[("plan", &plan), ("tasks", &tasks)],
        "Replan",
    );
    let review = env.follow_begin(&result, "plan-review");
    let decision = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "Conditions: none.",
    );
    let result = submit_outputs(
        env,
        work,
        &review,
        &[("decision", &decision)],
        "Preserve reviewed versions",
    );
    let scaffold = env.follow_begin(&result, "scaffold");
    let root = PathBuf::from(read(&input_path(&scaffold, "project")));
    let project = Proj { root };
    if extra_tasks {
        for name in ["third", "fourth"] {
            let task = if name == "third" { "T03" } else { "T04" };
            std::fs::write(
                project.root().join(format!("src/{name}.py")),
                format!("def {name}():\n    raise NotImplementedError  # {task}\n"),
            )
            .unwrap();
            std::fs::write(project.root().join(format!("tests/test_{name}.py")), format!("import pytest\n@pytest.mark.skip(reason=\"{task}\")\ndef test_{name}():\n    assert True\n")).unwrap();
        }
        project.gate();
        project.commit("feat(scaffold): Later signatures\n\nTask: scaffold\nAgent: cold-fixture");
    }
    submit_outputs(
        env,
        work,
        &scaffold,
        &[(
            "scaffold",
            &format!(
                "Complete\nCommit: {}\nPreserve verified tasks\n",
                project.head()
            ),
        )],
        "Incremental scaffold",
    )
}

fn finish_fixture_task(env: &Env, work: &str, project: &Proj, entry: &Value, task: &str) -> Value {
    let begun = env.follow_begin(entry, "implement");
    let source = input_path(&begun, "report");
    let prefix = cold_rows(&read(&source)).unwrap();
    let original = plan_baseline(&read(&input_path(&begun, "plan")));
    let base = project.head();
    let (name, code, test) = if task == "T02" {
        (
            "encode",
            "def encode(rows):\n    return \"|\".join(rows)\n".to_string(),
            unskip(TEST_ENCODE_PY_SKIPPED),
        )
    } else {
        (
            "third",
            "def third():\n    return True\n".to_string(),
            "def test_third():\n    assert True\n".to_string(),
        )
    };
    let files = [format!("src/{name}.py"), format!("tests/test_{name}.py")];
    std::fs::write(project.root().join(&files[0]), code).unwrap();
    std::fs::write(project.root().join(&files[1]), test).unwrap();
    project.gate();
    let candidate = project.commit(&format!(
        "feat: {task}\n\nTask: {task}\nAgent: cold-fixture"
    ));
    let change_path = output_file(&begun, "change");
    let change = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_done(task, &base, &candidate, 0, &[&files[0], &files[1]], ""),
        source.display(),
        verified_table(&prefix)
    );
    let result = submit_outputs(env, work, &begun, &[("change", &change)], "Task candidate");
    let verify = env.follow_begin(&result, "verify");
    assert_eq!(project.diff_names(&base, &candidate), files);
    let proof = env.dir.path().join(format!("advanced-proof-{task}.txt"));
    project.capture_gate(&base, &candidate, &proof);
    let row = format!(
        "| {task} | {base} | {candidate} | {} | {} | {} |",
        input_path(&verify, "decision").display(),
        output_file(&verify, "report").display(),
        proof.display()
    );
    let mut rows = prefix;
    rows.push(row);
    let first = if task == "T02" {
        "Accepted, all tasks complete"
    } else {
        "Accepted, next task T04"
    };
    let report = format!(
        "{}\nCommit: {candidate}\nOriginal baseline: {original}\nCurrent change: {}\nInheritance source: {}\nVerified specification: {}\nVerified plan: {}\nVerified tasks file: {}\nApproval source: {}\n\n{}",
        verify_report(
            first,
            task,
            0,
            &base,
            "- Original runs and Git references are in the table"
        ),
        change_path.display(),
        source.display(),
        input_path(&verify, "spec").display(),
        input_path(&verify, "plan").display(),
        input_path(&verify, "tasks").display(),
        input_path(&verify, "decision").display(),
        verified_table(&rows)
    );
    submit_outputs(env, work, &verify, &[("report", &report)], first)
}

fn prepare_three_task_history(env: &Env) -> (Value, String, String) {
    let (plan2, project, original, _) = prepare_cold_replan(env, "legal");
    let work = env.ok(&["work", "list"])["data"][0]["work_id"]
        .as_str()
        .unwrap()
        .to_string();
    let scaffold2 = review_fixture_replan(env, &work, &plan2, false);
    let verified2 = finish_fixture_task(env, &work, &project, &scaffold2, "T02");
    let review = env.follow_begin(&verified2, "review");
    let report = format!(
        "Rejected\nTask: overall\nBaseline: {original}\nOriginal baseline: {original}\nCommit: {}\nRepair round: 0\nFindings: Add README\n",
        project.head()
    );
    let result = submit_outputs(
        env,
        &work,
        &review,
        &[("report", &report)],
        "Overall repairs required",
    );
    let fix = env.follow_begin(&result, "fix");
    let latest_report = input_path(&fix, "verify_report");
    let verified_rows = cold_rows(&read(&latest_report)).unwrap();
    std::fs::write(
        project.root().join("README.md"),
        "Overall repairs recorded.\n",
    )
    .unwrap();
    project.gate();
    let fixed = project.commit("fix: Overall documentation\n\nTask: review\nAgent: cold-fixture");
    let change = format!(
        "Repairs complete\nTarget: Review report\nTask: review\nBaseline: {original}\nCommit: {fixed}\nOriginal baseline: {original}\nInheritance source: {}\nRepair round: 1\n\n{}",
        latest_report.display(),
        verified_table(&verified_rows)
    );
    let result = submit_outputs(
        env,
        &work,
        &fix,
        &[("change", &change)],
        "Overall repairs complete",
    );
    // A new human request selects a legal escalation edge; old overall repairs remain committed records, not verified task rows.
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md(
                "Revise plan",
                "Add T03/T04; overall repairs are committed and all prior facts retained.",
            ),
        )],
        "Human requests a new plan",
    );
    let plan3 = env.follow_begin(&result, "plan");
    assert_eq!(cold_replan(&plan3).unwrap().verified.len(), 2);
    let scaffold3 = review_fixture_replan(env, &work, &plan3, true);
    let verified3 = finish_fixture_task(env, &work, &project, &scaffold3, "T03");
    let pending4 = env.follow_begin(&verified3, "implement");
    let source = input_path(&pending4, "report");
    let rows = cold_rows(&read(&source)).unwrap();
    let change = format!(
        "{}\nOriginal baseline: {original}\nInheritance source: {}\n\n{}",
        change_stuck("T04", "Adjust remaining tasks"),
        source.display(),
        verified_table(&rows)
    );
    let result = submit_outputs(env, &work, &pending4, &[("change", &change)], "Blocked T04");
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md(
                "Revise plan",
                "Adjust only unverified T04 and preserve earlier tasks.",
            ),
        )],
        "Revise plan",
    );
    (env.follow_begin(&result, "plan"), original, fixed)
}

// Task: C002-T30
#[test]
fn fresh_replanner_accepts_stale_whole_review_fix_after_later_task_verification() {
    let env = Env::new();
    let (begun, original, fixed) = prepare_three_task_history(&env);
    let basis = cold_replan(&begun).unwrap();
    assert_eq!(basis.original, original);
    assert_eq!(basis.next_task.as_deref(), Some("T04"));
    assert_eq!(basis.verified.len(), 3);
    let old_fix = read(&input_path(&begun, "previous_fix_change"));
    assert_eq!(cold_field(&old_fix, "Task").unwrap(), "review");
    assert_eq!(cold_field(&old_fix, "Commit").unwrap(), fixed);
    assert_eq!(cold_rows(&old_fix).unwrap().len(), 2);
    assert!(basis.verified[0].contains("| T01 |"));
    assert!(basis.verified[2].contains("| T03 |"));
}

// Task: C002-T30
#[test]
fn fresh_replanner_rejects_deep_dropped_rows_failed_reports_denied_approval_and_cycles() {
    for (fault, expected) in [
        (
            "drop_earlier",
            "Historical verification loses or rewrites the cumulative prefix",
        ),
        (
            "failed_report",
            "Historical verification loses or rewrites the cumulative prefix",
        ),
        (
            "denied_approval",
            "An unapproved historical version cannot prove a verified task",
        ),
        ("cycle", "Inheritance cycle"),
    ] {
        let env = Env::new();
        let begun = file_replan_fixture(&env, 3);
        let valid = cold_replan(&begun).unwrap();
        assert_eq!(valid.verified.len(), 3);
        let directory = env.dir.path().join("cold-files");
        match fault {
            "drop_earlier" => {
                let report = directory.join("report-T02.md");
                std::fs::write(
                    &report,
                    read(&report).replace(&format!("{}\n", valid.verified[0]), ""),
                )
                .unwrap();
            }
            "failed_report" => {
                let report = directory.join("report-T02.md");
                std::fs::write(
                    &report,
                    read(&report).replacen("Accepted, all tasks complete", "Rejected", 1),
                )
                .unwrap();
            }
            "denied_approval" => {
                let approval = directory.join("decision-1.md");
                std::fs::write(
                    &approval,
                    read(&approval).replacen("Accepted", "Revise plan", 1),
                )
                .unwrap();
            }
            "cycle" => {
                let change = directory.join("change-T03.md");
                let text = read(&change);
                let source = cold_field(&text, "Inheritance source").unwrap();
                std::fs::write(&change, text.replace(&source, change.to_str().unwrap())).unwrap();
            }
            _ => unreachable!(),
        }
        let error = match cold_replan(&begun) {
            Ok(_) => panic!("Invalid history{fault} must not produce a new plan"),
            Err(error) => error,
        };
        assert!(error.contains(expected), "{fault}: {error}");
    }
}

// Task: C002-T30
#[test]
fn fresh_replanner_accepts_explicit_human_approval_correction_and_rejects_stale_version() {
    let env = Env::new();
    let (begun, _, _, _) = prepare_cold_replan(&env, "approval_correction");
    let result = cold_replan(&begun).unwrap();
    assert_eq!(result.verified.len(), 1);
    let row = result.verified[0]
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let approval = Path::new(row[3]);
    let report = read(Path::new(row[4]));
    let old = cold_field(&report, "Original approval source").unwrap();
    assert!(
        check_approval(
            &read(Path::new(&old)),
            Path::new(&cold_field(&report, "Verified specification").unwrap()),
            Path::new(&cold_field(&report, "Verified plan").unwrap())
        )
        .is_err()
    );
    let stale = read(approval).replace(
        &cold_field(&read(approval), "Approved plan").unwrap(),
        "wrong-plan-hash",
    );
    let stale_path = env.dir.path().join("stale-correction.md");
    std::fs::write(&stale_path, stale).unwrap();
    // The engine freezes artifacts; test the independent worker version oracle without editing submitted artifacts.
    assert!(
        check_approval(
            &read(&stale_path),
            Path::new(&cold_field(&report, "Verified specification").unwrap()),
            Path::new(&cold_field(&report, "Verified plan").unwrap())
        )
        .is_err()
    );
}

// Task: C002-T30
#[test]
fn failed_verification_without_current_change_stops_lineage_reconstruction() {
    let env = Env::new();
    let begun = file_replan_fixture(&env, 1);
    let original = cold_replan(&begun).unwrap().original;
    let prior = input_path(&begun, "previous_verification");
    let prefix = cold_rows(&read(&prior)).unwrap();
    let change = input_path(&begun, "previous_change");
    let report = format!(
        "Rejected\nTask: T02\nBaseline: {original}\nCommit: none\nOriginal baseline: {original}\nCurrent change: {}\nInheritance source: {}\nVerified specification: fixture-spec\nVerified plan: fixture-plan\nVerified tasks file: fixture-tasks\nApproval source: fixture-decision\n\n{}",
        change.display(),
        prior.display(),
        verified_table(&prefix)
    );
    let good = env.dir.path().join("failed-report-with-fields.md");
    std::fs::write(&good, &report).unwrap();
    assert_eq!(
        cold_lineage(&good, &original, &mut std::collections::BTreeSet::new()).unwrap(),
        prefix
    );
    let bad = env.dir.path().join("missing-current-change.md");
    std::fs::write(
        &bad,
        report.replace(&format!("Current change: {}\n", change.display()), ""),
    )
    .unwrap();
    assert!(
        cold_lineage(&bad, &original, &mut std::collections::BTreeSet::new())
            .unwrap_err()
            .contains("Current change")
    );
}
