# Storage, transactions, and recovery

English | [简体中文](storage.zh-CN.md)

This contract defines persistence and writes under ~/.sheltie. SQLite is the sole state authority (INV-7); directories contain projections or artifacts. Schema version is 4. Reject schema1/2/3 stores completely without migration or clearing (D-033).

## 1. SQLite

Store file: store.db. Use WAL, synchronous=FULL, foreign_keys=ON, busy_timeout=5000. Each CLI invocation opens one connection and closes it at exit; no pool.

### 1.1 Version and structure validation

PRAGMA user_version stores SCHEMA_VERSION=4, defined solely by the constant in sheltie-runtime/src/store/schema.rs.

1. Identify read-only first. Missing databases yield NOT_FOUND for reads; writes create them only after obtaining the root lock (§2).
2. For existing files identify user_version/sqlite_master through read-only connections. Wrong version yields STORE_SCHEMA_MISMATCH. Compare each expected CREATE TABLE statement ignoring whitespace. Before rejection, do not write main/existing WAL bytes, change journal mode, write PRAGMA, create tables, or checkpoint. D-039 permits shm maintenance and missing zero-byte WAL creation only: no WAL header/frame or existing WAL modification. Reject linked/special database/sidecar leaves before SQLite.
3. Only after structure validation may writable connections set WAL/synchronous=FULL.
4. No migration/clearing. A schema1 negative fixture must prove rejection and unchanged bytes.

Create DDL/user_version=4 in **one transaction**. Generate/validate a complete empty database in memory; use safe public serialization to obtain bytes. Under lock exclusively write owned tmp/store-init-<random-id>/store.db, sync file/parent, then publish that same opened object with NOREPLACE at root/store.db and sync source/target parents. SQLite must not open staging absolute paths or switch staging to WAL. Interrupted initialization leaves only owned tmp; final is absent or complete. Later writes may create independent staging databases; never interpret, initialize, or delete arbitrary existing schema0/1 files. The root lock serializes first creation.

### 1.2 Tables

```sql
CREATE TABLE workbooks (
  id          TEXT NOT NULL,
  version     TEXT NOT NULL,
  digest      TEXT NOT NULL,           -- 64 hexadecimal digits, workbook-digest/v2
  dir         TEXT NOT NULL,           -- Relative to management root
  added_at    TEXT NOT NULL,           -- RFC 3339 UTC
  PRIMARY KEY (id, version)
);

CREATE TABLE works (
  work_id     TEXT PRIMARY KEY,
  revision    INTEGER NOT NULL,
  status      TEXT NOT NULL,           -- active | blocked | succeeded | cancelled; redundant list column
  state_json  TEXT NOT NULL,           -- Complete WorkState, serde_json, deny_unknown_fields
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE work_sequence (
  day          TEXT PRIMARY KEY,      -- UTC YYYY-MM-DD
  last         INTEGER NOT NULL       -- Largest allocated daily sequence, 1..999
);

CREATE TABLE requests (
  request_id   TEXT PRIMARY KEY,
  intent_hash  TEXT NOT NULL,          -- sha256 of canonical RequestIntent JSON (§2.1)
  work_id      TEXT,                   -- Resolved full WorkId; NULL for Workbook writes
  reply_json   TEXT NOT NULL,          -- Complete commit-time ResponseSnapshot, cli-result/v4
  effects_json TEXT NOT NULL,          -- Effect intents/historical bytes (§3.2); deletion also needs .deleted
  published    INTEGER NOT NULL,       -- 0 incomplete; 1 complete; explicit replay can verify/restore write_file
  at           TEXT NOT NULL
);

CREATE TABLE audit (
  seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id      TEXT NOT NULL,          -- Empty for Workbook operations
  revision     INTEGER NOT NULL,       -- Postcommit Work revision; 0 for Workbook operations
  request_id   TEXT NOT NULL,
  principal    TEXT NOT NULL,          -- Actual OS principal (§1.3)
  command_json TEXT NOT NULL,          -- Command with large fields removed
  at           TEXT NOT NULL
);
```

