// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetRow

public extension GemAssetRow {
    static let wallet = GemAssetRow(title: .asset, showsSymbol: false, subtitle: .price, trailing: .balance)
}
