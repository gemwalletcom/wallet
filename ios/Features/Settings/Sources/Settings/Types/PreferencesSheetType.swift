// Copyright (c). Gem Wallet. All rights reserved.

enum PreferencesSheetType: Identifiable {
    case leverage
    case marginType
    case takeProfit
    case stopLoss

    var id: Self { self }
}
