// Copyright (c). Gem Wallet. All rights reserved.

import Components

enum PreferencesRowKind {
    case currency
    case language
    case appearance
    case networks
    case contacts
    case perpetuals
    case perpetualLeverage
    case perpetualTakeProfit
    case perpetualStopLoss
}

struct PreferencesRowViewModel: Identifiable {
    let id: String
    let kind: PreferencesRowKind
    let model: ListItemModel
}
