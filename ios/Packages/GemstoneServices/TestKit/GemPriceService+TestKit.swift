// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemPriceService
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension GemPriceService {
    static func mock(db: DB = .mock()) -> GemPriceService {
        GemPriceService(
            store: GemstonePriceStore(priceStore: .mock(db: db), fiatRateStore: .mock(db: db)),
        )
    }
}
