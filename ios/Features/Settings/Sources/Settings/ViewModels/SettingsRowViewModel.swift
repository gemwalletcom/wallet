// Copyright (c). Gem Wallet. All rights reserved.

import Components

enum SettingsRowDestination {
    case wallets
    case security
    case notifications
    case preferences
    case walletConnect
    case support
    case rewards
    case aboutUs
    case developer
}

struct SettingsRowViewModel: Identifiable {
    let id: String
    let destination: SettingsRowDestination
    let model: ListItemModel
}
