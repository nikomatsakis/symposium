# WIP: Custom predicate output as JSON Lines

## Summary

Change the custom predicate stdout protocol from a single JSON object to **JSON Lines** (one JSON object per line). The output is a non-exhaustive set of predicate records, making it easy to add new record types in the future without protocol changes.

## Current behavior

A custom predicate script communicates its result via:

- **Exit code**: 0 = pass, non-zero = fail.
- **Stdout** (optional): A single JSON object with selected crates for `source = "crate"` resolution:

```json
{"selectedCrates": [{"crate": "bp-crate", "version": "0.2.0"}]}
```

Empty stdout with exit 0 means "pass, no witness crates." Non-empty stdout that isn't valid witness JSON is treated as a failure.

## Proposed behavior

Stdout becomes a series of lines, each a self-contained JSON object (JSON Lines / JSONL format). Each line is a tagged union — the key identifies the record type:

```
{"selectedCrate": {"name": "serde", "version": "1.0.217"}}
{"selectedCrate": {"name": "tokio", "version": "1.40.0"}}
```

### Record types

The set of record types is **non-exhaustive** — new types may be added in future versions.

| Key | Value | Meaning |
|-----|-------|---------|
| `selectedCrate` | `{"name": "<crate>", "version": "<semver>"}` | A crate whose source should be fetched for `source = "crate"` skill groups. Equivalent to today's `selectedCrates` array entries. |

### Design decisions

1. **Breaking change.** The old `{"selectedCrates": [...]}` format is dropped with no transitional support. Custom predicates are recent enough that this is acceptable.

2. **Empty stdout still means "pass, no records."** Exit 0 with no output continues to work as a simple boolean predicate.

3. **Each line is independent.** A malformed line (not valid JSON, zero keys, etc.) fails the entire predicate. This keeps the protocol strict and avoids partial-state bugs.

4. **Field naming: `"name"` not `"crate"`.** The new per-line format uses `{"name": "...", "version": "..."}` inside `selectedCrate` records, replacing the old `"crate"` key.

5. **Unknown record types: warn and skip (open design question).** A line that is valid JSON with a single unrecognized key is warned and skipped, not fatal. This gives forward-compatibility (older Symposium + newer plugins degrade gracefully) but risks silent misconfiguration from typos (e.g., `{"selctedCrate": ...}` would be silently ignored). We may revisit this — possible alternatives include a strict mode, or a "did you mean?" heuristic for near-misses.

## Example

```sh
#!/bin/sh
# Select crates based on a feature-flag file.
FLAGS_FILE="$PWD/.feature-flags.json"

if [ ! -f "$FLAGS_FILE" ]; then
  exit 1  # predicate returns false — no flags file
fi

if jq -e '.experimental_serde' "$FLAGS_FILE" > /dev/null 2>&1; then
  printf '{"selectedCrate":{"name":"serde","version":"1.0.217"}}\n'
fi

exit 0
```

## Implementation plan

### Type boundary

The main crate's `src/predicate.rs` imports `SelectedCrate` directly from `symposium-sdk` and uses it in `CustomPredicateResult.witness`. The SDK defines the wire format; the main crate uses its deserialization. Types are shared across the crate boundary — changes must be coordinated and land together.

### Strategy

One commit for all code changes (SDK + parser + tests). Follow-up commit for docs. No transitional period since this is a clean break.

### Step 1: SDK types — `symposium-sdk/src/predicate.rs`

1. **Remove `PredicateOutput`** (the old single-object struct, lines 32–38, 76–83).
2. **Update `SelectedCrate` serde impls** — change the JSON field from `"crate"` to `"name"`:
   - `Serialize` impl (line 53): `s.serialize_field("crate", ...)` → `s.serialize_field("name", ...)`
   - `Deserialize` impl (line 63): `#[serde(rename = "crate")]` → `#[serde(rename = "name")]`
