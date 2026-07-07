// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum TabItem: Identifiable, CaseIterable {
    case wallet
    case activity
    case settings
    case markets

    var id: Self {
        self
    }
}