Persisted audit instruction-text summaries retain the exact machine marker `<n 字节>` (where n is the UTF-8 byte length). This marker participates in historical command qualification; translating it would invalidate existing request/audit bindings. It is a persistent format literal, not user-facing prose.

State_json is authoritative. Derive status when writing; reads may use it for list sorting/filtering, never business judgment. Effects_json records I/O completion, does not choose business edges, and is not a second Work state.

### 1.3 Principal

Audit/approval use actual invoking process identity: safe Rust effective-uid lookup on Unix, account name or uid:<number> if unavailable/non-UTF-8. Never USER/USERNAME (D-036). This records accounting facts, not human authentication (constitution §5).

### 1.4 Same-snapshot Work read closure

Store::read_work_bundle for status/result obtains WorkRow and complete requests/audit/effects associated on either request or audit side in one read-only transaction, without published=0 filtering. Validate bidirectional work/request/revision ownership: unique audit per request and complete unique revisions. Missing request/audit, index conflicts, invalid published, or invalid complete payload yield STORE_CORRUPT; validate effects even at published=1. Unassociated other-Work rows do not affect this Work.

Locate frozen original from Start snapshot/publication registration in that bundle; validate Workbook identity, graph, gates, paths, historical responses, and effects. Never combine newer state from another connection. After concurrent publication, a verified final original may be read while preserving snapshot effect facts. A snapshot-unpublished Start requires owner validation whether reading pending or final; completed-cleanup exceptions cannot bypass it. If concurrent publication and owner cleanup finish, boundedly refetch the complete bundle and strictly reload; never splice versions. Result projections take explicit state/graph/revision/effects_pending parameters, adding no state source.

Schema4 has number/superseded/replacement_reason and one strict new-response payload. Preserve/reject old schema1/2/3 without migration. Result/resume adoption remains D-040; replacement format is [D-041](../../docs/explanation/decisions/D-041-attempt-number-and-replacement.md). Directory digests and existing write/effect formats remain unchanged.

## 2. Writes and locking

### 2.1 RequestIntent and intent fingerprints

Runtime constructs RequestIntent before writes. One enum covers StartWork (user Workbook selector, Flow, raw/omitted name, start arguments), Work write verbs (resolved full WorkId plus node/Attempt/summary/reason/replacement target), AddWorkbook (lexical absolute source), and RemoveWorkbook (full ID/version). New WorkName normalization follows protocol pure rules; intent retains user arguments without replacing them with normalized aliases.

- Intent includes parameters/resolved targets only, no observations, clocks, or model-reported facts. Sort StartWork keys and preserve literals or lexically normalized absolute @file paths. SubmitAttempt @file also records only its source. Contents are first-execution observations. Add paths normalize lexically without filesystem access; canonicalize must not make replay depend on source existence.
- Intent_hash is canonical JSON sha256. Independent T07 vectors fix tags/field order/input-key order/nonfloat encoding; retain that serialization/shape. Changed observed digests/contents do not change intent. Same path/ID returns original; new ID rereads current bytes. Different raw names conflict even when normalized names match; omitted and explicit flow-name values differ.
- Resolve new prefixes read-only against works to one full ID, including terminal rows, without Workbook reads. Existing IDs first use requests.work_id: original prefix must match committed target, then build/compare intent with that full ID. Later ambiguous prefixes cannot break/rebind replay. Mismatch is REQUEST_CONFLICT.
- Omitted start versions resolve once and enter committed responses. Replay consults requests first, never latest/current Workbook/@file. Submit/add replay requires no surviving source file/directory.
- Self has no RequestIntent/request_id (protocol §1).

### 2.2 Management-root write lock

Resolve canonical roots under [architecture §5](../architecture.md). A non-UTF-8 SHELTIE_HOME is not an absent variable and must not fall back to HOME. Reads/writes resolve roots first; failure initializes no directory/Store.

Root/.lock is the whole-root exclusive fs4 file lock:

