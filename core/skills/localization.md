# Localization

Use when adding or changing a user-facing string produced by Core (price alerts, rewards, in-app notifications, support). Mobile app strings are separate; see the root [Localization](../../skills/localization.md) skill.

Strings live in the `localizer` crate (Fluent `.ftl` + `i18n_embed`), one file per language at `crates/localizer/i18n/<lang>/localizer.ftl`. `en` is the canonical key set and fallback. Maintain translations directly; there is no download step.

To add a string:

1. Add the key to `crates/localizer/i18n/en/localizer.ftl`, using `{$var}` for placeholders.
2. Add the same key, translated, to every other `<lang>/localizer.ftl` with identical key, placeholders, and emoji; only the prose changes. A key missing in a language silently falls back to `en`.
3. Expose it as a typed method on `LanguageLocalizer` in `crates/localizer/src/lib.rs` via the `fl!` macro, then call that method from consumers (`pricer`, `gem_rewards`, `in_app_notifications`, `support`). Never inline user-facing strings or reference raw keys outside `localizer`.

Fluent wraps interpolated arguments in isolation marks (`\u{2068}…\u{2069}`); account for them in test assertions (see `crates/localizer/tests/localizer.rs`).
