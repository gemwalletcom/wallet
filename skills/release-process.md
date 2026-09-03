# Release Process

Use when versioning, tagging, committing for review, rebasing, publishing, or triggering store builds.
## Branching

The repo follows a GitFlow-like release model:

- `main` tracks the latest production release
- `develop` is the primary integration branch
- `feature/...` branches start from `develop`
- `release/...` branches prepare production releases
- `hotfix/...` branches handle production fixes

## Versioning

- iOS, Android, tags, and Rust workspace packages use `Major.Minor.Patch`, for example `3.0.0`
- Internal build numbers remain separate and must keep increasing on every release
- Bump versions with:
  ```bash
  just bump patch
  just bump minor
  just bump major
  just bump 3.1.2
  ```
- `just bump` commits the version, creates a signed tag, and pushes commit and tag atomically. Tag creation is restricted to repository admins by a tag ruleset, so a non-admin's push is rejected as a whole. Hand the bump commit to an admin to push the tag (or create the GitHub release at that commit); `release_on_tag.yml` creates the GitHub release once the tag lands

## Commits

- Run the relevant tests, linters, and formatters before committing
- Write concise commit messages that explain the reason for the change, not just the file edits
- Do not add agent attribution trailers, `Co-Authored-By` lines, or session links to commits or PR descriptions. Match the repository style: a short imperative subject, optionally followed by a numbered list of the changes
- For a cross-stack feature, keep dependency-ordered commits that each build on their own (Core, then the Core provider or swap layer, then apps). Do not squash or re-split them without asking; that shape is what makes review and bisect work

## Release Builds

- The release tag is the input to everything downstream. `release_on_tag.yml` creates the GitHub release with generated notes on any tag push, and store builds check out the tag, so the tag must exist on the remote before a build is started.
- Android: `just android release` builds the Play AAB and `just android release-apk` the universal APK; channel variants come from `-Pchannel` (see `android/skills/release-and-verification.md`). The F-Droid build is reproduced from `android/reproducible/fdroid/`.
- iOS: `just generate-stone` builds the Rust static libraries and UniFFI sources the archive links, then the `Gem` scheme is archived in the Release configuration from `ios/Gem.xcodeproj`.
- When two releases with near-identical source differ in binary size or behavior, suspect the build machine's Rust toolchain and cargo cache state before source: archive both tags locally on one machine and compare.

## Publication Boundaries

- Treat commit, rebase, push, force-push, PR creation, issue creation, review replies, and thread resolution as separate actions. Perform only the actions the user requested.
- Follow the requested branch name and commit shape. Do not rename the branch, squash, or split commits unless requested or required by repository policy.
- Before rebasing, fetch the intended upstream and verify the current branch, worktree, local changes, and target base. Rerun focused verification after resolving conflicts.
- A rebase or amended commit does not authorize a push. When rewritten history must be published, verify the remote branch and use `git push --force-with-lease` rather than an unconditional force push.
- Do not continuously rebase merely because upstream moved. Rebase for a real conflict, an explicit request, or a stated release requirement.
- While a PR is under review, do not amend or force-push per comment. Batch fixes in the working tree until the review round ends, then push one update so reviewers can diff against what they commented on.

## Removing or Disabling Support

Use when deleting or hiding a chain, asset, provider, endpoint, serialized field, generated enum case, or persisted identifier.

- Inventory persisted and serialized values first. Existing wallets, accounts, transactions, preferences, and old app versions may still need to decode or migrate the value.
- Inspect actual callers in the currently supported shipped release tags; current-source compilation is not compatibility proof. State which current and legacy clients remain supported: expected failure in an intentionally unsupported client is a release decision, not automatically a blocker, but its user-visible behavior must be understood and documented.
- Keep the Core enum and wire value when historical data must stay decodable, and remove mobile exposure through the established mechanism (`#[typeshare(skip)]`) only after confirming that requirement.
- Add an explicit one-time migration on every platform that persists the affected data, and run that platform's migration tests. A fresh install or permanent runtime filtering on every launch is not evidence that upgrades are safe.
- Keep similarly named but still-supported chains and assets distinct: check raw identifiers, filenames, token metadata, and migration predicates so cleanup cannot cross the boundary.