- Every Work/Workbook/self write uses it. Create root/.lock only when allowed and preflight passes; create no other managed files/directories before locking. OS releases at process exit. Under lock: recheck schema → replay → recover effects → prepare → transact → publish → mark complete.
- Reads neither acquire nor create it.
- It coordinates local cooperating processes without constraining same-user manual edits. Keep SQLite revision CAS as transaction validation, without automatic business retries.
- Purge retains empty root and the same lock (D-038). Waiters recheck root/lock dev/inode and continue on that lock. Legal install/add may initialize empty Store; old Work operations return NOT_FOUND after deletion and never recreate old Works.

### 2.3 One write transaction

```text
Read-only preflight without lock: arguments/target identity, schema, request replay;
only misses read @file/Workbook and preflight; add checks source structure/type/limits only
Acquire root lock (create root/.lock only for legal writes)
Recheck schema/request/prerequisites under lock; recover prior effects first
BEGIN IMMEDIATE
  SELECT intent_hash, reply_json, effects_json, published FROM requests WHERE request_id = ?
    Same hash → ROLLBACK; recover and return original, replayed=true
    Different hash → ROLLBACK; REQUEST_CONFLICT
  SELECT revision FROM works WHERE work_id = ?       # Except start/Workbook writes
    Changed revision → ROLLBACK; REVISION_CONFLICT
  # New Work core::decide already produced Decision outside the transaction; replay skips core
  UPDATE/INSERT works OR INSERT/DELETE workbooks
  INSERT audit
  INSERT requests (intent_hash, reply_json, effects_json, published=0)
COMMIT
Publish effects (§3.2); only when complete UPDATE requests SET published=1
Release lock
```

- No file reads, digests, or model calls inside transactions. Observe before BEGIN and pass facts to core with Command.
- Decide runs outside transactions. Locking normally prevents conflicts, but preserve CAS. Return conflicts directly, without automatic reread/reobservation/redecision; callers query and choose the next operation.
- Prepare private pending before COMMIT. Final publication, briefs, sealing, and card refresh occur afterward; these are effects, not Work state.

## 3. Crash semantics and recovery

### 3.1 Windows

Processes may be killed at any point.

| Window | Database | Directory | Next operation |
| --- | --- | --- | --- |
| Preflight rejection | Unchanged; no DB in new root | Unchanged | Correct arguments/inputs; same ID allowed |
| Sequence allocated, staging prepared | No Work row; sequence gap/owned pending possible | Owned staging only; no final | Locked ownership/no-Store-reference check and cleanup; never reclaim sequence |
| Before COMMIT | No new business/request/audit; first legal initialization may leave empty schema4 | Owned staging/control objects only | Same/new ID reruns preflight |
| After COMMIT, before publication | Committed, published=0 | Pending payload is the sole original; **never age-clean** | Next write recovers under lock before new commands; same-ID returns original |
| During publication after rename/before mark | published=0 | Final object present | Verify ownership/digest; same object complete; different object fails without overwrite |
| Publication failure (disk/permissions) | Committed | Partial | EFFECT_PENDING/committed=true/request-id/original; Work revision; no state rollback |

### 3.2 Effect registration

Effects_json contains these six kinds:

| Kind | Fields | Recovery |
| --- | --- | --- |
| `publish_dir` | pending (pending/<id>/payload/), final, owner (work:<id> or workbook:<id>@<version>), digest, digest_root (workbook for Work, empty for Workbook) | Verify owner/full closure. Pending only: sync, NOREPLACE rename, make Workbook read-only. Final only: verify business row/success request/effect/full identity. Both/neither: stop without overwrite/recreation |
| `prepare_attempt` | work_id, attempt_id, dirs in parent-first order | Verify committed Attempt; safely create Attempt/engine/outputs/output parents; existing directories must have correct type/ownership, no links/file occupants |
| `write_file` | path, sha256, content (exact bytes) | Missing: write; same digest: no rewrite; different: integrity error, never conceal mutation |
| `seal_outputs` | refs (complete original ArtifactRefs) | Validate originals then read-only; missing/modified stops, no recreation/out-of-root chmod |
| `delete_dir` | pending, final, owner, predelete digest | Verify/move original to owned pending/delete same object; then durable completion marker (§3.3). Both absent without marker is not success |
| `refresh_status_card` | work_id | Generate from latest state_json; no historical bytes or old-snapshot overwrite |

