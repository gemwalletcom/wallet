# Pull Requests

Use when opening or editing a pull request. The body is for the reviewer: what is wrong, what is true now, and how that was shown, in plain words read in a minute. The diff and CI carry the rest.

## Title

The title follows the commit subject rule in [Release Process](release-process.md#commits): one imperative line of at most 72 characters with an optional area prefix and the TODO id.

## Body

`.github/pull_request_template.md` holds the shape with a hint per part; `gh pr create --body` does not read that file, so write the body to this shape yourself. Do not hard-wrap it.

- **Why:** what is wrong today, as the user or the code meets it, with one concrete case: the amount, screen or network. For a refactor, the rule written twice or the code nothing calls. For a feature, what the user could not do
- **What changed:** what is true now. One sentence per independent change, a numbered list in reading order for several changes or a large diff, the precedent it follows ("as iOS already does"), and "No behavior change" when that is so
- **Proof:** one result per [Proof of Done](quality-checks.md#proof-of-done) row: the test that fails on main, a Before | After screenshot table per platform, a live transaction, before and after numbers. What was not run, in one line. A result, not the commands that produced it

Add a line only when it applies:

- **Behavior change:** what users or dapps see differently
- **Not in this PR:** the sibling left out and where it is tracked
- **Deploy:** a setting, migration or rollout order the change needs
- `Closes #N`

Leave out what the diff and CI carry: file paths and identifiers, command lists, test counts, checklists, and tables whose Expected column repeats After. No empty headings or placeholders. A body that needs two Whys is two PRs.

## Example

```
**Why:** Chainflip sends a refund as a plain transfer unless the broker's parameters attach a contract message, which turns it into a contract call. Confirm checked those parameters except that field.

**What changed:** Confirm also requires the refund message to be empty, on EVM, Tron and Solana. The swap product doc gains the row.

**Proof:** the new refund case fails on main and passes here; the recorded broker fixtures and a live Tron USDT to Arbitrum USDC quote still pass.
```
