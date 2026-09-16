// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemAmountSign
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension TransactionInfoViewModel {
    static func mock(
        assetPrice: Price? = .mock(price: 1.5),
        feeAssetPrice: Price? = .mock(price: 0.5),
        feeValue: BigInt? = BigInt(10_000_000),
        sign: GemAmountSign = .none,
    ) -> TransactionInfoViewModel {
        TransactionInfoViewModel(
            currency: "USD",
            asset: .mock(),
            assetPrice: assetPrice,
            feeAsset: .mock(),
            feeAssetPrice: feeAssetPrice,
            value: BigInt(100_000_000),
            feeValue: feeValue,
            sign: sign,
        )
    }
}
