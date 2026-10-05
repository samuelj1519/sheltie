# Run the minimal code-change method

For authorized local repository changes with clear acceptance/check commands. Fixed stages/frozen reports do not replace actual code in the project; workers/reviewers verify reported commit/patch/tool references.

Current candidate0.3.0-rc.1 is unreleased. Build current sheltie and use a new explicit root, retaining old binaries/roots unchanged. Default method code-change@1.0.1 is English; code-change-zh-cn@1.0.0 is the maintained Chinese variant. Neither has a default gate or grants deployment/merge/publication authority.

## Inputs

Task.md states goal/constraints/results/acceptance. Project.md identifies absolute repository/scope/worktree prerequisites/check commands. Both freeze by bytes; changed goals require new Work, never live graph/input edits.

Implement → review → deliver; review back returns to implement. Implement/review allow three arrivals, one business-failure retry per arrival; deliver one arrival. Content rework follows edges; execution failure uses fail.

## Install/start

Run Cargo from repository root to locate actual executable. Example_home is a new absolute root used consistently:

```bash
example_home=/private/tmp/sheltie-code-change-example
cargo run -p sheltie-cli -- --home "$example_home" --json workbook add examples/code-change
cargo run -p sheltie-cli -- --home "$example_home" --json workbook show code-change
cargo run -p sheltie-cli -- --home "$example_home" --json work start --workbook code-change --flow default --input task=@/absolute/task.md --input project=@/absolute/project.md
```

Reuse installed versions without add; supply fresh task/project for new Work. Show start_inputs should be task/project. Replace <work> below with returned full ID. Explicit write request-id enables safe retry; reads reject it.

## Acquire/submit/rework

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json attempt begin <work> --node implement
```

Delegate brief_path. Worker reads frozen inputs, changes/checks authorized project, writes outputs.change with actual candidate/raw checks/exit/unrun/remaining.

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json attempt submit <work> --attempt implement#1.0 --summary "Actual completion status"
cargo run -p sheltie-cli -- --home "$example_home" --json attempt begin <work> --node review
```

Independent reviewer did not author code; reads brief/task, writes review, submits. Coordinator reads report/current next and chooses back/main. Successful execution is not accepted content; Accepted text never automatically routes.

Rework creates implement#2/review#2. Use actual begin IDs/paths. Previous-review binds a particular frozen report, not guessed newest directory files. Crashed/unable executors use fail according to next/max_retries; content findings submit/back.

## Resume

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json work status <work>
```

Check current revision/effects_pending/next/resume. Resume binds current brief/inputs/draft locations, not proof of draft existence/sealing. Replay next is historical; current action requires new status.

After handling old executor/shared workspace, continue original running Attempt. Reopening alone changes no state or creates a new Attempt through fail/begin. Reads do not recover pending effects; replay registered writes. Persistent errors stop with accurate cause.

## Results

After independent review/coordinator deliver selection, worker verifies same candidate in change/review, writes/submits delivery:

```bash
cargo run -p sheltie-cli -- --home "$example_home" --json work result <work>
```

Final=true exposes change/review bound inputs and delivery sealed output of that deliver Attempt. Paths/sha256/bytes/source identify terminal binding/sealing. Ref queries do not reprove bytes/reports. Acceptance/copy/merge/publication require their actual contracts/authority.

Final=false is empty. Final=true without declarations is explicitly empty; this method selects three slots. Inspect reports/code/remaining obligations; succeeded does not replace acceptance/independent quality.

## Evaluate English instructions

Use fresh equivalent authorized tasks for English/Chinese variants in separate roots. Capture brief language/first-line routing, review independence, explicit rework, and final provenance; compare human questions/repair rounds/observable costs/independent quality with conditions fixed beforehand. Record actual outcomes in the active package. This procedure is not evidence that model/host comparative evaluation ran.

Language links select repository source variants. Installed/frozen instructions and resources use the chosen language; sibling language directories are not included in that copy.