Publish digests are registered workbook-digest/v2 for Workbook and frozen-copy digests for Work; Work also validates ID and every start ArtifactRef. Digest_root accepts exactly these two fixed values. Completed final objects may have cleaned owner sidecars: current row, success request/effect, ID/version/digest establish identity. Pure completed replay does not locate/operate current objects. Delete digest is registered Workbook digest. Validate every string as a managed relative path before action.

Effects_json is UTF-8 JSON, strict objects with exactly kind/fields above. Paths are UTF-8 root-relative; sha256/digest are bare 64 lowercase hexadecimal digits. Content is UTF-8 text: decoded UTF-8 bytes are exact historical brief/stats bytes. Refs use complete protocol ArtifactRefs. Validate structure/path/digest before actions; invalid formats yield STORE_CORRUPT, no guessed defaults.

Recover incomplete requests by audit.seq. Validate complete effect closure before the first I/O. Publish/prepare/write historical files/seal or delete/refresh latest card in order. Mark published only after every required file/directory sync succeeds, then clean metadata. Pure completed replay skips directory effects; explicit begin replay validates only its own write_file bytes. Own failures return committed=true/request_id/original (+Work revision); old A blocking new B returns committed=false/B's ID/detail.pending_request_id/pending_original. Cards always use latest WorkState. Cleanup errors are stderr maintenance diagnostics, not changed success JSON/snapshots. Cleaned owners do not prevent current-row verification of completed finals.

Verify original-response qualification separately from effect completion. Valid snapshot metadata, unique audit, and business binding preserve verified originals even with corrupt effects. Corrupt metadata/binding must not project raw JSON as success. Remove targets come from audit; add identity must match commit-time PublishDir target/owner/digest. Begin host declarations exactly match frozen node. Historical state validates core-determinable operation/retry/graph rules without confusing current state/later visits. If independently unverifiable, omit original under protocol, retain confirmed identity/cause, and neither guess history nor rewrite Store.

### 3.3 Pending and cleanup

Pending contains only locked engine-owned preparation, committed unpublished Work/Workbook originals, and deletions. Each operation uses internal UUIDv7. Owner sidecar is single-line UTF-8 JSON plus newline, ordered format/internal_id/request_id/op; format=pending/v1, internal_id matches directory, op=start_work|add_workbook|remove_workbook. Sidecars stay outside final directories; effects record payload/business ownership.

Start/add exclusively create/fsync pending/<internal-id>.owner, fsync pending entry, then exclusively create pending/<internal-id>/payload/. Remove persists owner/container **without precreating payload**; postcommit identity/digest verification precedes moving final into payload. Before recovery/cleanup verify confined paths, sidecars, directory types, and Store references without link following. Sync original/required parents before rename and both parents afterward. Required sync failure leaves effects incomplete. Normal durability should not leave ownerless payloads.

After deletion exclusively create pending/<internal-id>.deleted with {"format":"delete-complete/v1","internal_id":"<internal-id>"} plus newline and fsync. While published=0, only a valid marker proves completion when final/payload are absent. Missing/corrupt marker yields EFFECT_PENDING/unknown outcome with original preserved; external deletion is not operational success. Externally deleted sole postcommit originals also stop, never recreate.

- Under lock delete only complete pending containers/sidecars with valid ownership and no effects_json references: precommit residue. Sidecar-only residue may be cleaned after validation. Interrupted sidecar fragments without directories remain with diagnostics but do not block other owned requests. Ownerless directories, mismatched markers, abnormal paths, or unclear references stop/report rather than infer ownership.
- Never delete committed published=0 originals before effect completion. Once published=1, validate Store records then clean empty containers/owner/deleted. Cleanup failures do not repeat effects or age-clean.

