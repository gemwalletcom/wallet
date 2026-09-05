# Terminology and locale profiles

Use this index for any language. Shared concepts describe product behavior; locale profiles record approved wording and regional conventions. Load only profiles in scope.

## Shared product concepts

- Redelegation moves existing stake to another validator. Its label may use staking terminology, but explanations must preserve that behavior.
- The invitation system earns points and spends them to redeem assets. Keep earning and redemption actions distinct, retain actual asset values, and do not confuse invitation points with staking rewards. Follow each locale's approved terminology rather than forcing one universal label.
- Generic recommendations and invitations are different concepts. Brands may have region-specific names.
- Compact labels, placeholder roles, and security ownership are contextual constraints, not translation preferences. Preserve who controls assets, knows a recovery phrase, authorizes an action, or accepts responsibility.

## Profiles

| Locale | Audience | Guidance |
|---|---|---|
| `zh-Hans` | Mainland China | [Chinese, mainland](locales/zh-Hans.md) |
| `zh-Hant` | Taiwan, as targeted by this project | [Chinese, Taiwan](locales/zh-Hant.md) |
| `ja` | Japan | [Japanese](locales/ja.md) |

## Add or update a locale

Create `locales/<repository-locale-id>.md` and add a row above. Keep it short and use these sections:

1. **Audience and conventions:** region, register, grammar, subject omission, and relevant UI or script conventions. Distinguish observed conventions from explicit product decisions.
2. **Approved terminology:** a table with concept/key scope, preferred wording, and constraints or exceptions. Record only confirmed decisions; do not promote a model's preference to a requirement.
3. **Compact labels:** exact approved strings or length constraints only for the relevant controls.

Use repository locale IDs and verify the actual audience; shared language or script does not imply shared regional wording. Add representative examples only when they prevent recurring mistakes. Keep procedural rules in `SKILL.md` and shared meanings here, rather than duplicating them in every profile.

When no approved term exists, consult current source and caller context and use normal local conventions for clear cases. Ask about uncertain product meaning or regional choices before changing affected text. Keep unanswered choices in the review report rather than recording them as approved. Update a profile when the user confirms a choice; do not silently extend it to other locales.
