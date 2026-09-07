---
name: translation-review
description: Review or improve Gem Wallet translations for contextual accuracy, native regional usage, consistency, and UI fit. Use for localization audits and scheduled reviews.
---

# Translation review

Follow the [localization guide](../../../skills/localization.md) for sources and generation, and [quality checks](../../../skills/quality-checks.md) for verification.

- Establish locales and scope across app, widget, permission, and backend messages. For incremental reviews, check changed source keys and related terminology; report coverage and deferred work.
- Read actual callers and interpolated values on each affected platform. Translate the intended action or state, not isolated English words. Preserve distinctions between earning, spending, ownership, and authorization.
- Use each locale's natural grammar, regional vocabulary, and register. Omit obvious subjects and possessives where natural, while retaining necessary actors and responsibility. Do not mechanically convert another locale or treat acceptable usage as an error.
- Keep terminology consistent within its product context. Use existing corrected strings and user decisions as the reference; do not duplicate translations in a separate glossary. If particular wording must stay fixed, identify its locale and message key.
- Keep text close to its current rendered length, especially compact controls. Preserve placeholders, argument roles, markup, plural rules, and safety meaning. Check actual sample values when character counts alone cannot establish fit.
- **If meaning, terminology, regional usage, or fit is uncertain, ask the user before changing that text.** Continue independent work; unattended reviews leave uncertain text unchanged and report the question.
- Edit only when authorized, preserve unrelated changes, and regenerate outputs from canonical sources. Scheduled reviews report findings without editing or publishing.
- Before handoff, make two cleanup passes: simplify and check consistency, then review the final diff after verification. Check keys, placeholders, generated outputs, and affected builds/tests; report failures and anything skipped. For documentation-only edits, validate links and run `git diff --check`.

Report substantive findings with the message key, proposed wording, and a brief contextual reason. Avoid preference-only churn and claims about how the original translation was produced.
