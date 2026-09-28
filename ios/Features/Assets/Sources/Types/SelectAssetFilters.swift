// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSelectAssetFlow
import Store

extension GemSelectAssetFlow {
    var requestScope: AssetsQueryScope {
        switch scope {
        case .wallet: .wallet
        case .allAssets: .allAssets
        }
    }
}
