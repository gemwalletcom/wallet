# Rewards Feature Module

This module implements the rewards program feature for Gem Wallet.

## Structure

The module follows the standard Gem Wallet feature architecture with two submodules:

### viewmodels
Contains the business logic and state management:
- `RewardsViewModel.kt` - Main ViewModel handling rewards data and actions
- `models/` - UI models for the rewards sections, redemption options and incoming codes

### presents
Contains the UI/presentation layer:
- `RewardsScreen.kt` - Main composable screen, binds `RewardsViewModel`
- `RewardsScene.kt` - Stateless rewards scene

## Usage

### Navigation

To navigate to the rewards screen:

```kotlin
navigator.openRewards()
```

### Integration

The app registers the rewards destination (`RewardsRoute`, `routes/Rewards.kt`) in its navigation graph:

```kotlin
rewards(
    onClose = onCancel
)
```

## Features

- Display user's referral code
- Show referral statistics (total referrals, rewards)
- Copy referral code to clipboard
- Share referral code

## Dependencies

- `ui` - Shared UI components
- `ui-models` - Domain models
- `data:repositories` - Data layer access
- Jetpack Compose
- Hilt for dependency injection
