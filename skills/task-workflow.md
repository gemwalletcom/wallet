# Task Workflow

Use for every task. Follow the requested scope, ground changes in existing code, and verify the result.

Do not hard-wrap prose at 80 columns in docs, skills, commit messages, PR bodies, or other written output. GUIs, editors, and terminals wrap to the reader's width; keep one paragraph per block and one list item per line. Fenced code, tables, and headings stay as they are.

## 1. Establish the Task Contract

- Classify the request: investigate, review, implement, or publish. Investigation and review are read-only unless fixes are requested. Publication follows [Release Process](release-process.md#publication-boundaries); authorization already given in the session remains valid.
- Confirm repository, checkout/worktree, branch, and `git status --short`. Preserve unrelated changes; avoid stashing the user's tree. If unavoidable, label the stash and verify it before restoring.
- Fetch remote state only when needed. Record the exact base/head for a review; compare PRs against their own bases, using `git range-diff` when they differ. Inventory review threads separately.
- Load applicable platform guides and the relevant topic sections once. Use the [architecture index](../docs/ARCHITECTURE.md#find-the-relevant-contract-and-example) instead of reading every design document. State assumptions affecting scope, security, or compatibility.

## 2. Ground the Change Before Editing

- Find the entry point, existing owner, immediate callers, analogous implementation, and relevant tests/testkit. Read them; naming a file or finding a symbol is not understanding its contract. Check the analogue against current guidance before reusing it.
- For behavior changes, briefly state what will change, which existing path will own it, and which example and checks apply. A few sentences suffice; this is a progress update, not an approval gate or a new design document.
- Trace the affected path far enough to establish behavior: inputs/configuration → decoding/storage → domain rule → runtime/signer → interface → consumer, including only the layers involved. A declaration, generated model, or registration does not prove working support.
- Before adding a type, helper, service, module, or dependency, identify why the existing owner cannot handle the requirement. Apply [Engineering Principles](engineering-principles.md#abstractions-must-earn-their-place).
- For large refactors or audits, identify the architectural starting point and affected surfaces, then freeze the review range. Keep observed behavior, proposals, and upstream compatibility claims distinct.

## 3. Run a Tight Implementation Loop

- Change the owning layer, then its consumers. For shared features, implement Core and its tests first, regenerate only at changed mobile contracts, then integrate the apps. Follow [Cross-Platform Awareness](cross-platform-awareness.md) for parity and generated files.
- Use narrow checks from [Quality Checks](quality-checks.md) while iterating. Preserve working checkpoints at Core → bindings → app boundaries.
- Delegate bounded independent searches or verbose verification when useful. The implementing agent still reads the affected owner and callers; delegation does not replace understanding them.
- If a correction rejects the design, re-examine ownership and remove the rejected path before adding another wrapper or fallback. Resolve repeated failures from evidence instead of stacking workarounds.
- Keep tool output focused. After compaction, recover the task contract, changed files, decisions, and verification state from the checkpoint and diff; reread guidance only if missing, changed, or newly relevant.
- **Cleanup round one**, before the closing verification batch: review the whole diff for unnecessary abstractions, duplicate paths, dead code, stale fixtures, imports, and inconsistent naming. Every new symbol needs a current consumer or contract. Keep cleanup within the task.

## 4. Verify the Actual Result

- Run the applicable [Quality Checks](quality-checks.md) from the changed checkout and directory, with the features that compile the affected code. Zero selected tests is not a pass. If concurrent checks contend or return unclear status, rerun the affected check individually.
- Report deterministic tests, compiled-only tests, and gated live checks separately. Correctness, performance, compatibility, and exact output are separate claims.
- After switching branches with changed generated contracts, check stale ignored artifacts before diagnosing app code. Regenerate only the affected outputs and repeat the focused check.

## 5. Hand Off a Reproducible State

- **Cleanup round two**, after verification: read each hunk as a reviewer. Remove remaining redundancy, unused API, comments prohibited by the style guide, and debugging leftovers. Rerun affected checks if source changes.
- Inspect final diff and status. Report what changed, exact commands and results, skipped/blocked checks, and remaining risks. Do not describe inspection, compilation, or an unexecuted test as runtime verification.
- Leave changes uncommitted and external systems unchanged unless the user authorized those actions.

## 6. Maintain Agent Guidance

- Evaluate corrections and costly discoveries at handoff. In implementation or guidance tasks, fix a demonstrably wrong guide alongside the change. Investigations and reviews remain read-only unless edits are authorized. Promote reusable lessons when guidance work is authorized; otherwise note their proposed home. Optional agent memory is a retrieval aid, not policy.
- Keep procedures in `skills/`, platform rules beside the platform, current contracts in `docs/`, and rationale in decision records. Never retain temporary paths, credentials, transient status, or unverified workarounds.
- Use [Guidance Refresh](../.agents/skills/guidance-refresh/SKILL.md) for periodic history sweeps. Verify guidance changes with the documentation checks in [Quality Checks](quality-checks.md).
