---
name: translation-review
description: Review or refresh Gem Wallet translations for contextual accuracy, consistent product terminology, and UI length. Use for localization audits and scheduled translation reviews; not for unrelated code reviews.
---

# Translation review

Read [the localization guide](../../../skills/localization.md), the applicable platform guides, and the [terminology index](references/terminology.md). Load only the locale profiles relevant to the requested scope. The localization guide maintains source locations and generation commands.

Review only unless the user authorizes fixes. Preserve existing worktree edits. Scheduled reviews report findings and questions without editing translations, committing, or publishing.

## Context before wording

Inventory the requested locales across app, widget, InfoPlist, and backend Fluent sources. For a full audit, read every message in scope; for an incremental audit, review changed English/source keys and their translations, then expand to related terminology and screens. Report coverage and anything deferred. Do not describe a sampled audit as exhaustive.

Trace ambiguous keys to actual iOS, Android, or backend consumers. Identify the UI role, domain meaning, and the producer and order of every interpolated value. A matching placeholder count does not prove correctness. For example, a rewards availability argument may be a duration on one platform and a date on another; a redemption argument may already include a localized action or a points symbol. Correct the producer when authorized if it causes duplicated wording across locales.

Apply the approved product glossary before general translation preferences. Keep terminology consistent across a feature, including notifications and errors. Scope replacements by meaning and key; staking rewards and invitation points are separate concepts. Do not mechanically replace a term everywhere. Distinguish genuine mistranslations from acceptable regional usage or stylistic preferences.

Treat each language and regional variant as a separate editorial locale. Establish the intended audience; do not infer a region from script alone or translate through another locale. Review native UI conventions, register, grammar, financial vocabulary, and brand usage. Shared product semantics do not authorize copying regional choices. When delegating, assign locale-specific reviews with their own context checks. A missing locale profile is not a blocker: inspect existing usage and callers, ask about unresolved product choices, and record confirmed decisions using the terminology index.

If intent, terminology, interpolated meaning, or fit is uncertain, ask a specific question before changing the affected text. Continue independent work. During an unattended review, report the question and leave that text unchanged; never use elapsed time as approval.

Use natural subject and possessive omission only where the target language permits it and context supplies the actor. Keep grammatically required pronouns, agreement, politeness, and explicit ownership or responsibility. First-person acceptance statements and shared referral messages may need an actor even when ordinary UI labels do not. Avoid literal English sentence structure and mechanically deleting “your” equivalents.

## Preserve fit and contracts

Keep replacements close to the existing visible length. Compact controls must retain their approved locale-specific length budget; do not apply one language’s character limit to every language. Do not expand them into explanatory labels. Compare visible characters and rendered width; inspect the layout when combining characters, script shaping, bidirectional text, font fallback, or interpolation makes counts misleading. Do not shorten away meaning or safety information to fit; ask about the tradeoff.

Keep keys, placeholders, argument order, markup, plural/select logic, brands, and units intact unless a verified bug requires a separately explained fix. Compare against English and sibling keys, but do not assume English wording is enough to resolve product context. Use actual sample values to verify interpolation without secrets.

## Fix and verify

Edit canonical Fluent sources, then run `just localize`; never hand-edit generated catalogs. Check duplicate/missing keys against English, placeholder parity, generated resource validity, and that unrelated locales did not change. Inspect the final diff for glossary drift and unexpected length growth. Follow the repository's two cleanup rounds and [quality checks](../../../skills/quality-checks.md), including real affected mobile/package builds and `cargo test -p localizer --all-features --locked` from `core` when backend messages change. Report blocked checks accurately.

For findings, include the key/path, current and proposed text, relevant caller context, reason, and old/new character counts for constrained labels. For fixes, report coverage, substantive examples, verification, and unresolved questions. Do not claim model output proves that a human translator or machine authored the original.
