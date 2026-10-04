use super::*;

// Task: C002-T30
#[test]
fn replan_binds_the_exact_plan_and_tasks_copies_reviewed_by_the_human() {
    let env = Env::new();
    let project = Proj::init(&env, "t30-reviewed-inputs");
    let original = project.head();
    let work = start_spec_dev(&env, &project);
    let spec = env.begin(&work, "spec");
    submit_outputs(&env, &work, &spec, &[("spec", SPEC_MD)], "规格完成");
    let plan = env.begin(&work, "plan");
    let plan_bytes = plan_md(&original, "- 初版\n");
    let tasks_bytes = tasks_md(&[("T01 示例", "src/example.py", "example_is_valid")]);
    submit_outputs(
        &env,
        &work,
        &plan,
        &[("plan", &plan_bytes), ("tasks", &tasks_bytes)],
        "方案完成",
    );
    let review = env.begin(&work, "plan-review");
    assert!(review["data"]["outputs"].get("reviewed-plan").is_some());
    assert!(review["data"]["outputs"].get("reviewed-tasks").is_some());
    let decision = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "重新拆分任务。",
    )
    .replacen("通过", "修改方案", 1);
    std::fs::write(output_file(&review, "decision"), &decision).unwrap();
    std::fs::write(output_file(&review, "reviewed-plan"), plan_bytes.as_bytes()).unwrap();
    let (rejected, code) = env.fail(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        review["data"]["attempt"].as_str().unwrap(),
        "--summary",
        "遗漏被审任务副本",
    ]);
    assert_eq!(code, 1);
    assert_eq!(rejected["error"]["code"], "OUTPUT_MISSING");
    let submitted = submit_outputs(&env, &work, &review, &[("decision", &decision)], "修改方案");
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

const VERIFIED_HEADER: &str = "## 已验证任务\n\n| 任务 | 任务基线 | 候选提交 | 审批来源 | 验证报告 | 原始证据 |\n| --- | --- | --- | --- | --- | --- |\n";

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
        .ok_or_else(|| format!("缺字段 {key}"))
}

fn cold_original(text: &str) -> Result<String, String> {
    if let Ok(value) = cold_field(text, "原始基线") {
        return Ok(value);
    }
    text.lines()
        .find(|line| line.starts_with("原始基线（"))
        .and_then(|line| line.rsplit('`').nth(1))
        .filter(|hash| hash.len() == 40)
        .map(str::to_string)
        .ok_or_else(|| "缺原始基线".to_string())
}

fn cold_rows(text: &str) -> Result<Vec<String>, String> {
    let section = text
        .split_once("## 已验证任务\n")
        .ok_or("缺已验证任务表")?
        .1;
    Ok(section
        .split("\n## ")
        .next()
        .unwrap()
        .lines()
        .filter(|line| line.trim_start().starts_with("| T"))
        .map(str::to_string)
        .collect())
}

