# Git message migration reference

English | [简体中文](../../zh-CN/history/git-message-migration.md)

The authorized English-default migration translated 260 reachable local commit messages. It preserved each commit tree, every non-parent header (including author/committer identities and timestamps), ordered parent topology, and machine trailers. Parent IDs and resolvable commit references inside translated messages follow the [old/new map](git-message-migration.tsv).

This map preserves original identity as provenance. Historical file trees and raw evidence were not translated or rewritten. Current document/tooling references use mapped local IDs; an original ID found within a frozen historical file can be resolved through this map. Original artifact SHA256 values are unchanged.

The initial local migration remapped branches, lightweight release/review tags, archive refs, and the stash in one guarded transaction (43 refs), preserving then-observed remote identity and byte-exact original mapping blobs. The later authorized project publication replaced only the main branch, initially retaining the original release tags.

The subsequent authorized remote cleanup keeps only the `main` branch and places `v0.1.0` and `v0.2.0` on their corresponding commits in its history. The old candidate branch and `v0.1.0-rc` tag were removed from the remote. Release records identify the current tag target as Release commit and preserve Original published commit for provenance; both source trees are identical. Historical release assets, checksums, and recorded workflow identities remain original; old runs are not reclassified as runs against remapped commits. This cleanup creates no new product release.

For a historical lookup, find the original ID in the first map column and use the second column with `git show <mapped-id>:<path>` or `git archive <mapped-id>`. The mapped commit contains exactly the original tree, including Chinese historical content. Refer to [historical lookup](../how-to/maintain-docs.md#read-original-historical-records).

The subsequent repository recreation removes the old GitHub PR references and workflow runs. Local archive references, reflogs, and Git objects outside `main` history were removed without retaining an original-history backup. The mapping preserves old commit IDs as provenance; use their mapped commits for historical source and evidence. The fixed snapshots within `main` remain available.

The new remote contains only `main` and its two release tags. Release assets are restored byte-for-byte, and the Wiki is generated again from committed source. The recreated GitHub records have new identities; they do not reproduce original workflow runs or establish new product acceptance.

The [C012 historical reference](changes/C012-english-default/README.md) points to the full adopted task, exact object verification, application record, and final checks in the fixed Git snapshot. Their scope does not expand historical product acceptance or publication evidence.
