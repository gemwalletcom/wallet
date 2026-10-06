# Task Workflow

Use for every task. Follow the requested scope, ground changes in existing code, and verify the result.

Do not hard-wrap prose in docs, skills, commit messages, or PR bodies. Keep one paragraph per block and one list item per line; leave code, tables, and headings as they are.

## 1. Establish the Task Contract

- Classify the request: investigate, review, implement, or publish. Investigation and review are read-only unless fixes are requested. Publication follows [Release Process](release-process.md#publication-boundaries); authorization already given in the session remains valid.
- Confirm repository, checkout, branch, and `git status --short`. Preserve unrelated changes; avoid stashing the user's tree. If unavoidable, label the stash and verify it before restoring.
- Fetch remote state only when needed. Record the exact base/head for a review; compare PRs against their own bases, using `git range-diff` when they differ. Inventory review threads separately.
- Load applicable platform guides and the relevant topic sections once. Use the [architecture index](../docs/ARCHITECTURE.md#find-the-relevant-contract-and-example) instead of reading every design document. State assumptions affecting scope, security, or compatibility.
- Before changing behavior, read the area's [Product](../docs/PRODUCT.md) page. Raise any necessary change to a stated rule with the user as a product decision before editing. Record settled important user-facing rules in that page in the same change, not every case.

## 2. Ground the Change Before Editing

- Read the entry point, owner, immediate callers, analogue, and relevant tests/testkit. Check the analogue against current contracts before reusing it.
- For behavior changes, state the intended change, owner, reference, and checks in a brief progress update; this does not require approval or a design document.
- Trace the involved layers from input through the domain rule to its consumer. Declarations, generated models, and registrations alone do not prove behavior.
- Before adding a type, helper, service, module, or dependency, identify why the existing owner cannot handle the requirement. Apply [Engineering Principles](engineering-principles.md#abstractions-must-earn-their-place).
- For large refactors or audits, identify the architectural starting point and affected surfaces, then freeze the review range. Keep observed behavior, proposals, and upstream compatibility claims distinct.

## 3. Run a Tight Implementation Loop

- Change the owner, then consumers: Core and tests → generation when required → apps. Apply [Cross-Platform Awareness](cross-platform-awareness.md).
- Use narrow checks from [Quality Checks](quality-checks.md) while iterating. Preserve working checkpoints at Core → bindings → app boundaries.
- Delegate bounded independent searches or verbose verification when useful. The implementing agent still reads the affected owner and callers; delegation does not replace understanding them.
- When a design is rejected, re-examine ownership and remove that path before replacing it. Resolve failures from evidence, not stacked workarounds.
- Keep output focused. After compaction, recover scope, changes, decisions, and checks from the checkpoint and diff; reread guidance only if missing, changed, or newly relevant.
- **Cleanup round one**, before the closing verification batch: review the whole diff for unnecessary abstractions, duplicate paths, dead code, stale fixtures, imports, and inconsistent naming. Every new symbol needs a current consumer or contract. Keep cleanup within the task.

## 4. Verify the Actual Result

- Run applicable [Quality Checks](quality-checks.md), including the [Security checklist](security.md#review-checklist) for wallet-critical changes. Execution and reporting requirements live there.
- After switching branches with changed generated contracts, check stale ignored artifacts before diagnosing app code. Regenerate only the affected outputs and repeat the focused check.

## 5. Hand Off a Reproducible State

- **Cleanup round two**, after verification: read each hunk as a reviewer. Remove remaining redundancy, unused API, comments prohibited by the style guide, and debugging leftovers. Rerun affected checks if source changes.
- Inspect final diff and status. Report changes, exact commands/results, skipped or blocked checks, and risks. For wallet-critical flows, explicitly surface skipped records, swallowed errors, and untested branches.
- Leave changes uncommitted and external systems unchanged unless the user authorized those actions.
- Local checks are the gate, not CI. Push only after every applicable local check passes, then let CI run in the background and start the next task instead of waiting on it. Name the checks that only CI can run in the handoff, and treat a CI failure as a finding for the next commit.

## 6. Maintain Agent Guidance

- At handoff, evaluate corrections and costly discoveries. Fix demonstrably wrong guidance during implementation or authorized guidance work; investigations and reviews stay read-only. Otherwise propose the lesson's home. Memory aids retrieval, not policy.
- Keep procedures in `skills/`, platform rules beside the platform, current contracts in `docs/`, and rationale in decision records. Never retain temporary paths, credentials, transient status, or unverified workarounds.
- Use [Guidance Refresh](../.agents/skills/guidance-refresh/SKILL.md) for periodic history sweeps. Verify guidance changes with the documentation checks in [Quality Checks](quality-checks.md).
