# Git message migration reference

English | [简体中文](git-message-migration.zh-CN.md)

The authorized English-default migration translated 260 reachable local commit messages. It preserved each commit tree, every non-parent header (including author/committer identities and timestamps), ordered parent topology, and machine trailers. Parent IDs and resolvable commit references inside translated messages follow the [old/new map](git-message-migration.tsv).

This map preserves original identity as provenance. Historical file trees and raw evidence were not translated or rewritten. Current document/tooling references use mapped local IDs; an original ID found within a frozen historical file can be resolved through this map. Original artifact SHA256 values are unchanged.

Local branches, lightweight release/review tags, commit archive refs, and the stash were remapped in one guarded transaction (43 refs). Remote-tracking refs retain observed upstream identity; original mapping blobs remain byte-exact. Published remote tags, releases, workflow runs, and GitHub commit URLs retain their original identity. Release records explicitly distinguish local mapped commits from original published commits. No push or new release is implied.

For a historical lookup, find the original ID in the first map column and use the second column with `git show <mapped-id>:<path>` or `git archive <mapped-id>`. The mapped commit contains exactly the original tree, including Chinese historical content. Refer to [historical lookup](../how-to/maintain-docs.md#read-original-historical-records).

A verified original-history bundle and raw reference/message snapshot are retained in the ignored local directory `output/C012-english-default-20261005/`. The bundle SHA256 is `3013ae9b61f68098526542f012cbaf1cdce0f20a5e83b2da08477ebd4863e294`. Preserve that backup before any later history cleanup. It is a local artifact rather than part of a source clone.

The adopted task, exact object verification, application record, and final checks live in the [C012 package](../../specs/changes/completed/C012-english-default/README.md). Their scope does not expand historical product acceptance or publication evidence.
