// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
import PrimitivesTestKit
@testable import Swap

extension PriceImpactViewModel {
    static func mock(fromValue: BigUInt, toValue: BigUInt) -> PriceImpactViewModel {
        let assetPrice = AssetPriceValue(asset: .mockEthereum(), price: .mock())
        return PriceImpactViewModel(
            fromAssetPrice: assetPrice,
            swapPriceImpact: assetPrice.swapValue(fromValue)
                .priceImpact(receive: assetPrice.swapValue(toValue)),
        )
    }
}
