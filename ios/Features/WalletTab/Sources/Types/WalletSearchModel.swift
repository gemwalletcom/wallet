// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct WalletSearchModel {
    var searchableQuery: String = .empty

    static var searchItemTypes: [SearchItemType] {
        [.asset, .perpetual, .list, .nft]
    }
}