fn cold_read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// Fresh worker的唯一入口是本次CLI任务书绑定；不接收原始基线或完成列表参数。
fn cold_replan(begun: &Value) -> Result<ColdReplan, String> {
    let inputs = begun["data"]["inputs"]
        .as_object()
        .ok_or("缺任务书输入表")?;
    let path = |name: &str| -> Result<PathBuf, String> {
        inputs
            .get(name)
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| format!("缺输入 {name}"))
    };
    let previous_plan = path("previous_plan")?;
    let previous_tasks = path("previous_tasks")?;
    let plan = cold_read(&previous_plan)?;
    let tasks = cold_read(&previous_tasks)?;
    let original = cold_original(&plan)?;
    if cold_original(&tasks)? != original {
        return Err("被审plan/tasks原始基线不一致".into());
    }
    let plan_rows = cold_lineage(
        &previous_plan,
        &original,
        &mut std::collections::BTreeSet::new(),
    )?;
    if cold_rows(&tasks)? != plan_rows {
        return Err("被审plan/tasks已验证前缀不一致".into());
    }
    let decision_path = path("decision")?;
    let decision = cold_read(&decision_path)?;
    if cold_field(&decision, "批准的方案")? != sha256_file(&previous_plan) {
        let report = cold_read(&path("previous_verification")?)?;
        let approval = cold_read(Path::new(&cold_field(&report, "审批来源")?))?;
        if approval.lines().next() != Some("继续")
            || cold_field(&approval, "更正原审批")? != decision_path.to_string_lossy()
            || cold_field(&report, "原审批来源")? != decision_path.to_string_lossy()
            || decision.lines().next() != Some("通过")
            || cold_field(&approval, "批准的方案")? != sha256_file(&previous_plan)
            || sha256_file(Path::new(&cold_field(&report, "被验方案")?))
                != sha256_file(&previous_plan)
        {
            return Err("被审方案与人工版本记录及显式更正不一致".into());
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
        return Err("最新验证表丢失被审方案的已验证前缀".into());
    }
    for row in &verified {
        let columns = row
            .split('|')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>();
        if columns.len() != 6 {
            return Err("验证行字段数不完整".into());
        }
        let report = cold_read(Path::new(columns[4]))?;
        let proof = cold_read(Path::new(columns[5]))?;
        let report_first = report.lines().next().unwrap_or_default();
        if !(report_first == "通过，全部完成" || report_first.starts_with("通过，下一任务 T"))
        {
            return Err("失败验证报告不能证明已验证任务".into());
        }
        let approval = cold_read(Path::new(columns[3]))?;
        match approval.lines().next() {
            Some("通过") => {}
            Some("继续") => {
                let prior_path = cold_field(&approval, "更正原审批")?;
                let prior = cold_read(Path::new(&prior_path))?;
                if prior.lines().next() != Some("通过")
                    || cold_field(&report, "原审批来源")? != prior_path
                {
                    return Err("人工更正必须引用原通过审批".into());
                }
            }
            _ => return Err("未批准的历史版本不能证明已验证任务".into()),
        }
        if cold_field(&report, "审批来源")? != columns[3] {
            return Err("审批来源与累计表不一致".into());
        }
        for stream in ["stdout", "stderr"] {
            let stream_path = cold_field(&proof, stream)?;
            std::fs::read(&stream_path).map_err(|error| format!("{stream_path}: {error}"))?;
            if sha256_file(Path::new(&stream_path))
                != cold_field(&proof, &format!("{stream}_sha256"))?
            {
                return Err("原始证据输出摘要不一致".into());
            }
        }

        if cold_original(&report)? != original
            || cold_field(&report, "任务")? != columns[0]
            || cold_field(&report, "基线")? != columns[1]
            || cold_field(&report, "提交")? != columns[2]
            || cold_field(&proof, "提交")? != columns[2]
            || cold_field(&proof, "退出码")? != "0"
        {
            return Err("历史验证行与原始证据不一致".into());
        }
        if project.git(&["merge-base", &original, columns[2]]) != original
            || project.git(&["merge-base", columns[1], columns[2]]) != columns[1]
            || !project
                .git(&["log", "-1", "--format=%B", columns[2]])
                .lines()
                .any(|line| line == format!("Task: {}", columns[0]))
        {
            return Err("验证提交与project Git不一致".into());
        }
        let expected_files = cold_field(&proof, "文件")?
            .split(',')
            .map(str::to_string)
            .collect::<Vec<_>>();
        if project.diff_names(columns[1], columns[2]) != expected_files {
            return Err("验证范围与原始文件集合不一致".into());
        }
        let old_tasks = cold_read(Path::new(&cold_field(&report, "被验任务")?))?;
        let heading = format!("## {} ", columns[0]);
        let card = old_tasks
            .split_once(&heading)
            .ok_or("历史任务卡缺失")?
            .1
            .split("\n## ")
            .next()
            .unwrap();
        let allowed = card
            .lines()
            .find_map(|line| line.strip_prefix("- 只改哪些文件："))
            .ok_or("历史任务卡缺文件白名单")?
            .split('、')
            .map(|item| item.trim().trim_matches('`'))
            .collect::<Vec<_>>();
        if expected_files
            .iter()
            .any(|file| !allowed.contains(&file.as_str()))
        {
            return Err("历史验证范围越过任务卡白名单".into());
        }
        let old_plan = cold_read(Path::new(&cold_field(&report, "被验方案")?))?;
        let gate_command = cold_field(&proof, "命令")?;
        if !old_plan.lines().any(|line| line == gate_command) {
            return Err("原始门禁命令与获批方案不一致".into());
        }
        if check_approval(
            &approval,
            Path::new(&cold_field(&report, "被验规格")?),
            Path::new(&cold_field(&report, "被验方案")?),
        )
        .is_err()
        {
            return Err("历史审批版本不一致".into());
        }
    }
    for name in ["previous_change", "previous_fix_change"] {
        let Some(record_path) = inputs.get(name).and_then(Value::as_str) else {
            continue;
        };
        let record = cold_read(Path::new(record_path))?;
        if cold_original(&record)? != original {
            return Err(format!("{name}重设原始基线"));
        }
        let prefix = cold_lineage(
            Path::new(record_path),
            &original,
            &mut std::collections::BTreeSet::new(),
        )?;
        let source = cold_read(Path::new(&cold_field(&record, "继承来源")?))?;
        if cold_rows(&source)? != prefix || !verified.starts_with(&prefix) {
            return Err(format!("{name}漏行或改写继承前缀"));
        }
        let candidate = cold_field(&record, "提交")?;
        let latest = verified
            .last()
            .map(|row| row.split('|').nth(3).unwrap().trim());
        let stale = candidate != "无"
            && latest.is_some_and(|latest| {
                project.git(&["merge-base", &candidate, latest]) == candidate
            });
        if !stale && prefix != verified {
            return Err(format!("{name}的当前待验证变更丢失旧完成行"));
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
        "条件：无。",
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
            &[("scaffold", "卡住\n批准摘要不符\n")],
            "批准失效",
        );
        let human = env.follow_begin(&stopped, "escalate");
        let decision = format!(
            "继续\n更正原审批: {}\n批准的规格: {}\n批准的方案: {}\n条件：无。\n",
            input_path(&human, "approval").display(),
            sha256_file(&input_path(&human, "spec")),
            sha256_file(&input_path(&human, "plan"))
        );
        submit_outputs(
            env,
            &work,
            &human,
            &[("decision", &decision)],
            "显式更正批准版本",
        );
    }
    let scaffold = scaffold_two_files(env, &work, &project);
    let implement = env.follow_begin(&scaffold, "implement");
    let (base, candidate) = commit_export_task(&project, "cold-fixture");
    let plan_source = input_path(&implement, "plan");
    let change = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
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
            "{change}\n审批更正来源: {}\n原审批来源: {}\n",
            input_path(&implement, "escalation").display(),
            input_path(&implement, "decision").display()
        )
    } else {
        change
    };
    let result = submit_outputs(env, &work, &implement, &[("change", &change)], "完成 T01");
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
        "{}\n提交: {candidate}\n原始基线: {original}\n本轮变更: {}\n继承来源: {}\n被验规格: {}\n被验方案: {}\n被验任务: {}\n审批来源: {}\n\n{}",
        verify_report(
            "通过，下一任务 T02",
            "T01",
            0,
            &base,
            "- 独立范围/门禁/批准核验通过"
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
            "{report}\n原审批来源: {}\n",
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
        "通过，下一任务 T02",
    );
    let pending = env.follow_begin(&result, "implement");
    let stuck = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        change_stuck("T02", "任务需要重规划，尚未验证。"),
        output_file(&verification, "report").display(),
        verified_table(&[row])
    );
    let result = submit_outputs(env, &work, &pending, &[("change", &stuck)], "卡住 T02");
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md(
                if mode == "continue" {
                    "继续"
                } else {
                    "改方案"
                },
                "保留T01及原始基线，只调整后续T02。",
            ),
        )],
        "改方案",
    );
    let next_node = if mode == "continue" {
        "implement"
    } else {
        "plan"
    };
    let replanner = env.follow_begin(&result, next_node);
    (replanner, project, original, candidate)
}