Tmp is unrelated disposable download/unpack staging; any process may clean it at any time. Successful CLI writes, after business response, take the existing lock and clean direct tmp children whose modification age is strictly >24 hours. Exactly24h, future, and unexpired remain. Directories use their own root mtime. Verify the same opened object and traverse handles; leaf symlinks delete links only. Special objects, hardlinks, or unprovable identity remain with stderr maintenance warnings; response/exit/history unchanged. Read-only/failed calls do no expiry maintenance and create no root/lock/tmp for it. Age/timestamps/names never justify pending deletion.

Unpublished read queries never recover. Locate publisher/pending from current row and validate owner. Completed cleaned finals validate through row/request/effect/digest without deleted owners. Mark corresponding object pending_publish=true even after rename before mark. Only unpublished add permits pending Flow start preflight. Bounded final→pending→final reads handle rename races; exhausted retries yield temporary IO. No HomeLock, business writes, or arbitrary-version fallback.

## 4. Artifact sealing

Workers write attempts/<node>/occurrence-*/attempt-*/outputs/. At submit runtime:

1. Open each output anchored to directory handles without symlink following. Fstat that same handle: regular, nlink=1, confined managed relative path within Attempt.
2. Check size, stream digest/count from same handle; actual observed count governs if fstat differs.
3. After core validation/commit reverify same handle identity and committed ArtifactRef bytes, make that object read-only and sync, then confirm the original path still names it. Failure is committed effect error.

**ArtifactRef.sha256 is authoritative, not read-only permissions.** Downstream binding recomputes digest; mismatch yields ARTIFACT_MODIFIED. Start inputs likewise become start-inputs/<key> with ArtifactRefs.

## 5. Workbook repository and directory digests

### 5.1 workbook-digest/v2

One SHA256 over exact bytes:

```text
ASCII "sheltie-workbook-digest/v2\0"
BE64(file_count)
For each regular file sorted by canonical UTF-8 relative-path bytes:
  BE64(path_byte_length) || path_bytes || BE64(content_byte_length) || content_bytes
```

- BE64 is unsigned eight-byte big-endian. Counts/lengths eliminate boundary ambiguity (O07).
- Byte-order paths without Unicode/case conversion; no dot/dotdot/NUL/empty segments.
- Empty directories do not contribute. Reject symlinks/hardlinks nlink>1/special files.
- Check per-file/aggregate limits (§5.3) before reading, then stream/count accurately; growth cannot bypass limits.
- No two-stage per-file hash concatenation substitutes for this format. Independent handwritten framing tests supply expected digests.

### 5.2 Add/remove transactions

Add:

1. Read-only readable source/root regular workbook.toml/structure/type/limits checks; rejection creates no Home/Store/lock.
2. Lock; persist owner/pending entry (§3.3), exclusively create payload, bounded copy, fsync each file.
3. **Only the final copy** determines manifest parsing, Flow parsing/compilation, digest, and registered ID/version. Concurrent source changes that produce invalid/incomplete copies reject; valid copies register actual bytes, not preflight metadata. Content failures may retain control objects/owned uncommitted pending but no business/request/audit/final. Repair source and reuse ID.
4. One transaction deduplicates then inserts workbooks/audit/requests with publish_dir and published=0. Primary-key conflict yields WORKBOOK_EXISTS.
5. Postcommit rename payload to workbooks/<id>/<version>/; entire tree/root read-only (dirs0555/files0444). After all effects mark published, then clean container/sidecar.

Each add operates only its own pending, with older recovery before new add.

Remove:

