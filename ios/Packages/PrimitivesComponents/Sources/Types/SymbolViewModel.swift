// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetIcon
import Primitives
import Style

public struct SymbolViewModel: Sendable, AmountDisplayable {
    private let symbol: String
    private let image: AssetImage?

    public init(asset: Asset, icon: GemAssetIcon?) {
        symbol = asset.symbol
        image = icon.map { AssetImage(icon: $0) }
    }

    public var amount: TextValue {
        TextValue(
            text: symbol,
            style: TextStyle(
                font: .body,
                color: Colors.black,
                fontWeight: .semibold,
            ),
            lineLimit: 1,
        )
    }

    public var fiat: TextValue? {
        nil
    }

    public var assetImage: AssetImage? {
        image
    }
}
