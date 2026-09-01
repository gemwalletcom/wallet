# Platform Operations

## Tooling and Platform Notes

- Xcode latest stable is expected
- Apple Silicon Macs are the default environment
- Apple framework docs are available at:
  `/Applications/Xcode.app/Contents/PlugIns/IDEIntelligenceChat.framework/Versions/A/Resources/AdditionalDocumentation`
- Use Xcode directly when you need interactive debugging for failing tests or build issues

## Localization

- iOS localization is generated under `Packages/Localization/`
- Run:
  ```bash
  just localize
  ```
- `just localize` writes both the `.xcstrings` catalogs and the typed `Localized.swift`/`WidgetLocalized.swift` accessors; no separate codegen step is needed

## Security

- Keep secrets and secure preferences in the Keychain-backed layers, not plain storage
- Keystore-related behavior belongs in the dedicated security packages
- Biometric and secure-storage flows should preserve existing platform behavior unless the task explicitly changes them