1. Require explicit version.
2. One transaction deduplicates, validates every works row through T14 persisted-state validation (ID/revision/status/state_json), rejects any STORE_CORRUPT without redundant-status prefilter, finds nonterminal references, and rolls back WORKBOOK_IN_USE/detail.works if any. Otherwise delete row and insert audit/request/delete_dir with original digest/ownership.
3. Postcommit verify final identity/digest with absent payload, move to owned payload/sync parents, safely delete same object/sync, exclusively write/sync deleted marker. Validate marker/owner/op/request binding before mark. Both absent without valid marker yields EFFECT_PENDING preserving original, no fabricated marker. Content/ownership mismatch stops.
4. Completed deletion is not repeated by replay. New same-ID/version lifecycles are neither deleted by old remove nor overwritten by old add. New add is a new request; intent hashes distinguish requests.

Copy rejects symlinks/hardlinks/nonregular/.. paths, files >32 MiB, aggregate >256 MiB; host metadata follows §5.3.

### 5.3 Source directories and host metadata

Version is one safe directory segment: additionally reject dot/dotdot/reserved .staging. Accept regular source files only. **Explicitly reject and name** Finder .DS_Store and similar host metadata, without silently ignoring bytes; users clean them before installation. Single-source-file reads are bounded at 32 MiB; Workbook aggregate at 256 MiB.

### 5.4 Frozen Work copy

After sequence allocation but before start inputs, persist owner/pending entry under lock, exclusively create payload, copy installed Workbook to payload/workbook/. Recompute digest against registration; mismatch STORE_CORRUPT. Only validated copies become read-only and receive payload/start-inputs/. Commit then publish_dir renames payload to works/<id>/. State records that copy's digest. Every later graph/instruction/resource read uses it; unpublished Works read the same pending original under §3.3.

- Remove does not remove Work definitions; nonterminal references are a safety restriction.
- Installed-directory tampering does not affect started Works; verify detects it.
- Terminal status remains complete after installed-version deletion.

Missing/mismatched frozen copies cause STORE_CORRUPT for every Work operation, without repository fallback (protected unpublished pending excepted).

### 5.5 Administrative replacement persistence invariants

AttemptId uses number only, no retry compatibility field. Complete schema4 requires replacement_reason (null or BoundedText); missing/unknown state fields reject. Superseded requires ended_at/reason and no summary/fail_reason/outputs; every other state has null replacement_reason. Each Occurrence has contiguous numbers, at most one superseded, a valid successor, and unique latest current running qualification.

Replace intent fixes full WorkId/old AttemptId/literal reason or lexical @file source, excluding clock/principal/contents. Audit records actual principal/time/original command and full materialized bounded reason, verbatim matching replaced reason. Other-command audit summary policies remain unchanged. Snapshot adjacent old/new identities, inputs/requires/paths/revision must match history/audit. Later successor completion does not invalidate replace history. Fail status proves only the original failed prefix.

One transaction records old superseded/new running/audit/request/effects. Precommit rejection revokes nothing and creates no final Attempt directory; postcommit recovery uses existing prepare/write/seal/refresh with real original bytes. Never split commits or add another counter store, lease, session, or executor registry.

## 6. Historical files and current projections

Brief/stats are historical: finalize before commit, register exact effects_json bytes, verify/restore through write_file for incomplete recovery and explicit completed replay, never recalculate latest state. Explicit historical verification syncs the same file/parent even when published, completing previously failed post-rename sync without changing matching bytes or repeating directory publication/prepare/sealing. Completed submit replay never reseals outputs. Cards project latest state/current resume, excluding live revision/effects_pending. Refresh failures retain published=0; replay never writes old cards.

## 7. Time and IDs

Use second-precision RFC3339 UTC exactly YYYY-MM-DDTHH:MM:SSZ (uppercase T/Z, no fractions/offsets). Runtime obtains SystemTime::now() and passes Context; default IDs are runtime UUIDv7. Core uses no clocks/randomness; tests supply fixed facts. Timestamp construction/reading validates format; lexical order therefore equals time order. From_unix_secs beyond year9999 saturates to 9999-12-31T23:59:59Z, preserving format.

### 7.1 Work sequence allocation

Work ID is <day>-<three-digit-seq>-<name>. Allocate under lock before materialization in an independent short transaction:

