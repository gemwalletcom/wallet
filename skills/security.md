# Security

Gem Wallet is a crypto wallet. Security-sensitive changes require extra scrutiny even when the code change looks small.

## Pause Before Editing These Areas

Before editing any of the following, confirm the task explicitly intends to change security behavior. If it does not, the change is probably wrong — stop and ask. Re-read this file in full before continuing.

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
- Treat provider-returned spenders, routers, transaction targets, calldata, and approval metadata as untrusted. Validate authority against an independent provider-and-chain policy before constructing an approval, and bind the final transaction target to the validated route.
- Exact approval amount reduces exposure but does not make an attacker-selected spender safe. Permit2 and multi-call approval flows require separate validation of each authority boundary.

## Storage and Auth

- Use the existing platform-secure storage layers instead of plain preferences or database storage for secrets
- Preserve biometric and device-auth requirements unless the task explicitly changes them
- Keep lock, unlock, and session flows aligned with current platform behavior
- Before adding recovery behavior, classify the stored material as user-owned and irreplaceable or safely regenerable. Never reset seed phrases, imported private keys, wallet passwords, or other irreplaceable material as a generic recovery strategy.
- Retry only errors known to be transient. Do not catch a broad exception class and retry persistent authentication-tag, integrity, decryption, or key-invalidated failures through the same broken storage path.
- Scope reset/recovery to the smallest regenerable namespace after tracing every consumer. A device-auth identity failure must not clear wallet secrets, password storage, or unrelated preferences.
- Describe a recovery fix by the observed paths it covers. Do not imply that one exception mapping or retry handles every platform keystore failure.
- Do not run Keystore or Keychain operations eagerly during app startup (`App.onCreate`, app init) or on threads where an uncaught throw kills the process (OkHttp interceptors, coroutine roots without handlers). Defer them to first use, keep them off the main thread, and map failures to a typed error the caller can handle.

## Cross-Platform Rule

- If a security-sensitive Core change affects mobile interfaces, generated outputs, platform build inputs, or app-side integration, regenerate bindings and verify the affected apps
- If only one app changes a shared security or transaction flow, call out the parity risk explicitly

## Optional External Security Skills

For deeper security review, an agent may load external security skill packs when they are installed or available in its environment. These are optional aids, not substitutes for this repository's rules or platform verification.

- [Trail of Bits Skills](https://github.com/trailofbits/skills): security research, vulnerability detection, audit workflows, static analysis, supply-chain review, constant-time analysis, zeroization audit, and related skills.
- [NVIDIA SkillSpector](https://github.com/NVIDIA/SkillSpector): scanner for AI agent skills that can help detect malicious patterns, unsafe instructions, and security risks in skill definitions.

Use external skills when the change touches wallet-critical flows, cryptography, signing, dependency or supply-chain risk, untrusted input handling, CI automation, or agent skill definitions. Do not install or run external tools in a way that uploads secrets, private code, wallet data, signing payloads, or credentials without explicit approval.

## Review Checklist

Before finishing a security-sensitive change, check:

1. No secret material is exposed in logs, errors, analytics, snapshots, tests, or local storage
2. Transaction inputs and outputs remain explicit, validated, and correctly typed
3. Existing auth and confirmation gates still execute on every required path
4. New external-input handling is validated against malformed or hostile input
5. Affected platforms were verified when mobile interfaces, generated outputs, platform build inputs, or app-side integration changed
6. Approval spenders, routers, transaction targets, and multi-call destinations are independently authorized rather than cross-checked only against values from the same response
