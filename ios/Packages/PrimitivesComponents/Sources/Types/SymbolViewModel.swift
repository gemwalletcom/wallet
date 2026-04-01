// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style

public struct SymbolViewModel: Sendable, AmountDisplayable {
    private let symbol: String
    private let image: AssetImage

    public init(asset: Asset) {
        symbol = asset.symbol
        image = AssetViewModel(asset: asset).assetImage
    }

    public init(assetId: AssetId) {
        symbol = assetId.tokenId?.truncate(first: 8, last: 6) ?? assetId.chain.asset.symbol
        image = AssetIdViewModel(assetId: assetId).assetImage
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

    public var fiat: TextValue? { nil }

    public var assetImage: AssetImage? { image }
}
