# Localization

Use when adding or changing a user-facing string produced by Core (price alerts, rewards, in-app notifications, support). Mobile app strings are separate; see the root [Localization](../../skills/localization.md) skill.

Strings live in the `localizer` crate (Fluent `.ftl`, read with `gem_fluent` and resolved by the crate itself), one file per language at `crates/localizer/i18n/<lang>/localizer.ftl`. The build script picks up every language folder and fails the build on a file it cannot read. `en` is the canonical key set and fallback. Maintain translations directly; there is no download step.

To add a string:

1. Add the key to `crates/localizer/i18n/en/localizer.ftl`, using `{$var}` for placeholders.
2. Add the same key, translated, to every other `<lang>/localizer.ftl` with identical key, placeholders, and emoji; only the prose changes. A key missing in a language silently falls back to `en`.
3. Expose it as a typed method on `LanguageLocalizer` in `crates/localizer/src/lib.rs`: a row in the `messages!` table (`key(var: &str);`, named exactly like the key and its placeholders) for a plain message, or a hand-written method that calls `self.format(id::key, &[("var", &value)])` when it combines messages. The build generates one `id` constant per `en` key, so a key that does not exist fails to compile. Then call that method from consumers (`services`, `rewards`, `in_app_notifications`). Never inline user-facing strings or reference raw keys outside `localizer`.

A message is one line and may use `{$var}` and a reference to another message (`{ other_key }`); anything else from Fluent, plural selectors included, fails the build. Write a count after a label (`💎 Points earned: {$value}`) so the message reads correctly with any number in every language.

Fluent wraps interpolated arguments in isolation marks (`\u{2068}…\u{2069}`); account for them in test assertions (see the tests in `crates/localizer/src/lib.rs`).
