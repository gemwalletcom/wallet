# Localization

Use this skill for shared localization flow and generated output locations.

- Shared localization updates flow through the monorepo root:
  ```bash
  just localize
  ```
- Mobile app localization source of truth lives in `localization/app/*.ftl`
- Use Fluent message IDs with underscores, for example `common_cancel`
- Fluent comments are supported (`#`, `##`, `###`) and ignored by generation; put source context comments in English `en.ftl`
- Add new keys in the matching prefix section (`common_*` under `# Common`, `wallet_*` under `# Wallet`, etc.)
- Add each new app key to every language file, translated for the context where the string is used
- iOS InfoPlist localization source lives in `localization/InfoPlist/*.ftl`
- iOS widget localization source lives in `localization/widget/*.ftl`
- iOS localization output lives under `ios/Packages/Localization/`
- Android localization output lives under `android/ui/src/main/res/`
- Treat generated localization outputs as generated artifacts, not hand-edited source
- Core/backend localization is separate under `core/crates/localizer/i18n/`
