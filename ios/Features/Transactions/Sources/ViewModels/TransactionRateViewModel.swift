// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemSwapRate
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct TransactionRateViewModel {
    private let rate: GemSwapRate?
    private let direction: AssetRateFormatter.Direction

    init(
        rate: GemSwapRate?,
        direction: AssetRateFormatter.Direction,
    ) {
        self.rate = rate
        self.direction = direction
    }
}

extension TransactionRateViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard
            let rate,
            let value = try? AssetRateFormatter().rate(
                fromAsset: rate.from.asset.map(),
                toAsset: rate.to.asset.map(),
                fromValue: BigInt(rate.from.value),
                toValue: BigInt(rate.to.value),
                direction: direction,
            )
        else {
            return .empty
        }
        return .rate(title: Localized.Buy.rate, value: value)
    }
}
