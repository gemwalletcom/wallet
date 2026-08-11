// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PriceService
import Store
import StoreTestKit

public extension PriceService {
    static func mock(db: DB = .mock()) -> Self {
        PriceService(
            priceStore: .mock(db: db),
            fiatRateStore: .mock(db: db),
        )
    }
}
