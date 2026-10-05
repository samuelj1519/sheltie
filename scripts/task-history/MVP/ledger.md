# MVP historical task mapping

Tooling only: verify task completion states and test ownership. Read original task bodies from the fixed Git snapshot.

| T01 | done |

| T02 | done |

| T03 | done |

| T04 | done |

| T05 | done |

| T06 | done |

| T07 | done |

| T08 | done |

| T09 | done |

| T10 | done |

| M1 | done |

| T11 | done |

| T12 | done |

| T13 | done |

| T14 | done |

| T15 | done |

| T16 | done |

| M2 | done |

| T17 | done |

| T18 | done |

| T19 | done |

| T20 | done |

| T21 | done |

| T22 | done |

| T23 | done |

| M3 | done |

| T24 | done |

| T25 | done |

| T26 | done |

### T01

### T02

**Tests.** `workbook_id_accepts_kebab_case`, `workbook_id_rejects_uppercase_and_double_dash`, `rel_path_rejects_dotdot_and_absolute`, `bounded_text_rejects_over_limit_bytes`, `sha256_hex_requires_64_lowercase_hex`, `attempt_id_formats_as_node_hash_n_dot_retry`, `work_id_builds_from_day_seq_and_name`, `work_name_normalizes_whitespace_and_case`, `work_name_rejects_over_48_bytes_and_bad_chars`, `work_id_rejects_seq_zero_or_over_999`.

### T03

**Tests.** `parses_minimal_manifest`, `rejects_wrong_schema_string`, `rejects_empty_flows`, `rejects_flow_path_with_dotdot`, `rejects_description_over_2kib`, `parses_requires_with_optional_fields`, `rejects_duplicate_require_kind_name`, `rejects_unknown_require_kind`, `rejects_require_digest_without_sha256_prefix`.

### T04

**Tests.** `parses_three_node_flow`, `instruction_requires_exactly_one_of_file_or_text`, `input_from_parses_start_resource_and_node_forms`, `input_from_parses_engine_stats_only`, `input_from_rejects_three_segments_for_start_and_node`, `input_from_resource_keeps_slashes_in_path`, `node_requires_parses_kind_colon_name`, `node_requires_rejects_bad_kind`, `input_required_defaults_true_and_parses_false`, `tier_defaults_standard_and_parses_strong`, `defaults_gate_false_visits_1_retries_1`, `rejects_max_visits_zero_or_over_32`, `rejects_unknown_edge_kind`.

### T05

**Tests.** `rejects_entry_not_a_node`, `rejects_self_loop_edge`, `rejects_duplicate_from_to`, `rejects_unreachable_node`, `rejects_graph_without_terminal_node`, `rejects_input_from_unknown_output`, `rejects_input_from_node_that_cannot_reach_consumer`, `rejects_gate_node_with_empty_text`, `rejects_missing_instruction_file`, `rejects_non_utf8_instruction`, `rejects_node_id_start_or_resource`, `rejects_node_id_engine`, `rejects_missing_resource_input_file`, `accepts_binary_resource_input`, `rejects_node_require_not_declared_in_manifest`, `rejects_duplicate_node_require`, `rejects_optional_engine_stats_input`, `rejects_tier_on_human_node`, `compiles_article_review_example`, `proptest_compile_never_panics`.

### T06

**Tests.** `start_sets_current_to_entry_occurrence_1`, `start_rejects_missing_start_input_key`, `start_rejects_extra_start_input_key`, `start_next_is_begin_entry_and_cancel`, `start_records_workbook_ref_and_frozen_inputs`.

### T07

**Tests.** `begin_on_entry_creates_running_attempt_with_frozen_inputs`, `begin_rejects_node_not_in_next`, `begin_via_edge_increments_visits_and_occurrence`, `begin_filters_edges_whose_target_hit_max_visits`, `begin_rejects_modified_upstream_artifact`, `begin_rejects_upstream_without_succeeded_attempt`, `begin_leaves_optional_input_unbound_when_upstream_has_no_attempt`, `begin_binds_optional_input_when_upstream_succeeded_later`, `begin_after_failed_attempt_increments_retry_not_occurrence`, `begin_binds_resource_input_under_frozen_workbook_dir`, `begin_reply_lists_node_requires`, `begin_records_entered_from_occurrence_and_edge_kind`, `retry_keeps_entered_from_of_first_attempt`, `begin_binds_engine_stats_and_emits_write_file`, `begin_emits_write_brief_effect`.

### T08

**Tests.** `submit_marks_attempt_succeeded_and_records_outputs`, `submit_rejects_when_attempt_not_running`, `submit_rejects_missing_required_output`, `submit_accepts_missing_optional_output`, `submit_on_gate_node_blocks_work`, `submit_on_terminal_node_succeeds_work`, `submit_on_terminal_gate_node_blocks_not_succeeds`, `submit_when_every_out_edge_target_hit_max_visits_blocks_no_legal_edge`, `fail_marks_attempt_failed_and_allows_retry`, `fail_at_max_retries_blocks_work`, `next_after_success_lists_out_edges_with_kind`, `next_when_blocked_gate_has_only_approve_and_cancel`.

### T09

**Tests.** `approve_unblocks_and_records_principal_and_time`, `approve_rejects_when_not_blocked_on_gate`, `approve_rejects_wrong_node`, `approve_on_terminal_node_succeeds_work`, `cancel_from_active_and_blocked`, `terminal_work_rejects_every_command_with_work_terminal`.

### T10

**Tests.** `brief_for_review_node`, `brief_for_node_with_requires`, `brief_for_node_without_requires_omits_section`, `brief_marks_unbound_optional_input_as_absent`, `brief_for_human_executor_ends_with_submit_command`, `status_card_blocked_on_gate`, `status_card_succeeded`, `stats_table_mid_flow`, `next_op_renders_begin_with_node_flag`, `next_op_begin_carries_executor_and_tier`, `status_card_lists_done_occurrences_in_order`, `stats_json_counts_visits_failures_and_entered_via`.

