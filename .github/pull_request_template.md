<!-- One row per thing a user, dapp or operator sees: the case, what should happen, what happened on main, what happens now. A short value in each cell, the value itself and never "wrong" or "fixed". Last, the nearest case that stays the same. Delete the table only when the change shows nowhere: a refactor, tests, docs. -->

| What | Expected | Before | After |
|---|---|---|---|
| | | | |

**Why:** <!-- What is wrong today, as the user or the code meets it, with one concrete case: the amount, screen or network. For a refactor, the rule written twice or the code nothing calls. For a feature, what the user could not do. -->

**What changed:** <!-- What is true now. One sentence per independent change; a numbered list in reading order when there are several or the diff is large. Name the precedent it follows ("as iOS already does"). End with "No behavior change" when that is so. -->

**Proof:** <!-- One result per Proof of Done row (skills/quality-checks.md): the test that fails on main, a live transaction, before and after numbers. What was not run, in one line. A result, not the commands that produced it. -->

<!-- Required when a screen changes: the same screen and state on main and on this branch, cropped to what changed, one row per affected platform. Attach the images through GitHub's form so GitHub hosts them; never commit one, and never capture a secret screen. A new screen has no Before; write "not run" in a cell not captured. Delete the table when no screen changes. -->

| | Before | After |
|---|---|---|
| iOS | | |
| Android | | |

<!-- Only when it applies, one line each:
**Behavior change:** what users or dapps see differently and no table row carries.
**Not in this PR:** the sibling left out and where it is tracked.
**Deploy:** a setting, migration or rollout order the change needs.
Closes #N
Leave out what the diff and CI carry: file paths and identifiers, command lists, test counts, checklists. A body that needs two Whys is two PRs. -->
