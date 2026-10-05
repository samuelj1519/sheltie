# Human approval correction and handoff

Only bound escalation with first line Continue, explicit Corrected approval: <original-decision-path>, Approved specification: <current-spec-sha256>, Approved plan: <current-plan-sha256>, and original decision first line Accepted authorizes corrected-version checks. Ordinary Continue is not approval. Preserve original conditions; explicitly changed human conditions are checked against correction.

Record Approval correction source: <escalation-path> in handoff. Later nodes follow that explicit reference even when new escalation is bound. Verify's Approval source and current cumulative row point to actual correction; also record Original approval source: <original-decision-path>. Without correction use decision. Missing references/digest mismatch/original rejection stop without guessed correction.
