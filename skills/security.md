# Security

Gem Wallet is a crypto wallet. Security-sensitive changes require extra scrutiny even when the code change looks small.

## Always Treat These Areas As High Risk

- Seed phrases, private keys, backup material, wallet import and export
- Transaction construction, signing, simulation, and submission
- Address parsing, chain selection, asset identifiers, and amount conversion
- Authentication, biometrics, lock flows, session handling, and secure preferences
- QR scanning, deep links, WalletConnect, browser-to-wallet handoff, and external payload parsing
- Any `core/` cryptographic, signing, encoding, or generated-model change

## Non-Negotiable Rules

- Never log, print, persist, or transmit secret material outside the approved secure-storage path
- Never add test fixtures containing real secrets, production credentials, or reusable wallet material
- Do not weaken existing confirmation, signing, simulation, or authentication checks for convenience
- Keep transaction-critical values explicit: chain, asset, amount, recipient, fee, nonce, calldata, and signature context must not become ambiguous
- Validate external input defensively and prefer existing parsers, mappers, and domain types over ad hoc string handling
- Prefer fail-closed behavior when security state is missing, invalid, or unsupported

## Storage and Auth

- Use the existing platform-secure storage layers instead of plain preferences or database storage for secrets
- Preserve biometric and device-auth requirements unless the task explicitly changes them
- Keep lock, unlock, and session flows aligned with current platform behavior

## Cross-Platform Rule

- If a security-sensitive behavior is shared through `core/`, regenerate bindings and verify both iOS and Android
- If only one app changes a shared security or transaction flow, call out the parity risk explicitly

## Review Checklist

Before finishing a security-sensitive change, check:

1. No secret material is exposed in logs, errors, analytics, snapshots, tests, or local storage
2. Transaction inputs and outputs remain explicit, validated, and correctly typed
3. Existing auth and confirmation gates still execute on every required path
4. New external-input handling is validated against malformed or hostile input
5. Both platforms were verified if shared `core/` behavior changed