// 负例只验证独立检查器；真实 CLI 的绑定与冻结由下面的冷接续正例覆盖。
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
            "T01 导出",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 编码",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
        (
            "T03 第三项",
            "src/third.py、tests/test_third.py",
            "test_third",
        ),
        (
            "T04 后续项",
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
            .map_or_else(|| "无".to_string(), |path| path.display().to_string());
        let plan = write(
            &format!("plan-{index}.md"),
            format!(
                "{}\n继承来源: {source}\n\n{}",
                plan_md(&original, ""),
                verified_table(&rows)
            ),
        );
        let tasks = write(
            &format!("tasks-{index}.md"),
            format!(
                "{}\n原始基线: {original}\n继承来源: {source}\n\n{}",
                tasks_md(&cards),
                verified_table(&rows)
            ),
        );
        let decision = write(
            &format!("decision-{index}.md"),
            approval_md(&sha256_file(&spec), &sha256_file(&plan), "条件：无。"),
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
                "完成\n原始基线: {original}\n继承来源: {}\n提交: {candidate}\n\n{}",
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
            "通过，全部完成\n任务: {task}\n基线: {base}\n提交: {candidate}\n原始基线: {original}\n本轮变更: {}\n继承来源: {}\n被验规格: {}\n被验方案: {}\n被验任务: {}\n审批来源: {}\n\n{}",
            change.display(), plan.display(), spec.display(), plan.display(), tasks.display(), decision.display(), verified_table(&rows)
        )).unwrap();
        previous_report = Some(report);
    };
    let change = write(
        "pending-change.md",
        format!(
            "卡住\n原始基线: {original}\n继承来源: {}\n提交: 无\n\n{}",
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
        "{}\n继承来源: {}\n\n{}",
        plan_md(&basis.original, "- 根据已绑定证据重排T02\n"),
        input_path(&begun, "previous_verification").display(),
        verified_table(&basis.verified)
    );
    let tasks = format!(
        "{}\n原始基线: {}\n继承来源: {}\n\n{}",
        tasks_md(&[
            (
                "T01 导出入口",
                "src/export.py、tests/test_export.py",
                "test_export"
            ),
            (
                "T02 编码处理",
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
        "保留旧事实后重规划",
    );
    let review = env.follow_begin(&result, "plan-review");
    let approval = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "条件：无。",
    );
    let result = submit_outputs(
        &env,
        &work_id,
        &review,
        &[("decision", &approval)],
        "批准重规划",
    );
    let scaffold = env.follow_begin(&result, "scaffold");
    let result = submit_outputs(
        &env,
        &work_id,
        &scaffold,
        &[(
            "scaffold",
            &format!(
                "完成\n骨架提交: {}\n已有T01保持，T02签名和测试未变\n",
                basis.project.head()
            ),
        )],
        "保留已完成任务",
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
    let candidate = project.commit("feat(encode): 编码处理\n\nTask: T02\nAgent: cold-fixture");
    let change = format!(
        "{}\n原始基线: {}\n继承来源: {}\n\n{}",
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
        "完成 T02",
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
        "{}\n提交: {candidate}\n原始基线: {}\n本轮变更: {}\n继承来源: {}\n被验规格: {}\n被验方案: {}\n被验任务: {}\n审批来源: {}\n\n{}",
        verify_report("通过，全部完成", "T02", 0, &base, "- 独立验证通过"),
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
        "通过，全部完成",
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
    assert_eq!(rows[0], basis.verified[0], "累计前缀原样保留");
}

// Task: C002-T30
#[test]
fn fresh_replanner_stops_on_reset_baseline_missing_fields_or_dropped_verified_task() {
    for (mutation, expected) in [
        ("reset_baseline", "previous_change重设原始基线"),
        ("missing_fields", "缺原始基线"),
        ("drop_completed", "记录漏行或改写来源链前缀"),
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
                text.replace(&format!("原始基线: {}\n", valid.original), ""),
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
            Ok(_) => panic!("fresh worker必须在{mutation}时停止，不交新方案"),
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
        return Err(format!("继承来源循环: {}", path.display()));
    }
    let result = (|| {
        let text = cold_read(path)?;
        if cold_original(&text)? != original {
            return Err("来源链原始基线不一致".into());
        }
        let rows = cold_rows(&text)?;
        let first = text.lines().next().unwrap_or_default();
        if matches!(first, "不通过" | "不通过，需要人" | "通过，全部完成")
            || first.starts_with("通过，下一任务 T")
        {
            let change_path = cold_field(&text, "本轮变更")?;
            for key in [
                "任务",
                "基线",
                "提交",
                "被验规格",
                "被验方案",
                "被验任务",
                "审批来源",
            ] {
                cold_field(&text, key)?;
            }
            let change = cold_read(Path::new(&change_path))?;
            let prefix = cold_lineage(Path::new(&change_path), original, active)?;
            if cold_field(&text, "继承来源")? != cold_field(&change, "继承来源")? {
                return Err("verify继承来源与本轮change不一致".into());
            }
            let passed = first == "通过，全部完成" || first.starts_with("通过，下一任务 T");
            if !rows.starts_with(&prefix) || rows.len() != prefix.len() + usize::from(passed) {
                return Err("历史验证报告漏行或改写累计前缀".into());
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
                    || last[0] != cold_field(&text, "任务")?
                    || last[1] != cold_field(&text, "基线")?
                    || last[2] != cold_field(&text, "提交")?
                    || Path::new(last[4]) != path
                {
                    return Err("本轮追加行与通过报告不一致".into());
                }
            }
        } else {
            let source = cold_field(&text, "继承来源")?;
            if source == "无" {
                if !rows.is_empty()
                    || !(text.starts_with("# 技术方案") || text.starts_with("# 任务清单"))
                {
                    return Err("只有首次方案/任务清单空表可作为来源链起点".into());
                }
            } else if cold_lineage(Path::new(&source), original, active)? != rows {
                return Err("记录漏行或改写来源链前缀".into());
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
    assert!(brief_text(&begun).contains("来自: escalate"));
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
    let candidate = project.commit("feat(encode): 继续T02\n\nTask: T02\nAgent: cold-fixture");
    let change = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        change_done(
            "T02",
            &base,
            &candidate,
            0,
            &["src/encode.py", "tests/test_encode.py"],
            "继续后保留T01，T02候选待验证。"
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
        "完成 T02，保留前缀",
    );
    let verification = env.follow_begin(&result, "verify");
    let recorded = input_path(&verification, "change");
    assert_eq!(
        cold_lineage(&recorded, &original, &mut std::collections::BTreeSet::new()).unwrap(),
        latest
    );
    let omitted = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        change_stuck("T02", "错误空表"),
        report_source.display(),
        verified_table(&[])
    );
    let bad = env.dir.path().join("omitted-prefix.md");
    std::fs::write(&bad, omitted).unwrap();
    assert!(
        cold_lineage(&bad, &original, &mut std::collections::BTreeSet::new())
            .unwrap_err()
            .contains("前缀")
    );
}

fn review_fixture_replan(env: &Env, work: &str, begun: &Value, extra_tasks: bool) -> Value {
    let original = cold_original(&read(&input_path(begun, "previous_plan"))).unwrap();
    let source = input_path(begun, "previous_verification");
    let rows = cold_rows(&read(&source)).unwrap();
    let plan = format!(
        "{}\n继承来源: {}\n\n{}",
        plan_md(&original, "- 重规划，保留原始事实\n"),
        source.display(),
        verified_table(&rows)
    );
    let mut cards = vec![
        (
            "T01 导出入口",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 编码处理",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
    ];
    if extra_tasks {
        cards.extend([
            (
                "T03 第三项",
                "src/third.py、tests/test_third.py",
                "test_third",
            ),
            (
                "T04 后续项",
                "src/fourth.py、tests/test_fourth.py",
                "test_fourth",
            ),
        ]);
    }
    let tasks = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        tasks_md(&cards),
        source.display(),
        verified_table(&rows)
    );
    let result = submit_outputs(
        env,
        work,
        begun,
        &[("plan", &plan), ("tasks", &tasks)],
        "重规划",
    );
    let review = env.follow_begin(&result, "plan-review");
    let decision = approval_md(
        &sha256_file(&input_path(&review, "spec")),
        &sha256_file(&input_path(&review, "plan")),
        "条件：无。",
    );
    let result = submit_outputs(
        env,
        work,
        &review,
        &[("decision", &decision)],
        "保存被审版本",
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
        project.commit("feat(scaffold): 后续签名\n\nTask: scaffold\nAgent: cold-fixture");
    }
    submit_outputs(
        env,
        work,
        &scaffold,
        &[(
            "scaffold",
            &format!("完成\n骨架提交: {}\n已验证任务保持\n", project.head()),
        )],
        "增量骨架",
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
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        change_done(task, &base, &candidate, 0, &[&files[0], &files[1]], ""),
        source.display(),
        verified_table(&prefix)
    );
    let result = submit_outputs(env, work, &begun, &[("change", &change)], "任务候选");
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
        "通过，全部完成"
    } else {
        "通过，下一任务 T04"
    };
    let report = format!(
        "{}\n提交: {candidate}\n原始基线: {original}\n本轮变更: {}\n继承来源: {}\n被验规格: {}\n被验方案: {}\n被验任务: {}\n审批来源: {}\n\n{}",
        verify_report(first, task, 0, &base, "- 原始运行与Git引用见表"),
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
        "不通过\n任务: 整体\n基线: {original}\n原始基线: {original}\n提交: {}\n修复轮次: 0\n发现: 补充README\n",
        project.head()
    );
    let result = submit_outputs(env, &work, &review, &[("report", &report)], "需要整体修复");
    let fix = env.follow_begin(&result, "fix");
    let latest_report = input_path(&fix, "verify_report");
    let verified_rows = cold_rows(&read(&latest_report)).unwrap();
    std::fs::write(project.root().join("README.md"), "整体修复已记录。\n").unwrap();
    project.gate();
    let fixed = project.commit("fix: 整体文档\n\nTask: review\nAgent: cold-fixture");
    let change = format!(
        "修复完成\n针对: 审查报告\n任务: review\n基线: {original}\n提交: {fixed}\n原始基线: {original}\n继承来源: {}\n修复轮次: 1\n\n{}",
        latest_report.display(),
        verified_table(&verified_rows)
    );
    let result = submit_outputs(env, &work, &fix, &[("change", &change)], "整体修复完成");
    // 新的人工需求使协调者选择合法升级边；旧review fix保持为已提交记录，不伪装成Task通过行。
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md("改方案", "新增T03/T04；整体修复已提交，保留全部旧事实。"),
        )],
        "人工要求新方案",
    );
    let plan3 = env.follow_begin(&result, "plan");
    assert_eq!(cold_replan(&plan3).unwrap().verified.len(), 2);
    let scaffold3 = review_fixture_replan(env, &work, &plan3, true);
    let verified3 = finish_fixture_task(env, &work, &project, &scaffold3, "T03");
    let pending4 = env.follow_begin(&verified3, "implement");
    let source = input_path(&pending4, "report");
    let rows = cold_rows(&read(&source)).unwrap();
    let change = format!(
        "{}\n原始基线: {original}\n继承来源: {}\n\n{}",
        change_stuck("T04", "需调整剩余任务"),
        source.display(),
        verified_table(&rows)
    );
    let result = submit_outputs(env, &work, &pending4, &[("change", &change)], "卡住 T04");
    let escalation = env.follow_begin(&result, "escalate");
    let result = submit_outputs(
        env,
        &work,
        &escalation,
        &[(
            "decision",
            &escalation_md("改方案", "只调整尚未验证的T04，保留先前任务。"),
        )],
        "改方案",
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
    assert_eq!(cold_field(&old_fix, "任务").unwrap(), "review");
    assert_eq!(cold_field(&old_fix, "提交").unwrap(), fixed);
    assert_eq!(cold_rows(&old_fix).unwrap().len(), 2);
    assert!(basis.verified[0].contains("| T01 |"));
    assert!(basis.verified[2].contains("| T03 |"));
}

// Task: C002-T30
#[test]
fn fresh_replanner_rejects_deep_dropped_rows_failed_reports_denied_approval_and_cycles() {
    for (fault, expected) in [
        ("drop_earlier", "历史验证报告漏行或改写累计前缀"),
        ("failed_report", "历史验证报告漏行或改写累计前缀"),
        ("denied_approval", "未批准的历史版本不能证明已验证任务"),
        ("cycle", "继承来源循环"),
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
                    read(&report).replacen("通过，全部完成", "不通过", 1),
                )
                .unwrap();
            }
            "denied_approval" => {
                let approval = directory.join("decision-1.md");
                std::fs::write(&approval, read(&approval).replacen("通过", "修改方案", 1)).unwrap();
            }
            "cycle" => {
                let change = directory.join("change-T03.md");
                let text = read(&change);
                let source = cold_field(&text, "继承来源").unwrap();
                std::fs::write(&change, text.replace(&source, change.to_str().unwrap())).unwrap();
            }
            _ => unreachable!(),
        }
        let error = match cold_replan(&begun) {
            Ok(_) => panic!("坏历史{fault}不得生成新方案"),
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
    let old = cold_field(&report, "原审批来源").unwrap();
    assert!(
        check_approval(
            &read(Path::new(&old)),
            Path::new(&cold_field(&report, "被验规格").unwrap()),
            Path::new(&cold_field(&report, "被验方案").unwrap())
        )
        .is_err()
    );
    let stale = read(approval).replace(
        &cold_field(&read(approval), "批准的方案").unwrap(),
        "wrong-plan-hash",
    );
    let stale_path = env.dir.path().join("stale-correction.md");
    std::fs::write(&stale_path, stale).unwrap();
    // 文件工件由引擎冻结；直接检查独立worker所用的版本oracle，不改已提交工件。
    assert!(
        check_approval(
            &read(&stale_path),
            Path::new(&cold_field(&report, "被验规格").unwrap()),
            Path::new(&cold_field(&report, "被验方案").unwrap())
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
        "不通过\n任务: T02\n基线: {original}\n提交: 无\n原始基线: {original}\n本轮变更: {}\n继承来源: {}\n被验规格: fixture-spec\n被验方案: fixture-plan\n被验任务: fixture-tasks\n审批来源: fixture-decision\n\n{}",
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
        report.replace(&format!("本轮变更: {}\n", change.display()), ""),
    )
    .unwrap();
    assert!(
        cold_lineage(&bad, &original, &mut std::collections::BTreeSet::new())
            .unwrap_err()
            .contains("本轮变更")
    );
}