3. **Add `PredicateEmitter<W: Write>`** with chainable methods:
   ```rust
   pub struct PredicateEmitter<W: Write> { writer: W }

   impl PredicateEmitter<io::Stdout> {
       pub fn stdout() -> Self;
   }

   impl<W: Write> PredicateEmitter<W> {
       pub fn new(writer: W) -> Self;
       pub fn selected_crate(&mut self, name: &str, version: &semver::Version) -> io::Result<&mut Self>;
   }
   ```
   Internally uses a private `SelectedCrateRecord` wrapper to produce `{"selectedCrate": {"name": ..., "version": ...}}` + newline per call.

   Usage:
   ```rust
   // One-liner
   PredicateEmitter::stdout().selected_crate("serde", &semver::Version::new(1, 0, 217)).unwrap();

   // Chained
   PredicateEmitter::stdout()
       .selected_crate("serde", &semver::Version::new(1, 0, 217))?
       .selected_crate("tokio", &semver::Version::new(1, 40, 0))?;
   ```
4. **Update module-level doc comment** to describe JSONL protocol.

### Step 2: SDK docs — `symposium-sdk/src/lib.rs`

Replace the "Custom predicates" doc example (lines 29–47) from `PredicateOutput` construction to `PredicateEmitter` usage.

### Step 3: Parser — `src/predicate.rs` `parse_witness_stdout`

Rewrite the function (lines 878–940) from single-object deserialization to line-by-line JSONL:

1. Empty stdout → `Some(vec![])` (pass, no crates).
2. Parse as UTF-8 (fail on invalid UTF-8).
3. For each non-blank line:
   - Parse as JSON object. Must have exactly one key.
   - `"selectedCrate"` → deserialize value as `SelectedCrate`, accumulate.
   - Unknown key → `tracing::warn!`, skip line.
   - Not valid JSON / not an object / zero or multiple keys → return `None` (fail).
4. Remove the `PredicateOutput` import (it no longer exists in the SDK).

No changes to `CustomPredicateResult` or `run_custom_predicate` — the return type stays `Option<Vec<SelectedCrate>>` and the in-memory cache continues to deduplicate within a single sync run.

### Step 4: Unit tests — `src/predicate.rs` mod tests

**Update existing tests:**
- `witness_custom_with_selected_crates` — change JSON to JSONL with `"name"` key
- `witness_custom_invalid_version_fails_predicate` — same format change
- `witness_custom_multiple_crates` — change from one JSON blob to multiple JSONL lines

**Add new tests:**
- Multi-line output: two `selectedCrate` lines, verify both accumulated
- Unknown record type: `{"futureFeature": {...}}` + valid `selectedCrate` → passes, valid crate collected
- Empty object `{}` → predicate fails
- Blank lines between valid records → fine, skipped

### Step 5: Integration tests — `tests/custom_predicates.rs`

Update `sync_custom_predicate_witness_drives_crate_source` (line 256) script from:
```sh
printf '{"selectedCrates":[{"crate":"bp-crate","version":"0.2.0"}]}'
```
to:
```sh
printf '{"selectedCrate":{"name":"bp-crate","version":"0.2.0"}}\n'
```

No fixture files need updating — all test scripts are generated inline by `write_script`.

### Step 6: Documentation (separate commit)

1. **`md/reference/plugin-definition.md`** (lines 493–505) — Rewrite "Witness output (stdout JSON)" subsection. Show JSONL example, describe record types table, explain empty stdout / malformed line / unknown key behavior.
2. **`md/reference/predicates.md`** (around line 30) — Note that custom predicates contribute witness crates via `selectedCrate` JSONL records.
3. **`md/reference/crate-predicates.md`** (around line 73) — Same note.
4. **`md/design/module-structure.md`** (around line 48) — Add paragraph on custom predicate JSONL protocol and `parse_witness_stdout` (currently says nothing about custom predicates).

### Build order

```
Steps 1+2 (SDK)  ─┬─→  Step 3 (parser)  ─→  Steps 4+5 (tests)
                  │
                  └─ Must land together: removing PredicateOutput
                     from SDK breaks the parser's import
```

Step 6 (docs) is a follow-up commit after tests pass.

## Open questions

- Is warn+skip for unknown keys the right long-term choice, or should we add a "did you mean?" heuristic or strict mode?
- Future record types to consider: `watchPath` (cache invalidation on mtime change), `watchEnv` (env var change detection), `diagnostic` (debug logging).
