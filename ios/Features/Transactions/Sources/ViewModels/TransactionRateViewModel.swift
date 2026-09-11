// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapRate
import Localization
import PrimitivesComponents

struct TransactionRateViewModel {
    private let rate: GemSwapRate?
    private let isInverse: Bool

    init(
        rate: GemSwapRate?,
        isInverse: Bool,
    ) {
        self.rate = rate
        self.isInverse = isInverse
    }
}

extension TransactionRateViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let rate else { return .empty }
        return .rate(title: Localized.Buy.rate, value: AssetRateViewModel(rate: rate).text(isInverse: isInverse))
    }
}