### T11

**Tests.** `two_step_has_one_main_edge_and_no_gate`, `article_review_has_back_edge_human_publish_and_resource_input`, `gated_release_first_node_is_gate`, `no_example_declares_requires`, `spec_dev_retro_reads_engine_stats`, `spec_dev_only_retro_is_gated_and_human_nodes_are_plan_review_and_escalate`, `spec_dev_strong_tier_nodes_are_spec_plan_scaffold_review`.

### T12

**Tests.** `home_prefers_cli_then_env_then_default`, `confine_rejects_dotdot_absolute_and_empty_segment`, `confine_rejects_symlink_escaping_root`, `resource_index_marks_non_utf8`.

### T13

**Tests.** `open_readonly_on_missing_db_is_not_found`, `open_rejects_wrong_user_version`, `commit_inserts_state_audit_and_request_atomically`, `commit_replays_same_request_id_and_payload`, `commit_rejects_same_request_id_different_payload`, `commit_rejects_stale_revision`, `status_column_mirrors_state_json`, `allocate_seq_starts_at_1_per_day_and_increments`, `allocate_seq_rejects_1000th_of_day`, `allocate_seq_under_two_threads_yields_distinct_numbers`.

### T14

**Tests.** `add_two_step_example_copies_and_marks_readonly`, `add_rejects_duplicate_id_version_with_workbook_exists`, `add_rejects_symlink_inside_workbook`, `add_rejects_file_over_32mib`, `add_failure_leaves_no_staging_and_no_row`, `load_recompiles_graph_from_installed_copy`, `list_orders_by_id_then_version`.

### T15

**Tests.** `remove_deletes_row_and_directory`, `remove_requires_explicit_version`, `remove_allows_when_only_terminal_works_reference_version`, `verify_reports_ok_for_untouched_install`, `verify_reports_missing_when_directory_gone`, `verify_all_when_filter_omitted`.

### T16

**Tests.** `two_step_runs_to_succeeded`, `start_allocates_work_id_with_today_and_seq_001`, `start_replay_returns_same_work_id_without_new_seq`, `start_copies_workbook_into_work_dir_readonly`, `begin_loads_graph_from_frozen_copy_not_repository`, `begin_writes_brief_md_with_absolute_input_paths`, `begin_binds_resource_input_to_frozen_copy_path`, `status_card_regenerated_after_each_commit`, `concurrent_writers_one_gets_revision_conflict`, `begin_on_tampered_frozen_copy_is_store_corrupt`, `missing_frozen_copy_is_store_corrupt_for_begin_and_status`, `tampered_resource_input_is_store_corrupt_not_artifact_modified`.

### T17

**Tests.** `workbook_add_prints_id_version_digest`, `workbook_add_json_has_ok_true_and_data`, `workbook_add_invalid_dir_exits_1_with_workbook_invalid`, `workbook_list_after_add_shows_one_row_marked_latest`, `workbook_show_lists_nodes_edges_and_requires`, `workbook_remove_without_version_exits_2`, `workbook_remove_then_list_is_empty`, `workbook_verify_exits_1_after_tamper`, `unknown_subcommand_exits_2`.

### T18

**Tests.** `work_start_creates_work_and_prints_next`, `work_list_shows_status_and_current`, `work_status_prints_status_card`, `work_stats_prints_table_and_json`, `work_status_json_matches_schema`, `work_cancel_then_any_write_is_work_terminal`, `work_id_prefix_resolves_when_unique`, `work_id_prefix_ambiguous_lists_candidates`, `work_start_with_chinese_name_creates_matching_directory`.

### T19

**Tests.** `attempt_fail_then_begin_retries_same_occurrence`.

### T20

**Tests.** `install_copies_current_exe_and_is_idempotent`, `install_prints_path_hint_and_does_not_touch_rc_by_default`, `update_reports_unavailable_when_no_asset_for_platform`, `rollback_swaps_prev_back`, `rollback_recovers_when_current_missing`, `uninstall_keeps_store_and_works`, `uninstall_purge_requires_yes`, `self_version_works_without_home`.

### T21

**Tests.** `review_back_edge_creates_second_draft_occurrence`, `next_after_review_offers_both_main_and_back_with_kinds`, `max_visits_exhaustion_blocks_with_no_legal_edge`, `human_executor_node_is_begun_and_submitted_like_agent`, `review_brief_lists_checklist_resource_with_frozen_path`.

### T22

**Tests.** `gate_node_success_blocks_work_and_next_has_only_approve_and_cancel`, `begin_next_node_before_approve_is_illegal_next`, `approve_records_os_user_and_unblocks`, `approve_on_terminal_gate_node_succeeds_work`, `modifying_upstream_output_makes_downstream_begin_fail_with_artifact_modified`, `submit_without_required_output_is_output_missing_and_attempt_stays_running`, `submit_oversize_output_is_output_too_large`, `remove_in_use_workbook_is_rejected_with_work_list`, `remove_after_work_succeeds_then_status_still_renders`, `editing_repository_copy_does_not_change_running_work_brief`, `add_second_version_marks_it_latest_and_start_defaults_to_it`.

### T23

**Tests.** `kill_before_commit_leaves_state_unchanged_and_replay_succeeds`, `kill_after_commit_leaves_state_advanced_and_replay_returns_original_reply_and_rewrites_brief`, `kill_between_update_renames_leaves_prev_and_rollback_recovers`, `same_request_id_same_payload_returns_replayed_true`, `same_request_id_different_payload_is_request_conflict`.

### T24

### T25

### T26
