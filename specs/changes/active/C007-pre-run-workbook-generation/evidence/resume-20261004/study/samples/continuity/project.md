{
  "workspace_source_commit": "00e010ba8466b6ffc78ba603f5c923174b3d12af",
  "initial_head": "351feb7ac22c21317a686693b732d5ae0c4b4bcc",
  "initial_tree": "80ea3046b1b3575f07e68313444c051ee7c5b7db",
  "repo_by_arm": {
    "native": "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/continuity/native",
    "sheltie": "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/continuity/sheltie"
  },
  "active_authority": "External C007 adopted study cards authorize this independent sample; historical no-active clone entry does not authorize proposed work, only this explicit task scope.",
  "allow_files": [
    "specs/guides/continuity-choices.md"
  ],
  "source_material_paths_relative_to_assigned_repo": [
    "specs/contracts/protocol.md",
    "specs/contracts/storage.md",
    "specs/changes/completed/C004-verifiable-delegation/README.md",
    "specs/changes/completed/C005-executor-continuity/spec.md",
    "specs/changes/completed/C005-executor-continuity/design.md",
    "skills/sheltie/SKILL.md"
  ],
  "current_method_dir": "/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/confirmed-input",
  "canonical_binary": "/private/tmp/sheltie-completion-20261003/t58-final-frozen-binaries/sheltie",
  "runner_environment": "gpt-6.1-sol/high inherited same for botharms; actual metadata recorded; macOS arm64 same tool permissions; globalhuman/modelhistory unknown",
  "checks": [
    [
      "scripts/check-docs.sh",
      "specs/guides/continuity-choices.md"
    ],
    [
      "git",
      "diff",
      "--cached",
      "--check"
    ]
  ],
  "candidate_rule": "Implementation stages exactly allow_files (including new files), runs checks with immutable raw output outside repo, creates local candidate commit with hooksPath=/dev/null and signingfalse; exact HEAD and allowfiles SHA bound. No othercommits/content changes beyond ownsample.",
  "patch_rule": {
    "generate_argv": [
      "git",
      "diff",
      "--binary",
      "--full-index",
      "351feb7ac22c21317a686693b732d5ae0c4b4bcc",
      "HEAD",
      "--",
      "specs/guides/continuity-choices.md"
    ],
    "untracked_policy": "Required newfiles must be staged/committed; compare full diff path set toallow_files and candidate, never plain unstaged diff.",
    "verify_repo_by_arm": {
      "native": "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native",
      "sheltie": "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/sheltie"
    },
    "verify_sequence": "Create declared verify repo from original archive/full initial snapshot; use git apply --check then git apply --index on complete patch, compare git write-tree equals candidate HEAD^{tree}; source and oppositearm nevermodified."
  },
  "native_outputs_rule": "coordinator predeclares report paths under run-dir and persists samephase/inputs/counters/candidate; no Sheltie calls",
  "sheltie_outputs_rule": "Only actualbegin outputs; publicCLI next/brief, readonlysealedinputs; no directmetadata writes",
  "raw_check_rule": "Each required check actual argv/cwd/relevantenv/stdout/stderr/exit+candidate, failure kept; compare declaredsource initial tree and allowed-path change set.",
  "shared_existing_doc_skill": {
    "path": "/Users/shushu/.agents/skills/tech-doc-style-chinese/SKILL.md",
    "SHA": "24c1c75482d47270d3c58bfdab24df3747334cc61ea8be94e98131236aa9b243",
    "both_arms_same": "Read and apply same existing skill guidance when authoring Chinese docs; no newinstallation. Relevant projectterms fromCONTEXT, task authorization explicit."
  },
  "candidate_commit_format": "docs(study): concise Chinese sample summary; body actual reason/checks; Change:C007, Task:C007-T05, Sample:<id>, Arm:<arm>, Agent:<actual worker>. These local sample commits are evidence, Root applies only final real patch underT07."
}