```text
BEGIN IMMEDIATE
  INSERT INTO work_sequence (day, last) VALUES (?, 1)
    ON CONFLICT(day) DO UPDATE SET last = last + 1
  SELECT last FROM work_sequence WHERE day = ?
    last > 999 → ROLLBACK; INVALID_REQUEST
COMMIT
```

- Share daily counters across methods/Flows/names, grouped only by UTC date.
- Never reclaim allocated numbers. Later failures/crashes leave gaps with no works row. Deterministic missing-input/invalid-name/missing-method rejection occurs first (GF-30).
- Day and created_at both derive from Context.now, avoiding midnight disagreement.
- Name is readable suffix; whole string is identity. Different sequences with the same name are different Works.
- Replay hits requests and allocates nothing.

## 8. Cleanup

MVP has no clear command. Manual Work deletion requires deleting directory then row, explicitly documented. Ownership-checked work clear remains [roadmap](../roadmap.md). Tmp may be deleted anytime (§3.3); pending is never age-cleaned.

## 9. Binary self management

```text
~/.sheltie/
  bin/
    sheltie          # Current
    sheltie.prev     # Previous for rollback
  tmp/               # Download/unpack
  .lock              # Root write lock
```

Install creates root/parents, copies current executable to tmp/sync, then atomically places bin/sheltie. Missing targets use NOREPLACE. Existing targets require nonsymlink managed regular single-link validation; identical bytes stay untouched, different bytes atomically replace. Reject linked/special/multilink targets without following/overwriting destinations. Preserve install_replaces_divergent_binary_instead_of_short_circuit. No shell writes; PATH is a hint only. Self writes take the root lock.

Update:

1. Fix release identity: latest if omitted, otherwise v<version>; manifest/assets use the same resolution.
2. Find target-triple archive/sha256 in that tag's manifest or UPDATE_UNAVAILABLE.
3. Download tmp/<uuid>/, verify archive digest; mismatch removes tmp and returns UPDATE_CHECKSUM_MISMATCH. Valid tar.gz/tar.xz extracts the unique regular sheltie (cargo-dist <artifact-without-extension>/sheltie). Thin assets are binaries without unpacking.
4. Rename bin/sheltie to bin/sheltie.prev, replacing old previous.
5. Rename tmp/<uuid>/sheltie to bin/sheltie.
6. Remove tmp/<uuid>/.

Crashes between4/5 leave previous without current. Running ~/.sheltie/bin/sheltie.prev self rollback must restore directly when current is absent. Unix permits replacing running executables; current process finishes on its old image.

Successful purge retains only empty root/original lock; remove main/WAL sidecars/workbooks/works/pending/tmp/bin. Reads create no Store. Legal add/install initialize on the same lock. Old Work calls return NOT_FOUND without recreation. Different schemas remain rejected without migration/clearing.

Update does not modify Store. Higher-schema binaries reject old Stores on next operation (§1.1); v0.2.0 also rejects schema2. Rollback needs a matching old root; old binaries must not write new roots. Binary rollback is not schema downgrade.

Cargo-dist generates GitHub Releases, archives, sha256 manifests, and install.sh from tags. Update reads dist-manifest.json, accepting thin {version,assets:[{platform,name,sha256}]} for local/tests and full cargo-dist manifests adapted by selfmgmt. SHELTIE_RELEASE_BASE local directories avoid networking and mirror latest/dist-manifest.json and v<version>/dist-manifest.json, using the same resolver. Network uses system curl, not axoupdater (D-30). Distribution contains only the sheltie executable.

### Internal bounds for cumulative blocked facts

Blocked_count is explicit GF-29 history, never recomputed/corrected. Trusted loading requires approvals.len()+current_is_blocked ≤ blocked_count ≤ attempts.len()+approvals.len(). Each block belongs to at most one Attempt end or approval; each approval proves a previous gate block, and current blocked proves another event. Violation yields STORE_CORRUPT preserving data; this is a necessary bound, not exact reconstruction. Core increments must neither panic on overflow nor wrap.
