# C005 historical task mapping

Tooling only: verify task completion states and test ownership. Read original task bodies from the fixed Git snapshot.

| C005-T00 | done |

| C005-T01 | done |

| C005-M1 | done |

| C005-T04 | done |

| C005-T02 | done |

| C005-T03 | done |

| C005-M2 | done |

### C005-T01: Define global interfaces and implement trusted foundations

**Tests.** `attempt_id_requires_number_and_rejects_retired_retry_field`, `attempt_number_allocation_checks_overflow_without_wraparound`, `replacement_atomically_ends_old_attempt_and_inherits_optional_frozen_inputs`, `second_replacement_rejects_without_removing_current_submit_or_fail_eligibility`, `replacement_does_not_consume_failure_budget_and_historical_fail_uses_original_prefix`, `zero_retries_still_allows_replacement_but_first_real_failure_blocks`, `replacement_quota_resets_for_new_occurrence_and_keeps_entered_from`, `superseded_submit_and_fail_reject_and_terminal_guard_takes_precedence`, `replacement_checks_target_and_qualification_before_reason_or_inputs`, `replacement_rejects_any_changed_frozen_observation_and_optional_rebinding`, `replacement_reason_accepts_exact_limit_and_rejects_one_more_byte_without_mutation`, `replacement_stats_are_fresh_post_state_bytes_and_old_binding_is_preserved`, `replacement_reason_field_is_required_nullable_and_unknown_fields_are_rejected`, `persisted_replacement_rejects_reason_combinations_number_gaps_and_second_superseded`, `replacement_records_one_atomic_pair_and_inherits_nonstats_refs_with_new_exact_stats`, `failed_history_uses_its_original_prefix_after_replacement_and_later_exhaustion`, `late_old_writes_and_second_replacement_are_rejected_without_new_records`, `replacement_replay_after_new_attempt_ends_preserves_reason_source_identity_and_bytes`, `qualification_precedes_reason_file_reads_and_modified_input_never_commits`, `replacement_and_submit_or_two_replacements_have_exactly_one_committed_winner`, `schema_three_and_missing_nullable_reason_are_rejected_without_rewriting_records`, `replacement_history_rejects_forged_reason_input_binding_and_snapshot_identity`, `replacement_complete_payloads_are_decoded_before_frozen_workbook_io`, `replacement_with_no_inputs_rejects_changed_entry_source_before_another_write`, `replacement_transaction_crash_boundaries_preserve_exact_committed_history`, `replacement_rejects_a_path_swap_after_opening_the_original_input`.

### C005-T02: Implement frozen replacement behavior and connect entry points

**Tests.** `replacement_is_atomic_and_business_failures_use_history_not_attempt_number`, `replacement_does_not_consume_zero_business_retries_or_approve_a_gate`, `replacement_replays_original_file_reason_and_rejects_conflicting_intent`, `replacement_reason_has_exact_limit_and_missing_identity_is_not_found`, `replacement_refuses_modified_frozen_input_without_revoking_the_running_attempt`, `replacement_crash_windows_preserve_atomic_state_and_exact_brief_and_stats`, `stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load`, `status_card_active_mid_flow`, `public_next_offers_current_replacement_once_and_restores_quota_on_new_occurrence`, `status_card_missing_is_regenerated_on_next_write`, `post_commit_card_failure_returns_committed_response`.

### C005-T03: Complete real continuation and instructions using the guide

### C005-T00

### C005-T04: Complete future card-recovery expectations

| C005-T05 | done |

| C005-T06 | done |

| C005-M3 | done |

### C005-T05

### C005-T06
