# Localization

Use this skill for shared localization flow and generated output locations.
## Localization

- Shared localization updates flow through the monorepo root:
  ```bash
  just localize
  ```
- iOS localization output lives under `ios/Packages/Localization/`
- Android localization output lives under `android/ui/src/main/res/`
- Treat generated localization outputs as generated artifacts, not hand-edited source
